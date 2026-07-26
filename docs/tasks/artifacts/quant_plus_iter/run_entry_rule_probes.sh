#!/usr/bin/env bash
# QUANT-PLUS-ITER.1 — the distinguishing factor between the "failing" scratch shape
# and the working VHDL shape, proven by DECLARED VERDICTS.
#
# ORIGINAL THESIS (`.1`, historical — the defect this driver was written to pin)
#   PGEN's canonical entry rule was `rule_order[0]` — the rule DEFINED FIRST in the
#   file. The charter's probe grammar defines `stmt` ABOVE `scratch`, so the generated
#   parser was entered at `stmt` and the `+` in `scratch` was never executed at all.
#   `+` is not defective.
#
# ⭐ SUPERSEDED BY `.2`: the entry rule is now DECLARED (`@entry: true`, attached to
#   the rule), and since step C a grammar that declares none is REFUSED. The
#   order-sensitivity this driver originally demonstrated is therefore IMPOSSIBLE by
#   construction. The driver is kept and RE-PINNED rather than deleted, so the record
#   shows the defect class closing:
#     §0 asserts the refusal (the fix),
#     §1 asserts the DECLARED entry decides dispatch regardless of file position,
#     §2 asserts the linter now NAMES the entry (`.1` measured that it never did).
#
# WHAT THIS DRIVER PROVES (all cheap — no parser rebuild required)
#   0. an undeclared grammar is REFUSED with an actionable message.
#   1. the declared entry decides dispatch in BOTH file orderings.
#   2. `--lint-grammar` names the resolved entry (it never did when `.1` ran).
#   3. `--report-certificate-coverage` refuses an unregistered grammar name.
#   4. the shipped grammars that "worked" defined their entry FIRST (vhdl_file,
#      and the scratch slot's own committed fixture) — which is why nothing caught it.
#
# The BEHAVIOURAL flip (default entry REJECTs `a;a;` at position 2 while
# `--entry-rule scratch` ACCEPTs it, in ONE binary) needs a probe rebuild and is
# captured in `entry_rule_flip_capture.txt`; the recipe is printed at the end.
#
# Run from the repository root. Exit 0 + "0 divergences" = every declared verdict held.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT" || exit 1

PIPELINE="./rust/target/debug/ast_pipeline"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/quant_plus_iter.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

DIVERGENCES=0
CASES=0

expect() { # expect <label> <expected> <actual>
  CASES=$((CASES + 1))
  if [[ "$2" == "$3" ]]; then
    printf '  ✅ %-58s %s\n' "$1" "$3"
  else
    printf '  ⛔ %-58s expected=%s actual=%s\n' "$1" "$2" "$3"
    DIVERGENCES=$((DIVERGENCES + 1))
  fi
}

echo "=== QUANT-PLUS-ITER.1 — entry-rule probes ==================================="

if [[ ! -x "$PIPELINE" ]]; then
  echo "⛔ missing $PIPELINE — build it with:"
  echo "     (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)"
  exit 1
fi
SURFACE="$("$PIPELINE" --report-feature-surface 2>&1 | head -1)"
echo "$SURFACE"
case "$SURFACE" in
  *ebnf_dual_run=true*) ;;
  *) echo "⛔ ast_pipeline lacks ebnf_dual_run (a plain build — e.g. a prior 'make focus_*' — overwrote it)."
     echo "   rebuild: (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)"
     exit 1 ;;
esac
echo

# ---------------------------------------------------------------------------
# The two grammars differ ONLY in the ORDER the two rules are defined.
# ---------------------------------------------------------------------------
# ⭐ RE-PINNED BY `.2` STEP C. When `.1` ran, these two grammars differed ONLY in
# definition order and that silently decided the entry rule. Since step C an
# undeclared grammar is REFUSED, so the defect class `.1` documented is now
# impossible by construction — the grammars below therefore DECLARE their entry,
# and the order-sensitivity assertion is replaced by the refusal assertion in §0.
cat > "$WORK/A_stmt_first.ebnf" <<'EOF'
stmt := "a" ";"
      | "b" ";"

@entry: true
scratch := stmt+
EOF

cat > "$WORK/B_scratch_first.ebnf" <<'EOF'
@entry: true
scratch := stmt+

stmt := "a" ";"
      | "b" ";"
