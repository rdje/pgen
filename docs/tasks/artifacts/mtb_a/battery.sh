#!/usr/bin/env bash
# MTB-A correctness battery — the .5.i.6 battery verbatim (sequential, ONE
# heavy job at a time, guard on the heavy compiles).
set -uo pipefail
S=/private/tmp/claude-501/-Users-richarddje-Documents-github-pgen/3cacb54e-9d8e-4922-b32f-a5d4069f35ef/scratchpad/mtb_a
R=/Users/richarddje/Documents/github/pgen
G="$R/scripts/run_with_memory_guard.sh --budget-mb 16384 --floor-pct 10"
overall=0

step() {
  local name="$1"; shift
  echo "=== BATTERY $name: START $(date +%H:%M:%S) ==="
  $G --timeout-s 7200 --marker "$S/bat_$name.marker" -- "$@" \
    > "$S/bat_$name.log" 2>&1
  local rc=$?
  echo "=== BATTERY $name done (exit $rc) ==="
  if [ $rc -ne 0 ]; then overall=1; echo "FAILED: $name" >> "$S/battery_failures.txt"; fi
}

# A. Dual-feature ast_pipeline rebuild (MANDATORY after focus_* — the MEMORY trap).
step dualbuild bash -c "cd '$R/rust' && cargo build --features 'generated_parsers ebnf_dual_run' --bin ast_pipeline"

# B. Debug probe rebuild at the final artifact vintage (already built this
#    session; cheap no-op rebuild keeps the vintage honest after the all-11 train).
step probe_rebuild bash -c "cd '$R/rust' && cargo build --features generated_parsers --bin parseability_probe"

# C. Full dual-feature lib suite (the 12.4GB-class compile + 956-test run;
#    includes the equivalence / combinator / semantic gate tests over the
#    fused MTB graphs).
step libsuite bash -c "cd '$R/rust' && cargo test --features 'generated_parsers ebnf_dual_run' --lib"

# D. Regex cert seeds 0/7/42 (spf pins 0/1/1; 267/9/258/0).
step cert bash -c "cd '$R/rust' && for s in 0 7 42; do ./target/debug/ast_pipeline ../grammars/regex.ebnf --report-certificate-coverage --entry-rule regex --count 40 --seed \$s 2>&1 | grep CERTIFICATE-COVERAGE; done"

# E–H. The make gates.
step ast_shape_contract_gate make -C "$R/rust" SHELL=/bin/bash ast_shape_contract_gate
step duality_hunt_gate make -C "$R/rust" SHELL=/bin/bash duality_hunt_gate
step regex_pcre2_compile_oracle_gate make -C "$R/rust" SHELL=/bin/bash regex_pcre2_compile_oracle_gate
step clippy_on_rust_change make -C "$R/rust" SHELL=/bin/bash clippy_on_rust_change

echo "BATTERY COMPLETE overall=$overall"
exit $overall
