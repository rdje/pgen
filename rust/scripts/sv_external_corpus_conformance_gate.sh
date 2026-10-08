#!/usr/bin/env bash
# rust/scripts/sv_external_corpus_conformance_gate.sh — the SystemVerilog external-corpus CONFORMANCE gate
# (DONE-BAR.3b, 2026-09-03). Leg 3 of the DONE-BAR bar is *"an officially-recognized third-party corpus,
# PASSING"* — and DONE-BAR.1 measured that the only gate reading this family's corpus was a TRIAGE gate,
# which is not a conformance gate. This one is:
#   1. it RE-MEASURES the vendored third-party corpus under the family's declared corpus roots —
#      stimuli/sv/subs (23 vendored suites) and stimuli/sv/uvm — through stimuli/run_external_corpus.sh,
#      both lanes (sv_2017 and verilog_2005), into a SCRATCH state dir (never the tracked artifacts);
#   2. it ADJUDICATES the fresh outcomes against the answer keys (stimuli/sv/adjudicate_external_corpus.py);
#   3. it COMPARES the fresh adjudication with the TRACKED one (stimuli/sv/corpus_conformance_compare.py):
#      any regression of the recorded answer-key agreement FAILS the gate (exit 1);
#   4. it records `full_pass` — zero unexplained divergences, and every row adjudicated under its DECLARED
#      reading (the preprocess→parse pipeline's re-adjudication for a row the raw lane defers to it; the
#      PERMANENT classes of both lanes named and held under their ceilings, never demanded — DONE-BAR.3f,
#      2026-10-08) — in summary.json, beside `burnable_no_verdict` (what still stands between the corpus and
#      the bar) and `full_pass_raw_lane` (the pre-.3f bit, `no_verdict == 0` on the raw lane, which was
#      unreachable by construction: 716 permanent + 4 933 pipeline-only rows). That bit, not the exit code,
#      is what the DONE-BAR register's leg3_surface pass_query reads. ⛔ A green exit is "no regression"; it
#      is NOT "the corpus passes". Conflating the two is the silent relabelling DONE-BAR exists to stop.
# State: rust/target/sv_external_corpus_conformance_gate/{summary.txt,summary.json,comparison.json,work,logs}
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
STATE_DIR="${PGEN_SV_EXTERNAL_CORPUS_CONFORMANCE_STATE_DIR:-$RUST_DIR/target/sv_external_corpus_conformance_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"
COMPARISON_JSON="$STATE_DIR/comparison.json"
CORPUS_ROOTS="stimuli/sv/subs stimuli/sv/uvm"
TRACKED_DIR="$ROOT_DIR/stimuli/sv/characterization"
TRACKED_MANIFEST="$TRACKED_DIR/adjudication_manifest.tsv"
TRACKED_MANIFEST_V2005="$TRACKED_DIR/adjudication_manifest_v2005.tsv"
PROBE="$RUST_DIR/target/release/parseability_probe"
RUNNER="$ROOT_DIR/stimuli/run_external_corpus.sh"
ADJUDICATOR="$ROOT_DIR/stimuli/sv/adjudicate_external_corpus.py"
COMPARE="$ROOT_DIR/stimuli/sv/corpus_conformance_compare.py"

require_tool() { command -v "$1" >/dev/null 2>&1 || { echo "error: required tool '$1' not found" >&2; exit 2; }; }
require_file() { [[ -s "$1" ]] || { echo "error: required file missing or empty: $1" >&2; exit 2; }; }
require_tool jq; require_tool python3; require_tool shasum
require_file "$TRACKED_MANIFEST"; require_file "$TRACKED_MANIFEST_V2005"; require_file "$RUNNER"; require_file "$ADJUDICATOR"; require_file "$COMPARE"
[[ -x "$PROBE" ]] || { echo "error: $PROBE is not built (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)" >&2; exit 2; }
for root in $CORPUS_ROOTS; do
    [[ -d "$ROOT_DIR/$root" ]] && [[ -n "$(ls -A "$ROOT_DIR/$root" 2>/dev/null)" ]] || { echo "error: corpus root $root is absent or empty (submodules not checked out) — this gate cannot run, and says so" >&2; exit 2; }
done

rm -rf "$WORK_DIR"; mkdir -p "$WORK_DIR/characterization" "$LOG_DIR"
: >"$SUMMARY_TXT"
run_logged() {
    local label="$1"; shift; local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then echo "    ok (${log_file})"; else echo "error: stage '$label' failed (log: $log_file)" >&2; tail -n 40 "$log_file" >&2 || true; return 1; fi
}
started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
# 1. re-measure both lanes into scratch (the binding parameters are adopted from the tracked reports)
run_logged "corpus_sv"     env PGEN_CORPUS_OUT_DIR="$WORK_DIR/characterization" bash "$RUNNER" sv
run_logged "corpus_sv2005" env PGEN_CORPUS_OUT_DIR="$WORK_DIR/characterization" bash "$RUNNER" sv2005
C="$WORK_DIR/characterization"
# 2. adjudicate the fresh outcomes against the answer keys
run_logged "adjudicate" python3 "$ADJUDICATOR" \
    --results "$C/results.tsv" --positions "$C/positions.tsv" --report "$C/characterization.md" \
    --results-v2005 "$C/results_v2005.tsv" --positions-v2005 "$C/positions_v2005.tsv" --report-v2005 "$C/characterization_v2005.md" \
    --out-manifest "$WORK_DIR/adjudication_manifest.tsv" --out-manifest-v2005 "$WORK_DIR/adjudication_manifest_v2005.tsv" \
    --out-summary "$WORK_DIR/adjudication_summary.md" --out-summary-v2005 "$WORK_DIR/adjudication_summary_v2005.md" \
    --out-lane-list "$WORK_DIR/v2005_lane_files.tsv"
