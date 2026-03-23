#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_REGEX_COMBINED_TELEMETRY_CONTRACT_STATE_DIR:-$RUST_DIR/target/regex_combined_telemetry_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

SOTA_EXIT_GATE_SCRIPT="$RUST_DIR/scripts/sota_exit_gate.sh"
SOTA_POLICY_ENV_FILE="${PGEN_REGEX_COMBINED_TELEMETRY_SOTA_POLICY_ENV_FILE:-$RUST_DIR/test_data/grammar_quality/regex_combined_telemetry_lightweight_v0.env}"
EXISTING_SOTA_EXIT_STATE_DIR="${PGEN_REGEX_COMBINED_TELEMETRY_EXISTING_SOTA_EXIT_STATE_DIR:-}"

require_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool '$tool' is not available in PATH" >&2
        exit 1
    fi
}

require_file() {
    local path="$1"
    if [[ ! -f "$path" ]]; then
        echo "error: missing required file '$path'" >&2
        exit 1
    fi
}

require_nonempty_file() {
    local path="$1"
    if [[ ! -s "$path" ]]; then
        echo "error: expected non-empty artifact '$path'" >&2
        exit 1
    fi
}

run_logged_with_env_file() {
    local label="$1"
    local env_file="$2"
    shift 2
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (
        set -a
        # shellcheck disable=SC1090
        source "$env_file"
        set +a
        "$@"
    ) >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        exit 1
    fi
}

extract_summary_value() {
    local path="$1"
    local key="$2"
    awk -v key="$key" 'index($0, key ": ") == 1 { print substr($0, length(key) + 3); found = 1 } END { if (!found) exit 1 }' "$path"
}

assert_equal() {
    local label="$1"
    local expected="$2"
    local observed="$3"
    if [[ "$expected" != "$observed" ]]; then
        echo "error: ${label} mismatch (expected '${expected}', observed '${observed}')" >&2
        exit 1
    fi
}

require_file "$SOTA_EXIT_GATE_SCRIPT"
require_file "$SOTA_POLICY_ENV_FILE"
require_tool jq

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

sota_state_dir="$WORK_DIR/sota_exit_gate"
if [[ -n "$EXISTING_SOTA_EXIT_STATE_DIR" ]]; then
    sota_state_dir="$EXISTING_SOTA_EXIT_STATE_DIR"
else
    run_logged_with_env_file "regex_combined_sota_exit_gate" "$SOTA_POLICY_ENV_FILE" \
        env PGEN_SOTA_EXIT_STATE_DIR="$sota_state_dir" \
        "$SOTA_EXIT_GATE_SCRIPT"
fi

sota_summary_txt="$sota_state_dir/summary.txt"
sota_summary_json="$sota_state_dir/summary.json"
require_nonempty_file "$sota_summary_txt"
require_nonempty_file "$sota_summary_json"

sota_exit_gate_name="$(jq -r '.gate' "$sota_summary_json")"
sota_exit_gate_version="$(jq -r '.version' "$sota_summary_json")"
sota_exit_generated_at_utc="$(jq -r '.generated_at_utc' "$sota_summary_json")"
sota_exit_status="$(jq -r '.status' "$sota_summary_json")"
sota_exit_summary_txt_from_json="$(jq -r '.proof_surfaces.summary_txt' "$sota_summary_json")"
sota_exit_summary_csv_from_json="$(jq -r '.proof_surfaces.summary_csv' "$sota_summary_json")"
sota_exit_summary_json_from_json="$(jq -r '.proof_surfaces.summary_json' "$sota_summary_json")"
sota_exit_required_failures="$(jq -r '.counts.required_failures' "$sota_summary_json")"
sota_exit_informational_failures="$(jq -r '.counts.informational_failures' "$sota_summary_json")"
sota_exit_all_failures="$(jq -r '.counts.all_failures' "$sota_summary_json")"
sota_exit_regex_family_summary_json_from_json="$(jq -r '.proof_surfaces.regex_parser_family_contract_summary_json' "$sota_summary_json")"
sota_exit_regex_family_status_summary_json_from_json="$(jq -r '.proof_surfaces.regex_parser_family_status_summary_json' "$sota_summary_json")"
sota_exit_regex_family_status_contract_summary_json_from_json="$(jq -r '.proof_surfaces.regex_parser_family_status_contract_summary_json' "$sota_summary_json")"
sota_exit_regex_primary_unmet="$(jq -r '.family_status.regex.primary_unmet_closure_criterion' "$sota_summary_json")"
sota_exit_regex_unmet_json="$(jq -cer '.family_status.regex.unmet_closure_criteria' "$sota_summary_json")"
sota_exit_regex_unmet_details_json="$(jq -cer '.family_status.regex.unmet_closure_criteria_details' "$sota_summary_json")"
sota_exit_regex_primary_unmet_detail="$(jq -r '.family_status_contract.regex.primary_unmet_detail_criterion' "$sota_summary_json")"
sota_exit_regex_unmet_detail_json="$(jq -cer '.family_status_contract.regex.unmet_closure_criteria' "$sota_summary_json")"
sota_exit_regex_unmet_detail_details_json="$(jq -cer '.family_status_contract.regex.unmet_closure_criteria_details' "$sota_summary_json")"

assert_equal \
    "SOTA exit gate name" \
    "$(extract_summary_value "$sota_summary_txt" "gate")" \
    "$sota_exit_gate_name"
assert_equal \
    "SOTA exit gate version" \
    "$(extract_summary_value "$sota_summary_txt" "version")" \
    "$sota_exit_gate_version"
assert_equal \
    "SOTA exit generated_at_utc" \
    "$(extract_summary_value "$sota_summary_txt" "generated_at_utc")" \
    "$sota_exit_generated_at_utc"
assert_equal \
    "SOTA exit summary txt path from JSON" \
    "$sota_summary_txt" \
    "$sota_exit_summary_txt_from_json"
assert_equal \
    "SOTA exit summary csv path from JSON" \
    "$(extract_summary_value "$sota_summary_txt" "summary_csv")" \
    "$sota_exit_summary_csv_from_json"
assert_equal \
    "SOTA exit summary json self-path" \
    "$sota_summary_json" \
    "$sota_exit_summary_json_from_json"
assert_equal \
    "SOTA exit status" \
    "passed" \
    "$sota_exit_status"
