#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_STATE_DIR:-$RUST_DIR/target/vhdl_parser_family_status_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_JSON="$STATE_DIR/summary.json"
SUMMARY_TXT="$STATE_DIR/summary.txt"
LIVE_TRACKER_FILE="$ROOT_DIR/LIVE_ACHIEVEMENT_STATUS.md"

VHDL_FAMILY_CONTRACT_GATE="$RUST_DIR/scripts/vhdl_parser_family_contract_gate.sh"
VHDL_COMBINED_TELEMETRY_GATE="$RUST_DIR/scripts/vhdl_combined_telemetry_contract_gate.sh"

EXISTING_VHDL_FAMILY_CONTRACT_STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_EXISTING_FAMILY_CONTRACT_STATE_DIR:-}"
EXISTING_VHDL_COMBINED_TELEMETRY_STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_EXISTING_COMBINED_TELEMETRY_STATE_DIR:-}"
EXISTING_VHDL_QUALITY_STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_EXISTING_QUALITY_STATE_DIR:-}"
EXISTING_VHDL_STRICT_PROMOTION_STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_EXISTING_STRICT_PROMOTION_STATE_DIR:-}"
EXISTING_VHDL_SOTA_EXIT_STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_EXISTING_SOTA_EXIT_STATE_DIR:-}"

DONE_RULE="Done requires a formally exhaustive, machine-checkable closure surface with no remaining parser rejection debt and no remaining coverage/gap debt for the family claim."

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

summary_value_from_txt() {
    local key="$1"
    local path="$2"
    local line
    line="$(grep -F "${key}: " "$path" | tail -n 1 || true)"
    if [[ -z "$line" ]]; then
        echo "error: missing key '${key}' in summary '$path'" >&2
        exit 1
    fi
    printf '%s\n' "${line#${key}: }"
}

markdown_table_status_for_row() {
    local row_match="$1"
    local path="$2"
    local line
    line="$(grep -F "$row_match" "$path" | head -n 1 || true)"
    if [[ -z "$line" ]]; then
        echo "error: missing live-tracker row containing '$row_match' in '$path'" >&2
        exit 1
    fi
    awk -F'|' '{print $3}' <<<"$line" | xargs
}

run_logged() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        exit 1
    fi
}

require_tool jq
require_file "$LIVE_TRACKER_FILE"
require_file "$VHDL_FAMILY_CONTRACT_GATE"
require_file "$VHDL_COMBINED_TELEMETRY_GATE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

vhdl_family_contract_state_dir="${EXISTING_VHDL_FAMILY_CONTRACT_STATE_DIR:-$WORK_DIR/vhdl_parser_family_contract_gate}"
vhdl_combined_telemetry_state_dir="${EXISTING_VHDL_COMBINED_TELEMETRY_STATE_DIR:-$WORK_DIR/vhdl_combined_telemetry_contract_gate}"

if [[ -z "$EXISTING_VHDL_FAMILY_CONTRACT_STATE_DIR" ]]; then
    vhdl_family_contract_cmd=(
        env
        PGEN_VHDL_FAMILY_CONTRACT_STATE_DIR="$vhdl_family_contract_state_dir"
    )
    if [[ -n "$EXISTING_VHDL_QUALITY_STATE_DIR" ]]; then
        vhdl_family_contract_cmd+=(
            PGEN_VHDL_FAMILY_CONTRACT_EXISTING_QUALITY_STATE_DIR="$EXISTING_VHDL_QUALITY_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_VHDL_STRICT_PROMOTION_STATE_DIR" ]]; then
        vhdl_family_contract_cmd+=(
            PGEN_VHDL_FAMILY_CONTRACT_EXISTING_STRICT_PROMOTION_STATE_DIR="$EXISTING_VHDL_STRICT_PROMOTION_STATE_DIR"
        )
    fi
    vhdl_family_contract_cmd+=("$VHDL_FAMILY_CONTRACT_GATE")
    run_logged "vhdl_parser_family_contract_gate" "${vhdl_family_contract_cmd[@]}"
