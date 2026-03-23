#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_VHDL_COMBINED_TELEMETRY_CONTRACT_STATE_DIR:-$RUST_DIR/target/vhdl_combined_telemetry_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

SOTA_EXIT_GATE_SCRIPT="$RUST_DIR/scripts/sota_exit_gate.sh"
SOTA_POLICY_ENV_FILE="${PGEN_VHDL_COMBINED_TELEMETRY_SOTA_POLICY_ENV_FILE:-$RUST_DIR/test_data/grammar_quality/vhdl_combined_telemetry_lightweight_v0.env}"
EXISTING_SOTA_EXIT_STATE_DIR="${PGEN_VHDL_COMBINED_TELEMETRY_EXISTING_SOTA_EXIT_STATE_DIR:-}"

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
    run_logged_with_env_file "vhdl_combined_sota_exit_gate" "$SOTA_POLICY_ENV_FILE" \
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
sota_exit_vhdl_family_summary_json_from_json="$(jq -r '.proof_surfaces.vhdl_parser_family_contract_summary_json' "$sota_summary_json")"
sota_exit_vhdl_family_status_summary_json_from_json="$(jq -r '.proof_surfaces.vhdl_parser_family_status_summary_json' "$sota_summary_json")"
sota_exit_vhdl_family_status_contract_summary_json_from_json="$(jq -r '.proof_surfaces.vhdl_parser_family_status_contract_summary_json' "$sota_summary_json")"
sota_exit_vhdl_primary_unmet="$(jq -r '.family_status.vhdl.primary_unmet_closure_criterion' "$sota_summary_json")"
sota_exit_vhdl_unmet_json="$(jq -cer '.family_status.vhdl.unmet_closure_criteria' "$sota_summary_json")"
sota_exit_vhdl_unmet_details_json="$(jq -cer '.family_status.vhdl.unmet_closure_criteria_details' "$sota_summary_json")"
sota_exit_vhdl_primary_unmet_detail="$(jq -r '.family_status_contract.vhdl.primary_unmet_detail_criterion' "$sota_summary_json")"
sota_exit_vhdl_unmet_detail_json="$(jq -cer '.family_status_contract.vhdl.unmet_closure_criteria' "$sota_summary_json")"
sota_exit_vhdl_unmet_detail_details_json="$(jq -cer '.family_status_contract.vhdl.unmet_closure_criteria_details' "$sota_summary_json")"

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

vhdl_family_summary_txt="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_contract_summary_txt")"
vhdl_family_summary_json="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_contract_summary_json")"
vhdl_family_status_summary_txt="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_summary_txt")"
vhdl_family_status_summary_json="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_summary_json")"
vhdl_family_status_contract_summary_txt="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_contract_summary_txt")"
vhdl_family_status_contract_summary_json="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_contract_summary_json")"

require_nonempty_file "$vhdl_family_summary_txt"
require_nonempty_file "$vhdl_family_summary_json"
require_nonempty_file "$vhdl_family_status_summary_txt"
require_nonempty_file "$vhdl_family_status_summary_json"
require_nonempty_file "$vhdl_family_status_contract_summary_txt"
require_nonempty_file "$vhdl_family_status_contract_summary_json"

assert_equal \
    "SOTA exit VHDL family contract summary json path" \
    "$vhdl_family_summary_json" \
    "$sota_exit_vhdl_family_summary_json_from_json"
assert_equal \
    "SOTA exit VHDL family status summary json path" \
    "$vhdl_family_status_summary_json" \
    "$sota_exit_vhdl_family_status_summary_json_from_json"
assert_equal \
    "SOTA exit VHDL family status contract summary json path" \
    "$vhdl_family_status_contract_summary_json" \
    "$sota_exit_vhdl_family_status_contract_summary_json_from_json"
vhdl_parser_family_status_gate="$(jq -r '.gate' "$vhdl_family_status_summary_json")"
vhdl_parser_family_status_gate_version="$(jq -r '.version' "$vhdl_family_status_summary_json")"
vhdl_parser_family_status_generated_at_utc="$(jq -r '.generated_at_utc' "$vhdl_family_status_summary_json")"
vhdl_parser_family_status_live_tracker_file="$(jq -r '.live_tracker_file' "$vhdl_family_status_summary_json")"
vhdl_parser_family_status_status_rule_done="$(jq -r '.status_rule_done' "$vhdl_family_status_summary_json")"
assert_equal \
    "VHDL family status gate name" \
    "$vhdl_parser_family_status_gate" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_gate")"
assert_equal \
    "VHDL family status gate version" \
    "$vhdl_parser_family_status_gate_version" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_gate_version")"
assert_equal \
    "VHDL family status generated at" \
    "$vhdl_parser_family_status_generated_at_utc" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_generated_at_utc")"
assert_equal \
    "VHDL family status live tracker file" \
    "$vhdl_parser_family_status_live_tracker_file" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_live_tracker_file")"
assert_equal \
    "VHDL family status status-rule done" \
    "$vhdl_parser_family_status_status_rule_done" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_status_rule_done")"

