#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC09_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc09_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc09_semantic_differential_report.json"

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

echo "==> SC-09 contract gate"
echo "state_dir: $STATE_DIR"

# Typed directive payload and validator/coherence contracts.
run_logged_rust "directive_registry_sc09_constraint_parser_contract" \
    cargo test --lib parses_semantic_constraint_expressions
run_logged_rust "directive_registry_sc09_requires_parser_contract" \
    cargo test --lib parses_semantic_reference_lists
run_logged_rust "directive_registry_sc09_implies_parser_contract" \
    cargo test --lib parses_semantic_implication_payloads
run_logged_rust "validator_sc09_invalid_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_relational_payloads
run_logged_rust "validator_sc09_missing_constraint_coherence_contract" \
    cargo test --lib semantic_validator_warns_when_relational_hints_present_without_constraint
run_logged_rust "validator_sc09_active_constraint_coherence_contract" \
    cargo test --lib semantic_validator_does_not_warn_on_relational_hint_when_constraint_present

# Parser/stimuli runtime relational steering contracts.
run_logged_rust "semantic_usage_sc09_codegen_policy_contract" \
    cargo test --lib semantic_usage_codegen_parses_relational_constraint_policy
run_logged_rust "semantic_usage_sc09_codegen_inactive_hint_contract" \
    cargo test --lib semantic_usage_codegen_disables_relational_hints_without_constraint
run_logged_rust "semantic_usage_sc09_codegen_rule_guard_contract" \
    cargo test --lib semantic_usage_codegen_emits_runtime_relational_guards_for_rule_methods
run_logged_rust "semantic_usage_sc09_codegen_helper_surface_contract" \
    cargo test --lib semantic_usage_codegen_declares_relational_runtime_helper_methods
run_logged_rust "semantic_usage_sc09_stimuli_filter_contract" \
    cargo test --lib semantic_usage_stimuli_relational_constraint_filters_cross_capture_values
run_logged_rust "semantic_usage_sc09_stimuli_implies_contract" \
    cargo test --lib semantic_usage_stimuli_relational_implies_enforced_during_generation
run_logged_rust "semantic_usage_sc09_stimuli_nested_named_contract" \
    cargo test --lib semantic_usage_stimuli_relational_supports_nested_named_paths
run_logged_rust "semantic_usage_sc09_stimuli_nested_positional_contract" \
    cargo test --lib semantic_usage_stimuli_relational_supports_positional_nested_paths
run_logged_rust "semantic_usage_sc09_stimuli_nonstructured_named_contract" \
    cargo test --lib semantic_usage_stimuli_relational_supports_nonstructured_named_paths
run_logged_rust "semantic_usage_sc09_stimuli_nonstructured_positional_contract" \
    cargo test --lib semantic_usage_stimuli_relational_supports_nonstructured_positional_paths
run_logged_rust "semantic_usage_sc09_stimuli_inactive_hint_contract" \
    cargo test --lib semantic_usage_stimuli_relational_hints_without_constraint_remain_inactive
run_logged_rust "semantic_usage_sc09_stimuli_unsat_diagnostics_contract" \
    cargo test --lib semantic_usage_stimuli_relational_unsat_reports_ranked_violation_summary

# Shared SC-09 semantic contract corpus (bootstrap + generated).
run_logged_rust "bootstrap_sc09_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc09_contract
run_logged_rust "generated_sc09_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc09_contract

# Differential taxonomy slice: SC-09 comparable corpus should remain parity-clean.
run_logged_rust "differential_sc09_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc09_contract --differential-comparable-only --differential-report-json "target/sc09_contract_gate/work/sc09_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-09 differential report at '$REPORT_JSON'" >&2
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
✅ SC-09 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
