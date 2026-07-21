#!/usr/bin/env bash
# PGEN-RGX-0078-0214 pre-registered land gate — the `-0212` HARDENED protocol
# as the primary (and only) corpus instrument:
#   floorval x3 (bench custody) -> 5 alternating bench rounds (steering) ->
#   interleaved corpus sweeps base1 -> cand1 -> base2 -> cand2 with per-sweep
#   foreign-load snapshots. Adjudication afterwards by adjudicate.py
#   (pooled per-cell minima; base floor-agreement gates).
#
# Base = preserved_probes/regex_perf_probe_carrier48_8d392176 copied to the
# scratch dir with its SHA banked BEFORE regeneration.
# Cand = rust/target/release/regex_perf_probe rebuilt after the change + the
# canonical all-11 regen train.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../../../.." && pwd)
OUT="$SCRIPT_DIR"
S=${R0214_SCRATCH:?set R0214_SCRATCH to the session scratch dir}
BASE="$S/regex_perf_probe_base"
CAND="$ROOT/rust/target/release/regex_perf_probe"
CORPUS="$ROOT/regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl"
GUARD=("$ROOT/scripts/run_with_memory_guard.sh" --budget-mb 16384 --floor-pct 10 --timeout-s 7200)

sha256() { shasum -a 256 "$1" | awk '{print $1}'; }

EXPECTED_BASE_SHA=$(cat "$S/base_probe.sha256")
base_sha=$(sha256 "$BASE")
candidate_sha=$(sha256 "$CAND")
artifact_sha=$(sha256 "$ROOT/generated/regex_parser.rs")
base_artifact_sha=$(awk '/regex_parser.rs/{print $1}' "$S/artifacts_pre_regen.sha256")
[[ "$base_sha" == "$EXPECTED_BASE_SHA" ]] || {
    echo "REFUSE: base sha $base_sha != $EXPECTED_BASE_SHA" >&2; exit 2; }
[[ "$artifact_sha" != "$base_artifact_sha" ]] || {
    echo "REFUSE: regex artifact unchanged — the emitter change did not regenerate" >&2; exit 2; }
for src in "$ROOT/rust/src/ast_pipeline/ast_based_generator.rs" \
           "$ROOT/generated/regex_parser.rs"; do
    [[ "$CAND" -nt "$src" ]] || {
        echo "REFUSE: candidate probe is older than $src" >&2; exit 2; }
done

{
    echo "base_sha=$base_sha"
    echo "candidate_sha=$candidate_sha"
    echo "candidate_regex_artifact_sha=$artifact_sha"
    echo "base_regex_artifact_sha=$base_artifact_sha"
    echo "started_at=$(date -Iseconds)"
} > "$OUT/custody.txt"

normalize_repo_paths() { perl -pi -e "s|\\Q$ROOT/\\E||g" "$1"; }

snapshot() {
    ps aux | awk '$3 > 50 {print $2, $3, $11}' >> "$OUT/load_snapshots_ab.txt"
    echo "--- $1 $(date -Iseconds)" >> "$OUT/load_snapshots_ab.txt"
}

bench() {
    "${GUARD[@]}" -- caffeinate -i "$1" --samples 2000 --warmup 200 > "$2" 2>&1
    normalize_repo_paths "$2"
}

sweep() {
    local probe=$1 tag=$2
    snapshot "before_$tag"
    "${GUARD[@]}" -- caffeinate -i "$probe" \
        --corpus-jsonl "$CORPUS" \
        --out-jsonl "$OUT/corpus_${tag}.jsonl" > "$OUT/corpus_${tag}_stdout.txt" 2>&1
    normalize_repo_paths "$OUT/corpus_${tag}_stdout.txt"
    snapshot "after_$tag"
}

: > "$OUT/load_snapshots_ab.txt"

for round in 1 2 3; do
    bench "$BASE" "$OUT/floorval_r${round}.txt"
done

for round in 1 2 3 4 5; do
    if (( round % 2 == 1 )); then sides=(base candidate); else sides=(candidate base); fi
    for side in "${sides[@]}"; do
        if [[ "$side" == base ]]; then
            bench "$BASE" "$OUT/round${round}_base.txt"
        else
            bench "$CAND" "$OUT/round${round}_candidate.txt"
        fi
    done
done

sweep "$BASE" base1
sweep "$CAND" cand1
sweep "$BASE" base2
sweep "$CAND" cand2

echo "completed_at=$(date -Iseconds)" >> "$OUT/custody.txt"
echo "AB COMPLETE"
