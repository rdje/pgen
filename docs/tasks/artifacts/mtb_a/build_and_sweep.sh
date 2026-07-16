#!/usr/bin/env bash
# MTB-A speed gate — build the candidate fat-LTO regex_perf_probe under the
# memory guard, then the alternated 5×2000 sweep vs the preserved D2-B base
# probe (0e97d43b, embedding regex_parser 8c26c97f — the shipped state).
set -euo pipefail
S=/private/tmp/claude-501/-Users-richarddje-Documents-github-pgen/3cacb54e-9d8e-4922-b32f-a5d4069f35ef/scratchpad/mtb_a
R=/Users/richarddje/Documents/github/pgen

echo "cand parser sha: $(shasum -a 256 "$R/generated/regex_parser.rs" | cut -c1-8)"
"$R/scripts/run_with_memory_guard.sh" --budget-mb 16384 --floor-pct 10 --timeout-s 3600 \
  --marker "$S/probe_build_cand.marker" -- \
  bash -c "cd '$R/rust' && cargo build --release --features generated_parsers --bin regex_perf_probe" \
  > "$S/probe_build_cand.log" 2>&1
cp "$R/rust/target/release/regex_perf_probe" "$S/probe_cand"
chmod +x "$S/probe_cand"
cp "$S/probe_d2b_base_0e97d43b" "$S/probe_base"
chmod +x "$S/probe_base"
echo "probe_base sha: $(shasum -a 256 "$S/probe_base" | cut -c1-8)"
echo "probe_cand sha: $(shasum -a 256 "$S/probe_cand" | cut -c1-8)"

cd "$S"
for round in 1 2 3 4 5; do
  if [ $((round % 2)) -eq 1 ]; then sides="base cand"; else sides="cand base"; fi
  for side in $sides; do
    "./probe_$side" --samples 2000 --warmup 200 > "round${round}_${side}.txt"
    echo "round $round $side done"
  done
done
echo "SWEEP COMPLETE"
