#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_VHDL_COMBINED_TELEMETRY_CONTRACT_STATE_DIR:-$RUST_DIR/target/vhdl_combined_telemetry_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

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
require_nonempty_file "$sota_summary_txt"
vhdl_family_summary_txt="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_contract_summary_txt")"
vhdl_family_status_summary_txt="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_summary_txt")"
vhdl_family_status_summary_json="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_summary_json")"
vhdl_family_status_contract_summary_txt="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_contract_summary_txt")"
vhdl_family_status_contract_summary_json="$(extract_summary_value "$sota_summary_txt" "vhdl_parser_family_status_contract_summary_json")"

require_nonempty_file "$vhdl_family_summary_txt"
require_nonempty_file "$vhdl_family_status_summary_txt"
require_nonempty_file "$vhdl_family_status_summary_json"
require_nonempty_file "$vhdl_family_status_contract_summary_txt"
require_nonempty_file "$vhdl_family_status_contract_summary_json"
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

{
    echo "VHDL Combined Telemetry Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "sota_state_dir: $sota_state_dir"
    echo "sota_policy_env_file: $SOTA_POLICY_ENV_FILE"
    echo "vhdl_parser_family_contract_summary_txt: $vhdl_family_summary_txt"
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

echo "✅ VHDL combined telemetry contract gate passed."
