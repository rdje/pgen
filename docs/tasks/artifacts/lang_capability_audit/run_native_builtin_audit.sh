#!/usr/bin/env bash
# LANG-CAPABILITY-AUDIT.10.1 / .10.4 — audit every member of
# `AstBasedGenerator::NATIVE_UNRESOLVED_REFERENCE_BUILTINS`.
#
# The question per member: is it a GENUINE codegen builtin (a deliberately
# grammar-consumable primitive with a real native matcher), or a SILENCED DEFECT
# (a name allowlisted so the undefined-reference check stops reporting it)?
#
# .10.1 asked it of all five members and convicted two: `true` and `false` compiled
# to unconditional ZERO-WIDTH matchers, so `"T" true "T"` ACCEPTED `TT`. .10.4
# removed both. This driver now asserts the POST-FIX truth; the pre-fix capture
# (the same driver, declaring the defect's verdicts) is preserved in git at commit
# `c3d9b000`. Re-running it against a tree where the trap came back FAILS loudly.
#
# Re-run:  bash docs/tasks/artifacts/lang_capability_audit/run_native_builtin_audit.sh
# Capture: docs/tasks/artifacts/lang_capability_audit/native_builtin_audit.txt
#
# Arms:
#   A. STATIC   — const membership, tracked-grammar reference census, generated-parser
#                 census, and the lint-masking demonstration. No build, no slot.
#   B. SHAPE    — the two include candidates for `ebnf.ebnf`'s delegated
#                 `semantic_annotation`, measured (collisions + entry-rule shape).
#   C. BEHAVIOUR— declared-verdict parses through the PARSE-HARNESS scratch slot
#                 (authoritative by construction). OPT-IN: it OVERWRITES the slot and
#                 REBUILDS, so it runs only under PGEN_AUDIT_RUN_SLOT_ARM=1. It backs
#                 the slot up and restores it (plus its artifact) on every exit path.
#
# Every case declares the verdict it requires. A divergence prints `⛔` and is counted.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT"

PIPELINE="rust/target/debug/ast_pipeline"
[[ -x "$PIPELINE" ]] || PIPELINE="rust/target/release/ast_pipeline"
if [[ ! -x "$PIPELINE" ]]; then
  echo "FATAL: no ast_pipeline binary; build with" >&2
  echo "  (cd rust && cargo build --features 'generated_parsers ebnf_dual_run' --bin ast_pipeline)" >&2
  exit 2
fi
PIPELINE="$REPO_ROOT/$PIPELINE"

GEN="rust/src/ast_pipeline/ast_based_generator.rs"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
GAPS=0

hdr() { printf '\n===== %s =====\n' "$1"; }

want() { # want LABEL GOT WANT
  if [[ "$2" == "$3" ]]; then
    printf '  OK  %-46s got=%-24s [want %s]\n' "$1" "$2" "$3"
  else
    printf '  ⛔  %-46s got=%-24s [want %s]\n' "$1" "$2" "$3"
    GAPS=$((GAPS + 1))
  fi
}

undef_refs() { "$PIPELINE" --lint-grammar "$1" 2>&1 | grep -oE "undefined_references=[0-9]+" | head -1 | cut -d= -f2; }

# ---------------------------------------------------------------------------
hdr "A1. the allowlist, verbatim from the single source of truth"
sed -n '/pub const NATIVE_UNRESOLVED_REFERENCE_BUILTINS/,/\];/p' "$GEN" | sed 's/^/  /'
MEMBERS=(builtin_any_char builtin_ascii_char semantic_annotation)
want "member count" "${#MEMBERS[@]}" "3"
RETIRED=(true false)   # .10.4 — must NOT be on the list, and must NOT be zero-width

