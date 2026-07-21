#!/usr/bin/env bash
# RGX-0089 perf-guardrail A/B — the -0212/-0214 HARDENED corpus protocol,
# corpus-only form (the bench rounds are a perf-fix steering instrument; this
# slice is a CORRECTNESS fix — the binding checks are the corpus MAX guardrail
# and the honestly-recorded geomean delta):
#   interleaved corpus sweeps base1 -> cand1 -> base2 -> cand2 with per-sweep
#   foreign-load snapshots; adjudication by adjudicate_0089.py (pooled
#   per-cell minima; base floor-agreement custody gates; verdict-flip check).
#
# Base = preserved_probes/regex_perf_probe_carrier48_8d392176 (the floor probe).
# Cand = rust/target/release/regex_perf_probe rebuilt at the RGX-0089 vintage.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../../../.." && pwd)
OUT="$SCRIPT_DIR"
BASE="$ROOT/preserved_probes/regex_perf_probe_carrier48_8d392176"
CAND="$ROOT/rust/target/release/regex_perf_probe"
CORPUS="$ROOT/regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl"
GUARD=("$ROOT/scripts/run_with_memory_guard.sh" --budget-mb 16384 --floor-pct 10 --timeout-s 7200)

sha256() { shasum -a 256 "$1" | awk '{print $1}'; }

base_sha=$(sha256 "$BASE")
candidate_sha=$(sha256 "$CAND")
artifact_sha=$(sha256 "$ROOT/generated/regex_parser.rs")
[[ "$base_sha" == 8d392176* ]] || {
    echo "REFUSE: base sha $base_sha does not carry the banked 8d392176 prefix" >&2; exit 2; }
[[ "$artifact_sha" == 62f490ab* ]] || {
    echo "REFUSE: candidate regex artifact is not the RGX-0089 regen (62f490ab*)" >&2; exit 2; }
for src in "$ROOT/grammars/regex.ebnf" "$ROOT/generated/regex_parser.rs"; do
    [[ "$CAND" -nt "$src" ]] || {
        echo "REFUSE: candidate probe is older than $src" >&2; exit 2; }
done

{
    echo "base_sha=$base_sha"
    echo "candidate_sha=$candidate_sha"
    echo "candidate_regex_artifact_sha=$artifact_sha"
    echo "started_at=$(date -Iseconds)"
} > "$OUT/custody.txt"

normalize_repo_paths() { perl -pi -e "s|\\Q$ROOT/\\E||g" "$1"; }

snapshot() {
    ps aux | awk '$3 > 50 {print $2, $3, $11}' >> "$OUT/load_snapshots_ab.txt"
    echo "--- $1 $(date -Iseconds)" >> "$OUT/load_snapshots_ab.txt"
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

sweep "$BASE" base1
sweep "$CAND" cand1
sweep "$BASE" base2
sweep "$CAND" cand2

echo "completed_at=$(date -Iseconds)" >> "$OUT/custody.txt"
echo "AB COMPLETE"
