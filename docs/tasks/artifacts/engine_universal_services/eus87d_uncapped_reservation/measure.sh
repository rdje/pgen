#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.87`(d) — the reading at the DEFAULT reservation and at a
# pinned one, on nested `int'(...)` casts.
#
#   bash docs/tasks/artifacts/engine_universal_services/eus87d_uncapped_reservation/measure.sh 4814 7000 7600
#
# RES= a byte count pins `PGEN_PARSE_STACK_BYTES`; RES=default leaves it unset, which is
# the arm that actually proves (d) — the capability must arrive with no operator action.
PROBE=${PROBE:-rust/target/release/parseability_probe}
TMO=${TMO:-900}
RES=${RES:-default}
DIR=${DIR:-preserved_probes/eus87d}
for n in "$@"; do
  s=$(date +%s.%N)
  if [ "$RES" = default ]; then
    err=$(PGEN_REPORT_STACK_PROBE=1 timeout "$TMO" "$PROBE" \
          --parse systemverilog "$DIR/cast_$n.sv" --profile sv_2017 2>&1 >/dev/null)
  else
    err=$(PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES="$RES" timeout "$TMO" "$PROBE" \
          --parse systemverilog "$DIR/cast_$n.sv" --profile sv_2017 2>&1 >/dev/null)
  fi
  rc=$?
  e=$(date +%s.%N)
  printf "casts=%-6s res=%-11s rc=%-4s %8.2fs\n    %s\n" "$n" "$RES" "$rc" \
    "$(echo "$e-$s"|bc)" "$(echo "$err" | grep -o 'stack_probe:.*' | head -1)"
done