assert_equal \
    "SOTA exit required failures" \
    "0" \
    "$sota_exit_required_failures"
assert_equal \
    "SOTA exit informational failures" \
    "$(extract_summary_value "$sota_summary_txt" "informational_failures")" \
    "$sota_exit_informational_failures"
assert_equal \
    "SOTA exit all failures" \
    "$(extract_summary_value "$sota_summary_txt" "all_failures")" \
    "$sota_exit_all_failures"

regex_family_summary_txt="$(extract_summary_value "$sota_summary_txt" "regex_parser_family_contract_summary_txt")"
regex_family_summary_json="$(extract_summary_value "$sota_summary_txt" "regex_parser_family_contract_summary_json")"
regex_family_status_summary_txt="$(extract_summary_value "$sota_summary_txt" "regex_parser_family_status_summary_txt")"
regex_family_status_summary_json="$(extract_summary_value "$sota_summary_txt" "regex_parser_family_status_summary_json")"
regex_family_status_contract_summary_txt="$(extract_summary_value "$sota_summary_txt" "regex_parser_family_status_contract_summary_txt")"
regex_family_status_contract_summary_json="$(extract_summary_value "$sota_summary_txt" "regex_parser_family_status_contract_summary_json")"

require_nonempty_file "$regex_family_summary_txt"
require_nonempty_file "$regex_family_summary_json"
require_nonempty_file "$regex_family_status_summary_txt"
require_nonempty_file "$regex_family_status_summary_json"
require_nonempty_file "$regex_family_status_contract_summary_txt"
require_nonempty_file "$regex_family_status_contract_summary_json"

assert_equal \
    "SOTA exit regex family contract summary json path" \
    "$regex_family_summary_json" \
    "$sota_exit_regex_family_summary_json_from_json"
assert_equal \
    "SOTA exit regex family status summary json path" \
    "$regex_family_status_summary_json" \
    "$sota_exit_regex_family_status_summary_json_from_json"
assert_equal \
    "SOTA exit regex family status contract summary json path" \
    "$regex_family_status_contract_summary_json" \
    "$sota_exit_regex_family_status_contract_summary_json_from_json"

regex_family_frontend_overall="$(extract_summary_value "$regex_family_summary_txt" "frontend_regex_overall")"
regex_family_dual_run_overall="$(extract_summary_value "$regex_family_summary_txt" "dual_run_regex_overall")"
regex_family_dual_run_raw_ast_status="$(extract_summary_value "$regex_family_summary_txt" "dual_run_regex_raw_ast_status")"
regex_family_dual_run_perl_rule_count="$(extract_summary_value "$regex_family_summary_txt" "dual_run_regex_perl_rule_count")"
regex_family_dual_run_rust_rule_count="$(extract_summary_value "$regex_family_summary_txt" "dual_run_regex_rust_rule_count")"
regex_family_dual_run_raw_ast_missing_on_perl_count="$(extract_summary_value "$regex_family_summary_txt" "dual_run_regex_raw_ast_missing_on_perl_count")"
regex_family_dual_run_raw_ast_missing_on_rust_count="$(extract_summary_value "$regex_family_summary_txt" "dual_run_regex_raw_ast_missing_on_rust_count")"
regex_family_frontend_state_dir="$(jq -r '.proof_surfaces.frontend_state_dir' "$regex_family_summary_json")"
regex_family_frontend_summary_txt="$(jq -r '.proof_surfaces.frontend_summary_txt' "$regex_family_summary_json")"
regex_family_frontend_summary_csv="$(jq -r '.proof_surfaces.frontend_summary_csv' "$regex_family_summary_json")"
regex_family_dual_run_state_dir="$(jq -r '.proof_surfaces.dual_run_state_dir' "$regex_family_summary_json")"
regex_family_dual_run_summary_txt="$(jq -r '.proof_surfaces.dual_run_summary_txt' "$regex_family_summary_json")"
regex_family_dual_run_summary_csv="$(jq -r '.proof_surfaces.dual_run_summary_csv' "$regex_family_summary_json")"
regex_family_dual_run_summary_json="$(jq -r '.proof_surfaces.dual_run_summary_json' "$regex_family_summary_json")"
regex_family_stimuli_state_dir="$(jq -r '.proof_surfaces.stimuli_state_dir' "$regex_family_summary_json")"
regex_family_stimuli_summary_txt="$(jq -r '.proof_surfaces.stimuli_summary_txt' "$regex_family_summary_json")"
regex_family_stimuli_summary_csv="$(jq -r '.proof_surfaces.stimuli_summary_csv' "$regex_family_summary_json")"
regex_family_stimuli_status="$(extract_summary_value "$regex_family_summary_txt" "stimuli_regex_status")"
regex_family_stimuli_initial_targets="$(extract_summary_value "$regex_family_summary_txt" "stimuli_regex_initial_targets")"
regex_family_stimuli_resolved_targets="$(extract_summary_value "$regex_family_summary_txt" "stimuli_regex_resolved_targets")"
regex_family_stimuli_final_targets="$(extract_summary_value "$regex_family_summary_txt" "stimuli_regex_final_targets")"
regex_family_stimuli_target_attempts="$(extract_summary_value "$regex_family_summary_txt" "stimuli_regex_target_attempts")"
regex_family_stimuli_stage0_successes="$(extract_summary_value "$regex_family_summary_txt" "stimuli_regex_stage0_successes")"
regex_family_stimuli_stage3_successes="$(extract_summary_value "$regex_family_summary_txt" "stimuli_regex_stage3_successes")"

