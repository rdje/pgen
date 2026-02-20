#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SC03_CONTRACT_STATE_DIR:-$RUST_DIR/target/sc03_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
REPORT_JSON="$WORK_DIR/sc03_semantic_differential_report.json"

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

echo "==> SC-03 contract gate"
echo "state_dir: $STATE_DIR"

# Typed routing surface and unknown-directive strictness policy checks.
run_logged_rust "directive_registry_contract" \
    cargo test --lib semantic_directive_registry
run_logged_rust "unknown_directive_warn_mode_contract" \
    cargo test --lib semantic_validator_warns_on_unknown_directive_in_warn_mode
run_logged_rust "unknown_directive_strict_mode_contract" \
    cargo test --lib semantic_validator_errors_on_unknown_directive_in_strict_mode
run_logged_rust "strict_warning_code_selection_contract" \
    cargo test --lib semantic_validator_promotes_selected_warning_codes_to_error
run_logged_rust "strict_warning_code_exclusion_contract" \
    cargo test --lib semantic_validator_keeps_unselected_warning_codes_as_warning
run_logged_rust "strict_warning_code_wildcard_contract" \
    cargo test --lib semantic_validator_promotes_all_semantic_warnings_with_wildcard

# Name-aware transform/literal routing guards across parser + stimuli.
run_logged_rust "semantic_usage_codegen_transform_named_contract" \
    cargo test --lib semantic_usage_codegen_applies_canonical_transform_on_regex_atom
run_logged_rust "semantic_usage_codegen_transform_raw_guard_contract" \
    cargo test --lib semantic_usage_codegen_ignores_raw_annotations_for_regex_atom
run_logged_rust "semantic_usage_codegen_transform_named_non_transform_guard_contract" \
    cargo test --lib semantic_usage_codegen_ignores_transformexpr_when_named_non_transform_directive
run_logged_rust "semantic_usage_stimuli_transform_named_contract" \
    cargo test --lib semantic_usage_stimuli_transformexpr_hint_overrides_regex_sampling
run_logged_rust "semantic_usage_stimuli_transform_noncanonical_guard_contract" \
    cargo test --lib semantic_usage_stimuli_noncanonical_transform_does_not_override_regex
run_logged_rust "semantic_usage_stimuli_literal_raw_contract" \
    cargo test --lib semantic_usage_stimuli_raw_quoted_content_returns_literal_hint
run_logged_rust "semantic_usage_stimuli_literal_named_guard_contract" \
    cargo test --lib semantic_usage_stimuli_raw_hint_requires_literalish_directive_when_named

# Shared SC-03 semantic contract corpus (bootstrap + generated).
run_logged_rust "bootstrap_sc03_contract_suite" \
    cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_sc03_contract
run_logged_rust "generated_sc03_contract_suite" \
    cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_sc03_contract

# Differential taxonomy slice: SC-03 comparable corpus should stay parity-clean.
run_logged_rust "differential_sc03_contract_slice" \
    cargo run --features generated_parsers --bin test_runner -- --differential --parser semantic --suite semantic_annotation_sc03_contract --differential-comparable-only --differential-report-json "target/sc03_contract_gate/work/sc03_semantic_differential_report.json"

if [[ ! -f "$REPORT_JSON" ]]; then
    echo "error: missing SC-03 differential report at '$REPORT_JSON'" >&2
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
✅ SC-03 contract gate passed.
Logs: $LOG_DIR
Report: $REPORT_JSON
EOF
