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
SV_PARSER_AGGREGATE_GATE="$RUST_DIR/scripts/sv_parser_aggregate_contract_gate.sh"
SV_PREPROCESSOR_AGGREGATE_GATE="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"
SV_PREPROCESSOR_REACHABILITY_GATE="$RUST_DIR/scripts/sv_preprocessor_reachability_closure_gate.sh"

EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_SV_STIMULI_QUALITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-}"

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
require_file "$SV_PARSER_AGGREGATE_GATE"
require_file "$SV_PREPROCESSOR_AGGREGATE_GATE"
require_file "$SV_PREPROCESSOR_REACHABILITY_GATE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

sv_syntax_closure_state_dir="$WORK_DIR/sv_syntax_closure_gate"
sv_parser_gate_state_dir="$WORK_DIR/sv_parser_aggregate_contract_gate"
sv_preprocessor_aggregate_state_dir="$WORK_DIR/sv_preprocessor_aggregate_contract_gate"
sv_preprocessor_reachability_state_dir="$WORK_DIR/sv_preprocessor_reachability_closure_gate"

if [[ -n "$EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR" ]]; then
    sv_syntax_closure_state_dir="$EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR"
else
    run_logged "sv_syntax_closure_gate" \
        env \
            PGEN_SV_SYNTAX_CLOSURE_STATE_DIR="$sv_syntax_closure_state_dir" \
            "$SV_SYNTAX_CLOSURE_GATE"
fi

if [[ -n "$EXISTING_SV_STIMULI_QUALITY_STATE_DIR" ]]; then
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
if [[ -n "$preprocessor_quality_state_dir" ]]; then
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

run_logged "sv_preprocessor_reachability_closure_gate" \
    env \
        PGEN_SV_PREPROCESSOR_REACHABILITY_CLOSURE_STATE_DIR="$sv_preprocessor_reachability_state_dir" \
        PGEN_SV_PREPROCESSOR_REACHABILITY_CLOSURE_EXISTING_QUALITY_STATE_DIR="$preprocessor_quality_state_dir" \
        "$SV_PREPROCESSOR_REACHABILITY_GATE"

sv_parser_summary_txt="$sv_parser_gate_state_dir/summary.txt"
sv_preprocessor_aggregate_summary_txt="$sv_preprocessor_aggregate_state_dir/summary.txt"
sv_preprocessor_reachability_summary_txt="$sv_preprocessor_reachability_state_dir/summary.txt"

require_nonempty_file "$sv_parser_summary_txt"
require_nonempty_file "$sv_preprocessor_aggregate_summary_txt"
require_nonempty_file "$sv_preprocessor_reachability_summary_txt"

sv_syntax_summary_json="$sv_syntax_closure_state_dir/summary.json"
require_nonempty_file "$sv_syntax_summary_json"

sv_syntax_status="$(jq -er '.status | strings' "$sv_syntax_summary_json")"
sv_syntax_failure_count="$(jq -er '.failure_count | numbers' "$sv_syntax_summary_json")"
sv_syntax_defined_rule_count="$(jq -er '.metrics.defined_rule_count | numbers' "$sv_syntax_summary_json")"
sv_syntax_unresolved_rule_reference_count="$(jq -er '.metrics.unresolved_rule_reference_count | numbers' "$sv_syntax_summary_json")"
sv_syntax_unreachable_rules="$(jq -er '.metrics.unreachable_rules | numbers' "$sv_syntax_summary_json")"
sv_syntax_unreachable_branches="$(jq -er '.metrics.unreachable_branches | numbers' "$sv_syntax_summary_json")"
sv_syntax_target_debt_count="$(jq -er '.metrics.target_debt_count | numbers' "$sv_syntax_summary_json")"

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

if [[ "$sv_syntax_closure_gate_green" == true && "$sv_generation_parser_rejections_zero" == true && "$sv_shadow_parser_rejections_zero" == true && "$sv_focused_replay_target_debt_zero" == true ]]; then
    sv_status="Done"
else
    sv_status="Mostly Done"
fi

svpp_aggregate_contract_green=true
svpp_reachability_closure_green=true
svpp_parser_rejections_zero=false
svpp_parseability_rejections_zero=false
svpp_stage3_targets_zero=false
svpp_stage4_targets_zero=false
svpp_stage3_rules_full=false
svpp_stage4_rules_full=false
svpp_stage3_branches_full=false
svpp_stage4_branches_full=false

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

