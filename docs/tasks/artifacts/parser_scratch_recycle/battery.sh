#!/usr/bin/env bash
# PGEN-RGX-0078-0209 correctness battery — the standing land protocol (the
# `-0202` shape). Sequential, ONE heavy job at a time, every step
# memory-guarded at 16384 MB.
set -uo pipefail
R=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)
S=${R0209_SCRATCH:?set R0209_SCRATCH to the session scratch dir}
G="$R/scripts/run_with_memory_guard.sh --budget-mb 16384 --floor-pct 10 --timeout-s 7200"
overall=0
rm -f "$S/battery_failures.txt"

step() {
  local name="$1"; shift
  echo "=== BATTERY $name: START $(date +%H:%M:%S) ==="
  $G --marker "$S/bat_$name.marker" -- caffeinate -i "$@" > "$S/bat_$name.log" 2>&1
  local rc=$?
  echo "=== BATTERY $name: done exit=$rc $(date +%H:%M:%S) ==="
  if [ $rc -ne 0 ]; then overall=1; echo "FAILED: $name" >> "$S/battery_failures.txt"; fi
}

# A. Dual-feature ast_pipeline rebuild — keeps the debug tool HEAD-coherent.
step dualbuild bash -c "cd $R/rust && cargo build --features 'generated_parsers ebnf_dual_run' --bin ast_pipeline"

# B. Debug probe rebuilt at the changed vintage.
step probe_rebuild bash -c "cd $R/rust && cargo build --features generated_parsers --bin parseability_probe"

# C. Full dual-feature lib suite — carries the ALL-11 differential-equivalence
#    gate plus the re-anchored thin-memo direct-index pins (`-0205`).
step libsuite bash -c "cd $R/rust && cargo test --features 'generated_parsers ebnf_dual_run' --lib"

# D. Regex certificate coverage, seeds 0/7/42 (banked: 268/9/259/0 fully_certified).
step cert bash -c "cd $R/rust && for s in 0 7 42; do ./target/debug/ast_pipeline ../grammars/regex.ebnf --report-certificate-coverage --entry-rule regex --count 40 --seed \$s 2>&1 | grep -E 'CERTIFICATE-COVERAGE|single_path_fallback'; done"

# E-H. The make gates.
step ast_shape_contract_gate         make -C "$R/rust" SHELL=/bin/bash ast_shape_contract_gate
step duality_hunt_gate               make -C "$R/rust" SHELL=/bin/bash duality_hunt_gate
step regex_pcre2_compile_oracle_gate make -C "$R/rust" SHELL=/bin/bash regex_pcre2_compile_oracle_gate
step clippy_on_rust_change           make -C "$R/rust" SHELL=/bin/bash clippy_on_rust_change

echo "BATTERY COMPLETE overall=$overall"
[ -f "$S/battery_failures.txt" ] && cat "$S/battery_failures.txt"
exit $overall
