#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_STATE_DIR:-$RUST_DIR/target/regex_parser_family_status_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_JSON="$STATE_DIR/summary.json"
SUMMARY_TXT="$STATE_DIR/summary.txt"
LIVE_TRACKER_FILE="$ROOT_DIR/LIVE_ACHIEVEMENT_STATUS.md"

REGEX_FAMILY_CONTRACT_GATE="$RUST_DIR/scripts/regex_parser_family_contract_gate.sh"

EXISTING_REGEX_FAMILY_CONTRACT_STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_EXISTING_FAMILY_CONTRACT_STATE_DIR:-}"
EXISTING_REGEX_FRONTEND_STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_EXISTING_FRONTEND_STATE_DIR:-}"
EXISTING_REGEX_DUAL_RUN_STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_EXISTING_DUAL_RUN_STATE_DIR:-}"
EXISTING_REGEX_STIMULI_STATE_DIR="${PGEN_REGEX_FAMILY_STATUS_EXISTING_STIMULI_STATE_DIR:-}"

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

top_level_summary_value_from_txt() {
    local key="$1"
    local path="$2"
    local line
    line="$(awk -v key="$key" 'index($0, key ": ") == 1 { print; exit }' "$path")"
    if [[ -z "$line" ]]; then
        echo "error: missing top-level key '${key}' in summary '$path'" >&2
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
require_file "$REGEX_FAMILY_CONTRACT_GATE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

regex_family_contract_state_dir="${EXISTING_REGEX_FAMILY_CONTRACT_STATE_DIR:-$WORK_DIR/regex_parser_family_contract_gate}"

if [[ -z "$EXISTING_REGEX_FAMILY_CONTRACT_STATE_DIR" ]]; then
    regex_family_contract_cmd=(
        env
        PGEN_REGEX_FAMILY_CONTRACT_STATE_DIR="$regex_family_contract_state_dir"
    )
    if [[ -n "$EXISTING_REGEX_FRONTEND_STATE_DIR" ]]; then
        regex_family_contract_cmd+=(
            PGEN_REGEX_FAMILY_CONTRACT_EXISTING_FRONTEND_STATE_DIR="$EXISTING_REGEX_FRONTEND_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_REGEX_DUAL_RUN_STATE_DIR" ]]; then
        regex_family_contract_cmd+=(
            PGEN_REGEX_FAMILY_CONTRACT_EXISTING_DUAL_RUN_STATE_DIR="$EXISTING_REGEX_DUAL_RUN_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_REGEX_STIMULI_STATE_DIR" ]]; then
        regex_family_contract_cmd+=(
            PGEN_REGEX_FAMILY_CONTRACT_EXISTING_STIMULI_STATE_DIR="$EXISTING_REGEX_STIMULI_STATE_DIR"
        )
    fi
    regex_family_contract_cmd+=("$REGEX_FAMILY_CONTRACT_GATE")
    run_logged "regex_parser_family_contract_gate" "${regex_family_contract_cmd[@]}"
fi

regex_family_contract_summary_txt="$regex_family_contract_state_dir/summary.txt"
regex_family_contract_summary_json="$regex_family_contract_state_dir/summary.json"

require_nonempty_file "$regex_family_contract_summary_txt"
require_nonempty_file "$regex_family_contract_summary_json"

regex_family_contract_gate_name="$(jq -r '.gate' "$regex_family_contract_summary_json")"
regex_family_contract_gate_version="$(jq -r '.version' "$regex_family_contract_summary_json")"
regex_family_contract_generated_at_utc="$(jq -r '.generated_at_utc' "$regex_family_contract_summary_json")"
regex_family_contract_state_dir_from_json="$(jq -r '.state_dir' "$regex_family_contract_summary_json")"
regex_family_contract_summary_txt_from_json="$(jq -r '.summary_txt' "$regex_family_contract_summary_json")"
regex_family_contract_summary_json_from_json="$(jq -r '.summary_json' "$regex_family_contract_summary_json")"

summary_regex_family_contract_state_dir="$(top_level_summary_value_from_txt "state_dir" "$regex_family_contract_summary_txt")"
summary_regex_family_contract_generated_at_utc="$(top_level_summary_value_from_txt "generated_at_utc" "$regex_family_contract_summary_txt")"
summary_regex_family_contract_summary_json="$(top_level_summary_value_from_txt "summary_json" "$regex_family_contract_summary_txt")"