EOF

cat > "$WORK/undeclared.ebnf" <<'EOF'
stmt := "a" ";"
scratch := stmt+
EOF

echo "--- 0. `.1`'s defect class is now IMPOSSIBLE (step C) ------------------------"
undeclared_out="$("$PIPELINE" "$WORK/undeclared.ebnf" --lint-grammar 2>&1)"; undeclared_rc=$?
expect "a grammar with no @entry is REFUSED (was silently mis-rooted)" "1" "$undeclared_rc"
expect "…and the refusal explains how to fix it" "1" \
       "$(printf '%s' "$undeclared_out" | grep -c 'must contain one and only one')"
echo

echo "--- 1. codegen: the DECLARED entry decides dispatch, not file order ----------"
for variant in A_stmt_first B_scratch_first; do
  "$PIPELINE" "$WORK/$variant.ebnf" --generate-parser --output "$WORK/$variant.rs" \
    > "$WORK/$variant.gen.log" 2>&1
  # the single dispatch line inside `pub fn parse(&mut self)`
  dispatch="$(awk '/pub fn parse\(&mut self\)/,/^    }$/' "$WORK/$variant.rs" \
              | grep -oE 'self\.parse_[a-z_]+\(\)' | head -1)"
  alias="$(grep -oE 'pub fn parse_full_[a-z_]+' "$WORK/$variant.rs" | head -1)"
  case "$variant" in
    A_stmt_first)    expect "A (entry declared LAST in file): dispatches to"    "self.parse_scratch()" "$dispatch"
                     expect "A (entry declared LAST in file): alias"           "pub fn parse_full_scratch" "$alias" ;;
    B_scratch_first) expect "B (entry declared FIRST in file): dispatches to"  "self.parse_scratch()" "$dispatch"
                     expect "B (entry declared FIRST in file): alias"          "pub fn parse_full_scratch" "$alias" ;;
  esac
done
echo

echo "--- 2. --lint-grammar on the failing shape: does anything warn? -------------"
lint_out="$("$PIPELINE" "$WORK/A_stmt_first.ebnf" --lint-grammar 2>&1)"
lint_rc=$?
expect "lint exit code (0 = grammar declared well-formed)" "0" "$lint_rc"
unreachable="$(printf '%s' "$lint_out" | grep -oE 'unreachable_rules=[0-9]+' | head -1)"
expect "lint unreachable_rules"                            "unreachable_rules=0" "$unreachable"
# ⭐ CHANGED BY `.2` — deliberately re-pinned, not deleted. When `.1` measured this,
# `--lint-grammar` never named the entry rule (0 mentions), which is precisely why a
# mis-rooted grammar was invisible. `.2` added the report, so the honest pin is now
# "it DOES name it", and the diagnostic gap `.1` documented is closed.
names_entry="$(printf '%s' "$lint_out" | grep -cE "\[info\] entry rule" || true)"
expect "lint now NAMES the resolved entry (.2 closed .1's gap)" "1" "$names_entry"
echo

echo "--- 3. --generate-parser at DEFAULT verbosity: is the entry named? ----------"
gen_out="$("$PIPELINE" "$WORK/A_stmt_first.ebnf" --generate-parser --output "$WORK/quiet.rs" 2>&1)"
entry_visible="$(printf '%s' "$gen_out" | grep -cE 'Entry rule determined' || true)"
expect "default-verbosity generation names the entry (0 = never)" "0" "$entry_visible"
echo

echo "--- 4. --report-certificate-coverage: the instrument that DOES see it -------"
# NOT ASSERTED MECHANICALLY HERE, and the reason is itself asserted below:
# certificate-coverage verifies witnesses through the REAL generated parser, so it
# only runs for a REGISTERED grammar name. This driver deliberately probes throwaway
# grammars in a temp dir (it installs nothing and rebuilds nothing), so the derived
# grammar name is never registered. The slot-loaded measurement — which is the whole
# point of section 4 — is captured verbatim in `entry_rule_flip_capture.txt`
# alongside the behavioural flip, since both require the scratch slot to be loaded.
cert_out="$(PGEN_CERT_COVERAGE_DUMP_ALL=1 "$PIPELINE" "$WORK/A_stmt_first.ebnf" \
            --report-certificate-coverage --count 40 --seed 0 2>&1)"