vhdl_quality_closed_loop_initial_status="$(extract_summary_value "$vhdl_family_summary_txt" "quality_closed_loop_initial_status")"
vhdl_quality_closed_loop_replay_status="$(extract_summary_value "$vhdl_family_summary_txt" "quality_closed_loop_replay_status")"
vhdl_quality_closed_loop_initial_targets="$(extract_summary_value "$vhdl_family_summary_txt" "quality_closed_loop_initial_targets")"
vhdl_quality_closed_loop_replay_targets="$(extract_summary_value "$vhdl_family_summary_txt" "quality_closed_loop_replay_targets")"
vhdl_family_quality_state_dir="$(jq -r '.proof_surfaces.quality_state_dir' "$vhdl_family_summary_json")"
vhdl_family_quality_summary_txt="$(jq -r '.proof_surfaces.quality_summary_txt' "$vhdl_family_summary_json")"
vhdl_family_quality_realistic_report_json="$(jq -r '.proof_surfaces.quality_realistic_report_json' "$vhdl_family_summary_json")"
vhdl_family_quality_parseability_report_json="$(jq -r '.proof_surfaces.quality_parseability_report_json' "$vhdl_family_summary_json")"
vhdl_family_strict_promotion_state_dir="$(jq -r '.proof_surfaces.strict_promotion_state_dir' "$vhdl_family_summary_json")"
vhdl_family_strict_promotion_summary_txt="$(jq -r '.proof_surfaces.strict_promotion_summary_txt' "$vhdl_family_summary_json")"
vhdl_family_strict_promotion_report_json="$(jq -r '.proof_surfaces.strict_promotion_report_json' "$vhdl_family_summary_json")"
vhdl_quality_parseability_generation_attempts_total="$(extract_summary_value "$vhdl_family_summary_txt" "quality_parseability_generation_attempts_total")"
vhdl_quality_parseability_generation_rejected_total="$(extract_summary_value "$vhdl_family_summary_txt" "quality_parseability_generation_rejected_total")"
vhdl_quality_realistic_cases_executed="$(extract_summary_value "$vhdl_family_summary_txt" "quality_realistic_cases_executed")"
vhdl_quality_realistic_expected_pass_total="$(extract_summary_value "$vhdl_family_summary_txt" "quality_realistic_expected_pass_total")"
vhdl_quality_realistic_expected_fail_total="$(extract_summary_value "$vhdl_family_summary_txt" "quality_realistic_expected_fail_total")"
vhdl_quality_realistic_observed_parse_pass_total="$(extract_summary_value "$vhdl_family_summary_txt" "quality_realistic_observed_parse_pass_total")"
vhdl_quality_realistic_observed_parse_fail_total="$(extract_summary_value "$vhdl_family_summary_txt" "quality_realistic_observed_parse_fail_total")"
vhdl_strict_promotion_recommendation="$(extract_summary_value "$vhdl_family_summary_txt" "strict_promotion_recommendation")"
vhdl_strict_promotion_eligible="$(extract_summary_value "$vhdl_family_summary_txt" "strict_promotion_eligible_for_required_strict_mode")"
vhdl_strict_promotion_primary_blocker="$(extract_summary_value "$vhdl_family_summary_txt" "strict_promotion_primary_blocker")"
vhdl_strict_promotion_trial_passed="$(extract_summary_value "$vhdl_family_summary_txt" "strict_promotion_trial_passed")"
vhdl_strict_promotion_observed_ratio_min="$(extract_summary_value "$vhdl_family_summary_txt" "strict_promotion_observed_ratio_min")"
vhdl_strict_promotion_observed_ratio_max="$(extract_summary_value "$vhdl_family_summary_txt" "strict_promotion_observed_ratio_max")"
vhdl_strict_promotion_observed_ratio_avg="$(extract_summary_value "$vhdl_family_summary_txt" "strict_promotion_observed_ratio_avg")"
vhdl_family_status_vhdl="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_status")"
vhdl_family_status_vhdl_tracker_status="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_tracker_status")"
vhdl_family_status_vhdl_tracker_alignment_ok="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_tracker_alignment_ok")"
vhdl_family_status_vhdl_unmet_closure_criteria_count="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_unmet_closure_criteria_count")"
vhdl_family_status_vhdl_primary_unmet_closure_criterion="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_primary_unmet_closure_criterion")"
vhdl_family_status_vhdl_unmet_closure_criteria_json="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_unmet_closure_criteria_json")"
vhdl_family_status_vhdl_unmet_closure_criteria_details_json="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_unmet_closure_criteria_details_json")"
vhdl_family_status_vhdl_closure_criteria_satisfied_count="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_closure_criteria_satisfied_count")"
vhdl_family_status_vhdl_closure_criteria_total_count="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_closure_criteria_total_count")"
vhdl_family_status_vhdl_closure_criteria_unsatisfied_count="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_closure_criteria_unsatisfied_count")"
vhdl_family_status_vhdl_family_contract_summary_txt="$(jq -r '.families[] | select(.family=="vhdl") | .proof_surfaces.family_contract_summary_txt' "$vhdl_family_status_summary_json")"
vhdl_family_status_vhdl_family_contract_summary_json="$(jq -r '.families[] | select(.family=="vhdl") | .proof_surfaces.family_contract_summary_json' "$vhdl_family_status_summary_json")"
vhdl_family_status_vhdl_family_contract_green="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_family_contract_green")"
vhdl_family_status_vhdl_quality_closed_loop_initial_status_pass="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_quality_closed_loop_initial_status_pass")"
vhdl_family_status_vhdl_quality_closed_loop_replay_status_pass="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_quality_closed_loop_replay_status_pass")"
vhdl_family_status_vhdl_quality_parseability_generation_parser_rejections_zero="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_quality_parseability_generation_parser_rejections_zero")"
vhdl_family_status_vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero")"
vhdl_family_status_vhdl_quality_closed_loop_replay_target_debt_zero="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_quality_closed_loop_replay_target_debt_zero")"
vhdl_family_status_vhdl_strict_promotion_recommendation_green="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_strict_promotion_recommendation_green")"
vhdl_family_status_vhdl_strict_promotion_eligible_for_required_strict_mode="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_strict_promotion_eligible_for_required_strict_mode")"
vhdl_family_status_vhdl_strict_promotion_primary_blocker_none="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_strict_promotion_primary_blocker_none")"
vhdl_family_status_vhdl_formal_exhaustive_closure_surface_green="$(extract_summary_value "$vhdl_family_status_summary_txt" "vhdl_formal_exhaustive_closure_surface_green")"
vhdl_family_status_contract_family_count="$(extract_summary_value "$vhdl_family_status_contract_summary_txt" "family_count")"
vhdl_family_status_contract_vhdl_tracker_alignment_ok="$(extract_summary_value "$vhdl_family_status_contract_summary_txt" "vhdl_tracker_alignment_ok")"
vhdl_family_status_contract_vhdl_false_criteria_count="$(extract_summary_value "$vhdl_family_status_contract_summary_txt" "vhdl_false_criteria_count")"
vhdl_family_status_contract_vhdl_unmet_details_count="$(extract_summary_value "$vhdl_family_status_contract_summary_txt" "vhdl_unmet_details_count")"
vhdl_family_status_contract_vhdl_primary_unmet_detail_criterion="$(extract_summary_value "$vhdl_family_status_contract_summary_txt" "vhdl_primary_unmet_detail_criterion")"
vhdl_family_status_contract_vhdl_unmet_closure_criteria_json="$(extract_summary_value "$vhdl_family_status_contract_summary_txt" "vhdl_unmet_closure_criteria_json")"
vhdl_family_status_contract_vhdl_unmet_closure_criteria_details_json="$(extract_summary_value "$vhdl_family_status_contract_summary_txt" "vhdl_unmet_closure_criteria_details_json")"
vhdl_family_status_contract_gate="$(jq -r '.gate' "$vhdl_family_status_contract_summary_json")"
vhdl_family_status_contract_gate_version="$(jq -r '.version' "$vhdl_family_status_contract_summary_json")"
vhdl_family_status_contract_generated_at_utc="$(jq -r '.generated_at_utc' "$vhdl_family_status_contract_summary_json")"
vhdl_family_status_contract_family_status_state_dir="$(jq -r '.family_status_state_dir' "$vhdl_family_status_contract_summary_json")"
vhdl_family_status_contract_family_status_summary_json="$(jq -r '.family_status_summary_json' "$vhdl_family_status_contract_summary_json")"
vhdl_family_status_contract_family_status_summary_txt="$(jq -r '.family_status_summary_txt' "$vhdl_family_status_contract_summary_json")"

