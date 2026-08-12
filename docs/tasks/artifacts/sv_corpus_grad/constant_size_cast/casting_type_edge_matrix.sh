#!/usr/bin/env bash
# `SV-CORPUS-GRAD.13c.2b` — the five-arm A/B that pins the defect to ONE grammar edge.
#
# Every arm is a tracked reproducer under `stimuli/sv/adjudication_repros/`, so the matrix is
# re-runnable and the inputs cannot drift away from the claim.
#
#   arm                                        casting_type resolves via        verdict
#   -----------------------------------------  ------------------------------  -------
#   control_size_cast_in_statement.sv          constant_primary (FIRST entry)  ACCEPT
#   control_simple_type_cast_in_constant.sv    simple_type (branch 1/5)        ACCEPT
#   control_param_name_cast_in_constant.sv     simple_type -> ps_type_ident    ACCEPT
#   defect_constant_size_cast.sv               constant_primary — CYCLE-BLOCKED REJECT
#   defect_constant_size_cast_corpus_shape.sv  constant_primary — CYCLE-BLOCKED REJECT
#
# ⇒ the discriminator is not "constant expression" and not "size cast": it is whether casting_type
# has a viable alternative OTHER than `constant_primary` at the seed position. The cycle guard
# (`💥 Infinite recursion detected in rule 'constant_primary'`) fires in the passing arms too and
# costs nothing there — see the leaf.
#
# Usage:  bash docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/casting_type_edge_matrix.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && while [ ! -f CLAUDE.md ] || [ ! -d grammars ]; do cd .. || exit 1; done; pwd)"
PROBE="$ROOT/rust/target/release/parseability_probe"
REPROS="$ROOT/stimuli/sv/adjudication_repros"

if [ ! -x "$PROBE" ]; then
  echo "REFUSE: $PROBE is missing — build it with:" >&2
  echo "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)" >&2
  exit 2
fi

status=0
run_arm() {
  local file="$1" expect="$2" note="$3" out verdict
  out="$("$PROBE" --parse systemverilog "$REPROS/$file" --profile sv_2017 2>&1 | tail -1)"
  case "$out" in
    parse_full\ passed*) verdict=ACCEPT ;;
    *) verdict="REJECT ${out##*[}" ; verdict="${verdict%]*}" ;;
  esac
  printf '%-44s %-8s expect=%-6s %s\n' "$file" "${verdict%% *}" "$expect" "$note"
  [ "${verdict%% *}" = "$expect" ] || { echo "  ⛔ MISMATCH: $out"; status=1; }
}

run_arm control_size_cast_in_statement.sv         ACCEPT "8'(1) in a STATEMENT expression"
run_arm control_simple_type_cast_in_constant.sv   ACCEPT "int'(1) — casting_type = simple_type"
run_arm control_param_name_cast_in_constant.sv    ACCEPT "W'(1)  — casting_type = ps_type_identifier"
run_arm defect_constant_size_cast.sv              REJECT "8'(1) in a CONSTANT expression"
run_arm defect_constant_size_cast_corpus_shape.sv REJECT "512'({…}) — the OpenTitan corpus shape"

echo
if [ "$status" = 0 ]; then
  echo "VERDICT: matrix reproduces — 3 ACCEPT / 2 REJECT, exactly as adjudicated."
else
  echo "VERDICT: an arm moved. If the two REJECT arms now ACCEPT, ENGINE-UNIVERSAL-SERVICES.13"
  echo "         landed and this leaf plus the MANIFEST expectations must be re-baselined."
fi
exit "$status"
