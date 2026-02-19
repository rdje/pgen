#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
GENERATED_DIR="$ROOT_DIR/generated"

STATE_DIR="${PGEN_ANNOTATION_ROBUSTNESS_STATE_DIR:-$RUST_DIR/target/annotation_robustness_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"

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
        --output "$WORK_DIR/return_samples.txt" \
        --coverage-output "$WORK_DIR/return_coverage.json" \
        --gap-report-json "$WORK_DIR/return_gap_report.json"

run_logged "generated_semantic_parseability_stimuli" \
    cargo run --features generated_parsers --bin ast_pipeline -- \
        "$SEMANTIC_JSON" \
        --generate-stimuli \
        --count "$SAMPLE_COUNT" \
        --seed "$SEMANTIC_SEED" \
        --validate-parseability \
        --output "$WORK_DIR/semantic_samples.txt" \
        --coverage-output "$WORK_DIR/semantic_coverage.json" \
        --gap-report-json "$WORK_DIR/semantic_gap_report.json"

cat <<EOF
✅ Annotation robustness gate passed.
Logs: $LOG_DIR
Artifacts: $WORK_DIR
EOF