assert_equal \
    "VHDL family quality closed-loop initial status" \
    "$vhdl_quality_closed_loop_initial_status" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_closed_loop_initial_status")"
assert_equal \
    "VHDL family quality closed-loop replay status" \
    "$vhdl_quality_closed_loop_replay_status" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_closed_loop_replay_status")"
assert_equal \
    "VHDL family quality closed-loop initial targets" \
    "$vhdl_quality_closed_loop_initial_targets" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_closed_loop_initial_targets")"
assert_equal \
    "VHDL family quality closed-loop replay targets" \
    "$vhdl_quality_closed_loop_replay_targets" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_closed_loop_replay_targets")"
assert_equal \
    "VHDL family quality parseability generation attempts" \
    "$vhdl_quality_parseability_generation_attempts_total" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_parseability_generation_attempts_total")"
assert_equal \
    "VHDL family quality parseability generation rejected" \
    "$vhdl_quality_parseability_generation_rejected_total" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_parseability_generation_rejected_total")"
assert_equal \
    "VHDL family quality realistic cases executed" \
    "$vhdl_quality_realistic_cases_executed" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_realistic_cases_executed")"
assert_equal \
    "VHDL family quality realistic expected pass total" \
    "$vhdl_quality_realistic_expected_pass_total" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_realistic_expected_pass_total")"
assert_equal \
    "VHDL family quality realistic expected fail total" \
    "$vhdl_quality_realistic_expected_fail_total" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_realistic_expected_fail_total")"
assert_equal \
    "VHDL family quality realistic observed pass total" \
    "$vhdl_quality_realistic_observed_parse_pass_total" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_realistic_observed_parse_pass_total")"
assert_equal \
    "VHDL family quality realistic observed fail total" \
    "$vhdl_quality_realistic_observed_parse_fail_total" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_realistic_observed_parse_fail_total")"
assert_equal \
    "VHDL family strict-promotion recommendation" \
    "$vhdl_strict_promotion_recommendation" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_recommendation")"
assert_equal \
    "VHDL family strict-promotion eligible" \
    "$vhdl_strict_promotion_eligible" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_eligible_for_required_strict_mode")"
assert_equal \
    "VHDL family strict-promotion primary blocker" \
    "$vhdl_strict_promotion_primary_blocker" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_primary_blocker")"
assert_equal \
    "VHDL family strict-promotion trial passed" \
    "$vhdl_strict_promotion_trial_passed" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_trial_passed")"
assert_equal \
    "VHDL family strict-promotion observed ratio min" \
    "$vhdl_strict_promotion_observed_ratio_min" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_observed_ratio_min")"
assert_equal \
    "VHDL family strict-promotion observed ratio max" \
    "$vhdl_strict_promotion_observed_ratio_max" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_observed_ratio_max")"
assert_equal \
    "VHDL family strict-promotion observed ratio avg" \
    "$vhdl_strict_promotion_observed_ratio_avg" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_observed_ratio_avg")"
assert_equal \
    "VHDL family contract summary json path" \
    "$vhdl_family_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_contract_summary_json")"
assert_equal \
    "VHDL family quality state dir" \
    "$vhdl_family_quality_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_state_dir")"
assert_equal \
    "VHDL family quality summary txt" \
    "$vhdl_family_quality_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_summary_txt")"
assert_equal \
    "VHDL family quality realistic report json" \
    "$vhdl_family_quality_realistic_report_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_realistic_report_json")"
assert_equal \
    "VHDL family quality parseability report json" \
    "$vhdl_family_quality_parseability_report_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_quality_parseability_report_json")"
assert_equal \
    "VHDL family strict-promotion state dir" \
    "$vhdl_family_strict_promotion_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_state_dir")"
assert_equal \
    "VHDL family strict-promotion summary txt" \
    "$vhdl_family_strict_promotion_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_summary_txt")"
assert_equal \
    "VHDL family strict-promotion report json" \
    "$vhdl_family_strict_promotion_report_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_strict_promotion_report_json")"
assert_equal \
    "VHDL family computed status" \
    "$vhdl_family_status_vhdl" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl")"
assert_equal \
    "VHDL family tracker status" \
    "$vhdl_family_status_vhdl_tracker_status" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_tracker_status")"
assert_equal \
    "VHDL family tracker alignment" \
    "$vhdl_family_status_vhdl_tracker_alignment_ok" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_tracker_alignment_ok")"
assert_equal \
    "VHDL family unmet closure criteria count" \
    "$vhdl_family_status_vhdl_unmet_closure_criteria_count" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_unmet_closure_criteria_count")"
assert_equal \
    "VHDL family primary unmet closure criterion" \
    "$vhdl_family_status_vhdl_primary_unmet_closure_criterion" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_primary_unmet_closure_criterion")"
assert_equal \
    "VHDL family unmet closure criteria json" \
    "$vhdl_family_status_vhdl_unmet_closure_criteria_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_unmet_closure_criteria_json")"
assert_equal \
    "VHDL family unmet closure criteria details json" \
    "$vhdl_family_status_vhdl_unmet_closure_criteria_details_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_unmet_closure_criteria_details_json")"
assert_equal \
    "VHDL family closure criteria satisfied count" \
    "$vhdl_family_status_vhdl_closure_criteria_satisfied_count" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_closure_criteria_satisfied_count")"
