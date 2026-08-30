#!/bin/bash
# ENGINE-UNIVERSAL-SERVICES.69 — reservation-vs-depth ladder.
# Compares the shipped 256 MiB default against the 96 MiB floor on identical inputs.
PROBE=rust/target/release/parseability_probe
TMO=${TMO:-90}
printf "%-8s %-12s %-6s %8s  %s\n" depth reservation rc secs verdict
for n in 200 500 1000 1500 2000 3000 3500 4000; do
  for res in 268435456 100663296; do
    s=$(date +%s.%N)
    out=$(PGEN_PARSE_STACK_BYTES=$res timeout "$TMO" "$PROBE" --parse systemverilog \
          preserved_probes/eus69/deep_$n.sv --profile sv_2017 2>&1)
    rc=$?
    e=$(date +%s.%N)
    printf "%-8s %-12s %-6s %8.2f  %s\n" "$n" "$res" "$rc" "$(echo "$e-$s"|bc)" \
      "$(echo "$out" | tr '\n' ' ' | cut -c1-110)"
  done
done
