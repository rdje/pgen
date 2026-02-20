#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC11_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc11_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc11_semantic_differential_report.json"

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

echo "==> SC-11 contract gate"
echo "state_dir: $STATE_DIR"

# Typed directive payload and validator/coherence contracts.
run_logged_rust "directive_registry_sc11_bool_payload_contract" \
    cargo test --lib parses_semantic_bool_values
run_logged_rust "directive_registry_sc11_known_directive_contract" \
    cargo test --lib recognizes_known_directives
run_logged_rust "validator_sc11_invalid_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_recovery_payloads
run_logged_rust "validator_sc11_missing_invalid_case_coherence_contract" \
    cargo test --lib semantic_validator_warns_when_negative_enabled_without_invalid_case
run_logged_rust "validator_sc11_active_coherence_contract" \
    cargo test --lib semantic_validator_does_not_warn_when_negative_and_invalid_case_enabled

# Parser/stimuli runtime negative-case steering contracts.
run_logged_rust "semantic_usage_sc11_codegen_policy_contract" \
    cargo test --lib semantic_usage_codegen_extracts_negative_case_policy
run_logged_rust "semantic_usage_sc11_codegen_type_surface_contract" \
    cargo test --lib semantic_usage_codegen_emits_negative_case_types_and_accessors
run_logged_rust "semantic_usage_sc11_codegen_runtime_hook_contract" \
    cargo test --lib semantic_usage_codegen_emits_negative_case_runtime_hooks_for_rules
run_logged_rust "semantic_usage_sc11_codegen_event_recording_contract" \
    cargo test --lib semantic_usage_codegen_records_negative_case_events_in_helper_methods
run_logged_rust "semantic_usage_sc11_stimuli_invalid_case_contract" \
    cargo test --lib semantic_usage_stimuli_invalid_case_mutates_entry_output
run_logged_rust "semantic_usage_sc11_stimuli_negative_marker_contract" \
    cargo test --lib semantic_usage_stimuli_invalid_case_plus_negative_appends_marker
run_logged_rust "semantic_usage_sc11_stimuli_negative_guard_contract" \
    cargo test --lib semantic_usage_stimuli_negative_requires_invalid_case_contract

# Shared SC-11 semantic contract corpus (bootstrap + generated).
run_logged_rust "bootstrap_sc11_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc11_contract
run_logged_rust "generated_sc11_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc11_contract

# Differential taxonomy slice: SC-11 comparable corpus should remain parity-clean.
run_logged_rust "differential_sc11_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc11_contract --differential-comparable-only --differential-report-json "target/sc11_contract_gate/work/sc11_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-11 differential report at '$REPORT_JSON'" >&2
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
✅ SC-11 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