if [[ "$regex_family_contract_gate_name" != "regex_parser_family_contract_gate" ]]; then
    echo "error: unexpected regex family-contract gate identity '$regex_family_contract_gate_name'" >&2
    exit 1
fi
if [[ ! "$regex_family_contract_gate_version" =~ ^[0-9]+$ ]]; then
    echo "error: regex family-contract gate version is not numeric: '$regex_family_contract_gate_version'" >&2
    exit 1
fi
if [[ -z "$regex_family_contract_generated_at_utc" ]]; then
    echo "error: regex family-contract generated_at_utc is empty" >&2
    exit 1
fi
if [[ "$summary_regex_family_contract_state_dir" != "$regex_family_contract_state_dir" ]]; then
    echo "error: regex family-contract state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$regex_family_contract_state_dir_from_json" != "$regex_family_contract_state_dir" ]]; then
    echo "error: regex family-contract state_dir mismatch in summary.json" >&2
    exit 1
fi
if [[ "$regex_family_contract_summary_txt_from_json" != "$regex_family_contract_summary_txt" ]]; then
    echo "error: regex family-contract summary_txt mismatch in summary.json" >&2
    exit 1
fi
if [[ "$summary_regex_family_contract_summary_json" != "$regex_family_contract_summary_json" ]]; then
    echo "error: regex family-contract summary_json mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$regex_family_contract_summary_json_from_json" != "$regex_family_contract_summary_json" ]]; then
    echo "error: regex family-contract summary_json mismatch in summary.json" >&2
    exit 1
fi
if [[ "$summary_regex_family_contract_generated_at_utc" != "$regex_family_contract_generated_at_utc" ]]; then
    echo "error: regex family-contract generated_at_utc mismatch between summary.txt and summary.json" >&2
    exit 1
fi

regex_frontend_overall="$(summary_value_from_txt "frontend_regex_overall" "$regex_family_contract_summary_txt")"
regex_dual_run_overall="$(summary_value_from_txt "dual_run_regex_overall" "$regex_family_contract_summary_txt")"
regex_dual_run_raw_ast_status="$(summary_value_from_txt "dual_run_regex_raw_ast_status" "$regex_family_contract_summary_txt")"
regex_dual_run_raw_ast_missing_on_perl_count="$(summary_value_from_txt "dual_run_regex_raw_ast_missing_on_perl_count" "$regex_family_contract_summary_txt")"
regex_dual_run_raw_ast_missing_on_rust_count="$(summary_value_from_txt "dual_run_regex_raw_ast_missing_on_rust_count" "$regex_family_contract_summary_txt")"
regex_dual_run_rust_rule_count="$(summary_value_from_txt "dual_run_regex_rust_rule_count" "$regex_family_contract_summary_txt")"
regex_stimuli_parseability_required="$(summary_value_from_txt "stimuli_regex_parseability_required" "$regex_family_contract_summary_txt")"
regex_stimuli_parseability_attempts_total="$(summary_value_from_txt "stimuli_regex_parseability_attempts_total" "$regex_family_contract_summary_txt")"
regex_stimuli_parseability_accepted_total="$(summary_value_from_txt "stimuli_regex_parseability_accepted_total" "$regex_family_contract_summary_txt")"
regex_stimuli_parseability_rejected_total="$(summary_value_from_txt "stimuli_regex_parseability_rejected_total" "$regex_family_contract_summary_txt")"
regex_stimuli_parseability_parser_rejections_total="$(summary_value_from_txt "stimuli_regex_parseability_parser_rejections_total" "$regex_family_contract_summary_txt")"
regex_stimuli_parseability_acceptance_rate_percent="$(summary_value_from_txt "stimuli_regex_parseability_acceptance_rate_percent" "$regex_family_contract_summary_txt")"
regex_stimuli_status="$(summary_value_from_txt "stimuli_regex_status" "$regex_family_contract_summary_txt")"
regex_stimuli_initial_targets="$(summary_value_from_txt "stimuli_regex_initial_targets" "$regex_family_contract_summary_txt")"
regex_stimuli_resolved_targets="$(summary_value_from_txt "stimuli_regex_resolved_targets" "$regex_family_contract_summary_txt")"
regex_stimuli_final_targets="$(summary_value_from_txt "stimuli_regex_final_targets" "$regex_family_contract_summary_txt")"
regex_stimuli_target_attempts="$(summary_value_from_txt "stimuli_regex_target_attempts" "$regex_family_contract_summary_txt")"
regex_stimuli_stage0_successes="$(summary_value_from_txt "stimuli_regex_stage0_successes" "$regex_family_contract_summary_txt")"
regex_stimuli_stage3_successes="$(summary_value_from_txt "stimuli_regex_stage3_successes" "$regex_family_contract_summary_txt")"

