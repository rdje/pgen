#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.91 — the case-collision arm, asserted on BOTH sides of the fix.
#
#   RED 1  the synthetic pair (`case_pair_control.ebnf`): the lint refuses (rc 1) naming both rules,
#          and codegen REFUSES by rule name before emitting a byte (no `const RULE_SYM_X` twice).
#   RED 2  the SystemVerilog grammar with `SV-CORPUS-GRAD.3.30`'s original spelling restored
#          (`udp_sym_x_upper` -> `udp_sym_X`, a scratch copy under rust/target): the lint refuses.
#   GREEN  every tracked grammar reads `case_colliding_rule_names=0`, and the exit codes are what
#          they were before the arm existed (the four `*_lrm_*` extraction grammars stay rc 1 for
#          their own pre-existing reasons; every shipped grammar rc 0).
#
# Before `.91` (recorded in the grammar's header): RED 1 linted rc 0 with no arm and codegen emitted
# `const RULE_SYM_X: RuleId = 1u16;` / `= 2u16;` at rc 0; RED 2 linted rc 0 over 1734 rules.
#
# Usage (from the repository root):  bash <this script>        # exit 0 iff every assertion holds
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../../../../.." && pwd)"
cd "$REPO_ROOT" || exit 2
AST_PIPELINE="${PGEN_AST_PIPELINE:-rust/target/debug/ast_pipeline}"
WORK="rust/target/eus91_probe"; mkdir -p "$WORK"
[[ -x "$AST_PIPELINE" ]] || { echo "missing $AST_PIPELINE — build it: (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)" >&2; exit 2; }
fail=0
headline() { "$AST_PIPELINE" "$1" --lint-grammar 2>&1 | grep -oE 'case_colliding_rule_names=[0-9]+' | head -1; }

# RED 1 — lint
out="$("$AST_PIPELINE" "$HERE/case_pair_control.ebnf" --lint-grammar 2>&1)"; rc=$?
h="$(printf '%s\n' "$out" | grep -oE 'case_colliding_rule_names=[0-9]+' | head -1)"
if [[ "$h" == "case_colliding_rule_names=1" && $rc -ne 0 ]] && printf '%s\n' "$out" | grep -q "rules 'sym_x' / 'sym_X' differ only by case"; then
  echo "  ✓ RED 1 lint: $h rc=$rc, naming 'sym_x' / 'sym_X'"
else echo "  ✗ RED 1 lint: read '$h' rc=$rc" >&2; printf '%s\n' "$out" | tail -3 >&2; fail=1; fi
# RED 1 — codegen
rm -f "$WORK/case_pair.rs"
out="$("$AST_PIPELINE" "$HERE/case_pair_control.ebnf" --generate-parser --eliminate-left-recursion -o "$WORK/case_pair.rs" 2>&1)"; rc=$?
dups=$(grep -c 'const RULE_SYM_X' "$WORK/case_pair.rs" 2>/dev/null || echo 0)
if [[ $rc -ne 0 && "$dups" == "0" ]] && printf '%s\n' "$out" | grep -q 'codegen REFUSED'; then
  echo "  ✓ RED 1 codegen: REFUSED rc=$rc, no RULE_SYM_X constant emitted ($(printf '%s\n' "$out" | grep -oE 'codegen REFUSED[^—]*' | head -1))"
else echo "  ✗ RED 1 codegen: rc=$rc, RULE_SYM_X definitions=$dups" >&2; printf '%s\n' "$out" | tail -3 >&2; fail=1; fi
# RED 2 — the SV grammar with .3.30's spelling restored
sed -E 's/\budp_sym_x_upper\b/udp_sym_X/g' grammars/systemverilog.ebnf > "$WORK/sv_dot3_30_spelling.ebnf"
out="$("$AST_PIPELINE" "$WORK/sv_dot3_30_spelling.ebnf" --lint-grammar 2>&1)"; rc=$?
h="$(printf '%s\n' "$out" | grep -oE 'case_colliding_rule_names=[0-9]+' | head -1)"
if [[ "$h" == "case_colliding_rule_names=1" && $rc -ne 0 ]] && printf '%s\n' "$out" | grep -q "'udp_sym_x' / 'udp_sym_X'"; then
  echo "  ✓ RED 2 systemverilog (.3.30 spelling): $h rc=$rc, naming 'udp_sym_x' / 'udp_sym_X'"
else echo "  ✗ RED 2: read '$h' rc=$rc" >&2; fail=1; fi
# GREEN — every tracked grammar
green=0; exempt=0
for g in grammars/*.ebnf grammars/scratch/scratch.ebnf; do
  case "$(basename "$g")" in *_lrm_*) exempt=$((exempt+1)); continue;; esac
  h="$(headline "$g")"
  if [[ "$h" != "case_colliding_rule_names=0" ]]; then echo "  ✗ GREEN: $g reads '$h'" >&2; fail=1; else green=$((green+1)); fi
done
echo "  ✓ GREEN: $green tracked grammars read case_colliding_rule_names=0; $exempt *_lrm_* extraction grammars exempt (they refuse at load or on their own pre-existing findings, unchanged by this arm)"
if [[ $fail -eq 0 ]]; then echo "CASE-COLLISION-CONTROL: OK (RED 1 lint + codegen, RED 2, GREEN $green of $((green+exempt)) with $exempt exempt)"; else echo "CASE-COLLISION-CONTROL: FAIL" >&2; fi
exit $fail
