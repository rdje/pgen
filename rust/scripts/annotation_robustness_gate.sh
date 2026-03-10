#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
GENERATED_DIR="$ROOT_DIR/generated"

STATE_DIR="${PGEN_ANNOTATION_ROBUSTNESS_STATE_DIR:-$RUST_DIR/target/annotation_robustness_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

SAMPLE_COUNT="${PGEN_ANNOTATION_ROBUSTNESS_COUNT:-32}"
RETURN_SEED="${PGEN_ANNOTATION_ROBUSTNESS_RETURN_SEED:-4242}"
SEMANTIC_SEED="${PGEN_ANNOTATION_ROBUSTNESS_SEMANTIC_SEED:-4343}"

if ! [[ "$SAMPLE_COUNT" =~ ^[0-9]+$ ]] || [[ "$SAMPLE_COUNT" -lt 1 ]]; then
    echo "error: PGEN_ANNOTATION_ROBUSTNESS_COUNT must be an integer >= 1" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

RETURN_JSON="$GENERATED_DIR/return_annotation.json"
SEMANTIC_JSON="$GENERATED_DIR/semantic_annotation.json"

if [[ ! -f "$RETURN_JSON" ]]; then
    echo "error: missing generated return annotation JSON at '$RETURN_JSON'" >&2
    exit 1
fi
if [[ ! -f "$SEMANTIC_JSON" ]]; then
    echo "error: missing generated semantic annotation JSON at '$SEMANTIC_JSON'" >&2
    exit 1
fi

echo "==> Annotation robustness gate"
echo "state_dir: $STATE_DIR"
echo "sample_count: $SAMPLE_COUNT"
echo "return_seed: $RETURN_SEED"
echo "semantic_seed: $SEMANTIC_SEED"

require_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool '$tool' is not available in PATH" >&2
        exit 1
    fi
}

require_nonempty_file() {
    local path="$1"
    if [[ ! -s "$path" ]]; then
        echo "error: expected non-empty artifact '$path'" >&2
        exit 1
    fi
}

assert_json() {
    local path="$1"
    local expr="$2"
    local message="$3"
    if ! jq -e "$expr" "$path" >/dev/null; then
        echo "error: $message (file: $path, expr: $expr)" >&2
        exit 1
    fi
}

parseability_summary_field_u64() {
    local path="$1"
    local field="$2"
    jq -er ".summary.${field} | numbers" "$path"
}

parseability_acceptance_rate_percent() {
    local path="$1"
    local attempts accepted
    attempts="$(parseability_summary_field_u64 "$path" "attempts")"
    accepted="$(parseability_summary_field_u64 "$path" "accepted")"
    perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$accepted" "$attempts"
}

run_logged() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    (
        cd "$RUST_DIR"
        "$@"
    ) >"$log_file" 2>&1
    echo "    ok (${log_file})"
}

require_tool jq
require_tool perl

echo "grammar,sample_count,seed,stimuli_output,coverage_output,gap_report_json,parseability_attempts_total,parseability_accepted_total,parseability_rejected_total,parseability_parser_rejections_total,parseability_generation_errors_total,parseability_empty_generations_total,parseability_acceptance_rate_percent,parseability_report_json,status" >"$SUMMARY_CSV"

# Advanced annotation suites (bootstrap)
run_logged "bootstrap_return_advanced_extraction" \
    cargo run --bin test_runner -- --parser return --suite return_annotation_advanced_extraction_tests
run_logged "bootstrap_return_stress" \
    cargo run --bin test_runner -- --parser return --suite return_annotation_stress_tests
run_logged "bootstrap_semantic_advanced" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_advanced_tests

# Advanced annotation suites (generated)
run_logged "generated_return_advanced_extraction" \
    cargo run --features generated_parsers --bin test_runner -- --parser return --suite return_annotation_advanced_extraction_tests
run_logged "generated_return_stress" \
    cargo run --features generated_parsers --bin test_runner -- --parser return --suite return_annotation_stress_tests
run_logged "generated_semantic_advanced" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_advanced_tests

# Parseability + coverage/gap checks with generated parsers
run_logged "generated_return_parseability_stimuli" \
    cargo run --features generated_parsers --bin ast_pipeline -- \
        "$RETURN_JSON" \
        --generate-stimuli \
        --count "$SAMPLE_COUNT" \
        --seed "$RETURN_SEED" \
        --validate-parseability \
        --parseability-report-json "$WORK_DIR/return_parseability_report.json" \
        --output "$WORK_DIR/return_samples.txt" \
        --coverage-output "$WORK_DIR/return_coverage.json" \
        --gap-report-json "$WORK_DIR/return_gap_report.json"
