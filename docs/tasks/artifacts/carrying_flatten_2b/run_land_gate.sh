#!/bin/bash
# RGX-0078.5.j.2 STEP-2b land gate — alternated 5x2000 geomean-of-best-mins.
# base = the preserved -0129 floor probe (4bbfb4e9, dv_bench/probe_cand)
# cand = the fresh STEP-2b fat-LTO+mimalloc probe (target/release/regex_perf_probe)
set -euo pipefail
R=/Users/richarddje/Documents/github/pgen
OUT="$(dirname "$0")"
BASE=$R/rust/target/generated_logs/dv_bench/probe_cand
CAND=$R/rust/target/release/regex_perf_probe
echo "base sha: $(shasum -a 256 "$BASE" | cut -c1-8)"
echo "cand sha: $(shasum -a 256 "$CAND" | cut -c1-8)"
for r in 1 2 3 4 5; do
  echo "== round $r base =="
  "$BASE" --samples 2000 --warmup 200 > "$OUT/round${r}_base.txt"
  echo "== round $r cand =="
  "$CAND" --samples 2000 --warmup 200 > "$OUT/round${r}_cand.txt"
done
echo "rounds done"
