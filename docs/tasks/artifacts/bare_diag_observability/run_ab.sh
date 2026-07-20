#!/usr/bin/env bash
# PGEN-RGX-0078-0201 pre-registered land gate (the `-0200` harness re-homed:
# the two probes embed DIFFERENT generated regex artifacts, so custody asserts
# each side's identity independently).
#
# Base  = preserved_probes/regex_perf_probe_neutrality_948cbd63 (the designated
#         immediate-parent baseline), copied to the scratch dir with its SHA
#         banked in $S/base_probe.sha256 BEFORE any source change.
# Cand  = rust/target/release/regex_perf_probe rebuilt after the change + the
#         canonical all-11 regen train.
# Runs are serialized and guarded; the five benchmark rounds alternate which
# side runs first to reduce order bias.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../../../.." && pwd)
OUT="$SCRIPT_DIR"
S=${R0201_SCRATCH:?set R0201_SCRATCH to the session scratch dir}
BASE="$S/regex_perf_probe_base"
CAND="$ROOT/rust/target/release/regex_perf_probe"
CORPUS="$ROOT/regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl"
GUARD=("$ROOT/scripts/run_with_memory_guard.sh" --budget-mb 16384 --floor-pct 10 --timeout-s 7200)

sha256() {
    shasum -a 256 "$1" | awk '{print $1}'
}

normalize_repo_paths() {
    local output=$1
    perl -pi -e "s|\\Q$ROOT/\\E||g" "$output"
}

EXPECTED_BASE_SHA=$(cat "$S/base_probe.sha256")
base_sha=$(sha256 "$BASE")
candidate_sha=$(sha256 "$CAND")
artifact_sha=$(sha256 "$ROOT/generated/regex_parser.rs")
base_artifact_sha=$(awk '/regex_parser.rs/{print $1}' "$S/artifacts_pre_regen.sha256")
[[ "$base_sha" == "$EXPECTED_BASE_SHA" ]] || {
    echo "REFUSE: base sha $base_sha != $EXPECTED_BASE_SHA" >&2
    exit 2
}
[[ "$artifact_sha" != "$base_artifact_sha" ]] || {
    echo "REFUSE: regex artifact unchanged — the emitter change did not regenerate" >&2
    exit 2
}
for src in "$ROOT/rust/src/ast_pipeline/semantic_runtime.rs" \
           "$ROOT/rust/src/ast_pipeline/ast_based_generator.rs" \
           "$ROOT/rust/src/ast_pipeline/ast_based_generator/cascade.rs" \
           "$ROOT/generated/regex_parser.rs"; do
    [[ "$CAND" -nt "$src" ]] || {
        echo "REFUSE: candidate probe is older than $src" >&2
        exit 2
    }
done

{
    echo "base_sha=$base_sha"
    echo "candidate_sha=$candidate_sha"
    echo "candidate_regex_artifact_sha=$artifact_sha"
    echo "base_regex_artifact_sha=$base_artifact_sha"
    echo "started_at=$(date -Iseconds)"
} > "$OUT/custody.txt"

bench() {
    local probe=$1
    local output=$2
    "${GUARD[@]}" -- caffeinate -i "$probe" --samples 2000 --warmup 200 > "$output" 2>&1
    normalize_repo_paths "$output"
}

sweep() {
    local probe=$1
    local jsonl=$2
    local output=$3
    "${GUARD[@]}" -- caffeinate -i "$probe" \
        --corpus-jsonl "$CORPUS" \
        --out-jsonl "$jsonl" > "$output" 2>&1
    normalize_repo_paths "$output"
}

for round in 1 2 3; do
    bench "$BASE" "$OUT/floorval_r${round}.txt"
done

for round in 1 2 3 4 5; do
    if (( round % 2 == 1 )); then
        sides=(base candidate)
    else
        sides=(candidate base)
    fi
    for side in "${sides[@]}"; do
        if [[ "$side" == base ]]; then
            bench "$BASE" "$OUT/round${round}_base.txt"
        else
            bench "$CAND" "$OUT/round${round}_candidate.txt"
        fi
    done
done

sweep "$BASE" "$OUT/corpus_base.jsonl" "$OUT/corpus_base_stdout.txt"
sweep "$CAND" "$OUT/corpus_candidate.jsonl" "$OUT/corpus_candidate_stdout.txt"

echo "completed_at=$(date -Iseconds)" >> "$OUT/custody.txt"
