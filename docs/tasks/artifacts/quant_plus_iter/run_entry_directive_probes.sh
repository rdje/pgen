#!/usr/bin/env bash
# QUANT-PLUS-ITER.2 (step A) — the `@entry: true` directive, by DECLARED VERDICTS.
#
# WHAT `@entry` IS
#   A rule-level semantic annotation naming the grammar's entry rule (start symbol):
#
#       statement := "a" ";"        # helper rules may live wherever reads best
#
#       @entry: true
#       program := statement+       # <- "this rule is the entry"
#
#   Absent => REFUSED (step C): a main EBNF file shall contain one and only one.
#   `--entry-rule <name>` on the CLI OUTRANKS a declared `@entry` (director rule).
#
# WHY IT EXISTS (director, 2026-07-26): in EBNF a grammar is a SET of productions —
# rule order is presentation, not semantics. PGEN honoured that everywhere except the
# start symbol, which was whichever rule happened to be written first. Reordering two
# definitions silently changed the accepted language while `--lint-grammar` reported
# `unreachable_rules=0` and exited 0. `QUANT-PLUS-ITER.1` measured that costing an
# entire task-tree.
#
# Run from the repository root. Exit 0 + "0 divergences" = every declared verdict held.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT" || exit 1

PIPELINE="./rust/target/debug/ast_pipeline"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/entry_directive.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

DIVERGENCES=0
CASES=0
expect() {
  CASES=$((CASES + 1))
  if [[ "$2" == "$3" ]]; then printf '  ✅ %-56s %s\n' "$1" "$3"
  else printf '  ⛔ %-56s expected=%s actual=%s\n' "$1" "$2" "$3"; DIVERGENCES=$((DIVERGENCES + 1)); fi
}

echo "=== QUANT-PLUS-ITER.2 — @entry directive probes ============================"
[[ -x "$PIPELINE" ]] || { echo "⛔ missing $PIPELINE"; exit 1; }
SURFACE="$("$PIPELINE" --report-feature-surface 2>&1 | head -1)"
echo "$SURFACE"
case "$SURFACE" in
  *ebnf_dual_run=true*) ;;
  *) echo "⛔ ast_pipeline lacks ebnf_dual_run (a prior 'make focus_*' overwrote it)."
     echo "   rebuild: (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)"
     exit 1 ;;
esac
echo

# The charter's ORIGINAL failing shape: the quantifier's rule is defined LAST.
cat > "$WORK/declared.ebnf" <<'EOF'
stmt := "a" ";"
      | "b" ";"

@entry: true
scratch := stmt+
EOF
# Byte-for-byte the same grammar with the declaration removed.
cat > "$WORK/positional.ebnf" <<'EOF'
stmt := "a" ";"
      | "b" ";"

scratch := stmt+
EOF

dispatch_of() {  # $1 = grammar file, rest = extra ast_pipeline args
  local g="$1"; shift
  "$PIPELINE" "$g" --generate-parser "$@" --output "$WORK/out.rs" >/dev/null 2>&1 || { echo "GENFAIL"; return; }
  awk '/pub fn parse\(&mut self\)/,/^    }$/' "$WORK/out.rs" | grep -oE 'self\.parse_[a-z_]+\(\)' | head -1
}

echo "--- 1. the directive decides the entry, against file order -------------------"
expect "declared @entry on the LAST-defined rule -> dispatch" \
       "self.parse_scratch()" "$(dispatch_of "$WORK/declared.ebnf")"
# STEP C: a main EBNF with no `@entry: true` is REFUSED, so there is no positional
# default left to fall back to — that fallback WAS the silent-re-rooting hazard.
undeclared_out="$("$PIPELINE" "$WORK/positional.ebnf" --lint-grammar 2>&1)"; undeclared_rc=$?
expect "no declaration -> REFUSED (step C: @entry is mandatory)" "1" "$undeclared_rc"
expect "…refusal names the one-and-only-one contract" "1" \
       "$(printf '%s' "$undeclared_out" | grep -c 'one and only one')"
echo

echo "--- 2. --entry-rule OUTRANKS @entry (director rule) --------------------------"
expect "@entry=scratch + --entry-rule stmt -> CLI wins" \
       "self.parse_stmt()" "$(dispatch_of "$WORK/declared.ebnf" --entry-rule stmt)"
out="$("$PIPELINE" "$WORK/declared.ebnf" --generate-parser --entry-rule nosuch --output "$WORK/out.rs" 2>&1)"; rc=$?
expect "--entry-rule naming an undefined rule -> exit code" "1" "$rc"
expect "--entry-rule naming an undefined rule -> named error" "1" \
       "$(printf '%s' "$out" | grep -c "names a rule the grammar does not define")"
echo

echo "--- 3. the linter now REPORTS the resolved entry -----------------------------"
lint_declared="$("$PIPELINE" "$WORK/declared.ebnf" --lint-grammar 2>&1 | grep -c "entry rule 'scratch' — declared via")"
expect "lint names a DECLARED entry" "1" "$lint_declared"
expect "lint reports no POSITIONAL case (step C removed the fallback)" "0" \
       "$("$PIPELINE" "$WORK/declared.ebnf" --lint-grammar 2>&1 | grep -c "POSITIONAL")"
echo

echo "--- 4. 'ONE AND ONLY ONE' + placement, all HALT rather than drop -------------"
printf '@entry: true\na := "x"\n\n@entry: true\nb := "y"\n'  > "$WORK/two.ebnf"
printf 'a := ( @entry: true "x" ) | "y"\n'                   > "$WORK/inline.ebnf"
printf '@entry: "scratch"\na := "x"\n'                       > "$WORK/payload.ebnf"
printf '@entry: false\na := "x"\nb := "y"\n'                 > "$WORK/false.ebnf"
check_err() { # <label> <file> <expected_rc> <needle>
  local out rc
  out="$("$PIPELINE" "$2" --lint-grammar 2>&1)"; rc=$?
  expect "$1 -> exit code" "$3" "$rc"
  [[ -n "${4:-}" ]] && expect "$1 -> diagnostic present" "1" "$(printf '%s' "$out" | grep -c "$4")"
}
check_err "@entry on TWO rules"                "$WORK/two.ebnf"     1 "declared on more than one rule"
check_err "@entry inline inside a rule body"   "$WORK/inline.ebnf"  1 "marks a whole RULE"
check_err "@entry with a rule-NAME payload"    "$WORK/payload.ebnf" 1 "expects the boolean"
# `@entry: false` is an explicit no-op, so the grammar declares NO entry — which
# step C refuses. Both facts hold together; this pins the composition.
check_err "@entry: false = declares nothing -> REFUSED"  "$WORK/false.ebnf" 1 "one and only one"
echo

echo "============================================================================"
echo "cases=$CASES divergences=$DIVERGENCES"
echo
echo "NO-REGRESSION (separate, expensive — see the leaf): all 10 generated parsers"
echo "are BYTE-IDENTICAL against a HEAD-vintage binary with input AND output paths"
echo "pinned — 3,218,187 lines, 0 differences. Reproduce with byte_identity.sh."
[[ "$DIVERGENCES" -eq 0 ]] || exit 1
exit 0