refused="$(printf '%s' "$cert_out" | grep -cE 'no generated parser is registered for grammar' || true)"
expect "cert-coverage refuses an UNREGISTERED grammar name (1 = refused)" "1" "$refused"
echo "  ℹ️  the slot-loaded run (grammar name 'scratch', registered) printed, with NO flags:"
echo "         CERTIFICATE-COVERAGE: grammar='scratch' entry='stmt' ... UNKNOWN=1 fully_certified=false"
echo "         WARNING ... 1 UNKNOWN rules have NO reach path from the entry"
echo "                 (dead-rule candidates — adjudicate via the linter): [\"scratch\"]"
echo "      verbatim transcript + recipe: entry_rule_flip_capture.txt"
echo

echo "--- 5. the shipped grammars define their entry FIRST (why nothing caught it) -"
vhdl_first="$(grep -oE '^[a-z_]+ *:=' grammars/vhdl.ebnf | head -1 | tr -d ' :=')"
expect "grammars/vhdl.ebnf first-defined rule"           "vhdl_file" "$vhdl_first"
scratch_first="$(grep -oE '^[a-z_]+ *:=' grammars/scratch/scratch.ebnf | head -1 | tr -d ' :=')"
expect "committed scratch fixture first-defined rule"    "scratch"   "$scratch_first"
echo

echo "--- 6. PRIOR ART: can the entry rule be DECLARED in the EBNF at all? --------"
meta_hits="$(grep -ciE 'entry_rule|start_rule|start_symbol' grammars/ebnf.ebnf || true)"
expect "grammars/ebnf.ebnf declares an entry/start production (0 = no)" "0" "$meta_hits"
reg_hits="$(grep -ciE '"entry_rule|"start_rule|"start_symbol' rust/src/ast_pipeline/semantic_directive_registry.rs || true)"
expect "semantic_directive_registry registers an entry directive (0 = no)" "0" "$reg_hits"
echo

echo "--- 7. the charter's named FIRST suspect, refuted by measurement -------------"
# The charter blamed the FIRST-set prune guard, emitted only when
# `top_level && layout_sensitivity().terminals`, claiming "the tiny grammar
# satisfies it and VHDL does not". `layout_sensitivity()` is compiled ONLY from a
# grammar-level `@whitespace_sensitive:` directive (else all facets false,
# semantic_runtime.rs:204-217). NEITHER grammar declares it => the guard is emitted
# for NEITHER, and the suspect cannot distinguish them in either direction.
ws_vhdl="$(grep -c 'whitespace_sensitive' grammars/vhdl.ebnf || true)"
expect "grammars/vhdl.ebnf declares @whitespace_sensitive (0 = no)"      "0" "$ws_vhdl"
ws_scratch="$(grep -c 'whitespace_sensitive' grammars/scratch/scratch.ebnf || true)"
expect "committed scratch fixture declares @whitespace_sensitive (0 = no)" "0" "$ws_scratch"
ws_probe="$(grep -c 'whitespace_sensitive' "$WORK/A_stmt_first.ebnf" || true)"
expect "the failing probe grammar declares @whitespace_sensitive (0 = no)"  "0" "$ws_probe"
echo

echo "============================================================================"
echo "cases=$CASES divergences=$DIVERGENCES"
echo
echo "The BEHAVIOURAL flip (needs a probe rebuild — see entry_rule_flip_capture.txt):"
cat <<'RECIPE'
  cp <A_stmt_first.ebnf> grammars/scratch/scratch.ebnf
  make -C rust SHELL=/bin/bash focus_scratch
  touch rust/src/lib.rs
  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)
  printf 'a;a;' > /tmp/in.txt
  ./rust/target/release/parseability_probe --parse scratch /tmp/in.txt                        # rc=1, position 2
  ./rust/target/release/parseability_probe --parse scratch /tmp/in.txt --entry-rule scratch   # rc=0, ACCEPT
  git checkout grammars/scratch/scratch.ebnf   # then regenerate + rebuild
  # NOTE: `make focus_scratch` overwrites target/debug/ast_pipeline with a
  # SINGLE-feature build; rebuild it dual-feature afterwards.
RECIPE
echo
[[ "$DIVERGENCES" -eq 0 ]] || exit 1
exit 0