regex_parser_family_status_gate="$(jq -r '.gate' "$regex_family_status_summary_json")"
regex_parser_family_status_gate_version="$(jq -r '.version' "$regex_family_status_summary_json")"
regex_parser_family_status_generated_at_utc="$(jq -r '.generated_at_utc' "$regex_family_status_summary_json")"
regex_parser_family_status_live_tracker_file="$(jq -r '.live_tracker_file' "$regex_family_status_summary_json")"
regex_parser_family_status_status_rule_done="$(jq -r '.status_rule_done' "$regex_family_status_summary_json")"
regex_family_status_regex="$(extract_summary_value "$regex_family_status_summary_txt" "regex_status")"
regex_family_status_regex_tracker_status="$(extract_summary_value "$regex_family_status_summary_txt" "regex_tracker_status")"
regex_family_status_regex_tracker_alignment_ok="$(extract_summary_value "$regex_family_status_summary_txt" "regex_tracker_alignment_ok")"
regex_family_status_regex_unmet_closure_criteria_count="$(extract_summary_value "$regex_family_status_summary_txt" "regex_unmet_closure_criteria_count")"
regex_family_status_regex_primary_unmet_closure_criterion="$(extract_summary_value "$regex_family_status_summary_txt" "regex_primary_unmet_closure_criterion")"
regex_family_status_regex_unmet_closure_criteria_json="$(extract_summary_value "$regex_family_status_summary_txt" "regex_unmet_closure_criteria_json")"
regex_family_status_regex_unmet_closure_criteria_details_json="$(extract_summary_value "$regex_family_status_summary_txt" "regex_unmet_closure_criteria_details_json")"
regex_family_status_regex_closure_criteria_satisfied_count="$(extract_summary_value "$regex_family_status_summary_txt" "regex_closure_criteria_satisfied_count")"
regex_family_status_regex_closure_criteria_total_count="$(extract_summary_value "$regex_family_status_summary_txt" "regex_closure_criteria_total_count")"
regex_family_status_regex_closure_criteria_unsatisfied_count="$(extract_summary_value "$regex_family_status_summary_txt" "regex_closure_criteria_unsatisfied_count")"
regex_family_status_regex_family_contract_summary_txt="$(jq -r '.families[] | select(.family=="regex") | .proof_surfaces.family_contract_summary_txt' "$regex_family_status_summary_json")"
regex_family_status_regex_family_contract_summary_json="$(jq -r '.families[] | select(.family=="regex") | .proof_surfaces.family_contract_summary_json' "$regex_family_status_summary_json")"
regex_family_status_regex_family_contract_green="$(extract_summary_value "$regex_family_status_summary_txt" "regex_family_contract_green")"
regex_family_status_regex_frontend_overall_pass="$(extract_summary_value "$regex_family_status_summary_txt" "regex_frontend_overall_pass")"
regex_family_status_regex_dual_run_overall_pass="$(extract_summary_value "$regex_family_status_summary_txt" "regex_dual_run_overall_pass")"
regex_family_status_regex_dual_run_raw_ast_missing_on_rust_zero="$(extract_summary_value "$regex_family_status_summary_txt" "regex_dual_run_raw_ast_missing_on_rust_zero")"
regex_family_status_regex_stimuli_status_pass="$(extract_summary_value "$regex_family_status_summary_txt" "regex_stimuli_status_pass")"
regex_family_status_regex_stimuli_final_target_debt_zero="$(extract_summary_value "$regex_family_status_summary_txt" "regex_stimuli_final_target_debt_zero")"
regex_family_status_regex_formal_exhaustive_closure_surface_green="$(extract_summary_value "$regex_family_status_summary_txt" "regex_formal_exhaustive_closure_surface_green")"

regex_family_status_contract_gate="$(jq -r '.gate' "$regex_family_status_contract_summary_json")"
regex_family_status_contract_gate_version="$(jq -r '.version' "$regex_family_status_contract_summary_json")"
regex_family_status_contract_generated_at_utc="$(jq -r '.generated_at_utc' "$regex_family_status_contract_summary_json")"
regex_family_status_contract_family_status_state_dir="$(jq -r '.family_status_state_dir' "$regex_family_status_contract_summary_json")"
regex_family_status_contract_family_status_summary_json="$(jq -r '.family_status_summary_json' "$regex_family_status_contract_summary_json")"
regex_family_status_contract_family_status_summary_txt="$(jq -r '.family_status_summary_txt' "$regex_family_status_contract_summary_json")"
regex_family_status_contract_family_count="$(extract_summary_value "$regex_family_status_contract_summary_txt" "family_count")"
regex_family_status_contract_regex_tracker_alignment_ok="$(extract_summary_value "$regex_family_status_contract_summary_txt" "regex_tracker_alignment_ok")"
regex_family_status_contract_regex_false_criteria_count="$(extract_summary_value "$regex_family_status_contract_summary_txt" "regex_false_criteria_count")"
regex_family_status_contract_regex_unmet_details_count="$(extract_summary_value "$regex_family_status_contract_summary_txt" "regex_unmet_details_count")"
regex_family_status_contract_regex_primary_unmet_detail_criterion="$(extract_summary_value "$regex_family_status_contract_summary_txt" "regex_primary_unmet_detail_criterion")"
regex_family_status_contract_regex_unmet_closure_criteria_json="$(extract_summary_value "$regex_family_status_contract_summary_txt" "regex_unmet_closure_criteria_json")"
regex_family_status_contract_regex_unmet_closure_criteria_details_json="$(extract_summary_value "$regex_family_status_contract_summary_txt" "regex_unmet_closure_criteria_details_json")"

assert_equal \
    "Regex family frontend overall" \
    "$regex_family_frontend_overall" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_frontend_overall")"
assert_equal \
    "Regex family dual-run overall" \
    "$regex_family_dual_run_overall" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_overall")"
assert_equal \
    "Regex family dual-run raw_ast status" \
    "$regex_family_dual_run_raw_ast_status" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_raw_ast_status")"
assert_equal \
    "Regex family dual-run perl rule count" \
    "$regex_family_dual_run_perl_rule_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_perl_rule_count")"
assert_equal \
    "Regex family dual-run rust rule count" \
    "$regex_family_dual_run_rust_rule_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_rust_rule_count")"
assert_equal \
    "Regex family dual-run raw_ast missing_on_perl count" \
    "$regex_family_dual_run_raw_ast_missing_on_perl_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_raw_ast_missing_on_perl_count")"
assert_equal \
    "Regex family dual-run raw_ast missing_on_rust count" \
    "$regex_family_dual_run_raw_ast_missing_on_rust_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_raw_ast_missing_on_rust_count")"
assert_equal \
    "Regex family frontend state dir" \
    "$regex_family_frontend_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_frontend_state_dir")"
assert_equal \
    "Regex family frontend summary txt" \
    "$regex_family_frontend_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_frontend_summary_txt")"
assert_equal \
    "Regex family frontend summary csv" \
    "$regex_family_frontend_summary_csv" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_frontend_summary_csv")"
assert_equal \
    "Regex family dual-run state dir" \
    "$regex_family_dual_run_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_state_dir")"
assert_equal \
    "Regex family dual-run summary txt" \
    "$regex_family_dual_run_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_summary_txt")"
