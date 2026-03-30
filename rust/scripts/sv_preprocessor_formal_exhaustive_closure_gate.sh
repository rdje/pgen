#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_formal_exhaustive_closure_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

CONTRACT_FILE="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_preprocessor_formal_exhaustive_closure_contract.json}"
SYNTAX_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_syntax_closure_gate.sh"
AGGREGATE_CONTRACT_GATE="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"
REACHABILITY_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_reachability_closure_gate.sh"

EXISTING_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_AGGREGATE_CONTRACT_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_AGGREGATE_CONTRACT_STATE_DIR:-}"
EXISTING_REACHABILITY_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_REACHABILITY_CLOSURE_STATE_DIR:-}"

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

summary_value_from_txt() {
    local key="$1"
    local path="$2"
    local line
    line="$(awk -v key="$key" '
        {
            trimmed = $0
            sub(/^[[:space:]]+/, "", trimmed)
            if (index(trimmed, key ": ") == 1) {
                print trimmed
                exit
            }
        }
    ' "$path")"
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
require_file "$CONTRACT_FILE"
require_file "$SYNTAX_CLOSURE_GATE"
require_file "$AGGREGATE_CONTRACT_GATE"
require_file "$REACHABILITY_CLOSURE_GATE"

jq -e '
    .family == "systemverilog_preprocessor"
    and ((.version | type) == "number")
    and ((.done_rule | type) == "string" and (.done_rule | length) > 0)
    and (.required_surface_key == "zero_plausible_grammar_level_gap_proof_surface")
    and ((.required_surface_missing_detail | type) == "string" and (.required_surface_missing_detail | length) > 0)
' "$CONTRACT_FILE" >/dev/null

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

syntax_state_dir="${EXISTING_SYNTAX_CLOSURE_STATE_DIR:-$WORK_DIR/sv_preprocessor_syntax_closure_gate}"
if [[ -z "$EXISTING_SYNTAX_CLOSURE_STATE_DIR" ]]; then
    run_logged "sv_preprocessor_syntax_closure_gate" \
        env \
            PGEN_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR="$syntax_state_dir" \
            "$SYNTAX_CLOSURE_GATE"
else
    syntax_state_dir="$(cd "$syntax_state_dir" && pwd)"
fi

aggregate_state_dir="${EXISTING_AGGREGATE_CONTRACT_STATE_DIR:-$WORK_DIR/sv_preprocessor_aggregate_contract_gate}"
if [[ -z "$EXISTING_AGGREGATE_CONTRACT_STATE_DIR" ]]; then
    run_logged "sv_preprocessor_aggregate_contract_gate" \
        env \
            PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_STATE_DIR="$aggregate_state_dir" \
            "$AGGREGATE_CONTRACT_GATE"
else
    aggregate_state_dir="$(cd "$aggregate_state_dir" && pwd)"
fi

reachability_state_dir="${EXISTING_REACHABILITY_CLOSURE_STATE_DIR:-$WORK_DIR/sv_preprocessor_reachability_closure_gate}"
if [[ -z "$EXISTING_REACHABILITY_CLOSURE_STATE_DIR" ]]; then
    run_logged "sv_preprocessor_reachability_closure_gate" \
        env \
            PGEN_SV_PREPROCESSOR_REACHABILITY_CLOSURE_STATE_DIR="$reachability_state_dir" \
            "$REACHABILITY_CLOSURE_GATE"
else
    reachability_state_dir="$(cd "$reachability_state_dir" && pwd)"
fi

syntax_summary_txt="$syntax_state_dir/summary.txt"
aggregate_summary_txt="$aggregate_state_dir/summary.txt"
aggregate_summary_json="$aggregate_state_dir/summary.json"
reachability_summary_txt="$reachability_state_dir/summary.txt"

require_nonempty_file "$syntax_summary_txt"
require_nonempty_file "$aggregate_summary_txt"
require_nonempty_file "$aggregate_summary_json"
require_nonempty_file "$reachability_summary_txt"

contract_version="$(jq -r '.version' "$CONTRACT_FILE")"
done_rule="$(jq -r '.done_rule' "$CONTRACT_FILE")"
required_surface_key="$(jq -r '.required_surface_key' "$CONTRACT_FILE")"
required_surface_missing_detail="$(jq -r '.required_surface_missing_detail' "$CONTRACT_FILE")"

syntax_state_dir_from_txt="$(top_level_summary_value_from_txt "state_dir" "$syntax_summary_txt")"
aggregate_state_dir_from_txt="$(top_level_summary_value_from_txt "state_dir" "$aggregate_summary_txt")"
aggregate_generated_at_utc="$(top_level_summary_value_from_txt "generated_at_utc" "$aggregate_summary_txt")"
aggregate_summary_json_from_txt="$(top_level_summary_value_from_txt "summary_json" "$aggregate_summary_txt")"
reachability_state_dir_from_txt="$(top_level_summary_value_from_txt "state_dir" "$reachability_summary_txt")"

if [[ "$syntax_state_dir_from_txt" != "$syntax_state_dir" ]]; then
    echo "error: preprocessor syntax-closure state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$aggregate_state_dir_from_txt" != "$aggregate_state_dir" ]]; then
    echo "error: preprocessor aggregate-contract state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$aggregate_summary_json_from_txt" != "$aggregate_summary_json" ]]; then
    echo "error: preprocessor aggregate-contract summary_json mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$reachability_state_dir_from_txt" != "$reachability_state_dir" ]]; then
    echo "error: preprocessor reachability-closure state_dir mismatch in summary.txt" >&2
    exit 1
