#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC13_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc13_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc13_semantic_differential_report.json"

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

echo "==> SC-13 contract gate"
echo "state_dir: $STATE_DIR"

run_logged_rust "directive_registry_sc13_known_directive_contract" \
    cargo test --lib recognizes_known_directives
run_logged_rust "validator_sc13_valid_profiles_payload_contract" \
    cargo test --lib semantic_validator_accepts_valid_profiles_payload
run_logged_rust "validator_sc13_invalid_profiles_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_profiles_payload
run_logged_rust "validator_sc13_valid_open_scope_payload_contract" \
    cargo test --lib semantic_validator_accepts_valid_open_scope_runtime_payload
run_logged_rust "validator_sc13_valid_close_scope_payload_contract" \
    cargo test --lib semantic_validator_accepts_valid_close_scope_runtime_payload
run_logged_rust "validator_sc13_valid_predicate_payload_contract" \
    cargo test --lib semantic_validator_accepts_valid_predicate_runtime_payload

run_logged_rust "semantic_usage_sc13_profile_extraction_contract" \
    cargo test --lib semantic_usage_codegen_extracts_rule_profiles_from_named_directive
run_logged_rust "semantic_usage_sc13_profile_guard_codegen_contract" \
    cargo test --lib generated_parser_profile_contract_emits_rule_profile_guard
run_logged_rust "semantic_runtime_sc13_scope_fact_contract" \
    cargo test --lib parses_open_scope_and_emit_fact_runtime_directives
run_logged_rust "semantic_runtime_sc13_predicate_contract" \
    cargo test --lib parses_predicate_runtime_directive_with_explicit_phase_and_view
run_logged_rust "semantic_runtime_sc13_scope_state_contract" \
    cargo test --lib built_in_predicates_respect_scope_changes_and_unknowns
run_logged_rust "semantic_runtime_sc13_compiled_views_contract" \
    cargo test --lib compiled_annotations_split_pre_predicates_from_effects

run_logged_rust "bootstrap_sc13_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc13_contract
run_logged_rust "generated_sc13_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc13_contract

run_logged_rust "differential_sc13_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc13_contract --differential-comparable-only --differential-report-json "target/sc13_contract_gate/work/sc13_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-13 differential report at '$REPORT_JSON'" >&2
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
✅ SC-13 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
