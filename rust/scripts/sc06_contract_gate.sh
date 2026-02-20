#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC06_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc06_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc06_semantic_differential_report.json"

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

echo "==> SC-06 contract gate"
echo "state_dir: $STATE_DIR"

# Typed directive/validator contract checks
run_logged_rust "directive_registry_sc06_branch_policy_parser" \
    cargo test --lib parses_semantic_branch_policy_values
run_logged_rust "directive_registry_sc06_capability_matrix_contract" \
    cargo test --lib directive_capability_matrix_reflects_runtime_surface
run_logged_rust "validator_sc06_branch_policy_invalid_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_branch_policy_payload
run_logged_rust "validator_sc06_branch_policy_valid_payload_contract" \
    cargo test --lib semantic_validator_accepts_valid_branch_policy_payloads

# Parser/stimuli runtime branch-selection contract checks
run_logged_rust "semantic_usage_sc06_codegen_branch_policy_contract" \
    cargo test --lib semantic_usage_codegen_parses_branch_policy_directive
run_logged_rust "semantic_usage_sc06_stimuli_ordered_contract" \
    cargo test --lib semantic_branch_policy_ordered_prefers_first_successful_branch
run_logged_rust "semantic_usage_sc06_stimuli_priority_first_contract" \
    cargo test --lib semantic_branch_policy_priority_first_prefers_high_priority_branch
run_logged_rust "semantic_usage_sc06_weighted_probabilities_determinism_contract" \
    cargo test --lib weighted_probabilities_are_deterministic_with_seed
run_logged_rust "semantic_usage_sc06_weighted_probabilities_fallback_contract" \
    cargo test --lib missing_probabilities_fallback_to_equal_weights

# Shared SC-06 semantic contract corpus (bootstrap + generated)
run_logged_rust "bootstrap_sc06_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc06_contract
run_logged_rust "generated_sc06_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc06_contract

# Differential taxonomy slice: SC-06 comparable corpus should remain parity-clean.
run_logged_rust "differential_sc06_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc06_contract --differential-comparable-only --differential-report-json "target/sc06_contract_gate/work/sc06_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-06 differential report at '$REPORT_JSON'" >&2
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
✅ SC-06 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