declare -a svpp_unmet=()
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

if [[ "$svpp_parser_rejections_zero" == true \
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

sv_unmet_json="$(jq -n '$ARGS.positional' --args "${sv_unmet[@]}")"
svpp_unmet_json="$(jq -n '$ARGS.positional' --args "${svpp_unmet[@]}")"

jq -n \
    --arg generated_at_utc "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
    --arg live_tracker_file "$LIVE_TRACKER_FILE" \
    --arg sv_status "$sv_status" \
    --arg sv_tracker_status "$live_tracker_sv_status" \
    --arg sv_syntax_summary_json "$sv_syntax_summary_json" \
    --arg sv_syntax_status "$sv_syntax_status" \
    --arg sv_syntax_failure_count "$sv_syntax_failure_count" \
    --arg sv_syntax_defined_rule_count "$sv_syntax_defined_rule_count" \
    --arg sv_syntax_unresolved_rule_reference_count "$sv_syntax_unresolved_rule_reference_count" \
    --arg sv_syntax_unreachable_rules "$sv_syntax_unreachable_rules" \
    --arg sv_syntax_unreachable_branches "$sv_syntax_unreachable_branches" \
    --arg sv_syntax_target_debt_count "$sv_syntax_target_debt_count" \
    --arg sv_parser_summary_txt "$sv_parser_summary_txt" \
    --arg sv_generation_parser_rejections_total "$sv_generation_parser_rejections_total" \
    --arg sv_shadow_parser_rejections_total "$sv_shadow_parser_rejections_total" \
    --arg sv_focused_replay_target_count "$sv_focused_replay_target_count" \
    --arg sv_focused_replay_covered_reachable_rules "$sv_focused_replay_covered_reachable_rules" \
    --arg sv_focused_replay_covered_reachable_branches "$sv_focused_replay_covered_reachable_branches" \
    --arg sv_replay_gap_target_primary_rule "$sv_replay_gap_target_primary_rule" \
    --argjson sv_syntax_closure_gate_green "$sv_syntax_closure_gate_green" \
    --argjson sv_generation_parser_rejections_zero "$sv_generation_parser_rejections_zero" \
    --argjson sv_shadow_parser_rejections_zero "$sv_shadow_parser_rejections_zero" \
    --argjson sv_focused_replay_target_debt_zero "$sv_focused_replay_target_debt_zero" \
    --argjson sv_unmet "$sv_unmet_json" \
    --arg svpp_status "$svpp_status" \
    --arg svpp_tracker_status "$live_tracker_svpp_status" \
    --arg svpp_aggregate_summary_txt "$sv_preprocessor_aggregate_summary_txt" \
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
    --argjson svpp_parser_rejections_zero "$svpp_parser_rejections_zero" \
    --argjson svpp_parseability_rejections_zero "$svpp_parseability_rejections_zero" \
    --argjson svpp_stage3_targets_zero "$svpp_stage3_targets_zero" \
    --argjson svpp_stage4_targets_zero "$svpp_stage4_targets_zero" \
    --argjson svpp_stage3_rules_full "$svpp_stage3_rules_full" \
    --argjson svpp_stage4_rules_full "$svpp_stage4_rules_full" \
    --argjson svpp_stage3_branches_full "$svpp_stage3_branches_full" \
    --argjson svpp_stage4_branches_full "$svpp_stage4_branches_full" \
    --argjson svpp_unmet "$svpp_unmet_json" \
    '
    {
      gate: "sv_parser_family_status_gate",
      version: 1,
      generated_at_utc: $generated_at_utc,
      live_tracker_file: $live_tracker_file,
      status_rule_done: "Done requires a formally exhaustive, machine-checkable closure surface with no remaining parser rejection debt and no remaining coverage/gap debt for the family claim.",
      families: [
        {
          family: "systemverilog",
          computed_status: $sv_status,
          live_tracker_status: $sv_tracker_status,
          proof_surfaces: {
            syntax_closure_summary_json: $sv_syntax_summary_json,
            parser_aggregate_summary_txt: $sv_parser_summary_txt
          },
          criteria: {
            syntax_closure_gate_green: $sv_syntax_closure_gate_green,
            parser_aggregate_contract_green: true,
            generation_parser_rejections_zero: $sv_generation_parser_rejections_zero,
            replay_shadow_parser_rejections_zero: $sv_shadow_parser_rejections_zero,
            focused_replay_target_debt_zero: $sv_focused_replay_target_debt_zero
          },
          metrics: {
            syntax_closure_status: $sv_syntax_status,
            syntax_closure_failure_count: ($sv_syntax_failure_count | tonumber),
            syntax_defined_rule_count: ($sv_syntax_defined_rule_count | tonumber),
            syntax_unresolved_rule_reference_count: ($sv_syntax_unresolved_rule_reference_count | tonumber),
            syntax_unreachable_rules: ($sv_syntax_unreachable_rules | tonumber),
            syntax_unreachable_branches: ($sv_syntax_unreachable_branches | tonumber),
            syntax_target_debt_count: ($sv_syntax_target_debt_count | tonumber),
            generation_parser_rejections_total: ($sv_generation_parser_rejections_total | tonumber),
            replay_shadow_parser_rejections_total: ($sv_shadow_parser_rejections_total | tonumber),
            focused_replay_target_count: ($sv_focused_replay_target_count | tonumber),
            focused_replay_covered_reachable_rules: ($sv_focused_replay_covered_reachable_rules | tonumber),
            focused_replay_covered_reachable_branches: ($sv_focused_replay_covered_reachable_branches | tonumber),
            replay_gap_target_primary_rule: $sv_replay_gap_target_primary_rule
          },
          unmet_closure_criteria: $sv_unmet
        },
        {
          family: "systemverilog_preprocessor",
          computed_status: $svpp_status,
          live_tracker_status: $svpp_tracker_status,
          proof_surfaces: {
            aggregate_summary_txt: $svpp_aggregate_summary_txt,
            reachability_summary_txt: $svpp_reachability_summary_txt
          },
          criteria: {
            aggregate_contract_green: true,
            reachability_closure_green: true,
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
          unmet_closure_criteria: $svpp_unmet
        }
      ]
    }
    ' >"$SUMMARY_JSON"
require_nonempty_file "$SUMMARY_JSON"

{
    echo "SV Parser Family Status Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "live_tracker_file: $LIVE_TRACKER_FILE"
    echo "summary_json: $SUMMARY_JSON"
    echo "systemverilog_status: $sv_status"
    echo "systemverilog_tracker_status: $live_tracker_sv_status"
    echo "systemverilog_syntax_closure_status: $sv_syntax_status"
    echo "systemverilog_syntax_closure_failure_count: $sv_syntax_failure_count"
    echo "systemverilog_syntax_defined_rule_count: $sv_syntax_defined_rule_count"
    echo "systemverilog_syntax_unresolved_rule_reference_count: $sv_syntax_unresolved_rule_reference_count"
    echo "systemverilog_syntax_unreachable_rules: $sv_syntax_unreachable_rules"
    echo "systemverilog_syntax_unreachable_branches: $sv_syntax_unreachable_branches"
    echo "systemverilog_syntax_target_debt_count: $sv_syntax_target_debt_count"
    echo "systemverilog_generation_parser_rejections_total: $sv_generation_parser_rejections_total"
    echo "systemverilog_replay_shadow_parser_rejections_total: $sv_shadow_parser_rejections_total"
    echo "systemverilog_focused_replay_target_count: $sv_focused_replay_target_count"
    echo "systemverilog_replay_gap_target_primary_rule: $sv_replay_gap_target_primary_rule"
    echo "systemverilog_unmet_closure_criteria_count: ${#sv_unmet[@]}"
    for idx in "${!sv_unmet[@]}"; do
        echo "systemverilog_unmet_closure_criterion[$idx]: ${sv_unmet[$idx]}"
    done
    echo "systemverilog_preprocessor_status: $svpp_status"
    echo "systemverilog_preprocessor_tracker_status: $live_tracker_svpp_status"
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
    echo "systemverilog_preprocessor_unmet_closure_criteria_count: ${#svpp_unmet[@]}"
    for idx in "${!svpp_unmet[@]}"; do
        echo "systemverilog_preprocessor_unmet_closure_criterion[$idx]: ${svpp_unmet[$idx]}"
    done
} | tee "$SUMMARY_TXT"

echo "✅ SV parser-family status gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
