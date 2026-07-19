#!/usr/bin/env bash
# G1-A A/B land gate (PGEN-RGX-0078-0166) — the -0158/-0160 C1 protocol verbatim.
#   base = the preserved floor probe 1d3fa0ee (bench 1,937.4 ns / corpus MAX 483,583 / geomean 1,263.4)
#   cand = this slice's fat-LTO+mimalloc build
# ONE runner at a time, alternated rounds, everything under caffeinate + guard.
set -uo pipefail
R=/Users/richarddje/Documents/github/pgen
S=/private/tmp/claude-501/-Users-richarddje-Documents-github-pgen/ebe196bd-d754-4a06-ae42-207d4a2d6e48/scratchpad/g1a
O=$R/docs/tasks/artifacts/g1a_emission
BASE=$R/preserved_probes/regex_perf_probe_c1_1d3fa0ee
CAND=$S/regex_perf_probe_g1a
CORPUS=$R/regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl
LADDER=$R/docs/tasks/artifacts/worst_cell_rootcause/ladder.jsonl
G="$R/scripts/run_with_memory_guard.sh --budget-mb 16384 --floor-pct 10 --timeout-s 7200"
mkdir -p "$O"

bench() { # bench <probe> <outfile>
  $G --marker "$S/ab.marker" -- "$1" --samples 2000 --warmup 200 > "$2" 2>&1
}
sweep() { # sweep <probe> <inputjsonl> <outjsonl> <stdout>
  $G --marker "$S/ab.marker" -- "$1" --corpus-jsonl "$2" --out-jsonl "$3" > "$4" 2>&1
}

echo "=== A/B START $(date +%H:%M:%S) ==="

# 1. Floor-validate the BASE probe best-of-3 against its banked numbers.
#    If base does not reproduce its floor, the host is not in a measurable state
#    and no delta computed against it would mean anything.
for r in 1 2 3; do
  echo "--- floorval r$r $(date +%H:%M:%S) ---"
  bench "$BASE" "$O/floorval_r$r.txt"
done

# 2. Alternated 5x2000 bench rounds (base,cand interleaved to cancel drift).
for r in 1 2 3 4 5; do
  echo "--- round$r base $(date +%H:%M:%S) ---"; bench "$BASE" "$O/round${r}_base.txt"
  echo "--- round$r cand $(date +%H:%M:%S) ---"; bench "$CAND" "$O/round${r}_cand.txt"
done

# 3. Full PCRE2 corpus sweep (2,189 cells) — the PRIMARY geomean instrument.
echo "--- corpus base $(date +%H:%M:%S) ---"
sweep "$BASE" "$CORPUS" "$O/corpus_base.jsonl" "$O/corpus_base_stdout.txt"
echo "--- corpus cand $(date +%H:%M:%S) ---"
sweep "$CAND" "$CORPUS" "$O/corpus_cand.jsonl" "$O/corpus_cand_stdout.txt"

# 4. The 39-cell nesting ladder (non-regression on the deep-nesting class).
echo "--- ladder base $(date +%H:%M:%S) ---"
sweep "$BASE" "$LADDER" "$O/ladder_base.jsonl" "$O/ladder_base_stdout.txt"
echo "--- ladder cand $(date +%H:%M:%S) ---"
sweep "$CAND" "$LADDER" "$O/ladder_cand.jsonl" "$O/ladder_cand_stdout.txt"

echo "=== A/B COMPLETE $(date +%H:%M:%S) ==="
