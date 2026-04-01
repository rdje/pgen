#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_zero_plausible_gap_proof_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

CONTRACT_FILE="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_preprocessor_zero_plausible_gap_proof_contract.json}"
SYNTAX_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_syntax_closure_gate.sh"
AGGREGATE_CONTRACT_GATE="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"
REACHABILITY_CLOSURE_GATE="$RUST_DIR/scripts/sv_preprocessor_reachability_closure_gate.sh"

EXISTING_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_EXISTING_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_AGGREGATE_CONTRACT_STATE_DIR="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_EXISTING_AGGREGATE_CONTRACT_STATE_DIR:-}"
EXISTING_REACHABILITY_CLOSURE_STATE_DIR="${PGEN_SV_PREPROCESSOR_ZERO_PLAUSIBLE_GAP_PROOF_EXISTING_REACHABILITY_CLOSURE_STATE_DIR:-}"

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

fraction_is_full() {
    local value="$1"
    if [[ "$value" =~ ^([0-9]+)/([0-9]+)$ ]]; then
        [[ "${BASH_REMATCH[1]}" == "${BASH_REMATCH[2]}" ]]
        return
    fi
    return 1
}

require_tool jq
require_file "$CONTRACT_FILE"
require_file "$SYNTAX_CLOSURE_GATE"
require_file "$AGGREGATE_CONTRACT_GATE"
require_file "$REACHABILITY_CLOSURE_GATE"

jq -e '
    .family == "systemverilog_preprocessor"
    and ((.version | type) == "number")
    and ((.description | type) == "string" and (.description | length) > 0)
    and ((.grammar_name | type) == "string" and (.grammar_name | length) > 0)
    and ((.entry_rule | type) == "string" and (.entry_rule | length) > 0)
    and ((.done_rule | type) == "string" and (.done_rule | length) > 0)
    and ((.allowed_unreachable_rules | type) == "array" and (.allowed_unreachable_rules | length) > 0)
    and ((.allowed_unreachable_branches | type) == "array" and (.allowed_unreachable_branches | length) > 0)
    and ((.required_unreachable_rule_reason | type) == "string" and (.required_unreachable_rule_reason | length) > 0)
    and ((.required_unreachable_branch_rule_name | type) == "string" and (.required_unreachable_branch_rule_name | length) > 0)
    and ((.required_unreachable_branch_reason | type) == "string" and (.required_unreachable_branch_reason | length) > 0)
    and ((.helper_only_whitelist_detail | type) == "string" and (.helper_only_whitelist_detail | length) > 0)
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
syntax_gap_json="$syntax_state_dir/work/systemverilog_preprocessor_syntax_probe_gap.json"
aggregate_summary_txt="$aggregate_state_dir/summary.txt"
reachability_summary_txt="$reachability_state_dir/summary.txt"

require_nonempty_file "$syntax_summary_txt"
require_nonempty_file "$syntax_gap_json"
require_nonempty_file "$aggregate_summary_txt"
require_nonempty_file "$reachability_summary_txt"

contract_version="$(jq -r '.version' "$CONTRACT_FILE")"
family="$(jq -r '.family' "$CONTRACT_FILE")"
grammar_name="$(jq -r '.grammar_name' "$CONTRACT_FILE")"
entry_rule="$(jq -r '.entry_rule' "$CONTRACT_FILE")"
done_rule="$(jq -r '.done_rule' "$CONTRACT_FILE")"
helper_only_whitelist_detail="$(jq -r '.helper_only_whitelist_detail' "$CONTRACT_FILE")"
required_unreachable_rule_reason="$(jq -r '.required_unreachable_rule_reason' "$CONTRACT_FILE")"
required_unreachable_branch_rule_name="$(jq -r '.required_unreachable_branch_rule_name' "$CONTRACT_FILE")"
required_unreachable_branch_reason="$(jq -r '.required_unreachable_branch_reason' "$CONTRACT_FILE")"
allowed_unreachable_rules_json="$(jq -cer '.allowed_unreachable_rules | sort' "$CONTRACT_FILE")"
allowed_unreachable_branches_json="$(jq -cer '.allowed_unreachable_branches | sort' "$CONTRACT_FILE")"

syntax_summary_state_dir="$(top_level_summary_value_from_txt "state_dir" "$syntax_summary_txt")"
aggregate_summary_state_dir="$(top_level_summary_value_from_txt "state_dir" "$aggregate_summary_txt")"
reachability_summary_state_dir="$(top_level_summary_value_from_txt "state_dir" "$reachability_summary_txt")"