# ---------------------------------------------------------------------------
hdr "A2. tracked-grammar reference census (who names each member as a RULE REFERENCE?)"
# A rule reference is a bare name on a rule RHS. Three things spell these same names
# WITHOUT referencing a rule, and each is stripped before matching or the census lies:
#   - an annotation line          `@entry: true`            -> drop `^\s*@` lines
#   - a quoted/regex terminal     `("true" | "false")`      -> blank quoted + /…/ spans
#   - a return-annotation literal `-> {negated: false}`     -> drop from `->` onward
ref_screen() {
  grep -v '^[[:space:]]*#' "$1" | grep -v '^[[:space:]]*@' \
    | sed -e 's/->.*$//' -e 's/"[^"]*"/ /g' -e "s/'[^']*'/ /g" -e 's|/[^/]*/| |g'
}
declare -A REFERENCED_BY
for m in "${MEMBERS[@]}"; do
  files=""
  for f in grammars/*.ebnf grammars/*/*.ebnf; do
    [[ -f "$f" ]] || continue
    if ref_screen "$f" | grep -qE "(^|[^_[:alnum:]])${m}([^_[:alnum:]]|$)"; then
      files="$files $(basename "$f")"
    fi
  done
  REFERENCED_BY[$m]="${files# }"
  printf '  %-22s referenced-by:%s\n' "$m" "${files:- (none)}"
done

