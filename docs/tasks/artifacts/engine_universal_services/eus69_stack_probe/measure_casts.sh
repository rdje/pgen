#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.69`(d) — the reading at a REAL REJECTION.
# Nested casts are the shape `.61`(a)'s shipped label was captured on (4 814
# casts = 81 675 frames on a 256 MiB stack), so they reach the ceiling where
# nested parens only approach it.
PROBE=rust/target/release/parseability_probe
TMO=${TMO:-600}
for n in "$@"; do
  for res in 268435456 100663296; do
    s=$(date +%s.%N)
    err=$(PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES=$res timeout "$TMO" "$PROBE" \
          --parse systemverilog preserved_probes/eus69/cast_$n.sv --profile sv_2017 2>&1 >/dev/null)
    rc=$?
    e=$(date +%s.%N)
    printf "casts=%-6s res=%-11s rc=%-4s %8.2fs\n    %s\n" "$n" "$res" "$rc" \
      "$(echo "$e-$s"|bc)" "$(echo "$err" | grep -o 'stack_probe:.*' | head -1)"
    printf "    label: %s\n" "$(echo "$err" | grep -o 'recursion ceiling CONTACTED[^]]*' | head -1 | cut -c1-150)"
  done
done