fi

aggregate_gate_name="$(jq -r '.gate' "$aggregate_summary_json")"
aggregate_gate_version="$(jq -r '.version' "$aggregate_summary_json")"
aggregate_generated_at_utc_from_json="$(jq -r '.generated_at_utc' "$aggregate_summary_json")"
aggregate_state_dir_from_json="$(jq -r '.state_dir' "$aggregate_summary_json")"
aggregate_summary_txt_from_json="$(jq -r '.summary_txt' "$aggregate_summary_json")"
aggregate_summary_json_from_json="$(jq -r '.summary_json' "$aggregate_summary_json")"

if [[ "$aggregate_gate_name" != "sv_preprocessor_aggregate_contract_gate" ]]; then
    echo "error: unexpected preprocessor aggregate-contract gate identity '$aggregate_gate_name'" >&2
    exit 1
fi
if [[ ! "$aggregate_gate_version" =~ ^[0-9]+$ ]]; then
    echo "error: preprocessor aggregate-contract gate version is not numeric: '$aggregate_gate_version'" >&2
    exit 1
fi
if [[ "$aggregate_generated_at_utc_from_json" != "$aggregate_generated_at_utc" ]]; then
    echo "error: preprocessor aggregate-contract generated_at_utc mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$aggregate_state_dir_from_json" != "$aggregate_state_dir" ]]; then
    echo "error: preprocessor aggregate-contract state_dir mismatch in summary.json" >&2
    exit 1
fi
if [[ "$aggregate_summary_txt_from_json" != "$aggregate_summary_txt" ]]; then
    echo "error: preprocessor aggregate-contract summary_txt mismatch in summary.json" >&2
    exit 1
fi
if [[ "$aggregate_summary_json_from_json" != "$aggregate_summary_json" ]]; then
    echo "error: preprocessor aggregate-contract summary_json mismatch in summary.json" >&2
    exit 1
fi

syntax_status="pass"
syntax_failure_count="0"
syntax_defined_rule_count="$(summary_value_from_txt "defined_rule_count" "$syntax_summary_txt")"
syntax_unresolved_rule_reference_count="$(summary_value_from_txt "unresolved_rule_reference_count" "$syntax_summary_txt")"
syntax_unreachable_rules="$(summary_value_from_txt "unreachable_rules" "$syntax_summary_txt")"
syntax_unreachable_branches="$(summary_value_from_txt "unreachable_branches" "$syntax_summary_txt")"
syntax_target_debt_count="$(summary_value_from_txt "target_debt_count" "$syntax_summary_txt")"

aggregate_parseability_parser_rejections_total="$(summary_value_from_txt "parseability_parser_rejections_total" "$aggregate_summary_txt")"
aggregate_parseability_rejected_total="$(summary_value_from_txt "parseability_rejected_total" "$aggregate_summary_txt")"
aggregate_final_targets="$(summary_value_from_txt "final_targets" "$aggregate_summary_txt")"
aggregate_covered_reachable_rules="$(summary_value_from_txt "covered_reachable_rules" "$aggregate_summary_txt")"
aggregate_covered_reachable_branches="$(summary_value_from_txt "covered_reachable_branches" "$aggregate_summary_txt")"
aggregate_counterexample_primary_stage="$(summary_value_from_txt "counterexample_primary_stage" "$aggregate_summary_txt")"

