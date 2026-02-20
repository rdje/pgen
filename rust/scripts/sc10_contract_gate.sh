#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC10_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc10_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc10_semantic_differential_report.json"

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

echo "==> SC-10 contract gate"
echo "state_dir: $STATE_DIR"

# Typed directive payload and validator/coherence contracts.
run_logged_rust "directive_registry_sc10_coverage_target_parser_contract" \
    cargo test --lib parses_semantic_coverage_target_weights
run_logged_rust "directive_registry_sc10_critical_path_bool_contract" \
    cargo test --lib parses_semantic_bool_values
run_logged_rust "validator_sc10_invalid_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_recovery_payloads
run_logged_rust "validator_sc10_strict_promotion_contract" \
    cargo test --lib semantic_validator_promotes_selected_warning_codes_to_error
run_logged_rust "validator_sc10_strict_exclusion_contract" \
    cargo test --lib semantic_validator_keeps_unselected_warning_codes_as_warning
run_logged_rust "validator_sc10_missing_coverage_target_coherence_contract" \
    cargo test --lib semantic_validator_warns_when_critical_path_enabled_without_coverage_target
run_logged_rust "validator_sc10_active_coherence_contract" \
    cargo test --lib semantic_validator_does_not_warn_when_critical_path_and_coverage_target_enabled

# Parser/stimuli runtime coverage-target steering contracts.
run_logged_rust "semantic_usage_sc10_codegen_policy_contract" \
    cargo test --lib semantic_usage_codegen_extracts_coverage_target_policy
run_logged_rust "semantic_usage_sc10_codegen_type_surface_contract" \
    cargo test --lib semantic_usage_codegen_emits_coverage_target_types_and_accessors
run_logged_rust "semantic_usage_sc10_codegen_runtime_hook_contract" \
    cargo test --lib semantic_usage_codegen_emits_coverage_target_runtime_hooks_for_rules
run_logged_rust "semantic_usage_sc10_codegen_event_recording_contract" \
    cargo test --lib semantic_usage_codegen_records_coverage_target_events_in_helper_methods
run_logged_rust "semantic_usage_sc10_stimuli_branch_bias_contract" \
    cargo test --lib semantic_usage_stimuli_coverage_target_biases_targeted_rule_branches
run_logged_rust "semantic_usage_sc10_stimuli_gap_priority_contract" \
    cargo test --lib semantic_usage_stimuli_coverage_target_boosts_gap_report_branch_priority

# Shared SC-10 semantic contract corpus (bootstrap + generated).
run_logged_rust "bootstrap_sc10_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc10_contract
run_logged_rust "generated_sc10_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc10_contract

# Differential taxonomy slice: SC-10 comparable corpus should remain parity-clean.
run_logged_rust "differential_sc10_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc10_contract --differential-comparable-only --differential-report-json "target/sc10_contract_gate/work/sc10_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-10 differential report at '$REPORT_JSON'" >&2
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
✅ SC-10 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