assert_equal \
    "Regex family dual-run summary csv" \
    "$regex_family_dual_run_summary_csv" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_summary_csv")"
assert_equal \
    "Regex family dual-run summary json" \
    "$regex_family_dual_run_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_dual_run_summary_json")"
assert_equal \
    "Regex family stimuli state dir" \
    "$regex_family_stimuli_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_state_dir")"
assert_equal \
    "Regex family stimuli summary txt" \
    "$regex_family_stimuli_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_summary_txt")"
assert_equal \
    "Regex family stimuli summary csv" \
    "$regex_family_stimuli_summary_csv" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_summary_csv")"
assert_equal \
    "Regex family stimuli status" \
    "$regex_family_stimuli_status" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_status")"
assert_equal \
    "Regex family stimuli initial targets" \
    "$regex_family_stimuli_initial_targets" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_initial_targets")"
assert_equal \
    "Regex family stimuli resolved targets" \
    "$regex_family_stimuli_resolved_targets" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_resolved_targets")"
assert_equal \
    "Regex family stimuli final targets" \
    "$regex_family_stimuli_final_targets" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_final_targets")"
assert_equal \
    "Regex family stimuli target attempts" \
    "$regex_family_stimuli_target_attempts" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_target_attempts")"
assert_equal \
    "Regex family stimuli stage0 successes" \
    "$regex_family_stimuli_stage0_successes" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_stage0_successes")"
assert_equal \
    "Regex family stimuli stage3 successes" \
    "$regex_family_stimuli_stage3_successes" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_stimuli_stage3_successes")"

assert_equal \
    "Regex family status gate name" \
    "$regex_parser_family_status_gate" \
    "$(extract_summary_value "$sota_summary_txt" "regex_parser_family_status_gate")"
assert_equal \
    "Regex family status gate version" \
    "$regex_parser_family_status_gate_version" \
    "$(extract_summary_value "$sota_summary_txt" "regex_parser_family_status_gate_version")"
assert_equal \
    "Regex family status generated at" \
    "$regex_parser_family_status_generated_at_utc" \
    "$(extract_summary_value "$sota_summary_txt" "regex_parser_family_status_generated_at_utc")"
assert_equal \
    "Regex family status live tracker file" \
    "$regex_parser_family_status_live_tracker_file" \
    "$(extract_summary_value "$sota_summary_txt" "regex_parser_family_status_live_tracker_file")"
assert_equal \
    "Regex family status rule done" \
    "$regex_parser_family_status_status_rule_done" \
    "$(extract_summary_value "$sota_summary_txt" "regex_parser_family_status_status_rule_done")"
assert_equal \
    "Regex family computed status" \
    "$regex_family_status_regex" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex")"
assert_equal \
    "Regex family tracker status" \
    "$regex_family_status_regex_tracker_status" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_tracker_status")"
assert_equal \
    "Regex family tracker alignment" \
    "$regex_family_status_regex_tracker_alignment_ok" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_tracker_alignment_ok")"
assert_equal \
    "Regex family unmet closure criteria count" \
    "$regex_family_status_regex_unmet_closure_criteria_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_unmet_closure_criteria_count")"
assert_equal \
    "Regex family primary unmet closure criterion" \
    "$regex_family_status_regex_primary_unmet_closure_criterion" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_primary_unmet_closure_criterion")"
assert_equal \
    "Regex family unmet closure criteria json" \
    "$regex_family_status_regex_unmet_closure_criteria_json" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_unmet_closure_criteria_json")"
assert_equal \
    "Regex family unmet closure criteria details json" \
    "$regex_family_status_regex_unmet_closure_criteria_details_json" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_unmet_closure_criteria_details_json")"
assert_equal \
    "Regex family closure criteria satisfied count" \
    "$regex_family_status_regex_closure_criteria_satisfied_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_closure_criteria_satisfied_count")"
assert_equal \
    "Regex family closure criteria total count" \
    "$regex_family_status_regex_closure_criteria_total_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_closure_criteria_total_count")"
assert_equal \
    "Regex family closure criteria unsatisfied count" \
    "$regex_family_status_regex_closure_criteria_unsatisfied_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_closure_criteria_unsatisfied_count")"
assert_equal \
    "Regex family status family contract summary txt path" \
    "$regex_family_status_regex_family_contract_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_family_contract_summary_txt")"
assert_equal \
    "Regex family status family contract summary json path" \
    "$regex_family_status_regex_family_contract_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_family_contract_summary_json")"
assert_equal \
    "Regex family contract green criterion" \
    "$regex_family_status_regex_family_contract_green" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_family_contract_green")"
assert_equal \
    "Regex family frontend overall pass criterion" \
    "$regex_family_status_regex_frontend_overall_pass" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_frontend_overall_pass")"
assert_equal \
    "Regex family dual-run overall pass criterion" \
    "$regex_family_status_regex_dual_run_overall_pass" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_dual_run_overall_pass")"
assert_equal \
    "Regex family dual-run raw-AST missing on rust zero criterion" \
    "$regex_family_status_regex_dual_run_raw_ast_missing_on_rust_zero" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_dual_run_raw_ast_missing_on_rust_zero")"
assert_equal \
    "Regex family stimuli status pass criterion" \
    "$regex_family_status_regex_stimuli_status_pass" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_stimuli_status_pass")"
assert_equal \
    "Regex family stimuli final target debt zero criterion" \
    "$regex_family_status_regex_stimuli_final_target_debt_zero" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_stimuli_final_target_debt_zero")"
assert_equal \
    "Regex family formal exhaustive closure surface green criterion" \
    "$regex_family_status_regex_formal_exhaustive_closure_surface_green" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_regex_formal_exhaustive_closure_surface_green")"

assert_equal \
    "Regex family status contract gate name" \
    "$regex_family_status_contract_gate" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_gate")"
assert_equal \
    "Regex family status contract gate version" \
    "$regex_family_status_contract_gate_version" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_gate_version")"
assert_equal \
    "Regex family status contract generated at" \
    "$regex_family_status_contract_generated_at_utc" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_generated_at_utc")"
assert_equal \
    "Regex family status contract family status state dir" \
    "$regex_family_status_contract_family_status_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_family_status_state_dir")"
assert_equal \
    "Regex family status contract family status summary json path" \
    "$regex_family_status_contract_family_status_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_family_status_summary_json")"
