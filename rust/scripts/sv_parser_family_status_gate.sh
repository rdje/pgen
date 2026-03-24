#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_FAMILY_STATUS_STATE_DIR:-$RUST_DIR/target/sv_parser_family_status_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_JSON="$STATE_DIR/summary.json"
SUMMARY_TXT="$STATE_DIR/summary.txt"
LIVE_TRACKER_FILE="$ROOT_DIR/LIVE_ACHIEVEMENT_STATUS.md"

SV_SYNTAX_CLOSURE_GATE="$RUST_DIR/scripts/sv_syntax_closure_gate.sh"
SV_PREPROCESSOR_SYNTAX_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_syntax_closure_gate.sh"
SV_PARSER_AGGREGATE_GATE="$RUST_DIR/scripts/sv_parser_aggregate_contract_gate.sh"
SV_PREPROCESSOR_AGGREGATE_GATE="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"
SV_PREPROCESSOR_REACHABILITY_GATE="$RUST_DIR/scripts/sv_preprocessor_reachability_closure_gate.sh"
SV_SEMANTIC_SCOPE_CONTRACT_GATE="$RUST_DIR/scripts/sv_semantic_scope_contract_gate.sh"

EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_SV_PARSER_AGGREGATE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_PARSER_AGGREGATE_STATE_DIR:-}"
EXISTING_SV_STIMULI_QUALITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-}"
EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR:-}"

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
    line="$(grep -E "^${key}: " "$path" | tail -n 1 || true)"
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

