#!/usr/bin/env bash
# RGX-0078.5.i.14 land gate — alternated 5x2000 geomean-of-mins.
# BASE = the preserved -0115 floor probe (281b34d3), validated to reproduce the
#        recorded 5046.8ns floor within day-variance (a deterministic -0115
#        full-11 fat-LTO+mimalloc build == a fresh -0115 build at this HEAD).
# CAND = the fresh full-11 .5.i.14 fat-LTO+mimalloc build (this session).
# Runs BASE then CAND each round (alternated) so per-round machine drift is shared.
set -uo pipefail
# run from the repo root
cd "$(git rev-parse --show-toplevel)"
BENCH=rust/target/generated_logs/spine_bench
CAND="$BENCH/probe_cand_full"
BASE="rust/target/generated_logs/termlit_emission/bench/probe_cand"
[ -x "$CAND" ] || { echo "MISSING cand probe: $CAND"; exit 1; }
[ -x "$BASE" ] || { echo "MISSING base probe: $BASE"; exit 1; }
echo "cand=$(shasum -a 256 "$CAND" | awk '{print $1}')"
echo "base=$(shasum -a 256 "$BASE" | awk '{print $1}')"
for r in 1 2 3 4 5; do
  echo "=== round $r ==="
  "$BASE" --samples 2000 --warmup 200 > "$BENCH/round${r}_base.txt" 2>&1
  "$CAND" --samples 2000 --warmup 200 > "$BENCH/round${r}_cand.txt" 2>&1
  echo "  round $r written"
done
echo "BENCH_DONE"
