#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC12_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc12_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc12_semantic_differential_report.json"

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

echo "==> SC-12 contract gate"
echo "state_dir: $STATE_DIR"

# Typed directive payload and validator/coherence contracts.
run_logged_rust "directive_registry_sc12_seed_group_parser_contract" \
    cargo test --lib parses_semantic_group_labels
run_logged_rust "directive_registry_sc12_deterministic_group_parser_contract" \
    cargo test --lib parses_semantic_deterministic_group_payloads
run_logged_rust "directive_registry_sc12_known_directive_contract" \
    cargo test --lib recognizes_known_directives
run_logged_rust "validator_sc12_invalid_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_recovery_payloads
run_logged_rust "validator_sc12_missing_deterministic_group_coherence_contract" \
    cargo test --lib semantic_validator_warns_when_seed_group_without_deterministic_group
run_logged_rust "validator_sc12_active_coherence_contract" \
    cargo test --lib semantic_validator_does_not_warn_when_seed_group_with_deterministic_group_enabled

# Parser/stimuli runtime deterministic partition contracts.
run_logged_rust "semantic_usage_sc12_codegen_policy_contract" \
    cargo test --lib semantic_usage_codegen_extracts_deterministic_partition_policy
run_logged_rust "semantic_usage_sc12_codegen_type_surface_contract" \
    cargo test --lib semantic_usage_codegen_emits_deterministic_partition_types_and_accessors
run_logged_rust "semantic_usage_sc12_codegen_runtime_hook_contract" \
    cargo test --lib semantic_usage_codegen_emits_deterministic_partition_runtime_hooks_for_rules
run_logged_rust "semantic_usage_sc12_codegen_event_recording_contract" \
    cargo test --lib semantic_usage_codegen_records_deterministic_partition_events_in_helper_methods
run_logged_rust "semantic_usage_sc12_codegen_runtime_order_contract" \
    cargo test --lib semantic_usage_codegen_uses_runtime_partition_order_for_ordered_or
run_logged_rust "semantic_usage_sc12_stimuli_seed_group_inactive_contract" \
    cargo test --lib semantic_usage_stimuli_seed_group_stays_inactive_without_deterministic_group
run_logged_rust "semantic_usage_sc12_stimuli_group_label_contract" \
    cargo test --lib semantic_usage_stimuli_deterministic_group_string_payload_enables_partition
run_logged_rust "semantic_usage_sc12_stimuli_order_independence_contract" \
    cargo test --lib semantic_usage_stimuli_deterministic_partitions_are_order_independent

# Shared SC-12 semantic contract corpus (bootstrap + generated).
run_logged_rust "bootstrap_sc12_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc12_contract
run_logged_rust "generated_sc12_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc12_contract

# Differential taxonomy slice: SC-12 comparable corpus should remain parity-clean.
run_logged_rust "differential_sc12_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc12_contract --differential-comparable-only --differential-report-json "target/sc12_contract_gate/work/sc12_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-12 differential report at '$REPORT_JSON'" >&2
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
✅ SC-12 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