require_nonempty_file "$WORK_DIR/return_samples.txt"
require_nonempty_file "$WORK_DIR/return_coverage.json"
require_nonempty_file "$WORK_DIR/return_gap_report.json"
require_nonempty_file "$WORK_DIR/return_parseability_report.json"
assert_json "$WORK_DIR/return_coverage.json" '.grammar_name == "return_annotation"' "return coverage grammar_name mismatch"
assert_json "$WORK_DIR/return_gap_report.json" '.grammar_name == "return_annotation"' "return gap grammar_name mismatch"
assert_json "$WORK_DIR/return_parseability_report.json" ".grammar_name == \"return_annotation\" and .summary.requested == $SAMPLE_COUNT and .summary.accepted == $SAMPLE_COUNT and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "return parseability report contract mismatch"

return_attempts_total="$(parseability_summary_field_u64 "$WORK_DIR/return_parseability_report.json" "attempts")"
return_accepted_total="$(parseability_summary_field_u64 "$WORK_DIR/return_parseability_report.json" "accepted")"
return_rejected_total="$(parseability_summary_field_u64 "$WORK_DIR/return_parseability_report.json" "rejected")"
return_parser_rejections_total="$(parseability_summary_field_u64 "$WORK_DIR/return_parseability_report.json" "parser_rejections")"
return_generation_errors_total="$(parseability_summary_field_u64 "$WORK_DIR/return_parseability_report.json" "generation_errors")"
return_empty_generations_total="$(parseability_summary_field_u64 "$WORK_DIR/return_parseability_report.json" "empty_generations")"
return_acceptance_rate_percent="$(parseability_acceptance_rate_percent "$WORK_DIR/return_parseability_report.json")"
echo "return_annotation,${SAMPLE_COUNT},${RETURN_SEED},$WORK_DIR/return_samples.txt,$WORK_DIR/return_coverage.json,$WORK_DIR/return_gap_report.json,${return_attempts_total},${return_accepted_total},${return_rejected_total},${return_parser_rejections_total},${return_generation_errors_total},${return_empty_generations_total},${return_acceptance_rate_percent},$WORK_DIR/return_parseability_report.json,pass" >>"$SUMMARY_CSV"

run_logged "generated_semantic_parseability_stimuli" \
    cargo run --features generated_parsers --bin ast_pipeline -- \
        "$SEMANTIC_JSON" \
        --generate-stimuli \
        --count "$SAMPLE_COUNT" \
        --seed "$SEMANTIC_SEED" \
        --validate-parseability \
        --parseability-report-json "$WORK_DIR/semantic_parseability_report.json" \
        --output "$WORK_DIR/semantic_samples.txt" \
        --coverage-output "$WORK_DIR/semantic_coverage.json" \
        --gap-report-json "$WORK_DIR/semantic_gap_report.json"
require_nonempty_file "$WORK_DIR/semantic_samples.txt"
require_nonempty_file "$WORK_DIR/semantic_coverage.json"
require_nonempty_file "$WORK_DIR/semantic_gap_report.json"
require_nonempty_file "$WORK_DIR/semantic_parseability_report.json"
assert_json "$WORK_DIR/semantic_coverage.json" '.grammar_name == "semantic_annotation"' "semantic coverage grammar_name mismatch"
assert_json "$WORK_DIR/semantic_gap_report.json" '.grammar_name == "semantic_annotation"' "semantic gap grammar_name mismatch"
assert_json "$WORK_DIR/semantic_parseability_report.json" ".grammar_name == \"semantic_annotation\" and .summary.requested == $SAMPLE_COUNT and .summary.accepted == $SAMPLE_COUNT and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "semantic parseability report contract mismatch"

semantic_attempts_total="$(parseability_summary_field_u64 "$WORK_DIR/semantic_parseability_report.json" "attempts")"
semantic_accepted_total="$(parseability_summary_field_u64 "$WORK_DIR/semantic_parseability_report.json" "accepted")"
semantic_rejected_total="$(parseability_summary_field_u64 "$WORK_DIR/semantic_parseability_report.json" "rejected")"
semantic_parser_rejections_total="$(parseability_summary_field_u64 "$WORK_DIR/semantic_parseability_report.json" "parser_rejections")"
semantic_generation_errors_total="$(parseability_summary_field_u64 "$WORK_DIR/semantic_parseability_report.json" "generation_errors")"
semantic_empty_generations_total="$(parseability_summary_field_u64 "$WORK_DIR/semantic_parseability_report.json" "empty_generations")"
semantic_acceptance_rate_percent="$(parseability_acceptance_rate_percent "$WORK_DIR/semantic_parseability_report.json")"
echo "semantic_annotation,${SAMPLE_COUNT},${SEMANTIC_SEED},$WORK_DIR/semantic_samples.txt,$WORK_DIR/semantic_coverage.json,$WORK_DIR/semantic_gap_report.json,${semantic_attempts_total},${semantic_accepted_total},${semantic_rejected_total},${semantic_parser_rejections_total},${semantic_generation_errors_total},${semantic_empty_generations_total},${semantic_acceptance_rate_percent},$WORK_DIR/semantic_parseability_report.json,pass" >>"$SUMMARY_CSV"

{
    echo "PGEN Annotation Robustness Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "sample_count: $SAMPLE_COUNT"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

cat <<EOF
✅ Annotation robustness gate passed.
Logs: $LOG_DIR
Artifacts: $WORK_DIR
EOF
