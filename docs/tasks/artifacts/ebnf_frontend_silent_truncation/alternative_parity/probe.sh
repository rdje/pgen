#!/usr/bin/env bash
# docs/tasks/artifacts/ebnf_frontend_silent_truncation/alternative_parity/probe.sh — EBNF-FRONTEND-SILENT-TRUNCATION.1's
# bank for EBNF-ALTERNATIVE-PARITY (doctrine 36). RED arms assert the refusal/disagreement TEXT, never rc alone, and the
# founding RED is LIVE, not mutated: the `.2` shape (a column-0 comment between two alternatives) is read as 2
# alternatives by the meta-parser and as 1 by the frontend TODAY. Scratch only; nothing tracked is written.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
G=scripts/check_ebnf_alternative_parity.sh; W=rust/target/alternative_parity_probe; rm -rf "$W"; mkdir -p "$W"; trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
pass=0; fail=0; ok(){ pass=$((pass+1)); printf 'PASS  %s\n' "$1"; }; ko(){ fail=$((fail+1)); printf 'FAIL  %s\n%s\n' "$1" "${2:-}"; }
[ -x rust/target/debug/ast_pipeline ] && [ -x rust/target/debug/ebnf_dual_run_diff ] || { echo "REFUSING: both binaries are needed (ast_pipeline, ebnf_dual_run_diff)"; exit 2; }
# 1 self-test on literals: both counters, every verdict cell, the frozen pre-fix envelope, a real-arm mutation
out="$(bash $G --self-test 2>&1)"; rc=$?; n=$(printf '%s\n' "$out" | grep -c '^  ✓')
[ $rc -eq 0 ] && [ "$n" -ge 10 ] && printf '%s' "$out" | grep -q 'source 1 vs IR 4' && ok "1  self-test rc 0, $n ✓ arms incl. the frozen pre-fix parseable_source_item (source 1 vs IR 4)" || ko "1 rc=$rc n=$n" "$out"
# 2 doctrine mode on the tree: OK, and exactly the four grammars neither reader can pair are NOT EVALUATED, by name
start=$(date +%s); out="$(bash $G 2>&1)"; rc=$?; el=$(( $(date +%s) - start ))
[ $rc -eq 0 ] && printf '%s' "$out" | grep -q 'EBNF-ALTERNATIVE-PARITY: OK (grammars=14 rules=[0-9]* not_evaluated=4 \[systemverilog_2017_lrm_extracted.ebnf, systemverilog_2023_lrm_extracted.ebnf, systemverilog_lrm_profiled_wrapper.ebnf, verilog_2005_lrm_extracted.ebnf\])' && ok "2  GREEN on the tree: 14 grammars agree; NOT EVALUATED = exactly the 3 LRM snapshots + the include wrapper (${el}s)" || ko "2 rc=$rc" "$(printf '%s' "$out" | tail -2)"
[ "$el" -le 60 ] && ok "3  cost: doctrine mode took ${el}s (bound 60 s)" || ko "3 took ${el}s"
# 4 LIVE RED — the .2 shape: a column-0 comment between alternatives; the meta-parser reads 2, the frontend 1
printf 'a := b\n# a column-0 comment\n   | c\nb := "b"\nc := "c"\n' > "$W/col0.ebnf"; printf '%s\n' "$W/col0.ebnf" > "$W/l1.txt"
out="$(PGEN_ALTERNATIVE_PARITY_GRAMMARS="$W/l1.txt" bash $G 2>&1)"; rc=$?
[ $rc -eq 1 ] && printf '%s' "$out" | grep -q 'a: source 2 vs IR 1' && ok "4  LIVE RED: the .2 column-0-comment shape -> rc 1, 'a: source 2 vs IR 1' (the meta-grammar continues the rule; the frontend does not)" || ko "4 rc=$rc" "$out"
# 5 RED: a population nothing can compare REFUSES (rc 2), never passes for free
printf 'grammars/verilog_2005_lrm_extracted.ebnf\n' > "$W/l2.txt"; out="$(PGEN_ALTERNATIVE_PARITY_GRAMMARS="$W/l2.txt" bash $G 2>&1)"; rc=$?
[ $rc -eq 2 ] && printf '%s' "$out" | grep -q 'nothing was compared' && ok "5  RED: an all-NOT-EVALUATED population -> rc 2 'nothing was compared'" || ko "5 rc=$rc" "$out"
# 6 GREEN sibling on the same wiring: one readable grammar -> OK with its rule count
printf 'grammars/json.ebnf\n' > "$W/l3.txt"; out="$(PGEN_ALTERNATIVE_PARITY_GRAMMARS="$W/l3.txt" bash $G 2>&1)"; rc=$?
[ $rc -eq 0 ] && printf '%s' "$out" | grep -q 'OK (grammars=1 rules=19 not_evaluated=0)' && ok "6  GREEN sibling: json.ebnf alone -> OK grammars=1 rules=19" || ko "6 rc=$rc" "$out"
# 7 include directive: NOT EVALUATED by name, and alone it is a refusal (arms describe different rule sets)
printf 'grammars/systemverilog_lrm_profiled_wrapper.ebnf\n' > "$W/l4.txt"; out="$(PGEN_ALTERNATIVE_PARITY_GRAMMARS="$W/l4.txt" bash $G 2>&1)"; rc=$?
[ $rc -eq 2 ] && printf '%s' "$out" | grep -q 'include directive(s) 1' && ok "7  include-directive grammar -> NOT EVALUATED by name; alone it refuses (rc 2)" || ko "7 rc=$rc" "$out"
# 8 the founding defect's own grammar, TODAY: parseable_source_item reads 1 vs 1 (the .6 fix holds)
out="$(bash $G --report 2>&1 | grep 'systemverilog.ebnf')"; printf '%s' "$out" | grep -q 'agree .*systemverilog.ebnf: 1512 rules' && ok "8  GREEN: grammars/systemverilog.ebnf agrees on all 1512 rules (parseable_source_item 1 vs 1 since .6)" || ko "8" "$out"
printf 'ALTERNATIVE-PARITY: %d/%d as declared\n' "$pass" "$((pass+fail))"; [ "$fail" -eq 0 ]