if [[ "$syntax_summary_state_dir" != "$syntax_state_dir" ]]; then
    echo "error: preprocessor syntax-closure state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$aggregate_summary_state_dir" != "$aggregate_state_dir" ]]; then
    echo "error: preprocessor aggregate-contract state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$reachability_summary_state_dir" != "$reachability_state_dir" ]]; then
    echo "error: preprocessor reachability-closure state_dir mismatch in summary.txt" >&2
    exit 1
fi

jq -e --arg grammar_name "$grammar_name" --arg entry_rule "$entry_rule" '
    .grammar_name == $grammar_name
    and .entry_rule == $entry_rule
    and (.summary | type == "object")
    and (.unreachable_rule_debt | type == "array")
    and (.unreachable_branch_debt | type == "array")
' "$syntax_gap_json" >/dev/null

syntax_status="pass"
syntax_failure_count="0"
syntax_unresolved_rule_reference_count="$(summary_value_from_txt "unresolved_rule_reference_count" "$syntax_summary_txt")"

aggregate_parseability_parser_rejections_total="$(summary_value_from_txt "parseability_parser_rejections_total" "$aggregate_summary_txt")"
aggregate_parseability_rejected_total="$(summary_value_from_txt "parseability_rejected_total" "$aggregate_summary_txt")"
aggregate_final_targets="$(summary_value_from_txt "final_targets" "$aggregate_summary_txt")"
aggregate_covered_reachable_rules="$(summary_value_from_txt "covered_reachable_rules" "$aggregate_summary_txt")"
aggregate_covered_reachable_branches="$(summary_value_from_txt "covered_reachable_branches" "$aggregate_summary_txt")"

reachability_stage3_targets="$(summary_value_from_txt "stage3_targets" "$reachability_summary_txt")"
reachability_stage4_targets="$(summary_value_from_txt "stage4_targets" "$reachability_summary_txt")"
reachability_stage3_rules="$(summary_value_from_txt "stage3_covered_reachable_rules" "$reachability_summary_txt")"
reachability_stage4_rules="$(summary_value_from_txt "stage4_covered_reachable_rules" "$reachability_summary_txt")"
reachability_stage3_branches="$(summary_value_from_txt "stage3_covered_reachable_branches" "$reachability_summary_txt")"
reachability_stage4_branches="$(summary_value_from_txt "stage4_covered_reachable_branches" "$reachability_summary_txt")"

observed_unreachable_rules_json="$(jq -cer '.unreachable_rule_debt | map(.rule_name) | sort' "$syntax_gap_json")"
observed_unreachable_branches_json="$(jq -cer '.unreachable_branch_debt | map(.branch_id) | sort' "$syntax_gap_json")"

syntax_preconditions_green=false
aggregate_preconditions_green=false
reachability_preconditions_green=false
helper_only_unreachable_rules_green=false
helper_only_unreachable_branches_green=false
helper_only_unreachable_surface_green=false

if [[ "$syntax_status" == "pass" && "$syntax_failure_count" == "0" && "$syntax_unresolved_rule_reference_count" == "0" ]]; then
    syntax_preconditions_green=true
fi

if [[ "$aggregate_parseability_parser_rejections_total" == "0" \
   && "$aggregate_parseability_rejected_total" == "0" \
   && "$aggregate_final_targets" == "0" ]] \
   && fraction_is_full "$aggregate_covered_reachable_rules" \
   && fraction_is_full "$aggregate_covered_reachable_branches"; then
    aggregate_preconditions_green=true
fi

if [[ "$reachability_stage3_targets" == "0" \
   && "$reachability_stage4_targets" == "0" ]] \
   && fraction_is_full "$reachability_stage3_rules" \
   && fraction_is_full "$reachability_stage4_rules" \
   && fraction_is_full "$reachability_stage3_branches" \
   && fraction_is_full "$reachability_stage4_branches"; then
    reachability_preconditions_green=true
fi

if [[ "$observed_unreachable_rules_json" == "$allowed_unreachable_rules_json" ]] \
   && jq -e --arg reason "$required_unreachable_rule_reason" '
        all(.unreachable_rule_debt[]?;
            .reachable == false
            and .reason == $reason
        )
   ' "$syntax_gap_json" >/dev/null; then
    helper_only_unreachable_rules_green=true
fi