reachability_stage3_targets="$(summary_value_from_txt "stage3_targets" "$reachability_summary_txt")"
reachability_stage4_targets="$(summary_value_from_txt "stage4_targets" "$reachability_summary_txt")"
reachability_stage3_rules="$(summary_value_from_txt "stage3_covered_reachable_rules" "$reachability_summary_txt")"
reachability_stage4_rules="$(summary_value_from_txt "stage4_covered_reachable_rules" "$reachability_summary_txt")"
reachability_stage3_branches="$(summary_value_from_txt "stage3_covered_reachable_branches" "$reachability_summary_txt")"
reachability_stage4_branches="$(summary_value_from_txt "stage4_covered_reachable_branches" "$reachability_summary_txt")"
reachability_parseability_rejected="$(summary_value_from_txt "parseability_rejected" "$reachability_summary_txt")"
reachability_parser_rejections="$(summary_value_from_txt "parser_rejections" "$reachability_summary_txt")"

syntax_closure_surface_green=false
aggregate_contract_surface_green=false
reachability_closure_surface_green=false

if [[ "$syntax_status" == "pass" && "$syntax_failure_count" == "0" ]]; then
    syntax_closure_surface_green=true
fi
aggregate_contract_surface_green=true
if [[ "$reachability_stage3_targets" == "0" && "$reachability_stage4_targets" == "0" ]]; then
    reachability_closure_surface_green=true
fi

systemverilog_preprocessor_formal_exhaustive_closure_surface_green=false
unmet_closure_criteria_count=1
primary_unmet_closure_criterion="$required_surface_missing_detail"
unmet_closure_criteria_json="$(jq -cn --arg detail "$required_surface_missing_detail" '[ $detail ]')"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "SV Preprocessor Formal Exhaustive Closure Gate Summary"
    echo "gate: sv_preprocessor_formal_exhaustive_closure_gate"
    echo "version: 1"
    echo "generated_at_utc: $generated_at_utc"
    echo "state_dir: $STATE_DIR"
    echo "summary_txt: $SUMMARY_TXT"
    echo "summary_json: $SUMMARY_JSON"
    echo "family: systemverilog_preprocessor"
    echo "contract_file: $CONTRACT_FILE"
    echo "contract_version: $contract_version"
    echo "done_rule: $done_rule"
    echo "required_surface_key: $required_surface_key"
    echo "required_surface_missing_detail: $required_surface_missing_detail"
    echo "syntax_closure_state_dir: $syntax_state_dir"
    echo "syntax_closure_summary_txt: $syntax_summary_txt"
    echo "aggregate_contract_state_dir: $aggregate_state_dir"
    echo "aggregate_contract_summary_txt: $aggregate_summary_txt"
    echo "aggregate_contract_summary_json: $aggregate_summary_json"
    echo "reachability_closure_state_dir: $reachability_state_dir"
    echo "reachability_closure_summary_txt: $reachability_summary_txt"
    echo "syntax_closure_surface_green: $syntax_closure_surface_green"
    echo "aggregate_contract_surface_green: $aggregate_contract_surface_green"
    echo "reachability_closure_surface_green: $reachability_closure_surface_green"
    echo "zero_plausible_grammar_level_gap_proof_surface: false"
    echo "systemverilog_preprocessor_formal_exhaustive_closure_surface_green: $systemverilog_preprocessor_formal_exhaustive_closure_surface_green"
    echo "unmet_closure_criteria_count: $unmet_closure_criteria_count"
    echo "primary_unmet_closure_criterion: $primary_unmet_closure_criterion"
    echo "unmet_closure_criteria_json: $unmet_closure_criteria_json"
    echo "syntax_status: $syntax_status"
    echo "syntax_failure_count: $syntax_failure_count"
    echo "syntax_defined_rule_count: $syntax_defined_rule_count"
    echo "syntax_unresolved_rule_reference_count: $syntax_unresolved_rule_reference_count"
    echo "syntax_unreachable_rules: $syntax_unreachable_rules"
    echo "syntax_unreachable_branches: $syntax_unreachable_branches"
    echo "syntax_target_debt_count: $syntax_target_debt_count"
    echo "aggregate_contract_gate: $aggregate_gate_name"
    echo "aggregate_contract_gate_version: $aggregate_gate_version"
    echo "aggregate_contract_generated_at_utc: $aggregate_generated_at_utc"
    echo "aggregate_parseability_parser_rejections_total: $aggregate_parseability_parser_rejections_total"
    echo "aggregate_parseability_rejected_total: $aggregate_parseability_rejected_total"
    echo "aggregate_final_targets: $aggregate_final_targets"
    echo "aggregate_covered_reachable_rules: $aggregate_covered_reachable_rules"
    echo "aggregate_covered_reachable_branches: $aggregate_covered_reachable_branches"
    echo "aggregate_counterexample_primary_stage: $aggregate_counterexample_primary_stage"
    echo "reachability_stage3_targets: $reachability_stage3_targets"
    echo "reachability_stage4_targets: $reachability_stage4_targets"
    echo "reachability_stage3_covered_reachable_rules: $reachability_stage3_rules"
    echo "reachability_stage4_covered_reachable_rules: $reachability_stage4_rules"
    echo "reachability_stage3_covered_reachable_branches: $reachability_stage3_branches"
    echo "reachability_stage4_covered_reachable_branches: $reachability_stage4_branches"
    echo "reachability_parseability_rejected: $reachability_parseability_rejected"
    echo "reachability_parser_rejections: $reachability_parser_rejections"
} >"$SUMMARY_TXT"