fi

if [[ -z "$EXISTING_VHDL_COMBINED_TELEMETRY_STATE_DIR" ]]; then
    vhdl_combined_telemetry_cmd=(
        env
        PGEN_VHDL_COMBINED_TELEMETRY_CONTRACT_STATE_DIR="$vhdl_combined_telemetry_state_dir"
    )
    if [[ -n "$EXISTING_VHDL_SOTA_EXIT_STATE_DIR" ]]; then
        vhdl_combined_telemetry_cmd+=(
            PGEN_VHDL_COMBINED_TELEMETRY_EXISTING_SOTA_EXIT_STATE_DIR="$EXISTING_VHDL_SOTA_EXIT_STATE_DIR"
        )
    fi
    vhdl_combined_telemetry_cmd+=("$VHDL_COMBINED_TELEMETRY_GATE")
    run_logged "vhdl_combined_telemetry_contract_gate" "${vhdl_combined_telemetry_cmd[@]}"
fi

vhdl_family_contract_summary_txt="$vhdl_family_contract_state_dir/summary.txt"
vhdl_combined_telemetry_summary_txt="$vhdl_combined_telemetry_state_dir/summary.txt"

require_nonempty_file "$vhdl_family_contract_summary_txt"
require_nonempty_file "$vhdl_combined_telemetry_summary_txt"

vhdl_quality_closed_loop_initial_status="$(summary_value_from_txt "quality_closed_loop_initial_status" "$vhdl_family_contract_summary_txt")"
vhdl_quality_closed_loop_replay_status="$(summary_value_from_txt "quality_closed_loop_replay_status" "$vhdl_family_contract_summary_txt")"
vhdl_quality_closed_loop_replay_targets="$(summary_value_from_txt "quality_closed_loop_replay_targets" "$vhdl_family_contract_summary_txt")"
vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total="$(summary_value_from_txt "quality_closed_loop_parseability_shadow_parser_rejections_total" "$vhdl_family_contract_summary_txt")"
vhdl_quality_parseability_generation_parser_rejections_total="$(summary_value_from_txt "quality_parseability_generation_parser_rejections_total" "$vhdl_family_contract_summary_txt")"
vhdl_quality_parseability_generation_rejected_total="$(summary_value_from_txt "quality_parseability_generation_rejected_total" "$vhdl_family_contract_summary_txt")"
vhdl_quality_realistic_cases_executed="$(summary_value_from_txt "quality_realistic_cases_executed" "$vhdl_family_contract_summary_txt")"
vhdl_quality_realistic_expected_pass_total="$(summary_value_from_txt "quality_realistic_expected_pass_total" "$vhdl_family_contract_summary_txt")"
vhdl_quality_realistic_expected_fail_total="$(summary_value_from_txt "quality_realistic_expected_fail_total" "$vhdl_family_contract_summary_txt")"
vhdl_quality_realistic_observed_parse_pass_total="$(summary_value_from_txt "quality_realistic_observed_parse_pass_total" "$vhdl_family_contract_summary_txt")"
vhdl_quality_realistic_observed_parse_fail_total="$(summary_value_from_txt "quality_realistic_observed_parse_fail_total" "$vhdl_family_contract_summary_txt")"
vhdl_strict_promotion_recommendation="$(summary_value_from_txt "strict_promotion_recommendation" "$vhdl_family_contract_summary_txt")"
vhdl_strict_promotion_eligible="$(summary_value_from_txt "strict_promotion_eligible_for_required_strict_mode" "$vhdl_family_contract_summary_txt")"
vhdl_strict_promotion_primary_blocker="$(summary_value_from_txt "strict_promotion_primary_blocker" "$vhdl_family_contract_summary_txt")"
vhdl_strict_promotion_trial_passed="$(summary_value_from_txt "strict_promotion_trial_passed" "$vhdl_family_contract_summary_txt")"