assert_equal \
    "Regex family status contract family status summary txt path" \
    "$regex_family_status_contract_family_status_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_family_status_summary_txt")"
assert_equal \
    "Regex family status contract family count" \
    "$regex_family_status_contract_family_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_family_count")"
assert_equal \
    "Regex family status contract tracker alignment" \
    "$regex_family_status_contract_regex_tracker_alignment_ok" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_regex_tracker_alignment_ok")"
assert_equal \
    "Regex family status contract false criteria count" \
    "$regex_family_status_contract_regex_false_criteria_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_regex_false_criteria_count")"
assert_equal \
    "Regex family status contract unmet details count" \
    "$regex_family_status_contract_regex_unmet_details_count" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_regex_unmet_details_count")"
assert_equal \
    "Regex family status contract primary unmet detail criterion" \
    "$regex_family_status_contract_regex_primary_unmet_detail_criterion" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_regex_primary_unmet_detail_criterion")"
assert_equal \
    "Regex family status contract unmet criteria json" \
    "$regex_family_status_contract_regex_unmet_closure_criteria_json" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_regex_unmet_closure_criteria_json")"
assert_equal \
    "Regex family status contract unmet criteria details json" \
    "$regex_family_status_contract_regex_unmet_closure_criteria_details_json" \
    "$(extract_summary_value "$sota_summary_txt" "regex_family_status_contract_regex_unmet_closure_criteria_details_json")"

assert_equal \
    "SOTA exit regex primary unmet closure criterion" \
    "$regex_family_status_regex_primary_unmet_closure_criterion" \
    "$sota_exit_regex_primary_unmet"
assert_equal \
    "SOTA exit regex unmet closure criteria json" \
    "$regex_family_status_regex_unmet_closure_criteria_json" \
    "$sota_exit_regex_unmet_json"
assert_equal \
    "SOTA exit regex unmet closure criteria details json" \
    "$regex_family_status_regex_unmet_closure_criteria_details_json" \
    "$sota_exit_regex_unmet_details_json"
assert_equal \
    "SOTA exit regex primary unmet detail criterion" \
    "$regex_family_status_contract_regex_primary_unmet_detail_criterion" \
    "$sota_exit_regex_primary_unmet_detail"
assert_equal \
    "SOTA exit regex unmet detail closure criteria json" \
    "$regex_family_status_contract_regex_unmet_closure_criteria_json" \
    "$sota_exit_regex_unmet_detail_json"