if [[ "$observed_unreachable_branches_json" == "$allowed_unreachable_branches_json" ]] \
   && jq -e --arg rule_name "$required_unreachable_branch_rule_name" --arg reason "$required_unreachable_branch_reason" '
        all(.unreachable_branch_debt[]?;
            .reachable == false
            and .rule_name == $rule_name
            and .reason == $reason
        )
   ' "$syntax_gap_json" >/dev/null; then
    helper_only_unreachable_branches_green=true
fi

if [[ "$helper_only_unreachable_rules_green" == true && "$helper_only_unreachable_branches_green" == true ]]; then
    helper_only_unreachable_surface_green=true
fi

zero_plausible_grammar_level_gap_proof_surface=false
declare -a unmet_proof_criteria=()

if [[ "$syntax_preconditions_green" != true ]]; then
    unmet_proof_criteria+=("Syntax preconditions regressed: syntax_status=${syntax_status}, syntax_failure_count=${syntax_failure_count}, syntax_unresolved_rule_reference_count=${syntax_unresolved_rule_reference_count}.")
fi
if [[ "$aggregate_preconditions_green" != true ]]; then
    unmet_proof_criteria+=("Aggregate preconditions regressed: parseability_parser_rejections_total=${aggregate_parseability_parser_rejections_total}, parseability_rejected_total=${aggregate_parseability_rejected_total}, final_targets=${aggregate_final_targets}, covered_reachable_rules=${aggregate_covered_reachable_rules}, covered_reachable_branches=${aggregate_covered_reachable_branches}.")
fi
if [[ "$reachability_preconditions_green" != true ]]; then
    unmet_proof_criteria+=("Reachability preconditions regressed: stage3_targets=${reachability_stage3_targets}, stage4_targets=${reachability_stage4_targets}, stage3_rules=${reachability_stage3_rules}, stage4_rules=${reachability_stage4_rules}, stage3_branches=${reachability_stage3_branches}, stage4_branches=${reachability_stage4_branches}.")
fi
if [[ "$helper_only_unreachable_surface_green" != true ]]; then
    unmet_proof_criteria+=("$helper_only_whitelist_detail")
fi

if [[ "${#unmet_proof_criteria[@]}" -eq 0 ]]; then
    zero_plausible_grammar_level_gap_proof_surface=true
fi

unmet_proof_criteria_count="${#unmet_proof_criteria[@]}"
primary_unmet_proof_criterion="<none>"
if [[ "$unmet_proof_criteria_count" -gt 0 ]]; then
    primary_unmet_proof_criterion="${unmet_proof_criteria[0]}"
fi
unmet_proof_criteria_json="$(jq -cn '$ARGS.positional' --args "${unmet_proof_criteria[@]}")"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "SV Preprocessor Zero Plausible Gap Proof Gate Summary"
    echo "gate: sv_preprocessor_zero_plausible_gap_proof_gate"
    echo "version: 1"
    echo "generated_at_utc: $generated_at_utc"
    echo "state_dir: $STATE_DIR"
    echo "summary_txt: $SUMMARY_TXT"
    echo "summary_json: $SUMMARY_JSON"
    echo "family: $family"
    echo "contract_file: $CONTRACT_FILE"
    echo "contract_version: $contract_version"
    echo "grammar_name: $grammar_name"
    echo "entry_rule: $entry_rule"
    echo "done_rule: $done_rule"
    echo "syntax_closure_state_dir: $syntax_state_dir"
    echo "syntax_closure_summary_txt: $syntax_summary_txt"
    echo "syntax_gap_json: $syntax_gap_json"
    echo "aggregate_contract_state_dir: $aggregate_state_dir"
    echo "aggregate_contract_summary_txt: $aggregate_summary_txt"
    echo "reachability_closure_state_dir: $reachability_state_dir"
    echo "reachability_closure_summary_txt: $reachability_summary_txt"
    echo "syntax_preconditions_green: $syntax_preconditions_green"
    echo "aggregate_preconditions_green: $aggregate_preconditions_green"
    echo "reachability_preconditions_green: $reachability_preconditions_green"
    echo "helper_only_unreachable_rules_green: $helper_only_unreachable_rules_green"
    echo "helper_only_unreachable_branches_green: $helper_only_unreachable_branches_green"
    echo "helper_only_unreachable_surface_green: $helper_only_unreachable_surface_green"
    echo "zero_plausible_grammar_level_gap_proof_surface: $zero_plausible_grammar_level_gap_proof_surface"
    echo "unmet_proof_criteria_count: $unmet_proof_criteria_count"
    echo "primary_unmet_proof_criterion: $primary_unmet_proof_criterion"
    echo "unmet_proof_criteria_json: $unmet_proof_criteria_json"
    echo "allowed_unreachable_rules_json: $allowed_unreachable_rules_json"
    echo "observed_unreachable_rules_json: $observed_unreachable_rules_json"
    echo "allowed_unreachable_branches_json: $allowed_unreachable_branches_json"
    echo "observed_unreachable_branches_json: $observed_unreachable_branches_json"
} >"$SUMMARY_TXT"