vhdl_combined_quality_initial_status="$(summary_value_from_txt "vhdl_family_quality_closed_loop_initial_status" "$vhdl_combined_telemetry_summary_txt")"
vhdl_combined_quality_replay_status="$(summary_value_from_txt "vhdl_family_quality_closed_loop_replay_status" "$vhdl_combined_telemetry_summary_txt")"
vhdl_combined_quality_replay_targets="$(summary_value_from_txt "vhdl_family_quality_closed_loop_replay_targets" "$vhdl_combined_telemetry_summary_txt")"
vhdl_combined_quality_parseability_rejected="$(summary_value_from_txt "vhdl_family_quality_parseability_generation_rejected_total" "$vhdl_combined_telemetry_summary_txt")"
vhdl_combined_recommendation="$(summary_value_from_txt "vhdl_family_strict_promotion_recommendation" "$vhdl_combined_telemetry_summary_txt")"
vhdl_combined_eligible="$(summary_value_from_txt "vhdl_family_strict_promotion_eligible_for_required_strict_mode" "$vhdl_combined_telemetry_summary_txt")"
vhdl_combined_primary_blocker="$(summary_value_from_txt "vhdl_family_strict_promotion_primary_blocker" "$vhdl_combined_telemetry_summary_txt")"

vhdl_family_contract_green=true
vhdl_aggregate_telemetry_contract_green=true
vhdl_quality_closed_loop_initial_status_pass=false
vhdl_quality_closed_loop_replay_status_pass=false
vhdl_quality_parseability_generation_parser_rejections_zero=false
vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero=false
vhdl_quality_closed_loop_replay_target_debt_zero=false
vhdl_strict_promotion_recommendation_green=false
vhdl_strict_promotion_eligible_for_required_strict_mode=false
vhdl_strict_promotion_primary_blocker_none=false
vhdl_formal_exhaustive_closure_surface_green=false

if [[ "$vhdl_quality_closed_loop_initial_status" == "pass" && "$vhdl_combined_quality_initial_status" == "pass" ]]; then
    vhdl_quality_closed_loop_initial_status_pass=true
fi
if [[ "$vhdl_quality_closed_loop_replay_status" == "pass" && "$vhdl_combined_quality_replay_status" == "pass" ]]; then
    vhdl_quality_closed_loop_replay_status_pass=true
fi
if [[ "$vhdl_quality_parseability_generation_parser_rejections_total" == "0" && "$vhdl_combined_quality_parseability_rejected" == "0" ]]; then
    vhdl_quality_parseability_generation_parser_rejections_zero=true
fi
if [[ "$vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total" == "0" ]]; then
    vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero=true
fi
if [[ "$vhdl_quality_closed_loop_replay_targets" == "0" && "$vhdl_combined_quality_replay_targets" == "0" ]]; then
    vhdl_quality_closed_loop_replay_target_debt_zero=true
fi
if [[ "$vhdl_strict_promotion_recommendation" == "enable_required_strict_mode" && "$vhdl_combined_recommendation" == "enable_required_strict_mode" ]]; then
    vhdl_strict_promotion_recommendation_green=true
fi
if [[ "$vhdl_strict_promotion_eligible" == "1" && "$vhdl_combined_eligible" == "1" ]]; then
    vhdl_strict_promotion_eligible_for_required_strict_mode=true
fi
if [[ "$vhdl_strict_promotion_primary_blocker" == "none" && "$vhdl_combined_primary_blocker" == "none" ]]; then
    vhdl_strict_promotion_primary_blocker_none=true
fi

vhdl_closure_criteria_total_count=11
vhdl_closure_criteria_satisfied_count=0
for criterion in \
    "$vhdl_family_contract_green" \
    "$vhdl_aggregate_telemetry_contract_green" \
    "$vhdl_quality_closed_loop_initial_status_pass" \
    "$vhdl_quality_closed_loop_replay_status_pass" \
    "$vhdl_quality_parseability_generation_parser_rejections_zero" \
    "$vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero" \
    "$vhdl_quality_closed_loop_replay_target_debt_zero" \
    "$vhdl_strict_promotion_recommendation_green" \
    "$vhdl_strict_promotion_eligible_for_required_strict_mode" \
    "$vhdl_strict_promotion_primary_blocker_none" \
    "$vhdl_formal_exhaustive_closure_surface_green"; do
    if [[ "$criterion" == true ]]; then
        ((vhdl_closure_criteria_satisfied_count += 1))
    fi
