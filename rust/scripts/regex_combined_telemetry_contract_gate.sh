#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_REGEX_COMBINED_TELEMETRY_CONTRACT_STATE_DIR:-$RUST_DIR/target/regex_combined_telemetry_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

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
require_nonempty_file "$sota_summary_txt"
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

{
    echo "Regex Combined Telemetry Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "sota_state_dir: $sota_state_dir"
    echo "sota_policy_env_file: $SOTA_POLICY_ENV_FILE"
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

echo "✅ Regex combined telemetry contract gate passed."
