#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_CONTRACT_STATE_DIR:-$RUST_DIR/target/vhdl_parser_family_status_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

VHDL_FAMILY_STATUS_GATE="$RUST_DIR/scripts/vhdl_parser_family_status_gate.sh"
EXISTING_FAMILY_STATUS_STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_CONTRACT_EXISTING_STATE_DIR:-}"

EXISTING_VHDL_FAMILY_CONTRACT_STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_CONTRACT_EXISTING_FAMILY_CONTRACT_STATE_DIR:-}"
EXISTING_VHDL_QUALITY_STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_CONTRACT_EXISTING_QUALITY_STATE_DIR:-}"
EXISTING_VHDL_STRICT_PROMOTION_STATE_DIR="${PGEN_VHDL_FAMILY_STATUS_CONTRACT_EXISTING_STRICT_PROMOTION_STATE_DIR:-}"

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

extract_summary_value() {
    local path="$1"
    local key="$2"
    local line
    line="$(grep -F "${key}: " "$path" | tail -n 1 || true)"
    if [[ -z "$line" ]]; then
        echo "error: missing key '${key}' in summary '$path'" >&2
        exit 1
    fi
    printf '%s\n' "${line#${key}: }"
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
require_file "$VHDL_FAMILY_STATUS_GATE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

family_status_state_dir="${EXISTING_FAMILY_STATUS_STATE_DIR:-$WORK_DIR/vhdl_parser_family_status_gate}"

if [[ -z "$EXISTING_FAMILY_STATUS_STATE_DIR" ]]; then
    family_status_env=(
        env
        PGEN_VHDL_FAMILY_STATUS_STATE_DIR="$family_status_state_dir"
    )
    if [[ -n "$EXISTING_VHDL_FAMILY_CONTRACT_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_VHDL_FAMILY_STATUS_EXISTING_FAMILY_CONTRACT_STATE_DIR="$EXISTING_VHDL_FAMILY_CONTRACT_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_VHDL_QUALITY_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_VHDL_FAMILY_STATUS_EXISTING_QUALITY_STATE_DIR="$EXISTING_VHDL_QUALITY_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_VHDL_STRICT_PROMOTION_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_VHDL_FAMILY_STATUS_EXISTING_STRICT_PROMOTION_STATE_DIR="$EXISTING_VHDL_STRICT_PROMOTION_STATE_DIR"
        )
    fi
    family_status_env+=("$VHDL_FAMILY_STATUS_GATE")
    run_logged "vhdl_parser_family_status_gate" "${family_status_env[@]}"
fi

family_status_summary_json="$family_status_state_dir/summary.json"
family_status_summary_txt="$family_status_state_dir/summary.txt"

require_nonempty_file "$family_status_summary_json"
require_nonempty_file "$family_status_summary_txt"

expected_criteria='["family_contract_green","quality_closed_loop_initial_status_pass","quality_closed_loop_replay_status_pass","quality_parseability_generation_parser_rejections_zero","quality_closed_loop_parseability_shadow_parser_rejections_zero","quality_closed_loop_replay_target_debt_zero","strict_promotion_recommendation_green","strict_promotion_eligible_for_required_strict_mode","strict_promotion_primary_blocker_none","formal_exhaustive_closure_surface_green"]'
expected_metrics='["quality_closed_loop_initial_status","quality_closed_loop_replay_status","quality_closed_loop_replay_targets","quality_closed_loop_parseability_shadow_parser_rejections_total","quality_parseability_generation_parser_rejections_total","quality_parseability_generation_rejected_total","quality_realistic_cases_executed","quality_realistic_expected_pass_total","quality_realistic_expected_fail_total","quality_realistic_observed_parse_pass_total","quality_realistic_observed_parse_fail_total","strict_promotion_recommendation","strict_promotion_eligible","strict_promotion_primary_blocker","strict_promotion_trial_passed"]'
expected_proof_surfaces='["family_contract_summary_txt"]'

jq -e \
    --argjson expected_criteria "$expected_criteria" \
    --argjson expected_metrics "$expected_metrics" \
    --argjson expected_proof_surfaces "$expected_proof_surfaces" \
    '
    . as $root
    | ($root.gate == "vhdl_parser_family_status_gate")
    and (($root.version | type) == "number")
    and (($root.generated_at_utc | type) == "string" and ($root.generated_at_utc | length) > 0)
    and (($root.live_tracker_file | type) == "string" and ($root.live_tracker_file | length) > 0)
    and (($root.status_rule_done | type) == "string" and ($root.status_rule_done | length) > 0)
    and (($root.families | length) == 1)
    and (($root.families[0].family) == "vhdl")
    and (
        $root.families[0] as $family
        | ($family | (
            has("family")
            and has("computed_status")
            and has("live_tracker_status")
            and has("tracker_alignment_ok")
            and has("primary_unmet_closure_criterion")
            and has("unmet_closure_criteria_count")
            and has("unmet_closure_criteria")
            and has("unmet_closure_criteria_details")
            and has("closure_criteria_total_count")
            and has("closure_criteria_satisfied_count")
            and has("closure_criteria_unsatisfied_count")
            and has("criteria")
            and has("metrics")
            and has("proof_surfaces")
        ))
        and (($family.tracker_alignment_ok | type) == "boolean")
        and ($family.tracker_alignment_ok == ($family.computed_status == $family.live_tracker_status))
        and (($family.primary_unmet_closure_criterion | type) == "string")
        and (($family.unmet_closure_criteria_count | type) == "number")
        and (($family.unmet_closure_criteria | type) == "array")
        and (($family.unmet_closure_criteria_details | type) == "array")
        and (($family.unmet_closure_criteria | length) == $family.unmet_closure_criteria_count)
        and (($family.unmet_closure_criteria_details | length) == $family.unmet_closure_criteria_count)
        and ($family.closure_criteria_satisfied_count + $family.closure_criteria_unsatisfied_count == $family.closure_criteria_total_count)
        and (($family.unmet_closure_criteria | length) == $family.closure_criteria_unsatisfied_count)
        and (([($family.criteria | to_entries[] | select(.value == false))] | length) == $family.closure_criteria_unsatisfied_count)
        and (
            if $family.unmet_closure_criteria_count == 0 then
                ($family.primary_unmet_closure_criterion == "<none>")
            else
                ($family.primary_unmet_closure_criterion == $family.unmet_closure_criteria[0])
            end
        )
        and (
            all($family.unmet_closure_criteria_details[]?;
                ((.criterion | type) == "string")
                and ((.evidence_key | type) == "string")
                and ((.observed | type) == "string")
                and ((.expected | type) == "string")
                and ((.detail | type) == "string")
                and ($family.criteria[.criterion] == false)
            )
        )
        and (all($expected_criteria[]; . as $k | ($family.criteria | has($k))))
        and (all($expected_metrics[]; . as $k | ($family.metrics | has($k))))
        and (all($expected_proof_surfaces[]; . as $k | ($family.proof_surfaces | has($k))))
    )
    ' "$family_status_summary_json" >/dev/null

vhdl_tracker_alignment_ok="$(jq -r '.families[0].tracker_alignment_ok' "$family_status_summary_json")"
vhdl_unsatisfied_count="$(jq -r '.families[0].closure_criteria_unsatisfied_count' "$family_status_summary_json")"
vhdl_false_criteria_count="$(jq -r '[.families[0].criteria | to_entries[] | select(.value == false)] | length' "$family_status_summary_json")"
vhdl_details_count="$(jq -r '.families[0].unmet_closure_criteria_details | length' "$family_status_summary_json")"
vhdl_primary_unmet_detail_criterion="$(jq -r '.families[0].unmet_closure_criteria_details[0].criterion // "<none>"' "$family_status_summary_json")"
vhdl_details_json="$(jq -cer '.families[0].unmet_closure_criteria_details' "$family_status_summary_json")"
vhdl_unmet_json="$(jq -cer '.families[0].unmet_closure_criteria' "$family_status_summary_json")"
vhdl_primary_unmet_from_json="$(jq -r '.families[0].primary_unmet_closure_criterion' "$family_status_summary_json")"

summary_vhdl_details_json="$(extract_summary_value "$family_status_summary_txt" "vhdl_unmet_closure_criteria_details_json")"
summary_vhdl_unmet_json="$(extract_summary_value "$family_status_summary_txt" "vhdl_unmet_closure_criteria_json")"
summary_vhdl_tracker_alignment="$(extract_summary_value "$family_status_summary_txt" "vhdl_tracker_alignment_ok")"
summary_vhdl_primary_unmet="$(extract_summary_value "$family_status_summary_txt" "vhdl_primary_unmet_closure_criterion")"

if [[ "$summary_vhdl_details_json" != "$vhdl_details_json" ]]; then
    echo "error: vhdl structured blocker json mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_vhdl_unmet_json" != "$vhdl_unmet_json" ]]; then
    echo "error: vhdl unmet-criteria json mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_vhdl_tracker_alignment" != "$vhdl_tracker_alignment_ok" ]]; then
    echo "error: vhdl tracker alignment mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_vhdl_primary_unmet" != "$vhdl_primary_unmet_from_json" ]]; then
    echo "error: vhdl primary unmet criterion mismatch between summary.txt and summary.json" >&2
    exit 1
fi

{
    echo "VHDL Parser Family Status Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "family_status_state_dir: $family_status_state_dir"
    echo "family_status_summary_json: $family_status_summary_json"
    echo "family_status_summary_txt: $family_status_summary_txt"
    echo "family_count: 1"
    echo "vhdl_tracker_alignment_ok: $vhdl_tracker_alignment_ok"
    echo "vhdl_false_criteria_count: $vhdl_false_criteria_count"
    echo "vhdl_unmet_details_count: $vhdl_details_count"
    echo "vhdl_primary_unmet_detail_criterion: $vhdl_primary_unmet_detail_criterion"
} | tee "$SUMMARY_TXT"

echo "✅ VHDL parser-family status contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
