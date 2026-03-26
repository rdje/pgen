#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC01_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc01_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc01_semantic_differential_report.json"

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

echo "==> SC-01 contract gate"
echo "state_dir: $STATE_DIR"

run_logged_rust "semantic_transform_parser_contract" \
    cargo test --lib parses_canonical_transform_expression
run_logged_rust "semantic_transform_whitespace_contract" \
    cargo test --lib parses_canonical_transform_with_whitespace
run_logged_rust "semantic_transform_noncanonical_rejection_contract" \
    cargo test --lib rejects_noncanonical_transform_expression

run_logged_rust "validator_sc01_canonical_transform_contract" \
    cargo test --lib semantic_validator_accepts_canonical_transform
run_logged_rust "validator_sc01_noncanonical_strict_contract" \
    cargo test --lib semantic_validator_strict_mode_promotes_noncanonical_to_error

run_logged_rust "semantic_usage_sc01_codegen_contract" \
    cargo test --lib semantic_usage_codegen_applies_canonical_transform_on_regex_atom
run_logged_rust "semantic_usage_sc01_codegen_path_target_contract" \
    cargo test --lib semantic_usage_codegen_accepts_path_target_type

run_logged_rust "semantic_usage_sc01_stimuli_integer_contract" \
    cargo test --lib semantic_usage_stimuli_transformexpr_hint_overrides_regex_sampling
run_logged_rust "semantic_usage_sc01_stimuli_float_bool_contract" \
    cargo test --lib semantic_usage_stimuli_transformexpr_hints_cover_float_and_bool
run_logged_rust "semantic_usage_sc01_stimuli_path_target_contract" \
    cargo test --lib semantic_usage_stimuli_transformexpr_supports_path_target_type
run_logged_rust "semantic_usage_sc01_stimuli_noncanonical_guard_contract" \
    cargo test --lib semantic_usage_stimuli_noncanonical_transform_does_not_override_regex

run_logged_rust "bootstrap_sc01_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc01_contract
run_logged_rust "generated_sc01_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc01_contract

run_logged_rust "differential_sc01_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc01_contract --differential-comparable-only --differential-report-json "target/sc01_contract_gate/work/sc01_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-01 differential report at '$REPORT_JSON'" >&2
    exit 1
fi

jq -e '
  (.mismatched_cases == 0) and
  (((.mismatch_category_counts // {}) | to_entries | map(.value) | add) // 0 == (.mismatched_cases // 0)) and
  (
    (((.mismatch_category_counts // {}) | keys) -
      ["baseline_success_candidate_failure", "baseline_failure_candidate_success", "normalized_output_mismatch"]
    ) | length
  ) == 0
' "$REPORT_JSON" >/dev/null

cat <<EOF
✅ SC-01 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
