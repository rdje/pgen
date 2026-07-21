#!/usr/bin/env bash
# PGEN-RGX-0078-0212 hardened corpus-sweep rerun (the pre-registered addendum
# protocol): four interleaved sweeps base1 -> cand1 -> base2 -> cand2, with a
# foreign-load `ps` snapshot banked before and after every sweep. Custody and
# adjudication are computed afterwards by adjudicate_rerun.py.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../../../.." && pwd)
OUT="$SCRIPT_DIR"
S=${R0212_SCRATCH:?set R0212_SCRATCH to the session scratch dir}
BASE="$S/regex_perf_probe_base"
CAND="$ROOT/rust/target/release/regex_perf_probe"
CORPUS="$ROOT/regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl"
GUARD=("$ROOT/scripts/run_with_memory_guard.sh" --budget-mb 16384 --floor-pct 10 --timeout-s 7200)

sha256() { shasum -a 256 "$1" | awk '{print $1}'; }
[[ "$(sha256 "$BASE")" == "$(cat "$S/base_probe.sha256")" ]] || {
    echo "REFUSE: base probe sha mismatch" >&2; exit 2; }

snapshot() {
    # Bank every non-system process above 50% CPU (the foreign-load tripwire).
    ps aux | awk '$3 > 50 {print $2, $3, $11}' >> "$OUT/rerun_load_snapshots.txt"
    echo "--- $1 $(date -Iseconds)" >> "$OUT/rerun_load_snapshots.txt"
}

sweep() {
    local probe=$1 tag=$2
    snapshot "before_$tag"
    "${GUARD[@]}" -- caffeinate -i "$probe" \
        --corpus-jsonl "$CORPUS" \
        --out-jsonl "$OUT/rerun_${tag}.jsonl" > "$OUT/rerun_${tag}_stdout.txt" 2>&1
    perl -pi -e "s|\\Q$ROOT/\\E||g" "$OUT/rerun_${tag}_stdout.txt"
    snapshot "after_$tag"
}

: > "$OUT/rerun_load_snapshots.txt"
echo "rerun_started_at=$(date -Iseconds)" >> "$OUT/custody.txt"
sweep "$BASE" base1
sweep "$CAND" cand1
sweep "$BASE" base2
sweep "$CAND" cand2
echo "rerun_completed_at=$(date -Iseconds)" >> "$OUT/custody.txt"