done
vhdl_closure_criteria_unsatisfied_count=$((vhdl_closure_criteria_total_count - vhdl_closure_criteria_satisfied_count))

declare -a vhdl_unmet=()
declare -a vhdl_unmet_details=()
if [[ "$vhdl_quality_closed_loop_initial_status_pass" != true ]]; then
    vhdl_unmet+=("quality_closed_loop_initial_status=${vhdl_quality_closed_loop_initial_status} != pass")
    vhdl_unmet_details+=("{\"criterion\":\"quality_closed_loop_initial_status_pass\",\"evidence_key\":\"quality_closed_loop_initial_status\",\"observed\":\"${vhdl_quality_closed_loop_initial_status}\",\"expected\":\"pass\",\"detail\":\"The VHDL closed-loop initial status must stay green before the family can be promoted.\"}")
fi
if [[ "$vhdl_quality_closed_loop_replay_status_pass" != true ]]; then
    vhdl_unmet+=("quality_closed_loop_replay_status=${vhdl_quality_closed_loop_replay_status} != pass")
    vhdl_unmet_details+=("{\"criterion\":\"quality_closed_loop_replay_status_pass\",\"evidence_key\":\"quality_closed_loop_replay_status\",\"observed\":\"${vhdl_quality_closed_loop_replay_status}\",\"expected\":\"pass\",\"detail\":\"The VHDL closed-loop replay status must stay green before the family can be promoted.\"}")
fi
if [[ "$vhdl_quality_parseability_generation_parser_rejections_zero" != true ]]; then
    vhdl_unmet+=("quality_parseability_generation_parser_rejections_total=${vhdl_quality_parseability_generation_parser_rejections_total} > 0")
    vhdl_unmet_details+=("{\"criterion\":\"quality_parseability_generation_parser_rejections_zero\",\"evidence_key\":\"quality_parseability_generation_parser_rejections_total\",\"observed\":\"${vhdl_quality_parseability_generation_parser_rejections_total}\",\"expected\":\"0\",\"detail\":\"Current VHDL generation parseability still has bounded parser rejections.\"}")
fi
if [[ "$vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero" != true ]]; then
    vhdl_unmet+=("quality_closed_loop_parseability_shadow_parser_rejections_total=${vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total} > 0")
    vhdl_unmet_details+=("{\"criterion\":\"quality_closed_loop_parseability_shadow_parser_rejections_zero\",\"evidence_key\":\"quality_closed_loop_parseability_shadow_parser_rejections_total\",\"observed\":\"${vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total}\",\"expected\":\"0\",\"detail\":\"Current VHDL closed-loop replay shadow still has bounded parser rejection debt.\"}")
fi
if [[ "$vhdl_quality_closed_loop_replay_target_debt_zero" != true ]]; then
    vhdl_unmet+=("quality_closed_loop_replay_targets=${vhdl_quality_closed_loop_replay_targets} > 0")
    vhdl_unmet_details+=("{\"criterion\":\"quality_closed_loop_replay_target_debt_zero\",\"evidence_key\":\"quality_closed_loop_replay_targets\",\"observed\":\"${vhdl_quality_closed_loop_replay_targets}\",\"expected\":\"0\",\"detail\":\"Current VHDL replay closed-loop target debt is still non-zero.\"}")