# 3. compare with the tracked adjudication — the exit code is the regression bit
set +e
python3 "$COMPARE" --tracked "$TRACKED_MANIFEST" --fresh "$WORK_DIR/adjudication_manifest.tsv" \
    --tracked-v2005 "$TRACKED_MANIFEST_V2005" --fresh-v2005 "$WORK_DIR/adjudication_manifest_v2005.tsv" \
    --out "$COMPARISON_JSON" >"$LOG_DIR/compare.log" 2>&1
compare_rc=$?
set -e
cat "$LOG_DIR/compare.log"
if [[ $compare_rc -eq 2 ]] || [[ ! -s "$COMPARISON_JSON" ]]; then echo "error: the comparison REFUSED (log: $LOG_DIR/compare.log)" >&2; exit 2; fi
# 4. the artifact
sha() { shasum -a 256 "$1" | cut -d' ' -f1; }
probe_fp="$("$PROBE" --parser-fingerprint 2>/dev/null | jq -r '.parsers.systemverilog // empty' 2>/dev/null || true)"
jq -n \
    --arg gate "sv_external_corpus_conformance_gate" --argjson version 1 \
    --arg started_at_utc "$started_at" --arg generated_at_utc "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
    --arg corpus_roots "$CORPUS_ROOTS" \
    --arg tracked_manifest_sha256 "$(sha "$TRACKED_MANIFEST")" --arg fresh_manifest_sha256 "$(sha "$WORK_DIR/adjudication_manifest.tsv")" \
    --arg tracked_manifest_v2005_sha256 "$(sha "$TRACKED_MANIFEST_V2005")" --arg fresh_manifest_v2005_sha256 "$(sha "$WORK_DIR/adjudication_manifest_v2005.tsv")" \
    --arg probe_sha256 "$(sha "$PROBE")" --arg systemverilog_parser_fingerprint "${probe_fp:-unknown}" \
    --argjson comparison "$(cat "$COMPARISON_JSON")" --argjson regression_exit "$compare_rc" \
    '{gate: $gate, version: $version, started_at_utc: $started_at_utc, generated_at_utc: $generated_at_utc,
      corpus_roots: ($corpus_roots | split(" ")),
      tracked_manifest_sha256: $tracked_manifest_sha256, fresh_manifest_sha256: $fresh_manifest_sha256,
      tracked_manifest_v2005_sha256: $tracked_manifest_v2005_sha256, fresh_manifest_v2005_sha256: $fresh_manifest_v2005_sha256,
      probe_sha256: $probe_sha256, systemverilog_parser_fingerprint: $systemverilog_parser_fingerprint,
      regressions_total: $comparison.regressions_total, improvements_total: $comparison.improvements_total,
      no_regression: ($regression_exit == 0), full_pass: $comparison.full_pass,
      full_pass_raw_lane: $comparison.full_pass_raw_lane, permanent_no_verdict: $comparison.permanent_no_verdict,
      burnable_no_verdict: $comparison.burnable_no_verdict, full_pass_requires: $comparison.full_pass_requires,
      sv_2017: ($comparison.sv_2017 | del(.regression_rows, .improvement_rows)),
      verilog_2005: ($comparison.verilog_2005 | del(.regression_rows, .improvement_rows)),
      comparison_json: "rust/target/sv_external_corpus_conformance_gate/comparison.json"}' >"$SUMMARY_JSON"
{
    echo "gate: sv_external_corpus_conformance_gate"
    echo "corpus_roots: $CORPUS_ROOTS"
    echo "no_regression: $([[ $compare_rc -eq 0 ]] && echo true || echo false)"
    echo "full_pass: $(jq -r '.full_pass' "$SUMMARY_JSON")"
    echo "full_pass_raw_lane: $(jq -r '.full_pass_raw_lane' "$SUMMARY_JSON")"
    echo "permanent_no_verdict: $(jq -r '.permanent_no_verdict' "$SUMMARY_JSON")"
    echo "burnable_no_verdict: $(jq -r '.burnable_no_verdict' "$SUMMARY_JSON")"
    echo "regressions_total: $(jq -r '.regressions_total' "$SUMMARY_JSON")"
    echo "improvements_total: $(jq -r '.improvements_total' "$SUMMARY_JSON")"
    for lane in sv_2017 verilog_2005; do
        for k in rows match explained unexplained routed no_verdict permanent_no_verdict burnable_no_verdict observed_pass_to_fail; do echo "${lane}_${k}: $(jq -r ".${lane}.${k}" "$SUMMARY_JSON")"; done
    done
    for k in declared_adjudicated declared_routed declared_no_verdict declared_bar readjudication_stale; do echo "sv_2017_${k}: $(jq -r ".sv_2017.${k}" "$SUMMARY_JSON")"; done
    echo "tracked_manifest_sha256: $(jq -r '.tracked_manifest_sha256' "$SUMMARY_JSON")"
    echo "fresh_manifest_sha256: $(jq -r '.fresh_manifest_sha256' "$SUMMARY_JSON")"
    echo "summary_json: $SUMMARY_JSON"
} >"$SUMMARY_TXT"
cat "$SUMMARY_TXT"
exit "$compare_rc"