regex_family_contract_green=true
regex_frontend_overall_pass=false
regex_dual_run_overall_pass=false
regex_dual_run_raw_ast_missing_on_rust_zero=false
regex_stimuli_status_pass=false
regex_stimuli_parseability_parser_rejections_zero=false
regex_stimuli_final_target_debt_zero=false
regex_formal_exhaustive_closure_surface_green=false

if [[ "$regex_frontend_overall" == "pass" ]]; then
    regex_frontend_overall_pass=true
fi
if [[ "$regex_dual_run_overall" == "pass" ]]; then
    regex_dual_run_overall_pass=true
fi
if [[ "$regex_dual_run_raw_ast_missing_on_rust_count" == "0" ]]; then
    regex_dual_run_raw_ast_missing_on_rust_zero=true
fi
if [[ "$regex_stimuli_status" == "pass" ]]; then
    regex_stimuli_status_pass=true
fi
if [[ "$regex_stimuli_parseability_parser_rejections_total" == "0" ]]; then
    regex_stimuli_parseability_parser_rejections_zero=true
fi
if [[ "$regex_stimuli_final_targets" == "0" ]]; then
    regex_stimuli_final_target_debt_zero=true
fi

regex_closure_criteria_total_count=8
regex_closure_criteria_satisfied_count=0
for criterion in \
    "$regex_family_contract_green" \
    "$regex_frontend_overall_pass" \
    "$regex_dual_run_overall_pass" \
    "$regex_dual_run_raw_ast_missing_on_rust_zero" \
    "$regex_stimuli_status_pass" \
    "$regex_stimuli_parseability_parser_rejections_zero" \
    "$regex_stimuli_final_target_debt_zero" \
    "$regex_formal_exhaustive_closure_surface_green"; do
    if [[ "$criterion" == true ]]; then
        ((regex_closure_criteria_satisfied_count += 1))
    fi
done
regex_closure_criteria_unsatisfied_count=$((regex_closure_criteria_total_count - regex_closure_criteria_satisfied_count))

declare -a regex_unmet=()
declare -a regex_unmet_details=()
if [[ "$regex_frontend_overall_pass" != true ]]; then
    regex_unmet+=("frontend_regex_overall=${regex_frontend_overall} != pass")
    regex_unmet_details+=("{\"criterion\":\"frontend_overall_pass\",\"evidence_key\":\"frontend_regex_overall\",\"observed\":\"${regex_frontend_overall}\",\"expected\":\"pass\",\"detail\":\"The regex frontend readiness surface must stay green before the family can be promoted.\"}")
fi
if [[ "$regex_dual_run_overall_pass" != true ]]; then
    regex_unmet+=("dual_run_regex_overall=${regex_dual_run_overall} != pass")
    regex_unmet_details+=("{\"criterion\":\"dual_run_overall_pass\",\"evidence_key\":\"dual_run_regex_overall\",\"observed\":\"${regex_dual_run_overall}\",\"expected\":\"pass\",\"detail\":\"The regex Rust-vs-Perl dual-run differential surface must stay green before the family can be promoted.\"}")