fi
if [[ "$vhdl_strict_promotion_recommendation_green" != true ]]; then
    vhdl_unmet+=("strict_promotion_recommendation=${vhdl_strict_promotion_recommendation} != enable_required_strict_mode")
    vhdl_unmet_details+=("{\"criterion\":\"strict_promotion_recommendation_green\",\"evidence_key\":\"strict_promotion_recommendation\",\"observed\":\"${vhdl_strict_promotion_recommendation}\",\"expected\":\"enable_required_strict_mode\",\"detail\":\"VHDL strict-promotion recommendation must stay green before the family can be promoted.\"}")
fi
if [[ "$vhdl_strict_promotion_eligible_for_required_strict_mode" != true ]]; then
    vhdl_unmet+=("strict_promotion_eligible_for_required_strict_mode=${vhdl_strict_promotion_eligible} != 1")
    vhdl_unmet_details+=("{\"criterion\":\"strict_promotion_eligible_for_required_strict_mode\",\"evidence_key\":\"strict_promotion_eligible_for_required_strict_mode\",\"observed\":\"${vhdl_strict_promotion_eligible}\",\"expected\":\"1\",\"detail\":\"VHDL strict-promotion eligibility must remain green before the family can be promoted.\"}")
fi
if [[ "$vhdl_strict_promotion_primary_blocker_none" != true ]]; then
    vhdl_unmet+=("strict_promotion_primary_blocker=${vhdl_strict_promotion_primary_blocker} != none")
    vhdl_unmet_details+=("{\"criterion\":\"strict_promotion_primary_blocker_none\",\"evidence_key\":\"strict_promotion_primary_blocker\",\"observed\":\"${vhdl_strict_promotion_primary_blocker}\",\"expected\":\"none\",\"detail\":\"The VHDL strict-promotion blocker surface must be empty before the family can be promoted.\"}")
fi
if [[ "$vhdl_formal_exhaustive_closure_surface_green" != true ]]; then
    vhdl_unmet+=("formal_exhaustive_closure_surface=missing")
    vhdl_unmet_details+=("{\"criterion\":\"formal_exhaustive_closure_surface_green\",\"evidence_key\":\"formal_exhaustive_closure_surface\",\"observed\":\"missing\",\"expected\":\"present_and_green\",\"detail\":\"No formal exhaustive VHDL closure gate is wired yet, so the repository Done rule cannot be met.\"}")
fi

vhdl_status="Not Started"
if [[ "$vhdl_family_contract_green" == true || "$vhdl_aggregate_telemetry_contract_green" == true ]]; then
    vhdl_status="In Progress"
fi
if [[ "$vhdl_family_contract_green" == true \
   && "$vhdl_aggregate_telemetry_contract_green" == true \
   && "$vhdl_quality_closed_loop_initial_status_pass" == true \
   && "$vhdl_quality_closed_loop_replay_status_pass" == true \
   && "$vhdl_quality_parseability_generation_parser_rejections_zero" == true \
   && "$vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero" == true \
   && "$vhdl_quality_closed_loop_replay_target_debt_zero" == true \
   && "$vhdl_strict_promotion_recommendation_green" == true \
   && "$vhdl_strict_promotion_eligible_for_required_strict_mode" == true \
   && "$vhdl_strict_promotion_primary_blocker_none" == true ]]; then
    vhdl_status="Mostly Done"
fi
if [[ "$vhdl_status" == "Mostly Done" && "$vhdl_formal_exhaustive_closure_surface_green" == true ]]; then
    vhdl_status="Done"
fi

