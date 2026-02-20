#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC04_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc04_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc04_semantic_differential_report.json"

mkdir -p "$LOG_DIR" "$WORK_DIR"

run_logged_rust() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (
        cd "$RUST_DIR"
        "$@"
    ) >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "    fail (${log_file})" >&2
        tail -n 40 "$log_file" >&2 || true
        exit 1
    fi
}

echo "==> SC-04 contract gate"
echo "state_dir: $STATE_DIR"

# Typed payload/coherence contract checks
run_logged_rust "validator_sc04_payload_parsers" \
    cargo test --lib parses_semantic_
run_logged_rust "validator_sc04_precedence_contract" \
    cargo test --lib semantic_validator_warns_on_token_steering_precedence_overlap
run_logged_rust "validator_sc04_regex_atom_contract_negative" \
    cargo test --lib grammar_aware_validation_warns_on_token_steering_without_regex_atom
run_logged_rust "validator_sc04_regex_atom_contract_positive" \
    cargo test --lib grammar_aware_validation_accepts_token_steering_on_regex_atom

# Parser/stimuli runtime steering contract checks
run_logged_rust "semantic_usage_sc04_codegen_token_class" \
    cargo test --lib semantic_usage_codegen_token_class_overrides_regex_atom_pattern
run_logged_rust "semantic_usage_sc04_codegen_charset" \
    cargo test --lib semantic_usage_codegen_charset_overrides_token_class_pattern
run_logged_rust "semantic_usage_sc04_codegen_pattern" \
    cargo test --lib semantic_usage_codegen_pattern_overrides_charset_and_token_class
run_logged_rust "semantic_usage_sc04_stimuli_token_class" \
    cargo test --lib semantic_usage_stimuli_token_class_overrides_regex_sampling_pattern
run_logged_rust "semantic_usage_sc04_stimuli_charset" \
    cargo test --lib semantic_usage_stimuli_charset_overrides_token_class_pattern
run_logged_rust "semantic_usage_sc04_stimuli_pattern" \
    cargo test --lib semantic_usage_stimuli_pattern_overrides_charset_and_token_class

# Round-trip shared contract slice for semantic token steering directives
run_logged_rust "bootstrap_sc04_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc04_contract
run_logged_rust "generated_sc04_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc04_contract

# Differential taxonomy slice: bootstrap vs generated must remain parity-clean for SC-04 corpus.
run_logged_rust "differential_sc04_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc04_contract --differential-comparable-only --differential-report-json "target/sc04_contract_gate/work/sc04_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-04 differential report at '$REPORT_JSON'" >&2
    exit 1
fi

jq -e '
  (.total_cases > 0) and
  (.mismatched_cases == 0) and
  (((.mismatch_category_counts // {}) | to_entries | map(.value) | add) // 0 == (.mismatched_cases // 0)) and
  (
    (((.mismatch_category_counts // {}) | keys) -
      ["baseline_success_candidate_failure", "baseline_failure_candidate_success", "normalized_output_mismatch"]
    ) | length
  ) == 0
' "$REPORT_JSON" >/dev/null

cat <<EOF
✅ SC-04 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
