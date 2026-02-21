#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC08_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc08_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc08_semantic_differential_report.json"

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

echo "==> SC-08 contract gate"
echo "state_dir: $STATE_DIR"

# Typed directive payload and validator/coherence contracts.
run_logged_rust "directive_registry_sc08_numeric_bounds_parser_contract" \
    cargo test --lib parses_semantic_float_and_bounds_payloads
run_logged_rust "directive_registry_sc08_len_bounds_parser_contract" \
    cargo test --lib parses_semantic_len_bounds_payloads
run_logged_rust "directive_registry_sc08_string_list_parser_contract" \
    cargo test --lib parses_semantic_string_lists_and_scalars
run_logged_rust "directive_registry_sc08_pattern_parser_contract" \
    cargo test --lib parses_semantic_pattern_payloads
run_logged_rust "directive_registry_sc08_known_directive_contract" \
    cargo test --lib recognizes_known_directives
run_logged_rust "validator_sc08_invalid_payload_contract" \
    cargo test --lib semantic_validator_warns_on_invalid_value_domain_payloads
run_logged_rust "validator_sc08_unsat_enum_regex_contract" \
    cargo test --lib semantic_validator_warns_on_unsatisfiable_enum_regex_intersection
run_logged_rust "validator_sc08_unsat_enum_range_contract" \
    cargo test --lib semantic_validator_warns_on_unsatisfiable_enum_range_intersection
run_logged_rust "validator_sc08_satisfiable_intersection_contract" \
    cargo test --lib semantic_validator_does_not_warn_when_enum_intersection_is_satisfiable

# Parser/stimuli runtime value-domain steering contracts.
run_logged_rust "semantic_usage_sc08_codegen_constraint_guards_contract" \
    cargo test --lib semantic_usage_codegen_emits_value_constraint_guards_for_regex_atoms
run_logged_rust "semantic_usage_sc08_codegen_numeric_range_guard_contract" \
    cargo test --lib semantic_usage_codegen_emits_numeric_range_constraint_guards
run_logged_rust "semantic_usage_sc08_stimuli_enum_constraint_contract" \
    cargo test --lib semantic_usage_stimuli_enum_constraints_filter_regex_sampling
run_logged_rust "semantic_usage_sc08_stimuli_range_constraint_contract" \
    cargo test --lib semantic_usage_stimuli_range_constraints_generate_in_domain_values
run_logged_rust "semantic_usage_sc08_stimuli_len_constraint_contract" \
    cargo test --lib semantic_usage_stimuli_len_constraints_generate_matching_lengths
run_logged_rust "semantic_usage_sc08_stimuli_composed_constraint_contract" \
    cargo test --lib semantic_usage_stimuli_regex_and_enum_constraints_compose

# Shared SC-08 semantic contract corpus (bootstrap + generated).
run_logged_rust "bootstrap_sc08_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc08_contract
run_logged_rust "generated_sc08_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc08_contract

# Differential taxonomy slice: SC-08 comparable corpus should remain parity-clean.
run_logged_rust "differential_sc08_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc08_contract --differential-comparable-only --differential-report-json "target/sc08_contract_gate/work/sc08_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-08 differential report at '$REPORT_JSON'" >&2
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
✅ SC-08 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