assert_equal \
    "VHDL family closure criteria total count" \
    "$vhdl_family_status_vhdl_closure_criteria_total_count" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_closure_criteria_total_count")"
assert_equal \
    "VHDL family closure criteria unsatisfied count" \
    "$vhdl_family_status_vhdl_closure_criteria_unsatisfied_count" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_closure_criteria_unsatisfied_count")"
assert_equal \
    "VHDL family status family contract summary txt path" \
    "$vhdl_family_status_vhdl_family_contract_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_family_contract_summary_txt")"
assert_equal \
    "VHDL family status family contract summary json path" \
    "$vhdl_family_status_vhdl_family_contract_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_family_contract_summary_json")"
assert_equal \
    "VHDL family contract green criterion" \
    "$vhdl_family_status_vhdl_family_contract_green" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_family_contract_green")"
assert_equal \
    "VHDL family closed-loop initial status pass criterion" \
    "$vhdl_family_status_vhdl_quality_closed_loop_initial_status_pass" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_quality_closed_loop_initial_status_pass")"
assert_equal \
    "VHDL family closed-loop replay status pass criterion" \
    "$vhdl_family_status_vhdl_quality_closed_loop_replay_status_pass" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_quality_closed_loop_replay_status_pass")"
assert_equal \
    "VHDL family generation parser rejections zero criterion" \
    "$vhdl_family_status_vhdl_quality_parseability_generation_parser_rejections_zero" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_quality_parseability_generation_parser_rejections_zero")"
assert_equal \
    "VHDL family replay-shadow parser rejections zero criterion" \
    "$vhdl_family_status_vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero")"
assert_equal \
    "VHDL family replay target debt zero criterion" \
    "$vhdl_family_status_vhdl_quality_closed_loop_replay_target_debt_zero" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_quality_closed_loop_replay_target_debt_zero")"
assert_equal \
    "VHDL family strict-promotion recommendation green criterion" \
    "$vhdl_family_status_vhdl_strict_promotion_recommendation_green" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_strict_promotion_recommendation_green")"
assert_equal \
    "VHDL family strict-promotion eligible criterion" \
    "$vhdl_family_status_vhdl_strict_promotion_eligible_for_required_strict_mode" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_strict_promotion_eligible_for_required_strict_mode")"
assert_equal \
    "VHDL family strict-promotion primary blocker none criterion" \
    "$vhdl_family_status_vhdl_strict_promotion_primary_blocker_none" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_strict_promotion_primary_blocker_none")"
assert_equal \
    "VHDL family formal exhaustive closure surface green criterion" \
    "$vhdl_family_status_vhdl_formal_exhaustive_closure_surface_green" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_vhdl_formal_exhaustive_closure_surface_green")"
assert_equal \
    "VHDL family status contract gate name" \
    "$vhdl_family_status_contract_gate" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_gate")"
assert_equal \
    "VHDL family status contract gate version" \
    "$vhdl_family_status_contract_gate_version" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_gate_version")"
assert_equal \
    "VHDL family status contract generated at" \
    "$vhdl_family_status_contract_generated_at_utc" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_generated_at_utc")"
assert_equal \
    "VHDL family status contract family status state dir" \
    "$vhdl_family_status_contract_family_status_state_dir" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_family_status_state_dir")"
assert_equal \
    "VHDL family status contract family status summary json path" \
    "$vhdl_family_status_contract_family_status_summary_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_family_status_summary_json")"
assert_equal \
    "VHDL family status contract family status summary txt path" \
    "$vhdl_family_status_contract_family_status_summary_txt" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_family_status_summary_txt")"
assert_equal \
    "VHDL family status contract family count" \
    "$vhdl_family_status_contract_family_count" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_family_count")"
assert_equal \
    "VHDL family status contract tracker alignment" \
    "$vhdl_family_status_contract_vhdl_tracker_alignment_ok" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_vhdl_tracker_alignment_ok")"
assert_equal \
    "VHDL family status contract false criteria count" \
    "$vhdl_family_status_contract_vhdl_false_criteria_count" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_vhdl_false_criteria_count")"
assert_equal \
    "VHDL family status contract unmet details count" \
    "$vhdl_family_status_contract_vhdl_unmet_details_count" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_vhdl_unmet_details_count")"
assert_equal \
    "VHDL family status contract primary unmet detail criterion" \
    "$vhdl_family_status_contract_vhdl_primary_unmet_detail_criterion" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_vhdl_primary_unmet_detail_criterion")"
assert_equal \
    "VHDL family status contract unmet criteria json" \
    "$vhdl_family_status_contract_vhdl_unmet_closure_criteria_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_vhdl_unmet_closure_criteria_json")"
assert_equal \
    "VHDL family status contract unmet criteria details json" \
    "$vhdl_family_status_contract_vhdl_unmet_closure_criteria_details_json" \
    "$(extract_summary_value "$sota_summary_txt" "vhdl_family_status_contract_vhdl_unmet_closure_criteria_details_json")"

assert_equal \
    "SOTA exit VHDL primary unmet closure criterion" \
    "$vhdl_family_status_vhdl_primary_unmet_closure_criterion" \
    "$sota_exit_vhdl_primary_unmet"
assert_equal \
    "SOTA exit VHDL unmet closure criteria json" \
    "$vhdl_family_status_vhdl_unmet_closure_criteria_json" \
    "$sota_exit_vhdl_unmet_json"
assert_equal \
    "SOTA exit VHDL unmet closure criteria details json" \
    "$vhdl_family_status_vhdl_unmet_closure_criteria_details_json" \
    "$sota_exit_vhdl_unmet_details_json"
assert_equal \
    "SOTA exit VHDL primary unmet detail criterion" \
    "$vhdl_family_status_contract_vhdl_primary_unmet_detail_criterion" \
    "$sota_exit_vhdl_primary_unmet_detail"
assert_equal \
    "SOTA exit VHDL unmet detail closure criteria json" \
    "$vhdl_family_status_contract_vhdl_unmet_closure_criteria_json" \
    "$sota_exit_vhdl_unmet_detail_json"
