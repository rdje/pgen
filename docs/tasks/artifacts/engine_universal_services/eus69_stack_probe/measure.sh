#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.69`(d) — what the depth probe actually reads at a
# refusal on REAL input, under two reservations.
#
# `.71` could only publish a per-frame cost as ARITHMETIC over
# `reservation ÷ frames` (an upper bound, <= 3.008 KB). This measures it.
PROBE=rust/target/release/parseability_probe
TMO=${TMO:-400}
printf "%-7s %-12s %-4s %8s  %s\n" depth reservation rc secs stack_probe
for n in 1000 2000 2400 2600 3000; do
  for res in 268435456 100663296; do
    s=$(date +%s.%N)
    err=$(PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES=$res timeout "$TMO" "$PROBE" \
          --parse systemverilog preserved_probes/eus69/deep_$n.sv --profile sv_2017 2>&1 >/dev/null)
    rc=$?
    e=$(date +%s.%N)
    printf "%-7s %-12s %-4s %8.2f  %s\n" "$n" "$res" "$rc" "$(echo "$e-$s"|bc)" \
      "$(echo "$err" | grep -o 'stack_probe:.*' | head -1)"
  done
done
