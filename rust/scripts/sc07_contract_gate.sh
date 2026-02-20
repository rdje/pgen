#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC07_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc07_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc07_semantic_differential_report.json"

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

echo "==> SC-07 contract gate"
echo "state_dir: $STATE_DIR"

# Typed directive parsing and validator/coherence contracts.
run_logged_rust "directive_registry_sc07_bool_payload_contract" \
    cargo test --lib parses_semantic_bool_values
run_logged_rust "directive_registry_sc07_marker_list_payload_contract" \
    cargo test --lib parses_semantic_string_lists_and_scalars
run_logged_rust "directive_registry_sc07_budget_payload_contract" \
    cargo test --lib parses_semantic_nonnegative_usize_values
run_logged_rust "directive_registry_sc07_known_directive_contract" \
    cargo test --lib recognizes_known_directives
run_logged_rust "validator_sc07_invalid_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_recovery_payloads
run_logged_rust "validator_sc07_budget_without_recover_contract" \
    cargo test --lib semantic_validator_warns_when_recover_budget_present_without_recover
run_logged_rust "validator_sc07_hints_without_recover_contract" \
    cargo test --lib semantic_validator_warns_when_recovery_hints_present_without_recover
run_logged_rust "validator_sc07_enabled_contract" \
    cargo test --lib semantic_validator_does_not_warn_when_recovery_hints_enabled

# Parser/stimuli runtime recovery steering contracts.
run_logged_rust "semantic_usage_sc07_codegen_extracts_hints_contract" \
    cargo test --lib semantic_usage_codegen_extracts_recovery_hints
run_logged_rust "semantic_usage_sc07_codegen_hook_enabled_contract" \
    cargo test --lib semantic_usage_codegen_emits_runtime_recovery_hook_when_recover_enabled
run_logged_rust "semantic_usage_sc07_codegen_hook_disabled_contract" \
    cargo test --lib semantic_usage_codegen_skips_runtime_recovery_hook_when_recover_not_enabled
run_logged_rust "semantic_usage_sc07_codegen_type_surface_contract" \
    cargo test --lib semantic_usage_codegen_declares_structured_recovery_types
run_logged_rust "semantic_usage_sc07_codegen_accessor_surface_contract" \
    cargo test --lib semantic_usage_codegen_emits_recovery_event_accessors
run_logged_rust "semantic_usage_sc07_codegen_event_recording_contract" \
    cargo test --lib semantic_usage_codegen_records_recovery_events_in_helper_methods
run_logged_rust "semantic_usage_sc07_stimuli_fallback_marker_contract" \
    cargo test --lib semantic_usage_stimuli_recovery_fallback_prefers_panic_until_marker
run_logged_rust "semantic_usage_sc07_stimuli_fallback_guard_contract" \
    cargo test --lib semantic_usage_stimuli_recovery_fallback_requires_recover_enabled
run_logged_rust "semantic_usage_sc07_stimuli_recovery_biased_contract" \
    cargo test --lib semantic_usage_stimuli_recovery_biased_mode_wraps_output_with_recovery_markers
run_logged_rust "semantic_usage_sc07_stimuli_near_sync_negative_contract" \
    cargo test --lib semantic_usage_stimuli_near_sync_negative_mode_emits_noise_plus_marker
run_logged_rust "semantic_usage_sc07_stimuli_near_sync_guard_contract" \
    cargo test --lib semantic_usage_stimuli_near_sync_negative_mode_requires_recover_contract

# Shared SC-07 semantic contract corpus (bootstrap + generated).
run_logged_rust "bootstrap_sc07_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc07_contract
run_logged_rust "generated_sc07_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc07_contract

# Differential taxonomy slice: SC-07 comparable corpus should remain parity-clean.
run_logged_rust "differential_sc07_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc07_contract --differential-comparable-only --differential-report-json "target/sc07_contract_gate/work/sc07_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-07 differential report at '$REPORT_JSON'" >&2
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
✅ SC-07 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
