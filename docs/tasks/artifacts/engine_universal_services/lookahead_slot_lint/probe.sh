#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.106 — the founding controls for `--lint-grammar`'s
# `template_binds_lookahead_child` class, kept as the evidence of the measurement that made it a
# gating ERROR. Each RED grammar drives two of the four structural kinds; the GREEN one is the same
# guard-opened group with the extraction CORRECTLY renumbered (`$2::3*`), which the shipped
# `list_of_type_assignments` also writes — it is the control that keeps the arm from degenerating
# into "any extraction on a guard-opened group is suspect", the rule that would fire on the tree.
# ⛔ The FOUNDING POPULATION is not copied here: it is `git show 9646dc98:grammars/systemverilog.ebnf`
# (release 1.0.229), reproducible from history, and it reads 8.
# Run: bash docs/tasks/artifacts/engine_universal_services/lookahead_slot_lint/probe.sh
set -u
cd "$(dirname "$0")/../../../../.." || exit 2
BIN=${PGEN_AST_PIPELINE_BIN:-rust/target/debug/ast_pipeline}
HERE=docs/tasks/artifacts/engine_universal_services/lookahead_slot_lint
if [ ! -x "$BIN" ]; then echo "NOT EVALUATED: $BIN is not built (needs --features \"generated_parsers ebnf_dual_run\")"; exit 0; fi
pass=0; fail=0
expect() { # expect <file> <count>
  n=$("$BIN" "$HERE/$1" --lint-grammar 2>&1 | grep -o 'template_binds_lookahead_child=[0-9]*' | head -1 | cut -d= -f2)
  if [ "${n:-x}" = "$2" ]; then pass=$((pass+1)); echo "  PASS  $1 reads $2"; else fail=$((fail+1)); echo "  FAIL  $1 reads ${n:-NOT-LOADABLE}, expected $2"; fi
}
expect red_la_slot_and_slice_child.ebnf 2
expect red_slice_arity_and_raw_guarded.ebnf 2
expect green_correctly_renumbered.ebnf 0
expect ../../../../../grammars/systemverilog.ebnf 0
echo "lookahead-slot-lint controls: ${pass}/$((pass+fail)) (fail=${fail})"
[ "$fail" -eq 0 ]
