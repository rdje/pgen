#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_parser_family_status_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

SV_PARSER_FAMILY_STATUS_GATE="$RUST_DIR/scripts/sv_parser_family_status_gate.sh"
EXISTING_FAMILY_STATUS_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_STATE_DIR:-}"

EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_SV_STIMULI_QUALITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-}"

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
require_file "$SV_PARSER_FAMILY_STATUS_GATE"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

family_status_state_dir="${EXISTING_FAMILY_STATUS_STATE_DIR:-$WORK_DIR/sv_parser_family_status_gate}"

if [[ -z "$EXISTING_FAMILY_STATUS_STATE_DIR" ]]; then
    family_status_env=(
        env
        PGEN_SV_FAMILY_STATUS_STATE_DIR="$family_status_state_dir"
    )
    if [[ -n "$EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR="$EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR="$EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_STIMULI_QUALITY_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_STIMULI_QUALITY_STATE_DIR="$EXISTING_SV_STIMULI_QUALITY_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR"
        )
    fi
    family_status_env+=("$SV_PARSER_FAMILY_STATUS_GATE")
    run_logged "sv_parser_family_status_gate" "${family_status_env[@]}"
fi

family_status_summary_json="$family_status_state_dir/summary.json"
family_status_summary_txt="$family_status_state_dir/summary.txt"

require_nonempty_file "$family_status_summary_json"
require_nonempty_file "$family_status_summary_txt"

main_expected_criteria='["syntax_closure_gate_green","parser_aggregate_contract_green","generation_parser_rejections_zero","replay_shadow_parser_rejections_zero","focused_replay_target_debt_zero"]'
main_expected_metrics='["syntax_closure_status","syntax_closure_failure_count","syntax_defined_rule_count","syntax_unresolved_rule_reference_count","syntax_unreachable_rules","syntax_unreachable_branches","syntax_target_debt_count","generation_parser_rejections_total","replay_shadow_parser_rejections_total","focused_replay_target_count","focused_replay_covered_reachable_rules","focused_replay_covered_reachable_branches","replay_gap_target_primary_rule"]'
main_expected_proof_surfaces='["syntax_closure_summary_json","parser_aggregate_summary_txt"]'
svpp_expected_criteria='["syntax_closure_gate_green","aggregate_contract_green","reachability_closure_green","parser_rejections_zero","parseability_rejections_zero","reachability_stage3_targets_zero","reachability_stage4_targets_zero","reachability_stage3_rules_full","reachability_stage4_rules_full","reachability_stage3_branches_full","reachability_stage4_branches_full"]'
svpp_expected_metrics='["syntax_closure_status","syntax_closure_failure_count","syntax_defined_rule_count","syntax_unresolved_rule_reference_count","syntax_unreachable_rules","syntax_unreachable_branches","syntax_target_debt_count","parseability_parser_rejections_total","parseability_rejected_total","final_targets","covered_reachable_rules","covered_reachable_branches","counterexample_primary_stage","reachability_stage3_targets","reachability_stage4_targets","reachability_stage3_rules","reachability_stage4_rules","reachability_stage3_branches","reachability_stage4_branches"]'
svpp_expected_proof_surfaces='["syntax_closure_summary_json","aggregate_summary_txt","reachability_summary_txt"]'

jq -e \
    --argjson main_expected_criteria "$main_expected_criteria" \
    --argjson main_expected_metrics "$main_expected_metrics" \
    --argjson main_expected_proof_surfaces "$main_expected_proof_surfaces" \
    --argjson svpp_expected_criteria "$svpp_expected_criteria" \
    --argjson svpp_expected_metrics "$svpp_expected_metrics" \
    --argjson svpp_expected_proof_surfaces "$svpp_expected_proof_surfaces" \
    '
    . as $root
    | ($root.gate == "sv_parser_family_status_gate")
    and (($root.version | type) == "number")
    and (($root.generated_at_utc | type) == "string" and ($root.generated_at_utc | length) > 0)
    and (($root.live_tracker_file | type) == "string" and ($root.live_tracker_file | length) > 0)
    and (($root.status_rule_done | type) == "string" and ($root.status_rule_done | length) > 0)
    and (($root.families | length) == 2)
    and (($root.families | map(.family) | sort) == ["systemverilog","systemverilog_preprocessor"])
    and (
        $root.families[]
        | (
            has("family")
            and has("computed_status")
            and has("live_tracker_status")
            and has("tracker_alignment_ok")
            and has("proof_surfaces")
            and has("closure_criteria_total_count")
            and has("closure_criteria_satisfied_count")
            and has("closure_criteria_unsatisfied_count")
            and has("criteria")
            and has("metrics")
            and has("unmet_closure_criteria")
            and has("unmet_closure_criteria_details")
        )
        and ((.tracker_alignment_ok | type) == "boolean")
        and (.tracker_alignment_ok == (.computed_status == .live_tracker_status))
        and ((.closure_criteria_total_count | type) == "number")
        and ((.closure_criteria_satisfied_count | type) == "number")
        and ((.closure_criteria_unsatisfied_count | type) == "number")
        and (.closure_criteria_satisfied_count + .closure_criteria_unsatisfied_count == .closure_criteria_total_count)
        and ((.unmet_closure_criteria | type) == "array")
        and ((.unmet_closure_criteria_details | type) == "array")
        and ((.unmet_closure_criteria | length) == .closure_criteria_unsatisfied_count)
        and ((.unmet_closure_criteria_details | length) == .closure_criteria_unsatisfied_count)
        and ((.unmet_closure_criteria_details | map(.detail)) == .unmet_closure_criteria)
        and (
            . as $family
            | all($family.unmet_closure_criteria_details[]?;
                ((.criterion | type) == "string")
                and ((.evidence_key | type) == "string")
                and ((.observed | type) == "string")
                and ((.expected | type) == "string")
                and ((.detail | type) == "string")
                and ($family.criteria[.criterion] == false)
            )
        )
        and (
            . as $family
            | ([($family.criteria | to_entries[] | select(.value == false))] | length) == $family.closure_criteria_unsatisfied_count
        )
        and (
            . as $family
            | if $family.family == "systemverilog" then
                (all($main_expected_criteria[]; . as $k | ($family.criteria | has($k))))
                and (all($main_expected_metrics[]; . as $k | ($family.metrics | has($k))))
                and (all($main_expected_proof_surfaces[]; . as $k | ($family.proof_surfaces | has($k))))
            elif $family.family == "systemverilog_preprocessor" then
                (all($svpp_expected_criteria[]; . as $k | ($family.criteria | has($k))))
                and (all($svpp_expected_metrics[]; . as $k | ($family.metrics | has($k))))
                and (all($svpp_expected_proof_surfaces[]; . as $k | ($family.proof_surfaces | has($k))))
            else
                false
            end
        )
    )
    ' "$family_status_summary_json" >/dev/null

main_tracker_alignment_ok="$(jq -r '.families[] | select(.family=="systemverilog") | .tracker_alignment_ok' "$family_status_summary_json")"
main_unsatisfied_count="$(jq -r '.families[] | select(.family=="systemverilog") | .closure_criteria_unsatisfied_count' "$family_status_summary_json")"
main_false_criteria_count="$(jq -r '[.families[] | select(.family=="systemverilog") | .criteria | to_entries[] | select(.value == false)] | length' "$family_status_summary_json")"
main_details_count="$(jq -r '.families[] | select(.family=="systemverilog") | (.unmet_closure_criteria_details | length)' "$family_status_summary_json")"
main_primary_detail_criterion="$(jq -r '.families[] | select(.family=="systemverilog") | (.unmet_closure_criteria_details[0].criterion // "<none>")' "$family_status_summary_json")"
main_details_json="$(jq -cer '.families[] | select(.family=="systemverilog") | .unmet_closure_criteria_details' "$family_status_summary_json")"

svpp_tracker_alignment_ok="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .tracker_alignment_ok' "$family_status_summary_json")"
svpp_unsatisfied_count="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .closure_criteria_unsatisfied_count' "$family_status_summary_json")"
svpp_false_criteria_count="$(jq -r '[.families[] | select(.family=="systemverilog_preprocessor") | .criteria | to_entries[] | select(.value == false)] | length' "$family_status_summary_json")"
svpp_details_count="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | (.unmet_closure_criteria_details | length)' "$family_status_summary_json")"
svpp_primary_detail_criterion="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | (.unmet_closure_criteria_details[0].criterion // "<none>")' "$family_status_summary_json")"
svpp_details_json="$(jq -cer '.families[] | select(.family=="systemverilog_preprocessor") | .unmet_closure_criteria_details' "$family_status_summary_json")"

summary_main_details_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_unmet_closure_criteria_details_json")"
summary_svpp_details_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_unmet_closure_criteria_details_json")"
summary_main_tracker_alignment="$(extract_summary_value "$family_status_summary_txt" "systemverilog_tracker_alignment_ok")"
summary_svpp_tracker_alignment="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_tracker_alignment_ok")"

