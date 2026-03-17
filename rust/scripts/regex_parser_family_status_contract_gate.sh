#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_CONTRACT_STATE_DIR:-$RUST_DIR/target/regex_parser_family_status_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

REGEX_FAMILY_STATUS_GATE="$RUST_DIR/scripts/regex_parser_family_status_gate.sh"
EXISTING_FAMILY_STATUS_STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_CONTRACT_EXISTING_STATE_DIR:-}"

EXISTING_REGEX_FAMILY_CONTRACT_STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_CONTRACT_EXISTING_FAMILY_CONTRACT_STATE_DIR:-}"
EXISTING_REGEX_FRONTEND_STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_CONTRACT_EXISTING_FRONTEND_STATE_DIR:-}"
EXISTING_REGEX_DUAL_RUN_STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_CONTRACT_EXISTING_DUAL_RUN_STATE_DIR:-}"
EXISTING_REGEX_STIMULI_STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_CONTRACT_EXISTING_STIMULI_STATE_DIR:-}"

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
require_file "$REGEX_FAMILY_STATUS_GATE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

family_status_state_dir="${EXISTING_FAMILY_STATUS_STATE_DIR:-$WORK_DIR/regex_parser_family_status_gate}"

if [[ -z "$EXISTING_FAMILY_STATUS_STATE_DIR" ]]; then
    family_status_env=(
        env
        PGEN_REGEX_FAMILY_STATUS_STATE_DIR="$family_status_state_dir"
    )
    if [[ -n "$EXISTING_REGEX_FAMILY_CONTRACT_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_REGEX_FAMILY_STATUS_EXISTING_FAMILY_CONTRACT_STATE_DIR="$EXISTING_REGEX_FAMILY_CONTRACT_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_REGEX_FRONTEND_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_REGEX_FAMILY_STATUS_EXISTING_FRONTEND_STATE_DIR="$EXISTING_REGEX_FRONTEND_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_REGEX_DUAL_RUN_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_REGEX_FAMILY_STATUS_EXISTING_DUAL_RUN_STATE_DIR="$EXISTING_REGEX_DUAL_RUN_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_REGEX_STIMULI_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_REGEX_FAMILY_STATUS_EXISTING_STIMULI_STATE_DIR="$EXISTING_REGEX_STIMULI_STATE_DIR"
        )
    fi
    family_status_env+=("$REGEX_FAMILY_STATUS_GATE")
    run_logged "regex_parser_family_status_gate" "${family_status_env[@]}"
fi

family_status_summary_json="$family_status_state_dir/summary.json"
family_status_summary_txt="$family_status_state_dir/summary.txt"

require_nonempty_file "$family_status_summary_json"
require_nonempty_file "$family_status_summary_txt"

expected_criteria='["family_contract_green","frontend_overall_pass","dual_run_overall_pass","dual_run_raw_ast_missing_on_rust_zero","stimuli_status_pass","stimuli_final_target_debt_zero","formal_exhaustive_closure_surface_green"]'
expected_metrics='["frontend_overall","dual_run_overall","dual_run_raw_ast_status","dual_run_raw_ast_missing_on_perl_count","dual_run_raw_ast_missing_on_rust_count","dual_run_rust_rule_count","stimuli_initial_targets","stimuli_resolved_targets","stimuli_final_targets","stimuli_target_attempts","stimuli_stage0_successes","stimuli_stage3_successes","stimuli_status"]'
expected_proof_surfaces='["family_contract_summary_txt","family_contract_summary_json"]'

jq -e \
    --argjson expected_criteria "$expected_criteria" \
    --argjson expected_metrics "$expected_metrics" \
    --argjson expected_proof_surfaces "$expected_proof_surfaces" \
    '
    . as $root
    | ($root.gate == "regex_parser_family_status_gate")
    and (($root.version | type) == "number")
    and (($root.generated_at_utc | type) == "string" and ($root.generated_at_utc | length) > 0)
    and (($root.live_tracker_file | type) == "string" and ($root.live_tracker_file | length) > 0)
    and (($root.status_rule_done | type) == "string" and ($root.status_rule_done | length) > 0)
    and (($root.families | length) == 1)
    and (($root.families[0].family) == "regex")
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

regex_tracker_alignment_ok="$(jq -r '.families[0].tracker_alignment_ok' "$family_status_summary_json")"
regex_false_criteria_count="$(jq -r '[.families[0].criteria | to_entries[] | select(.value == false)] | length' "$family_status_summary_json")"
regex_details_count="$(jq -r '.families[0].unmet_closure_criteria_details | length' "$family_status_summary_json")"
regex_primary_unmet_detail_criterion="$(jq -r '.families[0].unmet_closure_criteria_details[0].criterion // "<none>"' "$family_status_summary_json")"
regex_details_json="$(jq -cer '.families[0].unmet_closure_criteria_details' "$family_status_summary_json")"
regex_unmet_json="$(jq -cer '.families[0].unmet_closure_criteria' "$family_status_summary_json")"
regex_primary_unmet_from_json="$(jq -r '.families[0].primary_unmet_closure_criterion' "$family_status_summary_json")"

summary_regex_details_json="$(extract_summary_value "$family_status_summary_txt" "regex_unmet_closure_criteria_details_json")"
summary_regex_unmet_json="$(extract_summary_value "$family_status_summary_txt" "regex_unmet_closure_criteria_json")"
summary_regex_tracker_alignment="$(extract_summary_value "$family_status_summary_txt" "regex_tracker_alignment_ok")"
summary_regex_primary_unmet="$(extract_summary_value "$family_status_summary_txt" "regex_primary_unmet_closure_criterion")"

if [[ "$summary_regex_details_json" != "$regex_details_json" ]]; then
    echo "error: regex structured blocker json mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_regex_unmet_json" != "$regex_unmet_json" ]]; then
    echo "error: regex unmet-criteria json mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_regex_tracker_alignment" != "$regex_tracker_alignment_ok" ]]; then
    echo "error: regex tracker alignment mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_regex_primary_unmet" != "$regex_primary_unmet_from_json" ]]; then
    echo "error: regex primary unmet criterion mismatch between summary.txt and summary.json" >&2
    exit 1
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "Regex Parser Family Status Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "family_status_state_dir: $family_status_state_dir"
    echo "family_status_summary_json: $family_status_summary_json"
    echo "family_status_summary_txt: $family_status_summary_txt"
    echo "family_count: 1"
    echo "regex_tracker_alignment_ok: $regex_tracker_alignment_ok"
    echo "regex_false_criteria_count: $regex_false_criteria_count"
    echo "regex_unmet_details_count: $regex_details_count"
    echo "regex_primary_unmet_detail_criterion: $regex_primary_unmet_detail_criterion"
    echo "regex_unmet_closure_criteria_json: $regex_unmet_json"
    echo "regex_unmet_closure_criteria_details_json: $regex_details_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "regex_parser_family_status_contract_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg family_status_state_dir "$family_status_state_dir" \
    --arg family_status_summary_json "$family_status_summary_json" \
    --arg family_status_summary_txt "$family_status_summary_txt" \
    --argjson family_count 1 \
    --argjson regex_tracker_alignment_ok "$regex_tracker_alignment_ok" \
    --argjson regex_false_criteria_count "$regex_false_criteria_count" \
    --argjson regex_unmet_details_count "$regex_details_count" \
    --arg regex_primary_unmet_detail_criterion "$regex_primary_unmet_detail_criterion" \
    --argjson regex_unmet_closure_criteria "$regex_unmet_json" \
    --argjson regex_unmet_closure_criteria_details "$regex_details_json" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      family_status_state_dir: $family_status_state_dir,
      family_status_summary_json: $family_status_summary_json,
      family_status_summary_txt: $family_status_summary_txt,
      family_count: $family_count,
      families: [
        {
          family: "regex",
          tracker_alignment_ok: $regex_tracker_alignment_ok,
          false_criteria_count: $regex_false_criteria_count,
          unmet_details_count: $regex_unmet_details_count,
          primary_unmet_detail_criterion: $regex_primary_unmet_detail_criterion,
          unmet_closure_criteria: $regex_unmet_closure_criteria,
          unmet_closure_criteria_details: $regex_unmet_closure_criteria_details
        }
      ]
    }' >"$SUMMARY_JSON"

echo "✅ Regex parser-family status contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