assert_equal \
    "SOTA exit VHDL unmet detail closure criteria details json" \
    "$vhdl_family_status_contract_vhdl_unmet_closure_criteria_details_json" \
    "$sota_exit_vhdl_unmet_detail_details_json"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "VHDL Combined Telemetry Contract Gate Summary"
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
    echo "vhdl_parser_family_contract_summary_txt: $vhdl_family_summary_txt"
    echo "vhdl_parser_family_contract_summary_json: $vhdl_family_summary_json"
    echo "vhdl_family_quality_state_dir: $vhdl_family_quality_state_dir"
    echo "vhdl_family_quality_summary_txt: $vhdl_family_quality_summary_txt"
    echo "vhdl_family_quality_realistic_report_json: $vhdl_family_quality_realistic_report_json"
    echo "vhdl_family_quality_parseability_report_json: $vhdl_family_quality_parseability_report_json"
    echo "vhdl_family_strict_promotion_state_dir: $vhdl_family_strict_promotion_state_dir"
    echo "vhdl_family_strict_promotion_summary_txt: $vhdl_family_strict_promotion_summary_txt"
    echo "vhdl_family_strict_promotion_report_json: $vhdl_family_strict_promotion_report_json"
    echo "vhdl_parser_family_status_summary_txt: $vhdl_family_status_summary_txt"
    echo "vhdl_parser_family_status_summary_json: $vhdl_family_status_summary_json"
    echo "vhdl_parser_family_status_gate: $vhdl_parser_family_status_gate"
    echo "vhdl_parser_family_status_gate_version: $vhdl_parser_family_status_gate_version"
    echo "vhdl_parser_family_status_generated_at_utc: $vhdl_parser_family_status_generated_at_utc"
    echo "vhdl_parser_family_status_live_tracker_file: $vhdl_parser_family_status_live_tracker_file"
    echo "vhdl_parser_family_status_status_rule_done: $vhdl_parser_family_status_status_rule_done"
    echo "vhdl_parser_family_status_contract_summary_txt: $vhdl_family_status_contract_summary_txt"
    echo "vhdl_parser_family_status_contract_summary_json: $vhdl_family_status_contract_summary_json"
    echo "vhdl_family_status_contract_gate: $vhdl_family_status_contract_gate"
    echo "vhdl_family_status_contract_gate_version: $vhdl_family_status_contract_gate_version"
    echo "vhdl_family_status_contract_generated_at_utc: $vhdl_family_status_contract_generated_at_utc"
    echo "vhdl_family_status_contract_family_status_state_dir: $vhdl_family_status_contract_family_status_state_dir"
    echo "vhdl_family_status_contract_family_status_summary_json: $vhdl_family_status_contract_family_status_summary_json"
    echo "vhdl_family_status_contract_family_status_summary_txt: $vhdl_family_status_contract_family_status_summary_txt"
    echo "vhdl_family_quality_closed_loop_initial_status: $vhdl_quality_closed_loop_initial_status"
    echo "vhdl_family_quality_closed_loop_replay_status: $vhdl_quality_closed_loop_replay_status"
    echo "vhdl_family_quality_closed_loop_initial_targets: $vhdl_quality_closed_loop_initial_targets"
    echo "vhdl_family_quality_closed_loop_replay_targets: $vhdl_quality_closed_loop_replay_targets"
    echo "vhdl_family_quality_parseability_generation_attempts_total: $vhdl_quality_parseability_generation_attempts_total"
    echo "vhdl_family_quality_parseability_generation_rejected_total: $vhdl_quality_parseability_generation_rejected_total"
    echo "vhdl_family_quality_realistic_cases_executed: $vhdl_quality_realistic_cases_executed"
    echo "vhdl_family_quality_realistic_expected_pass_total: $vhdl_quality_realistic_expected_pass_total"
    echo "vhdl_family_quality_realistic_expected_fail_total: $vhdl_quality_realistic_expected_fail_total"
    echo "vhdl_family_quality_realistic_observed_parse_pass_total: $vhdl_quality_realistic_observed_parse_pass_total"
    echo "vhdl_family_quality_realistic_observed_parse_fail_total: $vhdl_quality_realistic_observed_parse_fail_total"
    echo "vhdl_family_strict_promotion_recommendation: $vhdl_strict_promotion_recommendation"
    echo "vhdl_family_strict_promotion_eligible_for_required_strict_mode: $vhdl_strict_promotion_eligible"
    echo "vhdl_family_strict_promotion_primary_blocker: $vhdl_strict_promotion_primary_blocker"
    echo "vhdl_family_strict_promotion_trial_passed: $vhdl_strict_promotion_trial_passed"
    echo "vhdl_family_strict_promotion_observed_ratio_min: $vhdl_strict_promotion_observed_ratio_min"
    echo "vhdl_family_strict_promotion_observed_ratio_max: $vhdl_strict_promotion_observed_ratio_max"
    echo "vhdl_family_strict_promotion_observed_ratio_avg: $vhdl_strict_promotion_observed_ratio_avg"
    echo "vhdl_family_status_vhdl: $vhdl_family_status_vhdl"
    echo "vhdl_family_status_vhdl_tracker_status: $vhdl_family_status_vhdl_tracker_status"
    echo "vhdl_family_status_vhdl_tracker_alignment_ok: $vhdl_family_status_vhdl_tracker_alignment_ok"
    echo "vhdl_family_status_vhdl_unmet_closure_criteria_count: $vhdl_family_status_vhdl_unmet_closure_criteria_count"
    echo "vhdl_family_status_vhdl_primary_unmet_closure_criterion: $vhdl_family_status_vhdl_primary_unmet_closure_criterion"
    echo "vhdl_family_status_vhdl_unmet_closure_criteria_json: $vhdl_family_status_vhdl_unmet_closure_criteria_json"
    echo "vhdl_family_status_vhdl_unmet_closure_criteria_details_json: $vhdl_family_status_vhdl_unmet_closure_criteria_details_json"
    echo "vhdl_family_status_vhdl_closure_criteria_satisfied_count: $vhdl_family_status_vhdl_closure_criteria_satisfied_count"
    echo "vhdl_family_status_vhdl_closure_criteria_total_count: $vhdl_family_status_vhdl_closure_criteria_total_count"
    echo "vhdl_family_status_vhdl_closure_criteria_unsatisfied_count: $vhdl_family_status_vhdl_closure_criteria_unsatisfied_count"
    echo "vhdl_family_status_vhdl_family_contract_summary_txt: $vhdl_family_status_vhdl_family_contract_summary_txt"
    echo "vhdl_family_status_vhdl_family_contract_summary_json: $vhdl_family_status_vhdl_family_contract_summary_json"
    echo "vhdl_family_status_vhdl_family_contract_green: $vhdl_family_status_vhdl_family_contract_green"
    echo "vhdl_family_status_vhdl_quality_closed_loop_initial_status_pass: $vhdl_family_status_vhdl_quality_closed_loop_initial_status_pass"
    echo "vhdl_family_status_vhdl_quality_closed_loop_replay_status_pass: $vhdl_family_status_vhdl_quality_closed_loop_replay_status_pass"
    echo "vhdl_family_status_vhdl_quality_parseability_generation_parser_rejections_zero: $vhdl_family_status_vhdl_quality_parseability_generation_parser_rejections_zero"
    echo "vhdl_family_status_vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero: $vhdl_family_status_vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero"
    echo "vhdl_family_status_vhdl_quality_closed_loop_replay_target_debt_zero: $vhdl_family_status_vhdl_quality_closed_loop_replay_target_debt_zero"
    echo "vhdl_family_status_vhdl_strict_promotion_recommendation_green: $vhdl_family_status_vhdl_strict_promotion_recommendation_green"
    echo "vhdl_family_status_vhdl_strict_promotion_eligible_for_required_strict_mode: $vhdl_family_status_vhdl_strict_promotion_eligible_for_required_strict_mode"
    echo "vhdl_family_status_vhdl_strict_promotion_primary_blocker_none: $vhdl_family_status_vhdl_strict_promotion_primary_blocker_none"
    echo "vhdl_family_status_vhdl_formal_exhaustive_closure_surface_green: $vhdl_family_status_vhdl_formal_exhaustive_closure_surface_green"
    echo "vhdl_family_status_contract_family_count: $vhdl_family_status_contract_family_count"
    echo "vhdl_family_status_contract_vhdl_tracker_alignment_ok: $vhdl_family_status_contract_vhdl_tracker_alignment_ok"
    echo "vhdl_family_status_contract_vhdl_false_criteria_count: $vhdl_family_status_contract_vhdl_false_criteria_count"
    echo "vhdl_family_status_contract_vhdl_unmet_details_count: $vhdl_family_status_contract_vhdl_unmet_details_count"
    echo "vhdl_family_status_contract_vhdl_primary_unmet_detail_criterion: $vhdl_family_status_contract_vhdl_primary_unmet_detail_criterion"
    echo "vhdl_family_status_contract_vhdl_unmet_closure_criteria_json: $vhdl_family_status_contract_vhdl_unmet_closure_criteria_json"
    echo "vhdl_family_status_contract_vhdl_unmet_closure_criteria_details_json: $vhdl_family_status_contract_vhdl_unmet_closure_criteria_details_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "vhdl_combined_telemetry_contract_gate" \
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
    --arg vhdl_parser_family_contract_summary_txt "$vhdl_family_summary_txt" \
    --arg vhdl_parser_family_contract_summary_json "$vhdl_family_summary_json" \
    --arg vhdl_parser_family_status_summary_txt "$vhdl_family_status_summary_txt" \
    --arg vhdl_parser_family_status_summary_json "$vhdl_family_status_summary_json" \
    --arg vhdl_parser_family_status_contract_summary_txt "$vhdl_family_status_contract_summary_txt" \
    --arg vhdl_parser_family_status_contract_summary_json "$vhdl_family_status_contract_summary_json" \
    --arg vhdl_family_quality_state_dir "$vhdl_family_quality_state_dir" \
    --arg vhdl_family_quality_summary_txt "$vhdl_family_quality_summary_txt" \
    --arg vhdl_family_quality_realistic_report_json "$vhdl_family_quality_realistic_report_json" \
    --arg vhdl_family_quality_parseability_report_json "$vhdl_family_quality_parseability_report_json" \
    --arg vhdl_family_strict_promotion_state_dir "$vhdl_family_strict_promotion_state_dir" \
    --arg vhdl_family_strict_promotion_summary_txt "$vhdl_family_strict_promotion_summary_txt" \
    --arg vhdl_family_strict_promotion_report_json "$vhdl_family_strict_promotion_report_json" \
    --arg vhdl_quality_closed_loop_initial_status "$vhdl_quality_closed_loop_initial_status" \
    --arg vhdl_quality_closed_loop_replay_status "$vhdl_quality_closed_loop_replay_status" \
    --argjson vhdl_quality_closed_loop_initial_targets "$vhdl_quality_closed_loop_initial_targets" \
    --argjson vhdl_quality_closed_loop_replay_targets "$vhdl_quality_closed_loop_replay_targets" \
    --argjson vhdl_quality_parseability_generation_attempts_total "$vhdl_quality_parseability_generation_attempts_total" \
    --argjson vhdl_quality_parseability_generation_rejected_total "$vhdl_quality_parseability_generation_rejected_total" \
    --argjson vhdl_quality_realistic_cases_executed "$vhdl_quality_realistic_cases_executed" \
    --argjson vhdl_quality_realistic_expected_pass_total "$vhdl_quality_realistic_expected_pass_total" \
    --argjson vhdl_quality_realistic_expected_fail_total "$vhdl_quality_realistic_expected_fail_total" \
    --argjson vhdl_quality_realistic_observed_parse_pass_total "$vhdl_quality_realistic_observed_parse_pass_total" \
    --argjson vhdl_quality_realistic_observed_parse_fail_total "$vhdl_quality_realistic_observed_parse_fail_total" \
    --arg vhdl_strict_promotion_recommendation "$vhdl_strict_promotion_recommendation" \
    --argjson vhdl_strict_promotion_eligible "$vhdl_strict_promotion_eligible" \
    --arg vhdl_strict_promotion_primary_blocker "$vhdl_strict_promotion_primary_blocker" \
    --argjson vhdl_strict_promotion_trial_passed "$vhdl_strict_promotion_trial_passed" \
    --argjson vhdl_strict_promotion_observed_ratio_min "$vhdl_strict_promotion_observed_ratio_min" \
    --argjson vhdl_strict_promotion_observed_ratio_max "$vhdl_strict_promotion_observed_ratio_max" \
    --argjson vhdl_strict_promotion_observed_ratio_avg "$vhdl_strict_promotion_observed_ratio_avg" \
    --arg vhdl_parser_family_status_gate "$vhdl_parser_family_status_gate" \
    --argjson vhdl_parser_family_status_gate_version "$vhdl_parser_family_status_gate_version" \
    --arg vhdl_parser_family_status_generated_at_utc "$vhdl_parser_family_status_generated_at_utc" \
    --arg vhdl_parser_family_status_live_tracker_file "$vhdl_parser_family_status_live_tracker_file" \
    --arg vhdl_parser_family_status_status_rule_done "$vhdl_parser_family_status_status_rule_done" \
    --arg vhdl_family_status_vhdl "$vhdl_family_status_vhdl" \
    --arg vhdl_family_status_vhdl_tracker_status "$vhdl_family_status_vhdl_tracker_status" \
    --argjson vhdl_family_status_vhdl_tracker_alignment_ok "$vhdl_family_status_vhdl_tracker_alignment_ok" \
    --argjson vhdl_family_status_vhdl_unmet_closure_criteria_count "$vhdl_family_status_vhdl_unmet_closure_criteria_count" \
    --arg vhdl_family_status_vhdl_primary_unmet_closure_criterion "$vhdl_family_status_vhdl_primary_unmet_closure_criterion" \
    --argjson vhdl_family_status_vhdl_unmet_closure_criteria_json "$vhdl_family_status_vhdl_unmet_closure_criteria_json" \
    --argjson vhdl_family_status_vhdl_unmet_closure_criteria_details_json "$vhdl_family_status_vhdl_unmet_closure_criteria_details_json" \
    --argjson vhdl_family_status_vhdl_closure_criteria_satisfied_count "$vhdl_family_status_vhdl_closure_criteria_satisfied_count" \
    --argjson vhdl_family_status_vhdl_closure_criteria_total_count "$vhdl_family_status_vhdl_closure_criteria_total_count" \
    --argjson vhdl_family_status_vhdl_closure_criteria_unsatisfied_count "$vhdl_family_status_vhdl_closure_criteria_unsatisfied_count" \
    --arg vhdl_family_status_vhdl_family_contract_summary_txt "$vhdl_family_status_vhdl_family_contract_summary_txt" \
    --arg vhdl_family_status_vhdl_family_contract_summary_json "$vhdl_family_status_vhdl_family_contract_summary_json" \
    --argjson vhdl_family_status_vhdl_family_contract_green "$vhdl_family_status_vhdl_family_contract_green" \
    --argjson vhdl_family_status_vhdl_quality_closed_loop_initial_status_pass "$vhdl_family_status_vhdl_quality_closed_loop_initial_status_pass" \
    --argjson vhdl_family_status_vhdl_quality_closed_loop_replay_status_pass "$vhdl_family_status_vhdl_quality_closed_loop_replay_status_pass" \
    --argjson vhdl_family_status_vhdl_quality_parseability_generation_parser_rejections_zero "$vhdl_family_status_vhdl_quality_parseability_generation_parser_rejections_zero" \
    --argjson vhdl_family_status_vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero "$vhdl_family_status_vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero" \
    --argjson vhdl_family_status_vhdl_quality_closed_loop_replay_target_debt_zero "$vhdl_family_status_vhdl_quality_closed_loop_replay_target_debt_zero" \
    --argjson vhdl_family_status_vhdl_strict_promotion_recommendation_green "$vhdl_family_status_vhdl_strict_promotion_recommendation_green" \
    --argjson vhdl_family_status_vhdl_strict_promotion_eligible_for_required_strict_mode "$vhdl_family_status_vhdl_strict_promotion_eligible_for_required_strict_mode" \
    --argjson vhdl_family_status_vhdl_strict_promotion_primary_blocker_none "$vhdl_family_status_vhdl_strict_promotion_primary_blocker_none" \
    --argjson vhdl_family_status_vhdl_formal_exhaustive_closure_surface_green "$vhdl_family_status_vhdl_formal_exhaustive_closure_surface_green" \
    --arg vhdl_family_status_contract_gate "$vhdl_family_status_contract_gate" \
    --argjson vhdl_family_status_contract_gate_version "$vhdl_family_status_contract_gate_version" \
    --arg vhdl_family_status_contract_generated_at_utc "$vhdl_family_status_contract_generated_at_utc" \
    --arg vhdl_family_status_contract_family_status_state_dir "$vhdl_family_status_contract_family_status_state_dir" \
    --arg vhdl_family_status_contract_family_status_summary_json "$vhdl_family_status_contract_family_status_summary_json" \
    --arg vhdl_family_status_contract_family_status_summary_txt "$vhdl_family_status_contract_family_status_summary_txt" \
    --argjson vhdl_family_status_contract_family_count "$vhdl_family_status_contract_family_count" \
    --argjson vhdl_family_status_contract_vhdl_tracker_alignment_ok "$vhdl_family_status_contract_vhdl_tracker_alignment_ok" \
    --argjson vhdl_family_status_contract_vhdl_false_criteria_count "$vhdl_family_status_contract_vhdl_false_criteria_count" \
    --argjson vhdl_family_status_contract_vhdl_unmet_details_count "$vhdl_family_status_contract_vhdl_unmet_details_count" \
    --arg vhdl_family_status_contract_vhdl_primary_unmet_detail_criterion "$vhdl_family_status_contract_vhdl_primary_unmet_detail_criterion" \
    --argjson vhdl_family_status_contract_vhdl_unmet_closure_criteria_json "$vhdl_family_status_contract_vhdl_unmet_closure_criteria_json" \
    --argjson vhdl_family_status_contract_vhdl_unmet_closure_criteria_details_json "$vhdl_family_status_contract_vhdl_unmet_closure_criteria_details_json" \
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
        vhdl_parser_family_contract_summary_txt: $vhdl_parser_family_contract_summary_txt,
        vhdl_parser_family_contract_summary_json: $vhdl_parser_family_contract_summary_json,
        vhdl_parser_family_status_summary_txt: $vhdl_parser_family_status_summary_txt,
        vhdl_parser_family_status_summary_json: $vhdl_parser_family_status_summary_json,
        vhdl_parser_family_status_contract_summary_txt: $vhdl_parser_family_status_contract_summary_txt,
        vhdl_parser_family_status_contract_summary_json: $vhdl_parser_family_status_contract_summary_json
      },
      family_contract: {
        quality_closed_loop_initial_status: $vhdl_quality_closed_loop_initial_status,
        quality_closed_loop_replay_status: $vhdl_quality_closed_loop_replay_status,
        quality_closed_loop_initial_targets: $vhdl_quality_closed_loop_initial_targets,
        quality_closed_loop_replay_targets: $vhdl_quality_closed_loop_replay_targets,
        quality_parseability_generation_attempts_total: $vhdl_quality_parseability_generation_attempts_total,
        quality_parseability_generation_rejected_total: $vhdl_quality_parseability_generation_rejected_total,
        quality_realistic_cases_executed: $vhdl_quality_realistic_cases_executed,
        quality_realistic_expected_pass_total: $vhdl_quality_realistic_expected_pass_total,
        quality_realistic_expected_fail_total: $vhdl_quality_realistic_expected_fail_total,
        quality_realistic_observed_parse_pass_total: $vhdl_quality_realistic_observed_parse_pass_total,
        quality_realistic_observed_parse_fail_total: $vhdl_quality_realistic_observed_parse_fail_total,
        strict_promotion_recommendation: $vhdl_strict_promotion_recommendation,
        strict_promotion_eligible_for_required_strict_mode: $vhdl_strict_promotion_eligible,
        strict_promotion_primary_blocker: $vhdl_strict_promotion_primary_blocker,
        strict_promotion_trial_passed: $vhdl_strict_promotion_trial_passed,
        strict_promotion_observed_ratio_min: $vhdl_strict_promotion_observed_ratio_min,
        strict_promotion_observed_ratio_max: $vhdl_strict_promotion_observed_ratio_max,
        strict_promotion_observed_ratio_avg: $vhdl_strict_promotion_observed_ratio_avg,
        proof_surfaces: {
          quality_state_dir: $vhdl_family_quality_state_dir,
          quality_summary_txt: $vhdl_family_quality_summary_txt,
          quality_realistic_report_json: $vhdl_family_quality_realistic_report_json,
          quality_parseability_report_json: $vhdl_family_quality_parseability_report_json,
          strict_promotion_state_dir: $vhdl_family_strict_promotion_state_dir,
          strict_promotion_summary_txt: $vhdl_family_strict_promotion_summary_txt,
          strict_promotion_report_json: $vhdl_family_strict_promotion_report_json
        }
      },
      family_status: {
        gate: $vhdl_parser_family_status_gate,
        version: $vhdl_parser_family_status_gate_version,
        generated_at_utc: $vhdl_parser_family_status_generated_at_utc,
        live_tracker_file: $vhdl_parser_family_status_live_tracker_file,
        status_rule_done: $vhdl_parser_family_status_status_rule_done,
        family: {
          name: "vhdl",
          computed_status: $vhdl_family_status_vhdl,
          live_tracker_status: $vhdl_family_status_vhdl_tracker_status,
          tracker_alignment_ok: $vhdl_family_status_vhdl_tracker_alignment_ok,
          unmet_closure_criteria_count: $vhdl_family_status_vhdl_unmet_closure_criteria_count,
          primary_unmet_closure_criterion: $vhdl_family_status_vhdl_primary_unmet_closure_criterion,
          unmet_closure_criteria: $vhdl_family_status_vhdl_unmet_closure_criteria_json,
          unmet_closure_criteria_details: $vhdl_family_status_vhdl_unmet_closure_criteria_details_json,
          closure_criteria_satisfied_count: $vhdl_family_status_vhdl_closure_criteria_satisfied_count,
          closure_criteria_total_count: $vhdl_family_status_vhdl_closure_criteria_total_count,
          closure_criteria_unsatisfied_count: $vhdl_family_status_vhdl_closure_criteria_unsatisfied_count,
          proof_surfaces: {
            family_contract_summary_txt: $vhdl_family_status_vhdl_family_contract_summary_txt,
            family_contract_summary_json: $vhdl_family_status_vhdl_family_contract_summary_json
          },
          criteria: {
            family_contract_green: $vhdl_family_status_vhdl_family_contract_green,
            quality_closed_loop_initial_status_pass: $vhdl_family_status_vhdl_quality_closed_loop_initial_status_pass,
            quality_closed_loop_replay_status_pass: $vhdl_family_status_vhdl_quality_closed_loop_replay_status_pass,
            quality_parseability_generation_parser_rejections_zero: $vhdl_family_status_vhdl_quality_parseability_generation_parser_rejections_zero,
            quality_closed_loop_parseability_shadow_parser_rejections_zero: $vhdl_family_status_vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero,
            quality_closed_loop_replay_target_debt_zero: $vhdl_family_status_vhdl_quality_closed_loop_replay_target_debt_zero,
            strict_promotion_recommendation_green: $vhdl_family_status_vhdl_strict_promotion_recommendation_green,
            strict_promotion_eligible_for_required_strict_mode: $vhdl_family_status_vhdl_strict_promotion_eligible_for_required_strict_mode,
            strict_promotion_primary_blocker_none: $vhdl_family_status_vhdl_strict_promotion_primary_blocker_none,
            formal_exhaustive_closure_surface_green: $vhdl_family_status_vhdl_formal_exhaustive_closure_surface_green
          }
        }
      },
      family_status_contract: {
        gate: $vhdl_family_status_contract_gate,
        version: $vhdl_family_status_contract_gate_version,
        generated_at_utc: $vhdl_family_status_contract_generated_at_utc,
        family_status_state_dir: $vhdl_family_status_contract_family_status_state_dir,
        family_status_summary_json: $vhdl_family_status_contract_family_status_summary_json,
        family_status_summary_txt: $vhdl_family_status_contract_family_status_summary_txt,
        family_count: $vhdl_family_status_contract_family_count,
        family: {
          name: "vhdl",
          tracker_alignment_ok: $vhdl_family_status_contract_vhdl_tracker_alignment_ok,
          false_criteria_count: $vhdl_family_status_contract_vhdl_false_criteria_count,
          unmet_details_count: $vhdl_family_status_contract_vhdl_unmet_details_count,
          primary_unmet_detail_criterion: $vhdl_family_status_contract_vhdl_primary_unmet_detail_criterion,
          unmet_closure_criteria: $vhdl_family_status_contract_vhdl_unmet_closure_criteria_json,
          unmet_closure_criteria_details: $vhdl_family_status_contract_vhdl_unmet_closure_criteria_details_json
        }
      }
    }' >"$SUMMARY_JSON"

require_nonempty_file "$SUMMARY_JSON"

echo "✅ VHDL combined telemetry contract gate passed."