jq -n \
    --arg gate "sv_preprocessor_zero_plausible_gap_proof_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg family "$family" \
    --arg contract_file "$CONTRACT_FILE" \
    --argjson contract_version "$contract_version" \
    --arg grammar_name "$grammar_name" \
    --arg entry_rule "$entry_rule" \
    --arg done_rule "$done_rule" \
    --arg syntax_closure_state_dir "$syntax_state_dir" \
    --arg syntax_closure_summary_txt "$syntax_summary_txt" \
    --arg syntax_gap_json "$syntax_gap_json" \
    --arg aggregate_contract_state_dir "$aggregate_state_dir" \
    --arg aggregate_contract_summary_txt "$aggregate_summary_txt" \
    --arg reachability_closure_state_dir "$reachability_state_dir" \
    --arg reachability_closure_summary_txt "$reachability_summary_txt" \
    --argjson syntax_preconditions_green "$syntax_preconditions_green" \
    --argjson aggregate_preconditions_green "$aggregate_preconditions_green" \
    --argjson reachability_preconditions_green "$reachability_preconditions_green" \
    --argjson helper_only_unreachable_rules_green "$helper_only_unreachable_rules_green" \
    --argjson helper_only_unreachable_branches_green "$helper_only_unreachable_branches_green" \
    --argjson helper_only_unreachable_surface_green "$helper_only_unreachable_surface_green" \
    --argjson zero_plausible_grammar_level_gap_proof_surface "$zero_plausible_grammar_level_gap_proof_surface" \
    --argjson unmet_proof_criteria_count "$unmet_proof_criteria_count" \
    --arg primary_unmet_proof_criterion "$primary_unmet_proof_criterion" \
    --argjson unmet_proof_criteria "$unmet_proof_criteria_json" \
    --argjson allowed_unreachable_rules "$allowed_unreachable_rules_json" \
    --argjson observed_unreachable_rules "$observed_unreachable_rules_json" \
    --argjson allowed_unreachable_branches "$allowed_unreachable_branches_json" \
    --argjson observed_unreachable_branches "$observed_unreachable_branches_json" \
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
        grammar_name: $grammar_name,
        entry_rule: $entry_rule,
        done_rule: $done_rule,
        zero_plausible_grammar_level_gap_proof_surface: $zero_plausible_grammar_level_gap_proof_surface,
        unmet_proof_criteria_count: $unmet_proof_criteria_count,
        primary_unmet_proof_criterion: $primary_unmet_proof_criterion,
        unmet_proof_criteria: $unmet_proof_criteria,
        proof_surfaces: {
            syntax_closure_state_dir: $syntax_closure_state_dir,
            syntax_closure_summary_txt: $syntax_closure_summary_txt,
            syntax_gap_json: $syntax_gap_json,
            aggregate_contract_state_dir: $aggregate_contract_state_dir,
            aggregate_contract_summary_txt: $aggregate_contract_summary_txt,
            reachability_closure_state_dir: $reachability_closure_state_dir,
            reachability_closure_summary_txt: $reachability_closure_summary_txt
        },
        criteria: {
            syntax_preconditions_green: $syntax_preconditions_green,
            aggregate_preconditions_green: $aggregate_preconditions_green,
            reachability_preconditions_green: $reachability_preconditions_green,
            helper_only_unreachable_rules_green: $helper_only_unreachable_rules_green,
            helper_only_unreachable_branches_green: $helper_only_unreachable_branches_green,
            helper_only_unreachable_surface_green: $helper_only_unreachable_surface_green
        },
        expected: {
            allowed_unreachable_rules: $allowed_unreachable_rules,
            allowed_unreachable_branches: $allowed_unreachable_branches
        },
        observed: {
            unreachable_rules: $observed_unreachable_rules,
            unreachable_branches: $observed_unreachable_branches
        }
    }' >"$SUMMARY_JSON"

echo "SV preprocessor zero-plausible-gap proof gate completed."