vhdl_tracker_status="$(markdown_table_status_for_row "| \`vhdl\` parser family |" "$LIVE_TRACKER_FILE")"
vhdl_tracker_alignment_ok=false
if [[ "$vhdl_status" == "$vhdl_tracker_status" ]]; then
    vhdl_tracker_alignment_ok=true
fi
if [[ "$vhdl_tracker_alignment_ok" != true ]]; then
    echo "error: VHDL tracker alignment mismatch: computed '${vhdl_status}' but tracker says '${vhdl_tracker_status}'" >&2
    exit 1
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
vhdl_unmet_count="${#vhdl_unmet[@]}"
vhdl_primary_unmet_closure_criterion="<none>"
if [[ "$vhdl_unmet_count" -gt 0 ]]; then
    vhdl_primary_unmet_closure_criterion="${vhdl_unmet[0]}"
fi
vhdl_unmet_json="$(printf '%s\n' "${vhdl_unmet[@]:-}" | jq -R . | jq -sc 'map(select(length > 0))')"
vhdl_unmet_details_json="$(printf '%s\n' "${vhdl_unmet_details[@]:-}" | jq -R . | jq -sc 'map(select(length > 0) | fromjson)')"

{
    echo "VHDL Parser Family Status Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "live_tracker_file: $LIVE_TRACKER_FILE"
    echo "status_rule_done: $DONE_RULE"
    echo "vhdl_status: $vhdl_status"
    echo "vhdl_tracker_status: $vhdl_tracker_status"
    echo "vhdl_tracker_alignment_ok: $vhdl_tracker_alignment_ok"
    echo "vhdl_unmet_closure_criteria_count: $vhdl_unmet_count"
    echo "vhdl_primary_unmet_closure_criterion: $vhdl_primary_unmet_closure_criterion"
    for i in "${!vhdl_unmet[@]}"; do
        echo "vhdl_unmet_closure_criterion[$i]: ${vhdl_unmet[$i]}"
    done
    echo "vhdl_unmet_closure_criteria_json: $vhdl_unmet_json"
    echo "vhdl_unmet_closure_criteria_details_json: $vhdl_unmet_details_json"
    echo "vhdl_closure_criteria_total_count: $vhdl_closure_criteria_total_count"
    echo "vhdl_closure_criteria_satisfied_count: $vhdl_closure_criteria_satisfied_count"
    echo "vhdl_closure_criteria_unsatisfied_count: $vhdl_closure_criteria_unsatisfied_count"
    echo "vhdl_family_contract_green: $vhdl_family_contract_green"
    echo "vhdl_aggregate_telemetry_contract_green: $vhdl_aggregate_telemetry_contract_green"
    echo "vhdl_quality_closed_loop_initial_status_pass: $vhdl_quality_closed_loop_initial_status_pass"
    echo "vhdl_quality_closed_loop_replay_status_pass: $vhdl_quality_closed_loop_replay_status_pass"
    echo "vhdl_quality_parseability_generation_parser_rejections_zero: $vhdl_quality_parseability_generation_parser_rejections_zero"
    echo "vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero: $vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero"
    echo "vhdl_quality_closed_loop_replay_target_debt_zero: $vhdl_quality_closed_loop_replay_target_debt_zero"
    echo "vhdl_strict_promotion_recommendation_green: $vhdl_strict_promotion_recommendation_green"
    echo "vhdl_strict_promotion_eligible_for_required_strict_mode: $vhdl_strict_promotion_eligible_for_required_strict_mode"
    echo "vhdl_strict_promotion_primary_blocker_none: $vhdl_strict_promotion_primary_blocker_none"
    echo "vhdl_formal_exhaustive_closure_surface_green: $vhdl_formal_exhaustive_closure_surface_green"
    echo "vhdl_quality_closed_loop_initial_status: $vhdl_quality_closed_loop_initial_status"
    echo "vhdl_quality_closed_loop_replay_status: $vhdl_quality_closed_loop_replay_status"
    echo "vhdl_quality_closed_loop_replay_targets: $vhdl_quality_closed_loop_replay_targets"
    echo "vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total: $vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total"
    echo "vhdl_quality_parseability_generation_parser_rejections_total: $vhdl_quality_parseability_generation_parser_rejections_total"
    echo "vhdl_quality_parseability_generation_rejected_total: $vhdl_quality_parseability_generation_rejected_total"
    echo "vhdl_quality_realistic_cases_executed: $vhdl_quality_realistic_cases_executed"
    echo "vhdl_quality_realistic_expected_pass_total: $vhdl_quality_realistic_expected_pass_total"
    echo "vhdl_quality_realistic_expected_fail_total: $vhdl_quality_realistic_expected_fail_total"
    echo "vhdl_quality_realistic_observed_parse_pass_total: $vhdl_quality_realistic_observed_parse_pass_total"
    echo "vhdl_quality_realistic_observed_parse_fail_total: $vhdl_quality_realistic_observed_parse_fail_total"
    echo "vhdl_strict_promotion_recommendation: $vhdl_strict_promotion_recommendation"
    echo "vhdl_strict_promotion_eligible: $vhdl_strict_promotion_eligible"
    echo "vhdl_strict_promotion_primary_blocker: $vhdl_strict_promotion_primary_blocker"
    echo "vhdl_strict_promotion_trial_passed: $vhdl_strict_promotion_trial_passed"
    echo "vhdl_family_contract_summary_txt: $vhdl_family_contract_summary_txt"
    echo "vhdl_combined_telemetry_summary_txt: $vhdl_combined_telemetry_summary_txt"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "vhdl_parser_family_status_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg live_tracker_file "$LIVE_TRACKER_FILE" \
    --arg status_rule_done "$DONE_RULE" \
    --arg vhdl_status "$vhdl_status" \
    --arg vhdl_tracker_status "$vhdl_tracker_status" \
    --argjson vhdl_tracker_alignment_ok "$vhdl_tracker_alignment_ok" \
    --arg vhdl_primary_unmet_closure_criterion "$vhdl_primary_unmet_closure_criterion" \
    --argjson vhdl_unmet_closure_criteria_count "$vhdl_unmet_count" \
    --argjson vhdl_unmet_closure_criteria "$vhdl_unmet_json" \
    --argjson vhdl_unmet_closure_criteria_details "$vhdl_unmet_details_json" \
    --argjson vhdl_closure_criteria_total_count "$vhdl_closure_criteria_total_count" \
    --argjson vhdl_closure_criteria_satisfied_count "$vhdl_closure_criteria_satisfied_count" \
    --argjson vhdl_closure_criteria_unsatisfied_count "$vhdl_closure_criteria_unsatisfied_count" \
    --argjson vhdl_family_contract_green "$vhdl_family_contract_green" \
    --argjson vhdl_aggregate_telemetry_contract_green "$vhdl_aggregate_telemetry_contract_green" \
    --argjson vhdl_quality_closed_loop_initial_status_pass "$vhdl_quality_closed_loop_initial_status_pass" \
    --argjson vhdl_quality_closed_loop_replay_status_pass "$vhdl_quality_closed_loop_replay_status_pass" \
    --argjson vhdl_quality_parseability_generation_parser_rejections_zero "$vhdl_quality_parseability_generation_parser_rejections_zero" \
    --argjson vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero "$vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero" \
    --argjson vhdl_quality_closed_loop_replay_target_debt_zero "$vhdl_quality_closed_loop_replay_target_debt_zero" \
    --argjson vhdl_strict_promotion_recommendation_green "$vhdl_strict_promotion_recommendation_green" \
    --argjson vhdl_strict_promotion_eligible_for_required_strict_mode "$vhdl_strict_promotion_eligible_for_required_strict_mode" \
    --argjson vhdl_strict_promotion_primary_blocker_none "$vhdl_strict_promotion_primary_blocker_none" \
    --argjson vhdl_formal_exhaustive_closure_surface_green "$vhdl_formal_exhaustive_closure_surface_green" \
    --arg vhdl_quality_closed_loop_initial_status "$vhdl_quality_closed_loop_initial_status" \
    --arg vhdl_quality_closed_loop_replay_status "$vhdl_quality_closed_loop_replay_status" \
    --argjson vhdl_quality_closed_loop_replay_targets "$vhdl_quality_closed_loop_replay_targets" \
    --argjson vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total "$vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total" \
    --argjson vhdl_quality_parseability_generation_parser_rejections_total "$vhdl_quality_parseability_generation_parser_rejections_total" \
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
    --arg vhdl_family_contract_summary_txt "$vhdl_family_contract_summary_txt" \
    --arg vhdl_combined_telemetry_summary_txt "$vhdl_combined_telemetry_summary_txt" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      live_tracker_file: $live_tracker_file,
      status_rule_done: $status_rule_done,
      families: [
        {
          family: "vhdl",
          computed_status: $vhdl_status,
          live_tracker_status: $vhdl_tracker_status,
          tracker_alignment_ok: $vhdl_tracker_alignment_ok,
          primary_unmet_closure_criterion: $vhdl_primary_unmet_closure_criterion,
          unmet_closure_criteria_count: $vhdl_unmet_closure_criteria_count,
          unmet_closure_criteria: $vhdl_unmet_closure_criteria,
          unmet_closure_criteria_details: $vhdl_unmet_closure_criteria_details,
          closure_criteria_total_count: $vhdl_closure_criteria_total_count,
          closure_criteria_satisfied_count: $vhdl_closure_criteria_satisfied_count,
          closure_criteria_unsatisfied_count: $vhdl_closure_criteria_unsatisfied_count,
          criteria: {
            family_contract_green: $vhdl_family_contract_green,
            aggregate_telemetry_contract_green: $vhdl_aggregate_telemetry_contract_green,
            quality_closed_loop_initial_status_pass: $vhdl_quality_closed_loop_initial_status_pass,
            quality_closed_loop_replay_status_pass: $vhdl_quality_closed_loop_replay_status_pass,
            quality_parseability_generation_parser_rejections_zero: $vhdl_quality_parseability_generation_parser_rejections_zero,
            quality_closed_loop_parseability_shadow_parser_rejections_zero: $vhdl_quality_closed_loop_parseability_shadow_parser_rejections_zero,
            quality_closed_loop_replay_target_debt_zero: $vhdl_quality_closed_loop_replay_target_debt_zero,
            strict_promotion_recommendation_green: $vhdl_strict_promotion_recommendation_green,
            strict_promotion_eligible_for_required_strict_mode: $vhdl_strict_promotion_eligible_for_required_strict_mode,
            strict_promotion_primary_blocker_none: $vhdl_strict_promotion_primary_blocker_none,
            formal_exhaustive_closure_surface_green: $vhdl_formal_exhaustive_closure_surface_green
          },
          metrics: {
            quality_closed_loop_initial_status: $vhdl_quality_closed_loop_initial_status,
            quality_closed_loop_replay_status: $vhdl_quality_closed_loop_replay_status,
            quality_closed_loop_replay_targets: $vhdl_quality_closed_loop_replay_targets,
            quality_closed_loop_parseability_shadow_parser_rejections_total: $vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total,
            quality_parseability_generation_parser_rejections_total: $vhdl_quality_parseability_generation_parser_rejections_total,
            quality_parseability_generation_rejected_total: $vhdl_quality_parseability_generation_rejected_total,
            quality_realistic_cases_executed: $vhdl_quality_realistic_cases_executed,
            quality_realistic_expected_pass_total: $vhdl_quality_realistic_expected_pass_total,
            quality_realistic_expected_fail_total: $vhdl_quality_realistic_expected_fail_total,
            quality_realistic_observed_parse_pass_total: $vhdl_quality_realistic_observed_parse_pass_total,
            quality_realistic_observed_parse_fail_total: $vhdl_quality_realistic_observed_parse_fail_total,
            strict_promotion_recommendation: $vhdl_strict_promotion_recommendation,
            strict_promotion_eligible: $vhdl_strict_promotion_eligible,
            strict_promotion_primary_blocker: $vhdl_strict_promotion_primary_blocker,
            strict_promotion_trial_passed: $vhdl_strict_promotion_trial_passed
          },
          proof_surfaces: {
            family_contract_summary_txt: $vhdl_family_contract_summary_txt,
            combined_telemetry_summary_txt: $vhdl_combined_telemetry_summary_txt
          }
        }
      ]
    }' >"$SUMMARY_JSON"

echo "✅ VHDL parser-family status gate passed."