fi
if [[ "$regex_dual_run_raw_ast_missing_on_rust_zero" != true ]]; then
    regex_unmet+=("dual_run_regex_raw_ast_missing_on_rust_count=${regex_dual_run_raw_ast_missing_on_rust_count} > 0")
    regex_unmet_details+=("{\"criterion\":\"dual_run_raw_ast_missing_on_rust_zero\",\"evidence_key\":\"dual_run_regex_raw_ast_missing_on_rust_count\",\"observed\":\"${regex_dual_run_raw_ast_missing_on_rust_count}\",\"expected\":\"0\",\"detail\":\"The Rust regex raw-AST export must not under-report rules relative to the legacy Perl export.\"}")
fi
if [[ "$regex_stimuli_status_pass" != true ]]; then
    regex_unmet+=("stimuli_regex_status=${regex_stimuli_status} != pass")
    regex_unmet_details+=("{\"criterion\":\"stimuli_status_pass\",\"evidence_key\":\"stimuli_regex_status\",\"observed\":\"${regex_stimuli_status}\",\"expected\":\"pass\",\"detail\":\"The regex closed-loop stimuli surface must stay green before the family can be promoted.\"}")
fi
if [[ "$regex_stimuli_parseability_parser_rejections_zero" != true ]]; then
    regex_unmet+=("stimuli_regex_parseability_parser_rejections_total=${regex_stimuli_parseability_parser_rejections_total} > 0")
    regex_unmet_details+=("{\"criterion\":\"stimuli_parseability_parser_rejections_zero\",\"evidence_key\":\"stimuli_regex_parseability_parser_rejections_total\",\"observed\":\"${regex_stimuli_parseability_parser_rejections_total}\",\"expected\":\"0\",\"detail\":\"The parser-backed regex quality surface still records parser rejections, so parser-backed quality remains bounded debt rather than closed proof.\"}")
fi
if [[ "$regex_stimuli_final_target_debt_zero" != true ]]; then
    regex_unmet+=("stimuli_regex_final_targets=${regex_stimuli_final_targets} > 0")
    regex_unmet_details+=("{\"criterion\":\"stimuli_final_target_debt_zero\",\"evidence_key\":\"stimuli_regex_final_targets\",\"observed\":\"${regex_stimuli_final_targets}\",\"expected\":\"0\",\"detail\":\"Current regex stimuli closure still has bounded remaining target debt.\"}")
fi
if [[ "$regex_formal_exhaustive_closure_surface_green" != true ]]; then
    regex_unmet+=("formal_exhaustive_closure_surface=missing")
    regex_unmet_details+=("{\"criterion\":\"formal_exhaustive_closure_surface_green\",\"evidence_key\":\"formal_exhaustive_closure_surface\",\"observed\":\"missing\",\"expected\":\"present_and_green\",\"detail\":\"No formal exhaustive regex closure gate is wired yet, so the repository Done rule cannot be met.\"}")
fi

regex_status="Not Started"
if [[ "$regex_family_contract_green" == true ]]; then
    regex_status="In Progress"
fi
if [[ "$regex_family_contract_green" == true \
   && "$regex_frontend_overall_pass" == true \
   && "$regex_dual_run_overall_pass" == true \
   && "$regex_dual_run_raw_ast_missing_on_rust_zero" == true \
   && "$regex_stimuli_status_pass" == true \
   && "$regex_stimuli_parseability_parser_rejections_zero" == true \
   && "$regex_stimuli_final_target_debt_zero" == true ]]; then
    regex_status="Mostly Done"
fi
if [[ "$regex_status" == "Mostly Done" && "$regex_formal_exhaustive_closure_surface_green" == true ]]; then
    regex_status="Done"
fi

