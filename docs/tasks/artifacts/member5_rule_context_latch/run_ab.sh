#!/usr/bin/env bash
# PGEN-RGX-0078-0176 member-5 land gate.
#
# The candidate is a LIB-ONLY semantic-runtime change: both probes must embed
# the same generated regex parser.  Custody is therefore asserted before any
# timed run.  Runs are serialized and guarded; the five benchmark rounds
# alternate which side runs first to reduce order bias.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../../../.." && pwd)
OUT="$SCRIPT_DIR"
BASE="$ROOT/preserved_probes/regex_perf_probe_c1_1d3fa0ee"
CAND="$ROOT/rust/target/release/regex_perf_probe"
CORPUS="$ROOT/regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl"
GUARD=("$ROOT/scripts/run_with_memory_guard.sh" --budget-mb 16384 --floor-pct 10 --timeout-s 7200)

EXPECTED_BASE_SHA=1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8
EXPECTED_ARTIFACT_PREFIX=e4924024

sha256() {
    shasum -a 256 "$1" | awk '{print $1}'
}

normalize_repo_paths() {
    local output=$1
    perl -pi -e "s|\\Q$ROOT/\\E||g" "$output"
}

base_sha=$(sha256 "$BASE")
artifact_sha=$(sha256 "$ROOT/generated/regex_parser.rs")
candidate_sha=$(sha256 "$CAND")
[[ "$base_sha" == "$EXPECTED_BASE_SHA" ]] || {
    echo "REFUSE: base sha $base_sha != $EXPECTED_BASE_SHA" >&2
    exit 2
}
[[ "${artifact_sha:0:8}" == "$EXPECTED_ARTIFACT_PREFIX" ]] || {
    echo "REFUSE: regex artifact sha ${artifact_sha:0:8} != $EXPECTED_ARTIFACT_PREFIX" >&2
    exit 2
}
[[ "$CAND" -nt "$ROOT/rust/src/ast_pipeline/semantic_runtime.rs" ]] || {
    echo "REFUSE: candidate probe is older than semantic_runtime.rs" >&2
    exit 2
}

{
    echo "base_sha=$base_sha"
    echo "candidate_sha=$candidate_sha"
    echo "regex_artifact_sha=$artifact_sha"
    echo "started_at=$(date -Iseconds)"
} > "$OUT/custody.txt"

bench() {
    local probe=$1
    local output=$2
    "${GUARD[@]}" -- "$probe" --samples 2000 --warmup 200 > "$output" 2>&1
    normalize_repo_paths "$output"
}

sweep() {
    local probe=$1
    local jsonl=$2
    local output=$3
    "${GUARD[@]}" -- "$probe" \
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