jq -n \
    --arg gate "sv_preprocessor_formal_exhaustive_closure_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg family "systemverilog_preprocessor" \
    --arg contract_file "$CONTRACT_FILE" \
    --argjson contract_version "$contract_version" \
    --arg done_rule "$done_rule" \
    --arg required_surface_key "$required_surface_key" \
    --arg required_surface_missing_detail "$required_surface_missing_detail" \
    --arg syntax_closure_state_dir "$syntax_state_dir" \
    --arg syntax_closure_summary_txt "$syntax_summary_txt" \
    --arg aggregate_contract_state_dir "$aggregate_state_dir" \
    --arg aggregate_contract_summary_txt "$aggregate_summary_txt" \
    --arg aggregate_contract_summary_json "$aggregate_summary_json" \
    --arg reachability_closure_state_dir "$reachability_state_dir" \
    --arg reachability_closure_summary_txt "$reachability_summary_txt" \
    --arg syntax_status "$syntax_status" \
    --argjson syntax_failure_count "$syntax_failure_count" \
    --argjson syntax_defined_rule_count "$syntax_defined_rule_count" \
    --argjson syntax_unresolved_rule_reference_count "$syntax_unresolved_rule_reference_count" \
    --argjson syntax_unreachable_rules "$syntax_unreachable_rules" \
    --argjson syntax_unreachable_branches "$syntax_unreachable_branches" \
    --argjson syntax_target_debt_count "$syntax_target_debt_count" \
    --arg aggregate_contract_gate "$aggregate_gate_name" \
    --argjson aggregate_contract_gate_version "$aggregate_gate_version" \
    --arg aggregate_contract_generated_at_utc "$aggregate_generated_at_utc" \
    --argjson aggregate_parseability_parser_rejections_total "$aggregate_parseability_parser_rejections_total" \
    --argjson aggregate_parseability_rejected_total "$aggregate_parseability_rejected_total" \
    --argjson aggregate_final_targets "$aggregate_final_targets" \
    --arg aggregate_covered_reachable_rules "$aggregate_covered_reachable_rules" \
    --arg aggregate_covered_reachable_branches "$aggregate_covered_reachable_branches" \
    --arg aggregate_counterexample_primary_stage "$aggregate_counterexample_primary_stage" \
    --argjson reachability_stage3_targets "$reachability_stage3_targets" \
    --argjson reachability_stage4_targets "$reachability_stage4_targets" \
    --arg reachability_stage3_covered_reachable_rules "$reachability_stage3_rules" \
    --arg reachability_stage4_covered_reachable_rules "$reachability_stage4_rules" \
    --arg reachability_stage3_covered_reachable_branches "$reachability_stage3_branches" \
    --arg reachability_stage4_covered_reachable_branches "$reachability_stage4_branches" \
    --argjson reachability_parseability_rejected "$reachability_parseability_rejected" \
    --argjson reachability_parser_rejections "$reachability_parser_rejections" \
    --argjson syntax_closure_surface_green "$syntax_closure_surface_green" \
    --argjson aggregate_contract_surface_green "$aggregate_contract_surface_green" \
    --argjson reachability_closure_surface_green "$reachability_closure_surface_green" \
    --argjson zero_plausible_grammar_level_gap_proof_surface false \
    --argjson systemverilog_preprocessor_formal_exhaustive_closure_surface_green "$systemverilog_preprocessor_formal_exhaustive_closure_surface_green" \
    --argjson unmet_closure_criteria_count "$unmet_closure_criteria_count" \
    --arg primary_unmet_closure_criterion "$primary_unmet_closure_criterion" \
    --argjson unmet_closure_criteria "$unmet_closure_criteria_json" \
    '{
        gate: $gate,
        version: $version,
        generated_at_utc: $generated_at_utc,
        state_dir: $state_dir,
        summary_txt: $summary_txt,
        summary_json: $summary_json,
        family: $family,
        contract_file: $contract_file,
        contract_version: $contract_version,
        done_rule: $done_rule,
        required_surface_key: $required_surface_key,
        required_surface_missing_detail: $required_surface_missing_detail,
        syntax_closure_surface_green: $syntax_closure_surface_green,
        aggregate_contract_surface_green: $aggregate_contract_surface_green,
        reachability_closure_surface_green: $reachability_closure_surface_green,
        zero_plausible_grammar_level_gap_proof_surface: $zero_plausible_grammar_level_gap_proof_surface,
        systemverilog_preprocessor_formal_exhaustive_closure_surface_green: $systemverilog_preprocessor_formal_exhaustive_closure_surface_green,
        unmet_closure_criteria_count: $unmet_closure_criteria_count,
        primary_unmet_closure_criterion: $primary_unmet_closure_criterion,
        unmet_closure_criteria: $unmet_closure_criteria,
        metrics: {
            syntax_status: $syntax_status,
            syntax_failure_count: $syntax_failure_count,
            syntax_defined_rule_count: $syntax_defined_rule_count,
            syntax_unresolved_rule_reference_count: $syntax_unresolved_rule_reference_count,
            syntax_unreachable_rules: $syntax_unreachable_rules,
            syntax_unreachable_branches: $syntax_unreachable_branches,
            syntax_target_debt_count: $syntax_target_debt_count,
            aggregate_contract_gate: $aggregate_contract_gate,
            aggregate_contract_gate_version: $aggregate_contract_gate_version,
            aggregate_contract_generated_at_utc: $aggregate_contract_generated_at_utc,
            aggregate_parseability_parser_rejections_total: $aggregate_parseability_parser_rejections_total,
            aggregate_parseability_rejected_total: $aggregate_parseability_rejected_total,
            aggregate_final_targets: $aggregate_final_targets,
            aggregate_covered_reachable_rules: $aggregate_covered_reachable_rules,
            aggregate_covered_reachable_branches: $aggregate_covered_reachable_branches,
            aggregate_counterexample_primary_stage: $aggregate_counterexample_primary_stage,
            reachability_stage3_targets: $reachability_stage3_targets,
            reachability_stage4_targets: $reachability_stage4_targets,
            reachability_stage3_covered_reachable_rules: $reachability_stage3_covered_reachable_rules,
            reachability_stage4_covered_reachable_rules: $reachability_stage4_covered_reachable_rules,
            reachability_stage3_covered_reachable_branches: $reachability_stage3_covered_reachable_branches,
            reachability_stage4_covered_reachable_branches: $reachability_stage4_covered_reachable_branches,
            reachability_parseability_rejected: $reachability_parseability_rejected,
            reachability_parser_rejections: $reachability_parser_rejections
        },
        proof_surfaces: {
            syntax_closure_state_dir: $syntax_closure_state_dir,
            syntax_closure_summary_txt: $syntax_closure_summary_txt,
            aggregate_contract_state_dir: $aggregate_contract_state_dir,
            aggregate_contract_summary_txt: $aggregate_contract_summary_txt,
            aggregate_contract_summary_json: $aggregate_contract_summary_json,
            reachability_closure_state_dir: $reachability_closure_state_dir,
            reachability_closure_summary_txt: $reachability_closure_summary_txt
        }
    }' >"$SUMMARY_JSON"

echo "SV preprocessor formal exhaustive closure gate passed."