regex_tracker_status="$(markdown_table_status_for_row "| \`regex\` parser family |" "$LIVE_TRACKER_FILE")"
regex_tracker_alignment_ok=false
if [[ "$regex_status" == "$regex_tracker_status" ]]; then
    regex_tracker_alignment_ok=true
fi
if [[ "$regex_tracker_alignment_ok" != true ]]; then
    echo "error: regex tracker alignment mismatch: computed '${regex_status}' but tracker says '${regex_tracker_status}'" >&2
    exit 1
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
regex_unmet_count="${#regex_unmet[@]}"
regex_primary_unmet_closure_criterion="<none>"
if [[ "$regex_unmet_count" -gt 0 ]]; then
    regex_primary_unmet_closure_criterion="${regex_unmet[0]}"
fi
regex_unmet_json="$(printf '%s\n' "${regex_unmet[@]:-}" | jq -R . | jq -sc 'map(select(length > 0))')"
regex_unmet_details_json="$(printf '%s\n' "${regex_unmet_details[@]:-}" | jq -R . | jq -sc 'map(select(length > 0) | fromjson)')"

{
    echo "Regex Parser Family Status Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "live_tracker_file: $LIVE_TRACKER_FILE"
    echo "status_rule_done: $DONE_RULE"
    echo "regex_status: $regex_status"
    echo "regex_tracker_status: $regex_tracker_status"
    echo "regex_tracker_alignment_ok: $regex_tracker_alignment_ok"
    echo "regex_unmet_closure_criteria_count: $regex_unmet_count"
    echo "regex_primary_unmet_closure_criterion: $regex_primary_unmet_closure_criterion"
    for i in "${!regex_unmet[@]}"; do
        echo "regex_unmet_closure_criterion[$i]: ${regex_unmet[$i]}"
    done
    echo "regex_unmet_closure_criteria_json: $regex_unmet_json"
    echo "regex_unmet_closure_criteria_details_json: $regex_unmet_details_json"
    echo "regex_closure_criteria_total_count: $regex_closure_criteria_total_count"
    echo "regex_closure_criteria_satisfied_count: $regex_closure_criteria_satisfied_count"
    echo "regex_closure_criteria_unsatisfied_count: $regex_closure_criteria_unsatisfied_count"
    echo "regex_family_contract_green: $regex_family_contract_green"
    echo "regex_frontend_overall_pass: $regex_frontend_overall_pass"
    echo "regex_dual_run_overall_pass: $regex_dual_run_overall_pass"
    echo "regex_dual_run_raw_ast_missing_on_rust_zero: $regex_dual_run_raw_ast_missing_on_rust_zero"
    echo "regex_stimuli_status_pass: $regex_stimuli_status_pass"
    echo "regex_stimuli_parseability_parser_rejections_zero: $regex_stimuli_parseability_parser_rejections_zero"
    echo "regex_stimuli_final_target_debt_zero: $regex_stimuli_final_target_debt_zero"
    echo "regex_formal_exhaustive_closure_surface_green: $regex_formal_exhaustive_closure_surface_green"
    echo "regex_frontend_overall: $regex_frontend_overall"
    echo "regex_dual_run_overall: $regex_dual_run_overall"
    echo "regex_dual_run_raw_ast_status: $regex_dual_run_raw_ast_status"
    echo "regex_dual_run_raw_ast_missing_on_perl_count: $regex_dual_run_raw_ast_missing_on_perl_count"
    echo "regex_dual_run_raw_ast_missing_on_rust_count: $regex_dual_run_raw_ast_missing_on_rust_count"
    echo "regex_dual_run_rust_rule_count: $regex_dual_run_rust_rule_count"
    echo "regex_stimuli_parseability_required: $regex_stimuli_parseability_required"
    echo "regex_stimuli_parseability_attempts_total: $regex_stimuli_parseability_attempts_total"
    echo "regex_stimuli_parseability_accepted_total: $regex_stimuli_parseability_accepted_total"
    echo "regex_stimuli_parseability_rejected_total: $regex_stimuli_parseability_rejected_total"
    echo "regex_stimuli_parseability_parser_rejections_total: $regex_stimuli_parseability_parser_rejections_total"
    echo "regex_stimuli_parseability_acceptance_rate_percent: $regex_stimuli_parseability_acceptance_rate_percent"
    echo "regex_stimuli_initial_targets: $regex_stimuli_initial_targets"
    echo "regex_stimuli_resolved_targets: $regex_stimuli_resolved_targets"
    echo "regex_stimuli_final_targets: $regex_stimuli_final_targets"
    echo "regex_stimuli_target_attempts: $regex_stimuli_target_attempts"
    echo "regex_stimuli_stage0_successes: $regex_stimuli_stage0_successes"
    echo "regex_stimuli_stage3_successes: $regex_stimuli_stage3_successes"
    echo "regex_stimuli_status: $regex_stimuli_status"
    echo "regex_family_contract_gate: $regex_family_contract_gate_name"
    echo "regex_family_contract_gate_version: $regex_family_contract_gate_version"
    echo "regex_family_contract_generated_at_utc: $regex_family_contract_generated_at_utc"
    echo "regex_family_contract_state_dir: $regex_family_contract_state_dir_from_json"
    echo "regex_family_contract_summary_txt: $regex_family_contract_summary_txt"
    echo "regex_family_contract_summary_json: $regex_family_contract_summary_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "regex_parser_family_status_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg live_tracker_file "$LIVE_TRACKER_FILE" \
    --arg status_rule_done "$DONE_RULE" \
    --arg regex_status "$regex_status" \
    --arg regex_tracker_status "$regex_tracker_status" \
    --argjson regex_tracker_alignment_ok "$regex_tracker_alignment_ok" \
    --arg regex_primary_unmet_closure_criterion "$regex_primary_unmet_closure_criterion" \
    --argjson regex_unmet_closure_criteria_count "$regex_unmet_count" \
    --argjson regex_unmet_closure_criteria "$regex_unmet_json" \
    --argjson regex_unmet_closure_criteria_details "$regex_unmet_details_json" \
    --argjson regex_closure_criteria_total_count "$regex_closure_criteria_total_count" \
    --argjson regex_closure_criteria_satisfied_count "$regex_closure_criteria_satisfied_count" \
    --argjson regex_closure_criteria_unsatisfied_count "$regex_closure_criteria_unsatisfied_count" \
    --argjson regex_family_contract_green "$regex_family_contract_green" \
    --argjson regex_frontend_overall_pass "$regex_frontend_overall_pass" \
    --argjson regex_dual_run_overall_pass "$regex_dual_run_overall_pass" \
    --argjson regex_dual_run_raw_ast_missing_on_rust_zero "$regex_dual_run_raw_ast_missing_on_rust_zero" \
    --argjson regex_stimuli_status_pass "$regex_stimuli_status_pass" \
    --argjson regex_stimuli_parseability_parser_rejections_zero "$regex_stimuli_parseability_parser_rejections_zero" \
    --argjson regex_stimuli_final_target_debt_zero "$regex_stimuli_final_target_debt_zero" \
    --argjson regex_formal_exhaustive_closure_surface_green "$regex_formal_exhaustive_closure_surface_green" \
    --arg regex_frontend_overall "$regex_frontend_overall" \
    --arg regex_dual_run_overall "$regex_dual_run_overall" \
    --arg regex_dual_run_raw_ast_status "$regex_dual_run_raw_ast_status" \
    --argjson regex_dual_run_raw_ast_missing_on_perl_count "$regex_dual_run_raw_ast_missing_on_perl_count" \
    --argjson regex_dual_run_raw_ast_missing_on_rust_count "$regex_dual_run_raw_ast_missing_on_rust_count" \
    --argjson regex_dual_run_rust_rule_count "$regex_dual_run_rust_rule_count" \
    --argjson regex_stimuli_parseability_required "$regex_stimuli_parseability_required" \
    --argjson regex_stimuli_parseability_attempts_total "$regex_stimuli_parseability_attempts_total" \
    --argjson regex_stimuli_parseability_accepted_total "$regex_stimuli_parseability_accepted_total" \
    --argjson regex_stimuli_parseability_rejected_total "$regex_stimuli_parseability_rejected_total" \
    --argjson regex_stimuli_parseability_parser_rejections_total "$regex_stimuli_parseability_parser_rejections_total" \
    --argjson regex_stimuli_parseability_acceptance_rate_percent "$regex_stimuli_parseability_acceptance_rate_percent" \
    --argjson regex_stimuli_initial_targets "$regex_stimuli_initial_targets" \
    --argjson regex_stimuli_resolved_targets "$regex_stimuli_resolved_targets" \
    --argjson regex_stimuli_final_targets "$regex_stimuli_final_targets" \
    --argjson regex_stimuli_target_attempts "$regex_stimuli_target_attempts" \
    --argjson regex_stimuli_stage0_successes "$regex_stimuli_stage0_successes" \
    --argjson regex_stimuli_stage3_successes "$regex_stimuli_stage3_successes" \
    --arg regex_stimuli_status "$regex_stimuli_status" \
    --arg regex_family_contract_gate "$regex_family_contract_gate_name" \
    --argjson regex_family_contract_gate_version "$regex_family_contract_gate_version" \
    --arg regex_family_contract_generated_at_utc "$regex_family_contract_generated_at_utc" \
    --arg regex_family_contract_state_dir "$regex_family_contract_state_dir_from_json" \
    --arg regex_family_contract_summary_txt "$regex_family_contract_summary_txt" \
    --arg regex_family_contract_summary_json "$regex_family_contract_summary_json" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      live_tracker_file: $live_tracker_file,
      status_rule_done: $status_rule_done,
      families: [
        {
          family: "regex",
          computed_status: $regex_status,
          live_tracker_status: $regex_tracker_status,
          tracker_alignment_ok: $regex_tracker_alignment_ok,
          primary_unmet_closure_criterion: $regex_primary_unmet_closure_criterion,
          unmet_closure_criteria_count: $regex_unmet_closure_criteria_count,
          unmet_closure_criteria: $regex_unmet_closure_criteria,
          unmet_closure_criteria_details: $regex_unmet_closure_criteria_details,
          closure_criteria_total_count: $regex_closure_criteria_total_count,
          closure_criteria_satisfied_count: $regex_closure_criteria_satisfied_count,
          closure_criteria_unsatisfied_count: $regex_closure_criteria_unsatisfied_count,
          criteria: {
            family_contract_green: $regex_family_contract_green,
            frontend_overall_pass: $regex_frontend_overall_pass,
            dual_run_overall_pass: $regex_dual_run_overall_pass,
            dual_run_raw_ast_missing_on_rust_zero: $regex_dual_run_raw_ast_missing_on_rust_zero,
            stimuli_status_pass: $regex_stimuli_status_pass,
            stimuli_parseability_parser_rejections_zero: $regex_stimuli_parseability_parser_rejections_zero,
            stimuli_final_target_debt_zero: $regex_stimuli_final_target_debt_zero,
            formal_exhaustive_closure_surface_green: $regex_formal_exhaustive_closure_surface_green
          },
            metrics: {
            frontend_overall: $regex_frontend_overall,
            dual_run_overall: $regex_dual_run_overall,
            dual_run_raw_ast_status: $regex_dual_run_raw_ast_status,
            dual_run_raw_ast_missing_on_perl_count: $regex_dual_run_raw_ast_missing_on_perl_count,
            dual_run_raw_ast_missing_on_rust_count: $regex_dual_run_raw_ast_missing_on_rust_count,
            dual_run_rust_rule_count: $regex_dual_run_rust_rule_count,
            stimuli_parseability_required: $regex_stimuli_parseability_required,
            stimuli_parseability_attempts_total: $regex_stimuli_parseability_attempts_total,
            stimuli_parseability_accepted_total: $regex_stimuli_parseability_accepted_total,
            stimuli_parseability_rejected_total: $regex_stimuli_parseability_rejected_total,
            stimuli_parseability_parser_rejections_total: $regex_stimuli_parseability_parser_rejections_total,
            stimuli_parseability_acceptance_rate_percent: $regex_stimuli_parseability_acceptance_rate_percent,
            stimuli_initial_targets: $regex_stimuli_initial_targets,
            stimuli_resolved_targets: $regex_stimuli_resolved_targets,
            stimuli_final_targets: $regex_stimuli_final_targets,
            stimuli_target_attempts: $regex_stimuli_target_attempts,
            stimuli_stage0_successes: $regex_stimuli_stage0_successes,
            stimuli_stage3_successes: $regex_stimuli_stage3_successes,
            stimuli_status: $regex_stimuli_status,
            family_contract_gate: $regex_family_contract_gate,
            family_contract_gate_version: $regex_family_contract_gate_version,
            family_contract_generated_at_utc: $regex_family_contract_generated_at_utc
          },
          proof_surfaces: {
            family_contract_state_dir: $regex_family_contract_state_dir,
            family_contract_summary_txt: $regex_family_contract_summary_txt,
            family_contract_summary_json: $regex_family_contract_summary_json
          }
        }
      ]
    }' >"$SUMMARY_JSON"

echo "✅ Regex parser-family status gate passed."