assert_equal \
    "SOTA exit regex unmet detail closure criteria details json" \
    "$regex_family_status_contract_regex_unmet_closure_criteria_details_json" \
    "$sota_exit_regex_unmet_detail_details_json"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "Regex Combined Telemetry Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "sota_state_dir: $sota_state_dir"
    echo "sota_policy_env_file: $SOTA_POLICY_ENV_FILE"
    echo "sota_exit_summary_txt: $sota_summary_txt"
    echo "sota_exit_summary_json: $sota_summary_json"
    echo "sota_exit_gate: $sota_exit_gate_name"
    echo "sota_exit_gate_version: $sota_exit_gate_version"
    echo "sota_exit_generated_at_utc: $sota_exit_generated_at_utc"
    echo "sota_exit_status: $sota_exit_status"
    echo "sota_exit_required_failures: $sota_exit_required_failures"
    echo "sota_exit_informational_failures: $sota_exit_informational_failures"
    echo "sota_exit_all_failures: $sota_exit_all_failures"
    echo "regex_parser_family_contract_summary_txt: $regex_family_summary_txt"
    echo "regex_parser_family_contract_summary_json: $regex_family_summary_json"
    echo "regex_parser_family_status_summary_txt: $regex_family_status_summary_txt"
    echo "regex_parser_family_status_summary_json: $regex_family_status_summary_json"
    echo "regex_parser_family_status_gate: $regex_parser_family_status_gate"
    echo "regex_parser_family_status_gate_version: $regex_parser_family_status_gate_version"
    echo "regex_parser_family_status_generated_at_utc: $regex_parser_family_status_generated_at_utc"
    echo "regex_parser_family_status_live_tracker_file: $regex_parser_family_status_live_tracker_file"
    echo "regex_parser_family_status_status_rule_done: $regex_parser_family_status_status_rule_done"
    echo "regex_parser_family_status_contract_summary_txt: $regex_family_status_contract_summary_txt"
    echo "regex_parser_family_status_contract_summary_json: $regex_family_status_contract_summary_json"
    echo "regex_family_frontend_state_dir: $regex_family_frontend_state_dir"
    echo "regex_family_frontend_summary_txt: $regex_family_frontend_summary_txt"
    echo "regex_family_frontend_summary_csv: $regex_family_frontend_summary_csv"
    echo "regex_family_dual_run_state_dir: $regex_family_dual_run_state_dir"
    echo "regex_family_dual_run_summary_txt: $regex_family_dual_run_summary_txt"
    echo "regex_family_dual_run_summary_csv: $regex_family_dual_run_summary_csv"
    echo "regex_family_dual_run_summary_json: $regex_family_dual_run_summary_json"
    echo "regex_family_stimuli_state_dir: $regex_family_stimuli_state_dir"
    echo "regex_family_stimuli_summary_txt: $regex_family_stimuli_summary_txt"
    echo "regex_family_stimuli_summary_csv: $regex_family_stimuli_summary_csv"
    echo "regex_family_status_contract_gate: $regex_family_status_contract_gate"
    echo "regex_family_status_contract_gate_version: $regex_family_status_contract_gate_version"
    echo "regex_family_status_contract_generated_at_utc: $regex_family_status_contract_generated_at_utc"
    echo "regex_family_status_contract_family_status_state_dir: $regex_family_status_contract_family_status_state_dir"
    echo "regex_family_status_contract_family_status_summary_json: $regex_family_status_contract_family_status_summary_json"
    echo "regex_family_status_contract_family_status_summary_txt: $regex_family_status_contract_family_status_summary_txt"
    echo "regex_family_frontend_overall: $regex_family_frontend_overall"
    echo "regex_family_dual_run_overall: $regex_family_dual_run_overall"
    echo "regex_family_dual_run_raw_ast_status: $regex_family_dual_run_raw_ast_status"
    echo "regex_family_dual_run_perl_rule_count: $regex_family_dual_run_perl_rule_count"
    echo "regex_family_dual_run_rust_rule_count: $regex_family_dual_run_rust_rule_count"
    echo "regex_family_dual_run_raw_ast_missing_on_perl_count: $regex_family_dual_run_raw_ast_missing_on_perl_count"
    echo "regex_family_dual_run_raw_ast_missing_on_rust_count: $regex_family_dual_run_raw_ast_missing_on_rust_count"
    echo "regex_family_stimuli_status: $regex_family_stimuli_status"
    echo "regex_family_stimuli_initial_targets: $regex_family_stimuli_initial_targets"
    echo "regex_family_stimuli_resolved_targets: $regex_family_stimuli_resolved_targets"
    echo "regex_family_stimuli_final_targets: $regex_family_stimuli_final_targets"
    echo "regex_family_stimuli_target_attempts: $regex_family_stimuli_target_attempts"
    echo "regex_family_stimuli_stage0_successes: $regex_family_stimuli_stage0_successes"
    echo "regex_family_stimuli_stage3_successes: $regex_family_stimuli_stage3_successes"
    echo "regex_family_status_regex: $regex_family_status_regex"
    echo "regex_family_status_regex_tracker_status: $regex_family_status_regex_tracker_status"
    echo "regex_family_status_regex_tracker_alignment_ok: $regex_family_status_regex_tracker_alignment_ok"
    echo "regex_family_status_regex_unmet_closure_criteria_count: $regex_family_status_regex_unmet_closure_criteria_count"
    echo "regex_family_status_regex_primary_unmet_closure_criterion: $regex_family_status_regex_primary_unmet_closure_criterion"
    echo "regex_family_status_regex_unmet_closure_criteria_json: $regex_family_status_regex_unmet_closure_criteria_json"
    echo "regex_family_status_regex_unmet_closure_criteria_details_json: $regex_family_status_regex_unmet_closure_criteria_details_json"
    echo "regex_family_status_regex_closure_criteria_satisfied_count: $regex_family_status_regex_closure_criteria_satisfied_count"
    echo "regex_family_status_regex_closure_criteria_total_count: $regex_family_status_regex_closure_criteria_total_count"
    echo "regex_family_status_regex_closure_criteria_unsatisfied_count: $regex_family_status_regex_closure_criteria_unsatisfied_count"
    echo "regex_family_status_regex_family_contract_summary_txt: $regex_family_status_regex_family_contract_summary_txt"
    echo "regex_family_status_regex_family_contract_summary_json: $regex_family_status_regex_family_contract_summary_json"
    echo "regex_family_status_regex_family_contract_green: $regex_family_status_regex_family_contract_green"
    echo "regex_family_status_regex_frontend_overall_pass: $regex_family_status_regex_frontend_overall_pass"
    echo "regex_family_status_regex_dual_run_overall_pass: $regex_family_status_regex_dual_run_overall_pass"
    echo "regex_family_status_regex_dual_run_raw_ast_missing_on_rust_zero: $regex_family_status_regex_dual_run_raw_ast_missing_on_rust_zero"
    echo "regex_family_status_regex_stimuli_status_pass: $regex_family_status_regex_stimuli_status_pass"
    echo "regex_family_status_regex_stimuli_final_target_debt_zero: $regex_family_status_regex_stimuli_final_target_debt_zero"
    echo "regex_family_status_regex_formal_exhaustive_closure_surface_green: $regex_family_status_regex_formal_exhaustive_closure_surface_green"
    echo "regex_family_status_contract_family_count: $regex_family_status_contract_family_count"
    echo "regex_family_status_contract_regex_tracker_alignment_ok: $regex_family_status_contract_regex_tracker_alignment_ok"
    echo "regex_family_status_contract_regex_false_criteria_count: $regex_family_status_contract_regex_false_criteria_count"
    echo "regex_family_status_contract_regex_unmet_details_count: $regex_family_status_contract_regex_unmet_details_count"
    echo "regex_family_status_contract_regex_primary_unmet_detail_criterion: $regex_family_status_contract_regex_primary_unmet_detail_criterion"
    echo "regex_family_status_contract_regex_unmet_closure_criteria_json: $regex_family_status_contract_regex_unmet_closure_criteria_json"
    echo "regex_family_status_contract_regex_unmet_closure_criteria_details_json: $regex_family_status_contract_regex_unmet_closure_criteria_details_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "regex_combined_telemetry_contract_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg sota_state_dir "$sota_state_dir" \
    --arg sota_summary_txt "$sota_summary_txt" \
    --arg sota_summary_json "$sota_summary_json" \
    --arg sota_policy_env_file "$SOTA_POLICY_ENV_FILE" \
    --arg sota_exit_gate "$sota_exit_gate_name" \
    --argjson sota_exit_gate_version "$sota_exit_gate_version" \
    --arg sota_exit_generated_at_utc "$sota_exit_generated_at_utc" \
    --arg sota_exit_status "$sota_exit_status" \
    --argjson sota_exit_required_failures "$sota_exit_required_failures" \
    --argjson sota_exit_informational_failures "$sota_exit_informational_failures" \
    --argjson sota_exit_all_failures "$sota_exit_all_failures" \
    --arg regex_parser_family_contract_summary_txt "$regex_family_summary_txt" \
    --arg regex_parser_family_contract_summary_json "$regex_family_summary_json" \
    --arg regex_parser_family_status_summary_txt "$regex_family_status_summary_txt" \
    --arg regex_parser_family_status_summary_json "$regex_family_status_summary_json" \
    --arg regex_parser_family_status_contract_summary_txt "$regex_family_status_contract_summary_txt" \
    --arg regex_parser_family_status_contract_summary_json "$regex_family_status_contract_summary_json" \
    --arg regex_family_frontend_state_dir "$regex_family_frontend_state_dir" \
    --arg regex_family_frontend_summary_txt "$regex_family_frontend_summary_txt" \
    --arg regex_family_frontend_summary_csv "$regex_family_frontend_summary_csv" \
    --arg regex_family_dual_run_state_dir "$regex_family_dual_run_state_dir" \
    --arg regex_family_dual_run_summary_txt "$regex_family_dual_run_summary_txt" \
    --arg regex_family_dual_run_summary_csv "$regex_family_dual_run_summary_csv" \
    --arg regex_family_dual_run_summary_json "$regex_family_dual_run_summary_json" \
    --arg regex_family_stimuli_state_dir "$regex_family_stimuli_state_dir" \
    --arg regex_family_stimuli_summary_txt "$regex_family_stimuli_summary_txt" \
    --arg regex_family_stimuli_summary_csv "$regex_family_stimuli_summary_csv" \
    --arg regex_family_frontend_overall "$regex_family_frontend_overall" \
    --arg regex_family_dual_run_overall "$regex_family_dual_run_overall" \
    --arg regex_family_dual_run_raw_ast_status "$regex_family_dual_run_raw_ast_status" \
    --argjson regex_family_dual_run_perl_rule_count "$regex_family_dual_run_perl_rule_count" \
    --argjson regex_family_dual_run_rust_rule_count "$regex_family_dual_run_rust_rule_count" \
    --argjson regex_family_dual_run_raw_ast_missing_on_perl_count "$regex_family_dual_run_raw_ast_missing_on_perl_count" \
    --argjson regex_family_dual_run_raw_ast_missing_on_rust_count "$regex_family_dual_run_raw_ast_missing_on_rust_count" \
    --arg regex_family_stimuli_status "$regex_family_stimuli_status" \
    --argjson regex_family_stimuli_initial_targets "$regex_family_stimuli_initial_targets" \
    --argjson regex_family_stimuli_resolved_targets "$regex_family_stimuli_resolved_targets" \
    --argjson regex_family_stimuli_final_targets "$regex_family_stimuli_final_targets" \
    --argjson regex_family_stimuli_target_attempts "$regex_family_stimuli_target_attempts" \
    --argjson regex_family_stimuli_stage0_successes "$regex_family_stimuli_stage0_successes" \
    --argjson regex_family_stimuli_stage3_successes "$regex_family_stimuli_stage3_successes" \
    --arg regex_parser_family_status_gate "$regex_parser_family_status_gate" \
    --argjson regex_parser_family_status_gate_version "$regex_parser_family_status_gate_version" \
    --arg regex_parser_family_status_generated_at_utc "$regex_parser_family_status_generated_at_utc" \
    --arg regex_parser_family_status_live_tracker_file "$regex_parser_family_status_live_tracker_file" \
    --arg regex_parser_family_status_status_rule_done "$regex_parser_family_status_status_rule_done" \
    --arg regex_family_status_regex "$regex_family_status_regex" \
    --arg regex_family_status_regex_tracker_status "$regex_family_status_regex_tracker_status" \
    --argjson regex_family_status_regex_tracker_alignment_ok "$regex_family_status_regex_tracker_alignment_ok" \
    --argjson regex_family_status_regex_unmet_closure_criteria_count "$regex_family_status_regex_unmet_closure_criteria_count" \
    --arg regex_family_status_regex_primary_unmet_closure_criterion "$regex_family_status_regex_primary_unmet_closure_criterion" \
    --argjson regex_family_status_regex_unmet_closure_criteria_json "$regex_family_status_regex_unmet_closure_criteria_json" \
    --argjson regex_family_status_regex_unmet_closure_criteria_details_json "$regex_family_status_regex_unmet_closure_criteria_details_json" \
    --argjson regex_family_status_regex_closure_criteria_satisfied_count "$regex_family_status_regex_closure_criteria_satisfied_count" \
    --argjson regex_family_status_regex_closure_criteria_total_count "$regex_family_status_regex_closure_criteria_total_count" \
    --argjson regex_family_status_regex_closure_criteria_unsatisfied_count "$regex_family_status_regex_closure_criteria_unsatisfied_count" \
    --arg regex_family_status_regex_family_contract_summary_txt "$regex_family_status_regex_family_contract_summary_txt" \
    --arg regex_family_status_regex_family_contract_summary_json "$regex_family_status_regex_family_contract_summary_json" \
    --argjson regex_family_status_regex_family_contract_green "$regex_family_status_regex_family_contract_green" \
    --argjson regex_family_status_regex_frontend_overall_pass "$regex_family_status_regex_frontend_overall_pass" \
    --argjson regex_family_status_regex_dual_run_overall_pass "$regex_family_status_regex_dual_run_overall_pass" \
    --argjson regex_family_status_regex_dual_run_raw_ast_missing_on_rust_zero "$regex_family_status_regex_dual_run_raw_ast_missing_on_rust_zero" \
    --argjson regex_family_status_regex_stimuli_status_pass "$regex_family_status_regex_stimuli_status_pass" \
    --argjson regex_family_status_regex_stimuli_final_target_debt_zero "$regex_family_status_regex_stimuli_final_target_debt_zero" \
    --argjson regex_family_status_regex_formal_exhaustive_closure_surface_green "$regex_family_status_regex_formal_exhaustive_closure_surface_green" \
    --arg regex_family_status_contract_gate "$regex_family_status_contract_gate" \
    --argjson regex_family_status_contract_gate_version "$regex_family_status_contract_gate_version" \
    --arg regex_family_status_contract_generated_at_utc "$regex_family_status_contract_generated_at_utc" \
    --arg regex_family_status_contract_family_status_state_dir "$regex_family_status_contract_family_status_state_dir" \
    --arg regex_family_status_contract_family_status_summary_json "$regex_family_status_contract_family_status_summary_json" \
    --arg regex_family_status_contract_family_status_summary_txt "$regex_family_status_contract_family_status_summary_txt" \
    --argjson regex_family_status_contract_family_count "$regex_family_status_contract_family_count" \
    --argjson regex_family_status_contract_regex_tracker_alignment_ok "$regex_family_status_contract_regex_tracker_alignment_ok" \
    --argjson regex_family_status_contract_regex_false_criteria_count "$regex_family_status_contract_regex_false_criteria_count" \
    --argjson regex_family_status_contract_regex_unmet_details_count "$regex_family_status_contract_regex_unmet_details_count" \
    --arg regex_family_status_contract_regex_primary_unmet_detail_criterion "$regex_family_status_contract_regex_primary_unmet_detail_criterion" \
    --argjson regex_family_status_contract_regex_unmet_closure_criteria_json "$regex_family_status_contract_regex_unmet_closure_criteria_json" \
    --argjson regex_family_status_contract_regex_unmet_closure_criteria_details_json "$regex_family_status_contract_regex_unmet_closure_criteria_details_json" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      sota: {
        state_dir: $sota_state_dir,
        summary_txt: $sota_summary_txt,
        summary_json: $sota_summary_json,
        gate: $sota_exit_gate,
        gate_version: $sota_exit_gate_version,
        generated_at_utc: $sota_exit_generated_at_utc,
        status: $sota_exit_status,
        counts: {
          required_failures: $sota_exit_required_failures,
          informational_failures: $sota_exit_informational_failures,
          all_failures: $sota_exit_all_failures
        },
        policy_env_file: $sota_policy_env_file
      },
      proof_surfaces: {
        sota_exit_summary_txt: $sota_summary_txt,
        sota_exit_summary_json: $sota_summary_json,
        regex_parser_family_contract_summary_txt: $regex_parser_family_contract_summary_txt,
        regex_parser_family_contract_summary_json: $regex_parser_family_contract_summary_json,
        regex_parser_family_status_summary_txt: $regex_parser_family_status_summary_txt,
        regex_parser_family_status_summary_json: $regex_parser_family_status_summary_json,
        regex_parser_family_status_contract_summary_txt: $regex_parser_family_status_contract_summary_txt,
        regex_parser_family_status_contract_summary_json: $regex_parser_family_status_contract_summary_json
      },
      family_contract: {
        frontend_overall: $regex_family_frontend_overall,
        dual_run_overall: $regex_family_dual_run_overall,
        dual_run_raw_ast_status: $regex_family_dual_run_raw_ast_status,
        dual_run_perl_rule_count: $regex_family_dual_run_perl_rule_count,
        dual_run_rust_rule_count: $regex_family_dual_run_rust_rule_count,
        dual_run_raw_ast_missing_on_perl_count: $regex_family_dual_run_raw_ast_missing_on_perl_count,
        dual_run_raw_ast_missing_on_rust_count: $regex_family_dual_run_raw_ast_missing_on_rust_count,
        stimuli_status: $regex_family_stimuli_status,
        stimuli_initial_targets: $regex_family_stimuli_initial_targets,
        stimuli_resolved_targets: $regex_family_stimuli_resolved_targets,
        stimuli_final_targets: $regex_family_stimuli_final_targets,
        stimuli_target_attempts: $regex_family_stimuli_target_attempts,
        stimuli_stage0_successes: $regex_family_stimuli_stage0_successes,
        stimuli_stage3_successes: $regex_family_stimuli_stage3_successes,
        proof_surfaces: {
          frontend_state_dir: $regex_family_frontend_state_dir,
          frontend_summary_txt: $regex_family_frontend_summary_txt,
          frontend_summary_csv: $regex_family_frontend_summary_csv,
          dual_run_state_dir: $regex_family_dual_run_state_dir,
          dual_run_summary_txt: $regex_family_dual_run_summary_txt,
          dual_run_summary_csv: $regex_family_dual_run_summary_csv,
          dual_run_summary_json: $regex_family_dual_run_summary_json,
          stimuli_state_dir: $regex_family_stimuli_state_dir,
          stimuli_summary_txt: $regex_family_stimuli_summary_txt,
          stimuli_summary_csv: $regex_family_stimuli_summary_csv
        }
      },
      family_status: {
        gate: $regex_parser_family_status_gate,
        version: $regex_parser_family_status_gate_version,
        generated_at_utc: $regex_parser_family_status_generated_at_utc,
        live_tracker_file: $regex_parser_family_status_live_tracker_file,
        status_rule_done: $regex_parser_family_status_status_rule_done,
        family: {
          name: "regex",
          computed_status: $regex_family_status_regex,
          live_tracker_status: $regex_family_status_regex_tracker_status,
          tracker_alignment_ok: $regex_family_status_regex_tracker_alignment_ok,
          unmet_closure_criteria_count: $regex_family_status_regex_unmet_closure_criteria_count,
          primary_unmet_closure_criterion: $regex_family_status_regex_primary_unmet_closure_criterion,
          unmet_closure_criteria: $regex_family_status_regex_unmet_closure_criteria_json,
          unmet_closure_criteria_details: $regex_family_status_regex_unmet_closure_criteria_details_json,
          closure_criteria_satisfied_count: $regex_family_status_regex_closure_criteria_satisfied_count,
          closure_criteria_total_count: $regex_family_status_regex_closure_criteria_total_count,
          closure_criteria_unsatisfied_count: $regex_family_status_regex_closure_criteria_unsatisfied_count,
          proof_surfaces: {
            family_contract_summary_txt: $regex_family_status_regex_family_contract_summary_txt,
            family_contract_summary_json: $regex_family_status_regex_family_contract_summary_json
          },
          criteria: {
            family_contract_green: $regex_family_status_regex_family_contract_green,
            frontend_overall_pass: $regex_family_status_regex_frontend_overall_pass,
            dual_run_overall_pass: $regex_family_status_regex_dual_run_overall_pass,
            dual_run_raw_ast_missing_on_rust_zero: $regex_family_status_regex_dual_run_raw_ast_missing_on_rust_zero,
            stimuli_status_pass: $regex_family_status_regex_stimuli_status_pass,
            stimuli_final_target_debt_zero: $regex_family_status_regex_stimuli_final_target_debt_zero,
            formal_exhaustive_closure_surface_green: $regex_family_status_regex_formal_exhaustive_closure_surface_green
          }
        }
      },
      family_status_contract: {
        gate: $regex_family_status_contract_gate,
        version: $regex_family_status_contract_gate_version,
        generated_at_utc: $regex_family_status_contract_generated_at_utc,
        family_status_state_dir: $regex_family_status_contract_family_status_state_dir,
        family_status_summary_json: $regex_family_status_contract_family_status_summary_json,
        family_status_summary_txt: $regex_family_status_contract_family_status_summary_txt,
        family_count: $regex_family_status_contract_family_count,
        family: {
          name: "regex",
          tracker_alignment_ok: $regex_family_status_contract_regex_tracker_alignment_ok,
          false_criteria_count: $regex_family_status_contract_regex_false_criteria_count,
          unmet_details_count: $regex_family_status_contract_regex_unmet_details_count,
          primary_unmet_detail_criterion: $regex_family_status_contract_regex_primary_unmet_detail_criterion,
          unmet_closure_criteria: $regex_family_status_contract_regex_unmet_closure_criteria_json,
          unmet_closure_criteria_details: $regex_family_status_contract_regex_unmet_closure_criteria_details_json
        }
      }
    }' >"$SUMMARY_JSON"

require_nonempty_file "$SUMMARY_JSON"

echo "✅ Regex combined telemetry contract gate passed."