if [[ "$summary_main_details_json" != "$main_details_json" ]]; then
    echo "error: main family structured blocker json mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_details_json" != "$svpp_details_json" ]]; then
    echo "error: preprocessor family structured blocker json mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_main_tracker_alignment" != "$main_tracker_alignment_ok" ]]; then
    echo "error: main family tracker alignment mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_tracker_alignment" != "$svpp_tracker_alignment_ok" ]]; then
    echo "error: preprocessor family tracker alignment mismatch between summary.txt and summary.json" >&2
    exit 1
fi

{
    echo "SV Parser Family Status Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "family_status_state_dir: $family_status_state_dir"
    echo "family_status_summary_json: $family_status_summary_json"
    echo "family_status_summary_txt: $family_status_summary_txt"
    echo "family_count: 2"
    echo "systemverilog_tracker_alignment_ok: $main_tracker_alignment_ok"
    echo "systemverilog_false_criteria_count: $main_false_criteria_count"
    echo "systemverilog_unmet_details_count: $main_details_count"
    echo "systemverilog_primary_unmet_detail_criterion: $main_primary_detail_criterion"
    echo "systemverilog_preprocessor_tracker_alignment_ok: $svpp_tracker_alignment_ok"
    echo "systemverilog_preprocessor_false_criteria_count: $svpp_false_criteria_count"
    echo "systemverilog_preprocessor_unmet_details_count: $svpp_details_count"
    echo "systemverilog_preprocessor_primary_unmet_detail_criterion: $svpp_primary_detail_criterion"
} | tee "$SUMMARY_TXT"

echo "✅ SV parser-family status contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
