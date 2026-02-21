#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC05_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc05_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc05_semantic_differential_report.json"

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

echo "==> SC-05 contract gate"
echo "state_dir: $STATE_DIR"

# Typed directive payload and validator/coherence contracts.
run_logged_rust "directive_registry_sc05_priority_parser_contract" \
    cargo test --lib parses_semantic_branch_priority_vectors
run_logged_rust "directive_registry_sc05_associativity_parser_contract" \
    cargo test --lib parses_semantic_associativity_values
run_logged_rust "directive_registry_sc05_known_directive_contract" \
    cargo test --lib recognizes_known_directives
run_logged_rust "validator_sc05_invalid_priority_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_priority_payload
run_logged_rust "validator_sc05_invalid_associativity_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_associativity_payload
run_logged_rust "validator_sc05_priority_precedence_conflict_contract" \
    cargo test --lib semantic_validator_warns_when_priority_and_precedence_both_present
run_logged_rust "validator_sc05_duplicate_directive_last_wins_contract" \
    cargo test --lib semantic_validator_warns_on_duplicate_directive_override_contract

# Parser/stimuli runtime precedence and associativity contracts.
run_logged_rust "semantic_usage_sc05_codegen_priority_parser_contract" \
    cargo test --lib semantic_usage_codegen_parses_branch_priorities
run_logged_rust "semantic_usage_sc05_codegen_associativity_parser_contract" \
    cargo test --lib semantic_usage_codegen_parses_associativity_directive
run_logged_rust "semantic_usage_sc05_codegen_priority_override_contract" \
    cargo test --lib semantic_usage_codegen_priority_overrides_precedence_regardless_of_order
run_logged_rust "semantic_usage_sc05_codegen_associativity_last_wins_contract" \
    cargo test --lib semantic_usage_codegen_last_associativity_directive_wins
run_logged_rust "semantic_usage_sc05_codegen_tiebreak_contract" \
    cargo test --lib semantic_usage_codegen_emits_priority_and_associativity_tiebreak_logic
run_logged_rust "semantic_usage_sc05_stimuli_priority_bias_contract" \
    cargo test --lib semantic_priority_directive_biases_branch_selection
run_logged_rust "semantic_usage_sc05_stimuli_priority_override_contract" \
    cargo test --lib semantic_priority_overrides_precedence_regardless_of_order
run_logged_rust "semantic_usage_sc05_stimuli_associativity_bias_contract" \
    cargo test --lib semantic_associativity_right_biases_ties_to_later_branches

# Shared SC-05 semantic contract corpus (bootstrap + generated).
run_logged_rust "bootstrap_sc05_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc05_contract
run_logged_rust "generated_sc05_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc05_contract

# Differential taxonomy slice: SC-05 comparable corpus should remain parity-clean.
run_logged_rust "differential_sc05_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc05_contract --differential-comparable-only --differential-report-json "target/sc05_contract_gate/work/sc05_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-05 differential report at '$REPORT_JSON'" >&2
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
✅ SC-05 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