fraction_is_full() {
    local fraction="$1"
    local numerator="${fraction%%/*}"
    local denominator="${fraction##*/}"
    [[ "$numerator" =~ ^[0-9]+$ ]] || return 1
    [[ "$denominator" =~ ^[0-9]+$ ]] || return 1
    [[ "$numerator" -eq "$denominator" ]]
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
require_file "$SV_SYNTAX_CLOSURE_GATE"
require_file "$SV_PREPROCESSOR_SYNTAX_CLOSURE_GATE"
require_file "$SV_PARSER_AGGREGATE_GATE"
require_file "$SV_PREPROCESSOR_AGGREGATE_GATE"
require_file "$SV_PREPROCESSOR_REACHABILITY_GATE"
require_file "$SV_SEMANTIC_SCOPE_CONTRACT_GATE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

sv_syntax_closure_state_dir="$WORK_DIR/sv_syntax_closure_gate"
sv_preprocessor_syntax_closure_state_dir="$WORK_DIR/sv_preprocessor_syntax_closure_gate"
sv_parser_gate_state_dir="$WORK_DIR/sv_parser_aggregate_contract_gate"
sv_preprocessor_aggregate_state_dir="$WORK_DIR/sv_preprocessor_aggregate_contract_gate"
sv_preprocessor_reachability_state_dir="$WORK_DIR/sv_preprocessor_reachability_closure_gate"
sv_semantic_scope_contract_state_dir="$WORK_DIR/sv_semantic_scope_contract_gate"

if [[ -n "$EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR" ]]; then
    sv_syntax_closure_state_dir="$EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR"
else
    run_logged "sv_syntax_closure_gate" \
        env \
            PGEN_SV_SYNTAX_CLOSURE_STATE_DIR="$sv_syntax_closure_state_dir" \
            "$SV_SYNTAX_CLOSURE_GATE"
fi

if [[ -n "$EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR" ]]; then
    sv_preprocessor_syntax_closure_state_dir="$EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR"
else
    run_logged "sv_preprocessor_syntax_closure_gate" \
        env \
            PGEN_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR="$sv_preprocessor_syntax_closure_state_dir" \
            "$SV_PREPROCESSOR_SYNTAX_CLOSURE_GATE"
fi

if [[ -n "$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR" ]]; then
    sv_parser_gate_state_dir="$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR"
elif [[ -n "$EXISTING_SV_STIMULI_QUALITY_STATE_DIR" ]]; then
    run_logged "sv_parser_aggregate_contract_gate" \
        env \
            PGEN_SV_PARSER_AGGREGATE_CONTRACT_STATE_DIR="$sv_parser_gate_state_dir" \
            PGEN_SV_PARSER_AGGREGATE_CONTRACT_EXISTING_SV_STIMULI_QUALITY_STATE_DIR="$EXISTING_SV_STIMULI_QUALITY_STATE_DIR" \
            "$SV_PARSER_AGGREGATE_GATE"
else
    run_logged "sv_parser_aggregate_contract_gate" \
        env \
            PGEN_SV_PARSER_AGGREGATE_CONTRACT_STATE_DIR="$sv_parser_gate_state_dir" \
            "$SV_PARSER_AGGREGATE_GATE"
fi

preprocessor_quality_state_dir="$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR"
if [[ -n "$EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR" ]]; then
    sv_preprocessor_aggregate_state_dir="$EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR"
elif [[ -n "$preprocessor_quality_state_dir" ]]; then
    run_logged "sv_preprocessor_aggregate_contract_gate" \
        env \
            PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_STATE_DIR="$sv_preprocessor_aggregate_state_dir" \
            PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_EXISTING_QUALITY_STATE_DIR="$preprocessor_quality_state_dir" \
            "$SV_PREPROCESSOR_AGGREGATE_GATE"
else
    run_logged "sv_preprocessor_aggregate_contract_gate" \
        env \
            PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_STATE_DIR="$sv_preprocessor_aggregate_state_dir" \
            "$SV_PREPROCESSOR_AGGREGATE_GATE"
    preprocessor_quality_state_dir="$sv_preprocessor_aggregate_state_dir/work/quality_state"
fi

if [[ -n "$EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR" ]]; then
    sv_preprocessor_reachability_state_dir="$EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR"
else
    run_logged "sv_preprocessor_reachability_closure_gate" \
        env \
            PGEN_SV_PREPROCESSOR_REACHABILITY_CLOSURE_STATE_DIR="$sv_preprocessor_reachability_state_dir" \
            PGEN_SV_PREPROCESSOR_REACHABILITY_CLOSURE_EXISTING_QUALITY_STATE_DIR="$preprocessor_quality_state_dir" \
            "$SV_PREPROCESSOR_REACHABILITY_GATE"
fi

if [[ -n "$EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR" ]]; then
    sv_semantic_scope_contract_state_dir="$EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR"
else
    run_logged "sv_semantic_scope_contract_gate" \
        env \
            PGEN_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR="$sv_semantic_scope_contract_state_dir" \
            "$SV_SEMANTIC_SCOPE_CONTRACT_GATE"
fi

sv_parser_summary_txt="$sv_parser_gate_state_dir/summary.txt"
sv_parser_summary_json="$sv_parser_gate_state_dir/summary.json"
sv_preprocessor_aggregate_summary_txt="$sv_preprocessor_aggregate_state_dir/summary.txt"
sv_preprocessor_aggregate_summary_json="$sv_preprocessor_aggregate_state_dir/summary.json"
sv_preprocessor_reachability_summary_txt="$sv_preprocessor_reachability_state_dir/summary.txt"
sv_syntax_summary_txt="$sv_syntax_closure_state_dir/summary.txt"
sv_semantic_scope_contract_summary_txt="$sv_semantic_scope_contract_state_dir/summary.txt"
svpp_syntax_summary_txt="$sv_preprocessor_syntax_closure_state_dir/summary.txt"
sv_semantic_scope_contract_summary_json="$sv_semantic_scope_contract_state_dir/summary.json"

require_nonempty_file "$sv_parser_summary_txt"
require_nonempty_file "$sv_parser_summary_json"
require_nonempty_file "$sv_preprocessor_aggregate_summary_txt"
require_nonempty_file "$sv_preprocessor_aggregate_summary_json"
require_nonempty_file "$sv_preprocessor_reachability_summary_txt"
require_nonempty_file "$sv_syntax_summary_txt"
require_nonempty_file "$sv_semantic_scope_contract_summary_txt"
require_nonempty_file "$svpp_syntax_summary_txt"
require_nonempty_file "$sv_semantic_scope_contract_summary_json"

sv_syntax_summary_json="$sv_syntax_closure_state_dir/summary.json"
require_nonempty_file "$sv_syntax_summary_json"

sv_syntax_status="$(jq -er '.status | strings' "$sv_syntax_summary_json")"
sv_syntax_failure_count="$(jq -er '.failure_count | numbers' "$sv_syntax_summary_json")"
sv_syntax_defined_rule_count="$(jq -er '.metrics.defined_rule_count | numbers' "$sv_syntax_summary_json")"
sv_syntax_unresolved_rule_reference_count="$(jq -er '.metrics.unresolved_rule_reference_count | numbers' "$sv_syntax_summary_json")"
sv_syntax_unreachable_rules="$(jq -er '.metrics.unreachable_rules | numbers' "$sv_syntax_summary_json")"
sv_syntax_unreachable_branches="$(jq -er '.metrics.unreachable_branches | numbers' "$sv_syntax_summary_json")"
sv_syntax_target_debt_count="$(jq -er '.metrics.target_debt_count | numbers' "$sv_syntax_summary_json")"
sv_semantic_scope_case_count="$(jq -er '.case_count | numbers' "$sv_semantic_scope_contract_summary_json")"
sv_semantic_scope_failed_count="$(jq -er '.failed_count | numbers' "$sv_semantic_scope_contract_summary_json")"

svpp_syntax_summary_json="$sv_preprocessor_syntax_closure_state_dir/summary.json"
require_nonempty_file "$svpp_syntax_summary_json"

svpp_syntax_status="$(jq -er '.status | strings' "$svpp_syntax_summary_json")"
svpp_syntax_failure_count="$(jq -er '.failure_count | numbers' "$svpp_syntax_summary_json")"
svpp_syntax_defined_rule_count="$(jq -er '.metrics.defined_rule_count | numbers' "$svpp_syntax_summary_json")"
svpp_syntax_unresolved_rule_reference_count="$(jq -er '.metrics.unresolved_rule_reference_count | numbers' "$svpp_syntax_summary_json")"
svpp_syntax_unreachable_rules="$(jq -er '.metrics.unreachable_rules | numbers' "$svpp_syntax_summary_json")"
svpp_syntax_unreachable_branches="$(jq -er '.metrics.unreachable_branches | numbers' "$svpp_syntax_summary_json")"
svpp_syntax_target_debt_count="$(jq -er '.metrics.target_debt_count | numbers' "$svpp_syntax_summary_json")"

sv_generation_parser_rejections_total="$(summary_value_from_txt "generation_parser_rejections_total" "$sv_parser_summary_txt")"
sv_shadow_parser_rejections_total="$(summary_value_from_txt "shadow_parser_rejections_total" "$sv_parser_summary_txt")"
sv_focused_replay_target_count="$(summary_value_from_txt "focused_replay_target_count" "$sv_parser_summary_txt")"
sv_focused_replay_covered_reachable_rules="$(summary_value_from_txt "focused_replay_covered_reachable_rules" "$sv_parser_summary_txt")"
sv_focused_replay_covered_reachable_branches="$(summary_value_from_txt "focused_replay_covered_reachable_branches" "$sv_parser_summary_txt")"
sv_replay_gap_target_primary_rule="$(summary_value_from_txt "replay_gap_target_primary_rule" "$sv_parser_summary_txt")"

svpp_parseability_parser_rejections_total="$(summary_value_from_txt "parseability_parser_rejections_total" "$sv_preprocessor_aggregate_summary_txt")"
svpp_parseability_rejected_total="$(summary_value_from_txt "parseability_rejected_total" "$sv_preprocessor_aggregate_summary_txt")"
svpp_final_targets="$(summary_value_from_txt "final_targets" "$sv_preprocessor_aggregate_summary_txt")"
svpp_covered_reachable_rules="$(summary_value_from_txt "covered_reachable_rules" "$sv_preprocessor_aggregate_summary_txt")"
svpp_covered_reachable_branches="$(summary_value_from_txt "covered_reachable_branches" "$sv_preprocessor_aggregate_summary_txt")"
svpp_counterexample_primary_stage="$(summary_value_from_txt "counterexample_primary_stage" "$sv_preprocessor_aggregate_summary_txt")"

svpp_reach_stage3_targets="$(summary_value_from_txt "stage3_targets" "$sv_preprocessor_reachability_summary_txt")"
svpp_reach_stage4_targets="$(summary_value_from_txt "stage4_targets" "$sv_preprocessor_reachability_summary_txt")"
svpp_reach_stage3_rules="$(summary_value_from_txt "stage3_covered_reachable_rules" "$sv_preprocessor_reachability_summary_txt")"
svpp_reach_stage4_rules="$(summary_value_from_txt "stage4_covered_reachable_rules" "$sv_preprocessor_reachability_summary_txt")"
svpp_reach_stage3_branches="$(summary_value_from_txt "stage3_covered_reachable_branches" "$sv_preprocessor_reachability_summary_txt")"
svpp_reach_stage4_branches="$(summary_value_from_txt "stage4_covered_reachable_branches" "$sv_preprocessor_reachability_summary_txt")"

sv_syntax_closure_gate_green=false
sv_aggregate_contract_green=true
sv_generation_parser_rejections_zero=false
sv_shadow_parser_rejections_zero=false
sv_focused_replay_target_debt_zero=false
sv_semantic_scope_contract_green=false

if [[ "$sv_syntax_status" == "pass" && "$sv_syntax_failure_count" == "0" ]]; then
    sv_syntax_closure_gate_green=true
fi
if [[ "$sv_generation_parser_rejections_total" == "0" ]]; then
    sv_generation_parser_rejections_zero=true
fi
if [[ "$sv_shadow_parser_rejections_total" == "0" ]]; then
    sv_shadow_parser_rejections_zero=true
fi
if [[ "$sv_focused_replay_target_count" == "0" ]]; then
    sv_focused_replay_target_debt_zero=true
fi
if [[ "$sv_semantic_scope_failed_count" == "0" ]]; then
    sv_semantic_scope_contract_green=true
fi

sv_closure_criteria_total_count=6
sv_closure_criteria_satisfied_count=0
if [[ "$sv_syntax_closure_gate_green" == true ]]; then
    ((sv_closure_criteria_satisfied_count += 1))
fi
if [[ "$sv_aggregate_contract_green" == true ]]; then
    ((sv_closure_criteria_satisfied_count += 1))
fi
if [[ "$sv_generation_parser_rejections_zero" == true ]]; then
    ((sv_closure_criteria_satisfied_count += 1))
fi
if [[ "$sv_shadow_parser_rejections_zero" == true ]]; then
    ((sv_closure_criteria_satisfied_count += 1))
fi
if [[ "$sv_focused_replay_target_debt_zero" == true ]]; then
    ((sv_closure_criteria_satisfied_count += 1))
fi
if [[ "$sv_semantic_scope_contract_green" == true ]]; then
    ((sv_closure_criteria_satisfied_count += 1))
fi

declare -a sv_unmet=()
if [[ "$sv_syntax_closure_gate_green" != true ]]; then
    sv_unmet+=("syntax_closure_gate_status=${sv_syntax_status} failure_count=${sv_syntax_failure_count}")
fi
if [[ "$sv_generation_parser_rejections_zero" != true ]]; then
    sv_unmet+=("generation_parser_rejections_total=${sv_generation_parser_rejections_total} > 0")
fi
if [[ "$sv_shadow_parser_rejections_zero" != true ]]; then
    sv_unmet+=("shadow_parser_rejections_total=${sv_shadow_parser_rejections_total} > 0")
fi
if [[ "$sv_focused_replay_target_debt_zero" != true ]]; then
    sv_unmet+=("focused_replay_target_count=${sv_focused_replay_target_count} > 0")
fi
if [[ "$sv_semantic_scope_contract_green" != true ]]; then
    sv_unmet+=("semantic_scope_failed_count=${sv_semantic_scope_failed_count} > 0")
fi

if [[ "$sv_syntax_closure_gate_green" == true && "$sv_generation_parser_rejections_zero" == true && "$sv_shadow_parser_rejections_zero" == true && "$sv_focused_replay_target_debt_zero" == true && "$sv_semantic_scope_contract_green" == true ]]; then
    sv_status="Done"
else
    sv_status="Mostly Done"
fi

svpp_aggregate_contract_green=true
svpp_reachability_closure_green=true
svpp_syntax_closure_gate_green=false
svpp_parser_rejections_zero=false
svpp_parseability_rejections_zero=false
svpp_stage3_targets_zero=false
svpp_stage4_targets_zero=false
svpp_stage3_rules_full=false
svpp_stage4_rules_full=false
svpp_stage3_branches_full=false
svpp_stage4_branches_full=false

if [[ "$svpp_syntax_status" == "pass" && "$svpp_syntax_failure_count" == "0" ]]; then
    svpp_syntax_closure_gate_green=true
fi
if [[ "$svpp_parseability_parser_rejections_total" == "0" ]]; then
    svpp_parser_rejections_zero=true
fi
if [[ "$svpp_parseability_rejected_total" == "0" ]]; then
    svpp_parseability_rejections_zero=true
fi
if [[ "$svpp_reach_stage3_targets" == "0" ]]; then
    svpp_stage3_targets_zero=true
fi
if [[ "$svpp_reach_stage4_targets" == "0" ]]; then
    svpp_stage4_targets_zero=true
fi
if fraction_is_full "$svpp_reach_stage3_rules"; then
    svpp_stage3_rules_full=true
fi
if fraction_is_full "$svpp_reach_stage4_rules"; then
    svpp_stage4_rules_full=true
fi
if fraction_is_full "$svpp_reach_stage3_branches"; then
    svpp_stage3_branches_full=true
fi
if fraction_is_full "$svpp_reach_stage4_branches"; then
    svpp_stage4_branches_full=true
fi

svpp_closure_criteria_total_count=11
svpp_closure_criteria_satisfied_count=0
if [[ "$svpp_syntax_closure_gate_green" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_aggregate_contract_green" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_reachability_closure_green" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_parser_rejections_zero" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_parseability_rejections_zero" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_stage3_targets_zero" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_stage4_targets_zero" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_stage3_rules_full" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_stage4_rules_full" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_stage3_branches_full" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi
if [[ "$svpp_stage4_branches_full" == true ]]; then
    ((svpp_closure_criteria_satisfied_count += 1))
fi

declare -a svpp_unmet=()
if [[ "$svpp_syntax_closure_gate_green" != true ]]; then
    svpp_unmet+=("syntax_closure_gate_status=${svpp_syntax_status} failure_count=${svpp_syntax_failure_count}")
fi
if [[ "$svpp_parser_rejections_zero" != true ]]; then
    svpp_unmet+=("parseability_parser_rejections_total=${svpp_parseability_parser_rejections_total} > 0")
fi
if [[ "$svpp_parseability_rejections_zero" != true ]]; then
    svpp_unmet+=("parseability_rejected_total=${svpp_parseability_rejected_total} > 0")
fi
if [[ "$svpp_stage3_targets_zero" != true ]]; then
    svpp_unmet+=("reachability_stage3_targets=${svpp_reach_stage3_targets} > 0")
fi
if [[ "$svpp_stage4_targets_zero" != true ]]; then
    svpp_unmet+=("reachability_stage4_targets=${svpp_reach_stage4_targets} > 0")
fi
if [[ "$svpp_stage3_rules_full" != true ]]; then
    svpp_unmet+=("reachability_stage3_covered_reachable_rules=${svpp_reach_stage3_rules} is not full")
fi
if [[ "$svpp_stage4_rules_full" != true ]]; then
    svpp_unmet+=("reachability_stage4_covered_reachable_rules=${svpp_reach_stage4_rules} is not full")
fi
if [[ "$svpp_stage3_branches_full" != true ]]; then
    svpp_unmet+=("reachability_stage3_covered_reachable_branches=${svpp_reach_stage3_branches} is not full")
fi
if [[ "$svpp_stage4_branches_full" != true ]]; then
    svpp_unmet+=("reachability_stage4_covered_reachable_branches=${svpp_reach_stage4_branches} is not full")
fi

if [[ "$svpp_syntax_closure_gate_green" == true \
   && "$svpp_parser_rejections_zero" == true \
   && "$svpp_parseability_rejections_zero" == true \
   && "$svpp_stage3_targets_zero" == true \
   && "$svpp_stage4_targets_zero" == true \
   && "$svpp_stage3_rules_full" == true \
   && "$svpp_stage4_rules_full" == true \
   && "$svpp_stage3_branches_full" == true \
   && "$svpp_stage4_branches_full" == true ]]; then
    svpp_status="Done"
else
    svpp_status="Mostly Done"
fi

live_tracker_sv_status="$(markdown_table_status_for_row "| \`systemverilog\` main parser" "$LIVE_TRACKER_FILE")"
live_tracker_svpp_status="$(markdown_table_status_for_row "| \`systemverilog_preprocessor\` frontend" "$LIVE_TRACKER_FILE")"

if [[ "$live_tracker_sv_status" != "$sv_status" ]]; then
    echo "error: live tracker systemverilog status mismatch: tracker='$live_tracker_sv_status' computed='$sv_status'" >&2
    exit 1
fi
if [[ "$live_tracker_svpp_status" != "$svpp_status" ]]; then
    echo "error: live tracker systemverilog_preprocessor status mismatch: tracker='$live_tracker_svpp_status' computed='$svpp_status'" >&2
    exit 1
fi

sv_tracker_alignment_ok=true
svpp_tracker_alignment_ok=true

sv_primary_unmet_closure_criterion="<none>"
if [[ "${#sv_unmet[@]}" -gt 0 ]]; then
    sv_primary_unmet_closure_criterion="${sv_unmet[0]}"
fi
svpp_primary_unmet_closure_criterion="<none>"
if [[ "${#svpp_unmet[@]}" -gt 0 ]]; then
    svpp_primary_unmet_closure_criterion="${svpp_unmet[0]}"
fi
sv_unmet_json="$(jq -cn '$ARGS.positional' --args "${sv_unmet[@]}")"
svpp_unmet_json="$(jq -cn '$ARGS.positional' --args "${svpp_unmet[@]}")"
generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

jq -n \
    --arg generated_at_utc "$generated_at_utc" \
    --arg live_tracker_file "$LIVE_TRACKER_FILE" \
    --arg sv_status "$sv_status" \
    --arg sv_tracker_status "$live_tracker_sv_status" \
    --argjson sv_tracker_alignment_ok "$sv_tracker_alignment_ok" \
    --arg sv_syntax_closure_state_dir "$sv_syntax_closure_state_dir" \
    --arg sv_syntax_summary_txt "$sv_syntax_summary_txt" \
    --arg sv_syntax_summary_json "$sv_syntax_summary_json" \
    --arg sv_syntax_status "$sv_syntax_status" \
    --arg sv_syntax_failure_count "$sv_syntax_failure_count" \
    --arg sv_syntax_defined_rule_count "$sv_syntax_defined_rule_count" \
    --arg sv_syntax_unresolved_rule_reference_count "$sv_syntax_unresolved_rule_reference_count" \
    --arg sv_syntax_unreachable_rules "$sv_syntax_unreachable_rules" \
    --arg sv_syntax_unreachable_branches "$sv_syntax_unreachable_branches" \
    --arg sv_syntax_target_debt_count "$sv_syntax_target_debt_count" \
    --arg sv_semantic_scope_contract_state_dir "$sv_semantic_scope_contract_state_dir" \
    --arg sv_semantic_scope_contract_summary_txt "$sv_semantic_scope_contract_summary_txt" \
    --arg sv_semantic_scope_contract_summary_json "$sv_semantic_scope_contract_summary_json" \
    --arg sv_parser_gate_state_dir "$sv_parser_gate_state_dir" \
    --arg sv_semantic_scope_case_count "$sv_semantic_scope_case_count" \
    --arg sv_semantic_scope_failed_count "$sv_semantic_scope_failed_count" \
    --arg sv_parser_summary_txt "$sv_parser_summary_txt" \
    --arg sv_parser_summary_json "$sv_parser_summary_json" \
    --arg sv_generation_parser_rejections_total "$sv_generation_parser_rejections_total" \
    --arg sv_shadow_parser_rejections_total "$sv_shadow_parser_rejections_total" \
    --arg sv_focused_replay_target_count "$sv_focused_replay_target_count" \
    --arg sv_focused_replay_covered_reachable_rules "$sv_focused_replay_covered_reachable_rules" \
    --arg sv_focused_replay_covered_reachable_branches "$sv_focused_replay_covered_reachable_branches" \
    --arg sv_replay_gap_target_primary_rule "$sv_replay_gap_target_primary_rule" \
    --arg sv_closure_criteria_total_count "$sv_closure_criteria_total_count" \
    --arg sv_closure_criteria_satisfied_count "$sv_closure_criteria_satisfied_count" \
    --arg sv_primary_unmet_closure_criterion "$sv_primary_unmet_closure_criterion" \
    --argjson sv_syntax_closure_gate_green "$sv_syntax_closure_gate_green" \
    --argjson sv_aggregate_contract_green "$sv_aggregate_contract_green" \
    --argjson sv_generation_parser_rejections_zero "$sv_generation_parser_rejections_zero" \
    --argjson sv_shadow_parser_rejections_zero "$sv_shadow_parser_rejections_zero" \
    --argjson sv_focused_replay_target_debt_zero "$sv_focused_replay_target_debt_zero" \
    --argjson sv_semantic_scope_contract_green "$sv_semantic_scope_contract_green" \
    --argjson sv_unmet "$sv_unmet_json" \
    --arg svpp_status "$svpp_status" \
    --arg svpp_tracker_status "$live_tracker_svpp_status" \
    --argjson svpp_tracker_alignment_ok "$svpp_tracker_alignment_ok" \
    --arg sv_preprocessor_syntax_closure_state_dir "$sv_preprocessor_syntax_closure_state_dir" \
    --arg svpp_syntax_summary_txt "$svpp_syntax_summary_txt" \
    --arg svpp_syntax_summary_json "$svpp_syntax_summary_json" \
    --arg svpp_syntax_status "$svpp_syntax_status" \
    --arg svpp_syntax_failure_count "$svpp_syntax_failure_count" \
    --arg svpp_syntax_defined_rule_count "$svpp_syntax_defined_rule_count" \
    --arg svpp_syntax_unresolved_rule_reference_count "$svpp_syntax_unresolved_rule_reference_count" \
    --arg svpp_syntax_unreachable_rules "$svpp_syntax_unreachable_rules" \
    --arg svpp_syntax_unreachable_branches "$svpp_syntax_unreachable_branches" \
    --arg svpp_syntax_target_debt_count "$svpp_syntax_target_debt_count" \
    --arg sv_preprocessor_aggregate_state_dir "$sv_preprocessor_aggregate_state_dir" \
    --arg svpp_aggregate_summary_txt "$sv_preprocessor_aggregate_summary_txt" \
    --arg svpp_aggregate_summary_json "$sv_preprocessor_aggregate_summary_json" \
    --arg sv_preprocessor_reachability_state_dir "$sv_preprocessor_reachability_state_dir" \
    --arg svpp_reachability_summary_txt "$sv_preprocessor_reachability_summary_txt" \
    --arg svpp_parseability_parser_rejections_total "$svpp_parseability_parser_rejections_total" \
    --arg svpp_parseability_rejected_total "$svpp_parseability_rejected_total" \
    --arg svpp_final_targets "$svpp_final_targets" \
    --arg svpp_covered_reachable_rules "$svpp_covered_reachable_rules" \
    --arg svpp_covered_reachable_branches "$svpp_covered_reachable_branches" \
    --arg svpp_counterexample_primary_stage "$svpp_counterexample_primary_stage" \
    --arg svpp_reach_stage3_targets "$svpp_reach_stage3_targets" \
    --arg svpp_reach_stage4_targets "$svpp_reach_stage4_targets" \
    --arg svpp_reach_stage3_rules "$svpp_reach_stage3_rules" \
    --arg svpp_reach_stage4_rules "$svpp_reach_stage4_rules" \
    --arg svpp_reach_stage3_branches "$svpp_reach_stage3_branches" \
    --arg svpp_reach_stage4_branches "$svpp_reach_stage4_branches" \
    --arg svpp_closure_criteria_total_count "$svpp_closure_criteria_total_count" \
    --arg svpp_closure_criteria_satisfied_count "$svpp_closure_criteria_satisfied_count" \
    --arg svpp_primary_unmet_closure_criterion "$svpp_primary_unmet_closure_criterion" \
    --argjson svpp_syntax_closure_gate_green "$svpp_syntax_closure_gate_green" \
    --argjson svpp_aggregate_contract_green "$svpp_aggregate_contract_green" \
    --argjson svpp_reachability_closure_green "$svpp_reachability_closure_green" \
    --argjson svpp_parser_rejections_zero "$svpp_parser_rejections_zero" \
    --argjson svpp_parseability_rejections_zero "$svpp_parseability_rejections_zero" \
    --argjson svpp_stage3_targets_zero "$svpp_stage3_targets_zero" \
    --argjson svpp_stage4_targets_zero "$svpp_stage4_targets_zero" \
    --argjson svpp_stage3_rules_full "$svpp_stage3_rules_full" \
    --argjson svpp_stage4_rules_full "$svpp_stage4_rules_full" \
    --argjson svpp_stage3_branches_full "$svpp_stage3_branches_full" \
    --argjson svpp_stage4_branches_full "$svpp_stage4_branches_full" \
    --argjson svpp_unmet "$svpp_unmet_json" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    '
    {
      gate: "sv_parser_family_status_gate",
      version: 4,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      live_tracker_file: $live_tracker_file,
      status_rule_done: "Done requires a formally exhaustive, machine-checkable closure surface with no remaining parser rejection debt and no remaining coverage/gap debt for the family claim.",
      families: [
        {
          family: "systemverilog",
          computed_status: $sv_status,
          live_tracker_status: $sv_tracker_status,
          tracker_alignment_ok: $sv_tracker_alignment_ok,
          proof_surfaces: {
            syntax_closure_state_dir: $sv_syntax_closure_state_dir,
            syntax_closure_summary_txt: $sv_syntax_summary_txt,
            syntax_closure_summary_json: $sv_syntax_summary_json,
            parser_aggregate_state_dir: $sv_parser_gate_state_dir,
            parser_aggregate_summary_txt: $sv_parser_summary_txt,
            parser_aggregate_summary_json: $sv_parser_summary_json,
            semantic_scope_contract_state_dir: $sv_semantic_scope_contract_state_dir,
            semantic_scope_contract_summary_txt: $sv_semantic_scope_contract_summary_txt,
            semantic_scope_contract_summary_json: $sv_semantic_scope_contract_summary_json
          },
          closure_criteria_total_count: ($sv_closure_criteria_total_count | tonumber),
          closure_criteria_satisfied_count: ($sv_closure_criteria_satisfied_count | tonumber),
          closure_criteria_unsatisfied_count: ($sv_unmet | length),
          primary_unmet_closure_criterion: $sv_primary_unmet_closure_criterion,
          criteria: {
            syntax_closure_gate_green: $sv_syntax_closure_gate_green,
            parser_aggregate_contract_green: $sv_aggregate_contract_green,
            generation_parser_rejections_zero: $sv_generation_parser_rejections_zero,
            replay_shadow_parser_rejections_zero: $sv_shadow_parser_rejections_zero,
            focused_replay_target_debt_zero: $sv_focused_replay_target_debt_zero,
            semantic_scope_contract_green: $sv_semantic_scope_contract_green
          },
          metrics: {
            syntax_closure_status: $sv_syntax_status,
            syntax_closure_failure_count: ($sv_syntax_failure_count | tonumber),
            syntax_defined_rule_count: ($sv_syntax_defined_rule_count | tonumber),
            syntax_unresolved_rule_reference_count: ($sv_syntax_unresolved_rule_reference_count | tonumber),
            syntax_unreachable_rules: ($sv_syntax_unreachable_rules | tonumber),
            syntax_unreachable_branches: ($sv_syntax_unreachable_branches | tonumber),
            syntax_target_debt_count: ($sv_syntax_target_debt_count | tonumber),
            semantic_scope_case_count: ($sv_semantic_scope_case_count | tonumber),
            semantic_scope_failed_count: ($sv_semantic_scope_failed_count | tonumber),
            generation_parser_rejections_total: ($sv_generation_parser_rejections_total | tonumber),
            replay_shadow_parser_rejections_total: ($sv_shadow_parser_rejections_total | tonumber),
            focused_replay_target_count: ($sv_focused_replay_target_count | tonumber),
            focused_replay_covered_reachable_rules: ($sv_focused_replay_covered_reachable_rules | tonumber),
            focused_replay_covered_reachable_branches: ($sv_focused_replay_covered_reachable_branches | tonumber),
            replay_gap_target_primary_rule: $sv_replay_gap_target_primary_rule
          },
          unmet_closure_criteria: $sv_unmet,
          unmet_closure_criteria_details: (
            []
            + (if $sv_syntax_closure_gate_green then [] else [{
                criterion: "syntax_closure_gate_green",
                evidence_key: "syntax_closure_status",
                observed: ("status=" + $sv_syntax_status + " failure_count=" + $sv_syntax_failure_count),
                expected: "status=pass failure_count=0",
                detail: ("syntax_closure_gate_status=" + $sv_syntax_status + " failure_count=" + $sv_syntax_failure_count)
              }] end)
            + (if $sv_generation_parser_rejections_zero then [] else [{
                criterion: "generation_parser_rejections_zero",
                evidence_key: "generation_parser_rejections_total",
                observed: $sv_generation_parser_rejections_total,
                expected: "0",
                detail: ("generation_parser_rejections_total=" + $sv_generation_parser_rejections_total + " > 0")
              }] end)
            + (if $sv_shadow_parser_rejections_zero then [] else [{
                criterion: "replay_shadow_parser_rejections_zero",
                evidence_key: "replay_shadow_parser_rejections_total",
                observed: $sv_shadow_parser_rejections_total,
                expected: "0",
                detail: ("shadow_parser_rejections_total=" + $sv_shadow_parser_rejections_total + " > 0")
              }] end)
            + (if $sv_focused_replay_target_debt_zero then [] else [{
                criterion: "focused_replay_target_debt_zero",
                evidence_key: "focused_replay_target_count",
                observed: $sv_focused_replay_target_count,
                expected: "0",
                detail: ("focused_replay_target_count=" + $sv_focused_replay_target_count + " > 0")
              }] end)
            + (if $sv_semantic_scope_contract_green then [] else [{
                criterion: "semantic_scope_contract_green",
                evidence_key: "semantic_scope_failed_count",
                observed: $sv_semantic_scope_failed_count,
                expected: "0",
                detail: ("semantic_scope_failed_count=" + $sv_semantic_scope_failed_count + " > 0")
              }] end)
          )
        },
        {
          family: "systemverilog_preprocessor",
          computed_status: $svpp_status,
          live_tracker_status: $svpp_tracker_status,
          tracker_alignment_ok: $svpp_tracker_alignment_ok,
          proof_surfaces: {
            syntax_closure_state_dir: $sv_preprocessor_syntax_closure_state_dir,
            syntax_closure_summary_txt: $svpp_syntax_summary_txt,
            syntax_closure_summary_json: $svpp_syntax_summary_json,
            aggregate_state_dir: $sv_preprocessor_aggregate_state_dir,
            aggregate_summary_txt: $svpp_aggregate_summary_txt,
            aggregate_summary_json: $svpp_aggregate_summary_json,
            reachability_state_dir: $sv_preprocessor_reachability_state_dir,
            reachability_summary_txt: $svpp_reachability_summary_txt
          },
          closure_criteria_total_count: ($svpp_closure_criteria_total_count | tonumber),
          closure_criteria_satisfied_count: ($svpp_closure_criteria_satisfied_count | tonumber),
          closure_criteria_unsatisfied_count: ($svpp_unmet | length),
          primary_unmet_closure_criterion: $svpp_primary_unmet_closure_criterion,
          criteria: {
            syntax_closure_gate_green: $svpp_syntax_closure_gate_green,
            aggregate_contract_green: $svpp_aggregate_contract_green,
            reachability_closure_green: $svpp_reachability_closure_green,
            parser_rejections_zero: $svpp_parser_rejections_zero,
            parseability_rejections_zero: $svpp_parseability_rejections_zero,
            reachability_stage3_targets_zero: $svpp_stage3_targets_zero,
            reachability_stage4_targets_zero: $svpp_stage4_targets_zero,
            reachability_stage3_rules_full: $svpp_stage3_rules_full,
            reachability_stage4_rules_full: $svpp_stage4_rules_full,
            reachability_stage3_branches_full: $svpp_stage3_branches_full,
            reachability_stage4_branches_full: $svpp_stage4_branches_full
          },
          metrics: {
            syntax_closure_status: $svpp_syntax_status,
            syntax_closure_failure_count: ($svpp_syntax_failure_count | tonumber),
            syntax_defined_rule_count: ($svpp_syntax_defined_rule_count | tonumber),
            syntax_unresolved_rule_reference_count: ($svpp_syntax_unresolved_rule_reference_count | tonumber),
            syntax_unreachable_rules: ($svpp_syntax_unreachable_rules | tonumber),
            syntax_unreachable_branches: ($svpp_syntax_unreachable_branches | tonumber),
            syntax_target_debt_count: ($svpp_syntax_target_debt_count | tonumber),
            parseability_parser_rejections_total: ($svpp_parseability_parser_rejections_total | tonumber),
            parseability_rejected_total: ($svpp_parseability_rejected_total | tonumber),
            final_targets: ($svpp_final_targets | tonumber),
            covered_reachable_rules: $svpp_covered_reachable_rules,
            covered_reachable_branches: $svpp_covered_reachable_branches,
            counterexample_primary_stage: $svpp_counterexample_primary_stage,
            reachability_stage3_targets: ($svpp_reach_stage3_targets | tonumber),
            reachability_stage4_targets: ($svpp_reach_stage4_targets | tonumber),
            reachability_stage3_rules: $svpp_reach_stage3_rules,
            reachability_stage4_rules: $svpp_reach_stage4_rules,
            reachability_stage3_branches: $svpp_reach_stage3_branches,
            reachability_stage4_branches: $svpp_reach_stage4_branches
          },
          unmet_closure_criteria: $svpp_unmet,
          unmet_closure_criteria_details: (
            []
            + (if $svpp_syntax_closure_gate_green then [] else [{
                criterion: "syntax_closure_gate_green",
                evidence_key: "syntax_closure_status",
                observed: ("status=" + $svpp_syntax_status + " failure_count=" + $svpp_syntax_failure_count),
                expected: "status=pass failure_count=0",
                detail: ("syntax_closure_gate_status=" + $svpp_syntax_status + " failure_count=" + $svpp_syntax_failure_count)
              }] end)
            + (if $svpp_parser_rejections_zero then [] else [{
                criterion: "parser_rejections_zero",
                evidence_key: "parseability_parser_rejections_total",
                observed: $svpp_parseability_parser_rejections_total,
                expected: "0",
                detail: ("parseability_parser_rejections_total=" + $svpp_parseability_parser_rejections_total + " > 0")
              }] end)
            + (if $svpp_parseability_rejections_zero then [] else [{
                criterion: "parseability_rejections_zero",
                evidence_key: "parseability_rejected_total",
                observed: $svpp_parseability_rejected_total,
                expected: "0",
                detail: ("parseability_rejected_total=" + $svpp_parseability_rejected_total + " > 0")
              }] end)
            + (if $svpp_stage3_targets_zero then [] else [{
                criterion: "reachability_stage3_targets_zero",
                evidence_key: "reachability_stage3_targets",
                observed: $svpp_reach_stage3_targets,
                expected: "0",
                detail: ("reachability_stage3_targets=" + $svpp_reach_stage3_targets + " > 0")
              }] end)
            + (if $svpp_stage4_targets_zero then [] else [{
                criterion: "reachability_stage4_targets_zero",
                evidence_key: "reachability_stage4_targets",
                observed: $svpp_reach_stage4_targets,
                expected: "0",
                detail: ("reachability_stage4_targets=" + $svpp_reach_stage4_targets + " > 0")
              }] end)
            + (if $svpp_stage3_rules_full then [] else [{
                criterion: "reachability_stage3_rules_full",
                evidence_key: "reachability_stage3_rules",
                observed: $svpp_reach_stage3_rules,
                expected: "full",
                detail: ("reachability_stage3_covered_reachable_rules=" + $svpp_reach_stage3_rules + " is not full")
              }] end)
            + (if $svpp_stage4_rules_full then [] else [{
                criterion: "reachability_stage4_rules_full",
                evidence_key: "reachability_stage4_rules",
                observed: $svpp_reach_stage4_rules,
                expected: "full",
                detail: ("reachability_stage4_covered_reachable_rules=" + $svpp_reach_stage4_rules + " is not full")
              }] end)
            + (if $svpp_stage3_branches_full then [] else [{
                criterion: "reachability_stage3_branches_full",
                evidence_key: "reachability_stage3_branches",
                observed: $svpp_reach_stage3_branches,
                expected: "full",
                detail: ("reachability_stage3_covered_reachable_branches=" + $svpp_reach_stage3_branches + " is not full")
              }] end)
            + (if $svpp_stage4_branches_full then [] else [{
                criterion: "reachability_stage4_branches_full",
                evidence_key: "reachability_stage4_branches",
                observed: $svpp_reach_stage4_branches,
                expected: "full",
                detail: ("reachability_stage4_covered_reachable_branches=" + $svpp_reach_stage4_branches + " is not full")
              }] end)
          )
        }
      ]
    }
    ' >"$SUMMARY_JSON"
require_nonempty_file "$SUMMARY_JSON"

sv_unmet_details_json="$(jq -cer '.families[] | select(.family=="systemverilog") | .unmet_closure_criteria_details' "$SUMMARY_JSON")"
svpp_unmet_details_json="$(jq -cer '.families[] | select(.family=="systemverilog_preprocessor") | .unmet_closure_criteria_details' "$SUMMARY_JSON")"

{
    echo "SV Parser Family Status Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "live_tracker_file: $LIVE_TRACKER_FILE"
    echo "summary_json: $SUMMARY_JSON"
    echo "systemverilog_status: $sv_status"
    echo "systemverilog_tracker_status: $live_tracker_sv_status"
    echo "systemverilog_tracker_alignment_ok: $sv_tracker_alignment_ok"
    echo "systemverilog_syntax_closure_status: $sv_syntax_status"
    echo "systemverilog_syntax_closure_failure_count: $sv_syntax_failure_count"
    echo "systemverilog_syntax_defined_rule_count: $sv_syntax_defined_rule_count"
    echo "systemverilog_syntax_unresolved_rule_reference_count: $sv_syntax_unresolved_rule_reference_count"
    echo "systemverilog_syntax_unreachable_rules: $sv_syntax_unreachable_rules"
    echo "systemverilog_syntax_unreachable_branches: $sv_syntax_unreachable_branches"
    echo "systemverilog_syntax_target_debt_count: $sv_syntax_target_debt_count"
    echo "systemverilog_syntax_closure_state_dir: $sv_syntax_closure_state_dir"
    echo "systemverilog_syntax_closure_summary_txt: $sv_syntax_summary_txt"
    echo "systemverilog_syntax_closure_summary_json: $sv_syntax_summary_json"
    echo "systemverilog_parser_aggregate_state_dir: $sv_parser_gate_state_dir"
    echo "systemverilog_parser_aggregate_summary_txt: $sv_parser_summary_txt"
    echo "systemverilog_parser_aggregate_summary_json: $sv_parser_summary_json"
    echo "systemverilog_semantic_scope_contract_state_dir: $sv_semantic_scope_contract_state_dir"
    echo "systemverilog_semantic_scope_contract_summary_txt: $sv_semantic_scope_contract_summary_txt"
    echo "systemverilog_semantic_scope_contract_summary_json: $sv_semantic_scope_contract_summary_json"
    echo "systemverilog_semantic_scope_case_count: $sv_semantic_scope_case_count"
    echo "systemverilog_semantic_scope_failed_count: $sv_semantic_scope_failed_count"
    echo "systemverilog_generation_parser_rejections_total: $sv_generation_parser_rejections_total"
    echo "systemverilog_replay_shadow_parser_rejections_total: $sv_shadow_parser_rejections_total"
    echo "systemverilog_focused_replay_target_count: $sv_focused_replay_target_count"
    echo "systemverilog_replay_gap_target_primary_rule: $sv_replay_gap_target_primary_rule"
    echo "systemverilog_closure_criteria_total_count: $sv_closure_criteria_total_count"
    echo "systemverilog_closure_criteria_satisfied_count: $sv_closure_criteria_satisfied_count"
    echo "systemverilog_closure_criteria_unsatisfied_count: ${#sv_unmet[@]}"
    echo "systemverilog_unmet_closure_criteria_count: ${#sv_unmet[@]}"
    echo "systemverilog_primary_unmet_closure_criterion: $sv_primary_unmet_closure_criterion"
    echo "systemverilog_unmet_closure_criteria_json: $sv_unmet_json"
    echo "systemverilog_unmet_closure_criteria_details_json: $sv_unmet_details_json"
    for idx in "${!sv_unmet[@]}"; do
        echo "systemverilog_unmet_closure_criterion[$idx]: ${sv_unmet[$idx]}"
    done
    echo "systemverilog_preprocessor_status: $svpp_status"
    echo "systemverilog_preprocessor_tracker_status: $live_tracker_svpp_status"
    echo "systemverilog_preprocessor_tracker_alignment_ok: $svpp_tracker_alignment_ok"
    echo "systemverilog_preprocessor_syntax_closure_status: $svpp_syntax_status"
    echo "systemverilog_preprocessor_syntax_closure_failure_count: $svpp_syntax_failure_count"
    echo "systemverilog_preprocessor_syntax_defined_rule_count: $svpp_syntax_defined_rule_count"
    echo "systemverilog_preprocessor_syntax_unresolved_rule_reference_count: $svpp_syntax_unresolved_rule_reference_count"
    echo "systemverilog_preprocessor_syntax_unreachable_rules: $svpp_syntax_unreachable_rules"
    echo "systemverilog_preprocessor_syntax_unreachable_branches: $svpp_syntax_unreachable_branches"
    echo "systemverilog_preprocessor_syntax_target_debt_count: $svpp_syntax_target_debt_count"
    echo "systemverilog_preprocessor_syntax_closure_state_dir: $sv_preprocessor_syntax_closure_state_dir"
    echo "systemverilog_preprocessor_syntax_closure_summary_txt: $svpp_syntax_summary_txt"
    echo "systemverilog_preprocessor_syntax_closure_summary_json: $svpp_syntax_summary_json"
    echo "systemverilog_preprocessor_aggregate_state_dir: $sv_preprocessor_aggregate_state_dir"
    echo "systemverilog_preprocessor_aggregate_summary_txt: $sv_preprocessor_aggregate_summary_txt"
    echo "systemverilog_preprocessor_aggregate_summary_json: $sv_preprocessor_aggregate_summary_json"
    echo "systemverilog_preprocessor_reachability_state_dir: $sv_preprocessor_reachability_state_dir"
    echo "systemverilog_preprocessor_reachability_summary_txt: $sv_preprocessor_reachability_summary_txt"
    echo "systemverilog_preprocessor_parseability_parser_rejections_total: $svpp_parseability_parser_rejections_total"
    echo "systemverilog_preprocessor_parseability_rejected_total: $svpp_parseability_rejected_total"
    echo "systemverilog_preprocessor_final_targets: $svpp_final_targets"
    echo "systemverilog_preprocessor_counterexample_primary_stage: $svpp_counterexample_primary_stage"
    echo "systemverilog_preprocessor_reachability_stage3_targets: $svpp_reach_stage3_targets"
    echo "systemverilog_preprocessor_reachability_stage4_targets: $svpp_reach_stage4_targets"
    echo "systemverilog_preprocessor_reachability_stage3_rules: $svpp_reach_stage3_rules"
    echo "systemverilog_preprocessor_reachability_stage4_rules: $svpp_reach_stage4_rules"
    echo "systemverilog_preprocessor_reachability_stage3_branches: $svpp_reach_stage3_branches"
    echo "systemverilog_preprocessor_reachability_stage4_branches: $svpp_reach_stage4_branches"
    echo "systemverilog_preprocessor_closure_criteria_total_count: $svpp_closure_criteria_total_count"
    echo "systemverilog_preprocessor_closure_criteria_satisfied_count: $svpp_closure_criteria_satisfied_count"
    echo "systemverilog_preprocessor_closure_criteria_unsatisfied_count: ${#svpp_unmet[@]}"
    echo "systemverilog_preprocessor_unmet_closure_criteria_count: ${#svpp_unmet[@]}"
    echo "systemverilog_preprocessor_primary_unmet_closure_criterion: $svpp_primary_unmet_closure_criterion"
    echo "systemverilog_preprocessor_unmet_closure_criteria_json: $svpp_unmet_json"
    echo "systemverilog_preprocessor_unmet_closure_criteria_details_json: $svpp_unmet_details_json"
    for idx in "${!svpp_unmet[@]}"; do
        echo "systemverilog_preprocessor_unmet_closure_criterion[$idx]: ${svpp_unmet[$idx]}"
    done
} | tee "$SUMMARY_TXT"

echo "✅ SV parser-family status gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