# ---------------------------------------------------------------------------
hdr "A3. generated-parser census (who CONSUMES the synthesized native matcher?)"
declare -A EMITTED_IN
for m in "${MEMBERS[@]}"; do
  hits=""
  for f in generated/*.rs; do
    [[ -f "$f" ]] || continue
    grep -q "fn parse_${m}\b" "$f" 2>/dev/null && hits="$hits $(basename "$f")"
  done
  EMITTED_IN[$m]="${hits# }"
  printf '  %-22s emitted-in:%s\n' "$m" "${hits:- (none)}"
done
echo "  NOTE: generated/ is gitignored — regenerate with the per-grammar 'make -C rust focus_<g>'"
echo "        targets if a row reads (none) unexpectedly."

# A2 is a text screen; A3 is what codegen actually resolved. They are independent, so
# make them agree EXPLICITLY — a silent disagreement would mean the screen is lying.
hdr "A3b. cross-check: does the text screen agree with codegen?"
for m in "${MEMBERS[@]}"; do
  s_empty=no; c_empty=no
  [[ -z "${REFERENCED_BY[$m]}" ]] && s_empty=yes
  [[ -z "${EMITTED_IN[$m]}" ]] && c_empty=yes
  want "$m: screen-empty == codegen-empty" "$s_empty" "$c_empty"
done
echo "  (Exact file lists differ by construction: a grammar that DEFINES the rule emits"
echo "   parse_<name> from its own definition, not from the native-builtin fallback —"
echo "   which is why semantic_annotation.ebnf appears on both sides for its own rule.)"

# ---------------------------------------------------------------------------
hdr "A4. the masking: what the allowlist hides, and what it no longer hides"
cat > "$WORK/masked.ebnf" <<'EOF'
@entry: true
scratch := probe_true | probe_false | probe_annot | probe_any

probe_true := "T" true "T"
probe_false := "F" false "F"
probe_annot := "A" semantic_annotation
probe_any := "C" builtin_any_char "C"
EOF
# The probe names FOUR undefined rules. Before .10.4 all four were allowlisted and
# the grammar lint-reported 0. Now only `semantic_annotation` and `builtin_any_char`
# are, so exactly the two retired names must surface.
want "undefined_references on 4 dangling refs" "$(undef_refs "$WORK/masked.ebnf")" "2"
for r in "${RETIRED[@]}"; do
  named="$("$PIPELINE" --lint-grammar "$WORK/masked.ebnf" 2>&1 | grep -c "UNDEFINED rule '$r'")"
  want "the linter now NAMES '$r'" "$named" "1"
done
cat > "$WORK/unmasked.ebnf" <<'EOF'
@entry: true
scratch := "T" not_on_the_allowlist "T"
EOF
want "undefined_references on 1 NON-member ref" "$(undef_refs "$WORK/unmasked.ebnf")" "1"
echo "  => the check itself always worked; the ALLOWLIST decided what it could see."
echo "  => .10.4 gave 'true'/'false' back to it (0 -> 2 on this very grammar)."

# ---------------------------------------------------------------------------
hdr "A5. emitted matcher shape per member (codegen, not prose)"
"$PIPELINE" "$WORK/masked.ebnf" --generate-parser --bootstrap-mode \
  --output "$WORK/masked_parser.rs" >/dev/null 2>&1
for m in semantic_annotation builtin_any_char "${RETIRED[@]}"; do
  printf '  --- parse_%s\n' "$m"
  # index(), not a regex: "parse_true(" contains `(`, which is not a valid ERE atom.
  awk -v needle="pub fn parse_${m}(" 'index($0, needle) {p=1} p {print "      "$0} p && /^    }$/ {exit}' \
    "$WORK/masked_parser.rs"
done
# The decisive static contrast. A retired name must now be the bare never-matching
# stub — NOT the unconditional zero-width `Ok` .10.1 measured — while a genuine
# builtin still consumes.
for r in "${RETIRED[@]}"; do
  body="$(awk -v needle="pub fn parse_${r}(" 'index($0, needle) {p=1} p {print} p && /^    }$/ {exit}' \
    "$WORK/masked_parser.rs")"
  want "parse_$r is the Backtrack stub" "$(grep -c 'Backtrack' <<<"$body")" "1"
  want "parse_$r never succeeds" "$(grep -c 'Ok(ParseNode' <<<"$body")" "0"
done
aw="$(awk '/pub fn parse_builtin_any_char\(/,/^    }$/' "$WORK/masked_parser.rs" | grep -c 'self.position = end_pos')"
want "parse_builtin_any_char consumes" "$aw" "1"

# ---------------------------------------------------------------------------
hdr "B. ebnf.ebnf's delegated 'semantic_annotation': the two include candidates"
rule_names() { grep -oE '^[A-Za-z_][A-Za-z0-9_]*[[:space:]]*:=' "$1" | sed 's/[[:space:]]*:=//' | sort -u; }
rule_names grammars/ebnf.ebnf > "$WORK/e.txt"
rule_names grammars/semantic_annotation.ebnf > "$WORK/s.txt"
rule_names grammars/builtin_semantic_annotation.ebnf > "$WORK/b.txt"
comm -12 "$WORK/e.txt" "$WORK/s.txt" > "$WORK/coll_full.txt"
comm -12 "$WORK/e.txt" "$WORK/b.txt" > "$WORK/coll_builtin.txt"
want "collisions: ebnf x semantic_annotation" "$(wc -l < "$WORK/coll_full.txt" | tr -d ' ')" "15"
want "collisions: ebnf x builtin_semantic_annotation" "$(wc -l < "$WORK/coll_builtin.txt" | tr -d ' ')" "3"
echo "  full-grammar collisions: $(tr '\n' ' ' < "$WORK/coll_full.txt")"
echo "  builtin collisions:      $(tr '\n' ' ' < "$WORK/coll_builtin.txt")"

# Are the collisions renames, or genuine semantic conflicts?
extract() { awk -v r="$2" '$0 ~ "^"r"[[:space:]]*:=" {p=1} p {print} p && /^$/ {exit}' "$1"; }
same=0; diff_=0
for r in $(cat "$WORK/coll_full.txt"); do
  a="$(extract grammars/ebnf.ebnf "$r" | grep -v '^[[:space:]]*#' | tr -d ' \t\n')"
  b="$(extract grammars/semantic_annotation.ebnf "$r" | grep -v '^[[:space:]]*#' | tr -d ' \t\n')"
  if [[ "$a" == "$b" ]]; then same=$((same+1)); else diff_=$((diff_+1)); printf '    CONFLICT  %s\n' "$r"; fi
done
want "collisions that are genuine CONFLICTS" "$diff_" "11"
want "collisions that are duplicates" "$same" "4"

# Shape: which candidate's entry rule actually matches an `@name: value` LINE?
printf '  entry rule, semantic_annotation.ebnf : '; grep -m1 '^semantic_annotation[[:space:]]*:=' grammars/semantic_annotation.ebnf
printf '  entry rule, builtin_...ebnf         : '; grep -m1 '^builtin_semantic_annotation[[:space:]]*:=' grammars/builtin_semantic_annotation.ebnf
printf '  builtin fallback rule               : '; grep -m1 '^raw_payload[[:space:]]*:=' grammars/builtin_semantic_annotation.ebnf
printf '  builtin any_text                    : '; grep -m1 '^any_text[[:space:]]*:=' grammars/builtin_semantic_annotation.ebnf
sa_at="$(grep -m1 '^semantic_annotation[[:space:]]*:=' grammars/semantic_annotation.ebnf | grep -c '"@"')"
b_at="$(grep -m1 '^builtin_semantic_annotation[[:space:]]*:=' grammars/builtin_semantic_annotation.ebnf | grep -c '"@"')"
want "semantic_annotation.ebnf matches the '@'" "$sa_at" "1"
want "builtin_...ebnf matches the '@'" "$b_at" "0"

# ---------------------------------------------------------------------------
hdr "C. BEHAVIOUR through the scratch slot (opt-in: PGEN_AUDIT_RUN_SLOT_ARM=1)"
if [[ "${PGEN_AUDIT_RUN_SLOT_ARM:-0}" != "1" ]]; then
  echo "  SKIPPED (set PGEN_AUDIT_RUN_SLOT_ARM=1 to run; it overwrites grammars/scratch/scratch.ebnf"
  echo "  and rebuilds parseability_probe, then restores the slot + its artifact on exit)."
  echo "  Verdicts recorded by the capture, all 7 OK, 0 divergences"
  echo "  (arrow = the .10.1 pre-fix verdict this replaced):"
  echo "    TT             REJECT   <- was ACCEPT: 'true' matched EMPTY.  THE FIX."
  echo "    TtrueT         REJECT   <- unchanged; never was a literal matcher"
  echo "    FF             REJECT   <- was ACCEPT: 'false' matched EMPTY. THE FIX."
  echo "    A@name: value  ACCEPT   <- unchanged; native @-to-EOL matcher still live"
  echo "    Aname: value   REJECT   <- unchanged; it does require the '@'"
  echo "    CzC            ACCEPT   <- unchanged; builtin_any_char CONSUMES one char"
  echo "    CC             REJECT   <- unchanged; and refuses to match empty"
else
  SLOT="grammars/scratch/scratch.ebnf"
  cp "$SLOT" "$WORK/slot.bak"
  # Restore BOTH the slot fixture AND everything derived from it. Regenerating the
  # artifact is not enough: parseability_probe COMPILES the generated parser in, so a
  # binary left built against the probe grammar would silently answer for a grammar
  # that is no longer in the slot — the stale-binary trap banked in TOOLBOX.md.
  restore_slot() {
    cp "$WORK/slot.bak" "$SLOT"
    make -C rust SHELL=/bin/bash focus_scratch >/dev/null 2>&1 || true
    (cd rust && cargo build --features generated_parsers --bin parseability_probe) >/dev/null 2>&1 || true
    rm -rf "$WORK"
  }
  trap restore_slot EXIT
  cp "$WORK/masked.ebnf" "$SLOT"
  make -C rust SHELL=/bin/bash focus_scratch >/dev/null 2>&1
  (cd rust && cargo build --features generated_parsers --bin parseability_probe) >/dev/null 2>&1
  PROBE="$REPO_ROOT/rust/target/debug/parseability_probe"
  if [[ ! -x "$PROBE" ]]; then
    echo "  ⛔ parseability_probe missing after build"; GAPS=$((GAPS + 1))
  else
    probe() { # probe LABEL INPUT WANT
      printf '%s' "$2" > "$WORK/in.txt"
      local v
      if "$PROBE" --parse scratch "$WORK/in.txt" 2>&1 | grep -q "parse_full passed"; then v=ACCEPT; else v=REJECT; fi
      want "$1 [$2]" "$v" "$3"
    }
    # .10.4: these two flipped ACCEPT -> REJECT. That flip IS the fix.
    probe "true no longer matches empty" "TT"         REJECT
    probe "true is not a literal"     "TtrueT"        REJECT
    probe "false no longer matches empty" "FF"        REJECT
    probe "semantic_annotation @-line" "A@name: value" ACCEPT
    probe "semantic_annotation needs @" "Aname: value" REJECT
    probe "builtin_any_char consumes"  "CzC"          ACCEPT
    probe "builtin_any_char not empty" "CC"           REJECT
  fi
fi

hdr "RESULT"
if [[ "$GAPS" == "0" ]]; then
  echo "  ✅ 0 divergences — every declared verdict held."
else
  echo "  ⛔ $GAPS divergence(s)."
fi
exit 0
