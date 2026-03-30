#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_parser_family_status_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

SV_PARSER_FAMILY_STATUS_GATE="$RUST_DIR/scripts/sv_parser_family_status_gate.sh"
EXISTING_FAMILY_STATUS_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_STATE_DIR:-}"

EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR:-}"
EXISTING_SV_PARSER_AGGREGATE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PARSER_AGGREGATE_STATE_DIR:-}"
EXISTING_SV_STIMULI_QUALITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-}"
EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR:-}"
EXISTING_SV_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR:-}"
EXISTING_SV_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="${PGEN_SV_FAMILY_STATUS_CONTRACT_EXISTING_SV_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-}"

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
    if [[ -n "$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_PARSER_AGGREGATE_STATE_DIR="$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR="$EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR="$EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR="$EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR="$EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR="$EXISTING_SV_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_SV_EXTERNAL_CORPUS_TRIAGE_STATE_DIR" ]]; then
        family_status_env+=(
            PGEN_SV_FAMILY_STATUS_EXISTING_SV_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="$EXISTING_SV_EXTERNAL_CORPUS_TRIAGE_STATE_DIR"
        )
    fi
    family_status_env+=("$SV_PARSER_FAMILY_STATUS_GATE")
    run_logged "sv_parser_family_status_gate" "${family_status_env[@]}"
fi

family_status_summary_json="$family_status_state_dir/summary.json"
family_status_summary_txt="$family_status_state_dir/summary.txt"

require_nonempty_file "$family_status_summary_json"
require_nonempty_file "$family_status_summary_txt"

main_expected_criteria='["syntax_closure_gate_green","parser_aggregate_contract_green","generation_parser_rejections_zero","replay_shadow_parser_rejections_zero","focused_replay_target_debt_zero","semantic_scope_contract_green","formal_exhaustive_closure_surface_green"]'
main_expected_metrics='["syntax_closure_status","syntax_closure_failure_count","syntax_defined_rule_count","syntax_unresolved_rule_reference_count","syntax_unreachable_rules","syntax_unreachable_branches","syntax_target_debt_count","semantic_scope_case_count","semantic_scope_failed_count","generation_parser_rejections_total","replay_shadow_parser_rejections_total","focused_replay_target_count","focused_replay_covered_reachable_rules","focused_replay_covered_reachable_branches","replay_gap_target_primary_rule","formal_exhaustive_closure_gate","formal_exhaustive_closure_gate_version","formal_exhaustive_closure_generated_at_utc","formal_exhaustive_closure_primary_unmet_closure_criterion","formal_exhaustive_closure_unmet_closure_criteria_count"]'
main_expected_proof_surfaces='["syntax_closure_state_dir","syntax_closure_summary_txt","syntax_closure_summary_json","parser_aggregate_state_dir","parser_aggregate_summary_txt","parser_aggregate_summary_json","semantic_scope_contract_state_dir","semantic_scope_contract_summary_txt","semantic_scope_contract_summary_json","formal_exhaustive_closure_state_dir","formal_exhaustive_closure_summary_txt","formal_exhaustive_closure_summary_json"]'
svpp_expected_criteria='["syntax_closure_gate_green","aggregate_contract_green","reachability_closure_green","parser_rejections_zero","parseability_rejections_zero","reachability_stage3_targets_zero","reachability_stage4_targets_zero","reachability_stage3_rules_full","reachability_stage4_rules_full","reachability_stage3_branches_full","reachability_stage4_branches_full","formal_exhaustive_closure_surface_green"]'
svpp_expected_metrics='["syntax_closure_status","syntax_closure_failure_count","syntax_defined_rule_count","syntax_unresolved_rule_reference_count","syntax_unreachable_rules","syntax_unreachable_branches","syntax_target_debt_count","parseability_parser_rejections_total","parseability_rejected_total","final_targets","covered_reachable_rules","covered_reachable_branches","counterexample_primary_stage","reachability_stage3_targets","reachability_stage4_targets","reachability_stage3_rules","reachability_stage4_rules","reachability_stage3_branches","reachability_stage4_branches","formal_exhaustive_closure_gate","formal_exhaustive_closure_gate_version","formal_exhaustive_closure_generated_at_utc","formal_exhaustive_closure_primary_unmet_closure_criterion","formal_exhaustive_closure_unmet_closure_criteria_count"]'
svpp_expected_proof_surfaces='["syntax_closure_state_dir","syntax_closure_summary_txt","syntax_closure_summary_json","aggregate_state_dir","aggregate_summary_txt","aggregate_summary_json","reachability_state_dir","reachability_summary_txt","formal_exhaustive_closure_state_dir","formal_exhaustive_closure_summary_txt","formal_exhaustive_closure_summary_json"]'

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
main_primary_unmet_from_json="$(jq -r '.families[] | select(.family=="systemverilog") | .primary_unmet_closure_criterion' "$family_status_summary_json")"
main_unmet_json="$(jq -cer '.families[] | select(.family=="systemverilog") | .unmet_closure_criteria' "$family_status_summary_json")"
main_details_json="$(jq -cer '.families[] | select(.family=="systemverilog") | .unmet_closure_criteria_details' "$family_status_summary_json")"

svpp_tracker_alignment_ok="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .tracker_alignment_ok' "$family_status_summary_json")"
svpp_unsatisfied_count="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .closure_criteria_unsatisfied_count' "$family_status_summary_json")"
svpp_false_criteria_count="$(jq -r '[.families[] | select(.family=="systemverilog_preprocessor") | .criteria | to_entries[] | select(.value == false)] | length' "$family_status_summary_json")"
svpp_details_count="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | (.unmet_closure_criteria_details | length)' "$family_status_summary_json")"
svpp_primary_detail_criterion="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | (.unmet_closure_criteria_details[0].criterion // "<none>")' "$family_status_summary_json")"
svpp_primary_unmet_from_json="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .primary_unmet_closure_criterion' "$family_status_summary_json")"
svpp_unmet_json="$(jq -cer '.families[] | select(.family=="systemverilog_preprocessor") | .unmet_closure_criteria' "$family_status_summary_json")"
svpp_details_json="$(jq -cer '.families[] | select(.family=="systemverilog_preprocessor") | .unmet_closure_criteria_details' "$family_status_summary_json")"

summary_main_primary_unmet="$(extract_summary_value "$family_status_summary_txt" "systemverilog_primary_unmet_closure_criterion")"
summary_main_unmet_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_unmet_closure_criteria_json")"
summary_main_details_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_unmet_closure_criteria_details_json")"
summary_svpp_primary_unmet="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_primary_unmet_closure_criterion")"
summary_svpp_unmet_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_unmet_closure_criteria_json")"
summary_svpp_details_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_unmet_closure_criteria_details_json")"
summary_main_tracker_alignment="$(extract_summary_value "$family_status_summary_txt" "systemverilog_tracker_alignment_ok")"
summary_svpp_tracker_alignment="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_tracker_alignment_ok")"
summary_main_syntax_closure_state_dir="$(extract_summary_value "$family_status_summary_txt" "systemverilog_syntax_closure_state_dir")"
summary_main_syntax_closure_summary_txt="$(extract_summary_value "$family_status_summary_txt" "systemverilog_syntax_closure_summary_txt")"
summary_main_syntax_closure_summary_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_syntax_closure_summary_json")"
summary_main_parser_aggregate_state_dir="$(extract_summary_value "$family_status_summary_txt" "systemverilog_parser_aggregate_state_dir")"
summary_main_parser_aggregate_summary_txt="$(extract_summary_value "$family_status_summary_txt" "systemverilog_parser_aggregate_summary_txt")"
summary_main_parser_aggregate_summary_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_parser_aggregate_summary_json")"
summary_main_semantic_scope_contract_state_dir="$(extract_summary_value "$family_status_summary_txt" "systemverilog_semantic_scope_contract_state_dir")"
summary_main_semantic_scope_contract_summary_txt="$(extract_summary_value "$family_status_summary_txt" "systemverilog_semantic_scope_contract_summary_txt")"
summary_main_semantic_scope_contract_summary_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_semantic_scope_contract_summary_json")"
summary_main_formal_exhaustive_closure_gate="$(extract_summary_value "$family_status_summary_txt" "systemverilog_formal_exhaustive_closure_gate")"
summary_main_formal_exhaustive_closure_gate_version="$(extract_summary_value "$family_status_summary_txt" "systemverilog_formal_exhaustive_closure_gate_version")"
summary_main_formal_exhaustive_closure_generated_at_utc="$(extract_summary_value "$family_status_summary_txt" "systemverilog_formal_exhaustive_closure_generated_at_utc")"
summary_main_formal_exhaustive_closure_state_dir="$(extract_summary_value "$family_status_summary_txt" "systemverilog_formal_exhaustive_closure_state_dir")"
summary_main_formal_exhaustive_closure_summary_txt="$(extract_summary_value "$family_status_summary_txt" "systemverilog_formal_exhaustive_closure_summary_txt")"
summary_main_formal_exhaustive_closure_summary_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_formal_exhaustive_closure_summary_json")"
summary_main_formal_exhaustive_primary_unmet="$(extract_summary_value "$family_status_summary_txt" "systemverilog_formal_exhaustive_closure_primary_unmet_closure_criterion")"
summary_main_formal_exhaustive_unmet_count="$(extract_summary_value "$family_status_summary_txt" "systemverilog_formal_exhaustive_closure_unmet_closure_criteria_count")"
summary_svpp_syntax_closure_state_dir="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_syntax_closure_state_dir")"
summary_svpp_syntax_closure_summary_txt="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_syntax_closure_summary_txt")"
summary_svpp_syntax_closure_summary_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_syntax_closure_summary_json")"
summary_svpp_aggregate_state_dir="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_aggregate_state_dir")"
summary_svpp_aggregate_summary_txt="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_aggregate_summary_txt")"
summary_svpp_aggregate_summary_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_aggregate_summary_json")"
summary_svpp_reachability_state_dir="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_reachability_state_dir")"
summary_svpp_reachability_summary_txt="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_reachability_summary_txt")"
summary_svpp_formal_exhaustive_closure_gate="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_formal_exhaustive_closure_gate")"
summary_svpp_formal_exhaustive_closure_gate_version="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_formal_exhaustive_closure_gate_version")"
summary_svpp_formal_exhaustive_closure_generated_at_utc="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_formal_exhaustive_closure_generated_at_utc")"
summary_svpp_formal_exhaustive_closure_state_dir="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_formal_exhaustive_closure_state_dir")"
summary_svpp_formal_exhaustive_closure_summary_txt="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_formal_exhaustive_closure_summary_txt")"
summary_svpp_formal_exhaustive_closure_summary_json="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_formal_exhaustive_closure_summary_json")"
summary_svpp_formal_exhaustive_primary_unmet="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_formal_exhaustive_closure_primary_unmet_closure_criterion")"
summary_svpp_formal_exhaustive_unmet_count="$(extract_summary_value "$family_status_summary_txt" "systemverilog_preprocessor_formal_exhaustive_closure_unmet_closure_criteria_count")"
main_syntax_closure_state_dir="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.syntax_closure_state_dir' "$family_status_summary_json")"
main_syntax_closure_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.syntax_closure_summary_txt' "$family_status_summary_json")"
main_syntax_closure_summary_json="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.syntax_closure_summary_json' "$family_status_summary_json")"
main_parser_aggregate_state_dir="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.parser_aggregate_state_dir' "$family_status_summary_json")"
main_parser_aggregate_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.parser_aggregate_summary_txt' "$family_status_summary_json")"
main_parser_aggregate_summary_json="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.parser_aggregate_summary_json' "$family_status_summary_json")"
main_semantic_scope_contract_state_dir="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.semantic_scope_contract_state_dir' "$family_status_summary_json")"
main_semantic_scope_contract_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.semantic_scope_contract_summary_txt' "$family_status_summary_json")"
main_semantic_scope_contract_summary_json="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.semantic_scope_contract_summary_json' "$family_status_summary_json")"
main_formal_exhaustive_closure_gate="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.formal_exhaustive_closure_gate' "$family_status_summary_json")"
main_formal_exhaustive_closure_gate_version="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.formal_exhaustive_closure_gate_version' "$family_status_summary_json")"
main_formal_exhaustive_closure_generated_at_utc="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.formal_exhaustive_closure_generated_at_utc' "$family_status_summary_json")"
main_formal_exhaustive_closure_primary_unmet_closure_criterion="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.formal_exhaustive_closure_primary_unmet_closure_criterion' "$family_status_summary_json")"
main_formal_exhaustive_closure_unmet_closure_criteria_count="$(jq -r '.families[] | select(.family=="systemverilog") | .metrics.formal_exhaustive_closure_unmet_closure_criteria_count' "$family_status_summary_json")"
main_formal_exhaustive_closure_state_dir="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.formal_exhaustive_closure_state_dir' "$family_status_summary_json")"
main_formal_exhaustive_closure_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.formal_exhaustive_closure_summary_txt' "$family_status_summary_json")"
main_formal_exhaustive_closure_summary_json="$(jq -r '.families[] | select(.family=="systemverilog") | .proof_surfaces.formal_exhaustive_closure_summary_json' "$family_status_summary_json")"
svpp_syntax_closure_state_dir="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.syntax_closure_state_dir' "$family_status_summary_json")"
svpp_syntax_closure_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.syntax_closure_summary_txt' "$family_status_summary_json")"
svpp_syntax_closure_summary_json="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.syntax_closure_summary_json' "$family_status_summary_json")"
svpp_aggregate_state_dir="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.aggregate_state_dir' "$family_status_summary_json")"
svpp_aggregate_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.aggregate_summary_txt' "$family_status_summary_json")"
svpp_aggregate_summary_json="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.aggregate_summary_json' "$family_status_summary_json")"
svpp_reachability_state_dir="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.reachability_state_dir' "$family_status_summary_json")"
svpp_reachability_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.reachability_summary_txt' "$family_status_summary_json")"
svpp_formal_exhaustive_closure_gate="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.formal_exhaustive_closure_gate' "$family_status_summary_json")"
svpp_formal_exhaustive_closure_gate_version="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.formal_exhaustive_closure_gate_version' "$family_status_summary_json")"
svpp_formal_exhaustive_closure_generated_at_utc="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.formal_exhaustive_closure_generated_at_utc' "$family_status_summary_json")"
svpp_formal_exhaustive_closure_primary_unmet_closure_criterion="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.formal_exhaustive_closure_primary_unmet_closure_criterion' "$family_status_summary_json")"
svpp_formal_exhaustive_closure_unmet_closure_criteria_count="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .metrics.formal_exhaustive_closure_unmet_closure_criteria_count' "$family_status_summary_json")"
svpp_formal_exhaustive_closure_state_dir="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.formal_exhaustive_closure_state_dir' "$family_status_summary_json")"
svpp_formal_exhaustive_closure_summary_txt="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.formal_exhaustive_closure_summary_txt' "$family_status_summary_json")"
svpp_formal_exhaustive_closure_summary_json="$(jq -r '.families[] | select(.family=="systemverilog_preprocessor") | .proof_surfaces.formal_exhaustive_closure_summary_json' "$family_status_summary_json")"

if [[ "$summary_main_unmet_json" != "$main_unmet_json" ]]; then
    echo "error: main family unmet-criteria json mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_main_primary_unmet" != "$main_primary_unmet_from_json" ]]; then
    echo "error: main family primary unmet criterion mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_main_details_json" != "$main_details_json" ]]; then
    echo "error: main family structured blocker json mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_unmet_json" != "$svpp_unmet_json" ]]; then
    echo "error: preprocessor family unmet-criteria json mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_primary_unmet" != "$svpp_primary_unmet_from_json" ]]; then
    echo "error: preprocessor family primary unmet criterion mismatch between summary.txt and summary.json" >&2
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
if [[ "$summary_main_syntax_closure_state_dir" != "$main_syntax_closure_state_dir" || "$summary_main_syntax_closure_summary_txt" != "$main_syntax_closure_summary_txt" || "$summary_main_syntax_closure_summary_json" != "$main_syntax_closure_summary_json" || "$summary_main_parser_aggregate_state_dir" != "$main_parser_aggregate_state_dir" || "$summary_main_parser_aggregate_summary_txt" != "$main_parser_aggregate_summary_txt" || "$summary_main_parser_aggregate_summary_json" != "$main_parser_aggregate_summary_json" || "$summary_main_semantic_scope_contract_state_dir" != "$main_semantic_scope_contract_state_dir" || "$summary_main_semantic_scope_contract_summary_txt" != "$main_semantic_scope_contract_summary_txt" || "$summary_main_semantic_scope_contract_summary_json" != "$main_semantic_scope_contract_summary_json" ]]; then
    echo "error: main family proof-surface path mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_main_formal_exhaustive_closure_gate" != "$main_formal_exhaustive_closure_gate" ]]; then
    echo "error: main family formal-closure gate mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_main_formal_exhaustive_closure_gate_version" != "$main_formal_exhaustive_closure_gate_version" ]]; then
    echo "error: main family formal-closure gate version mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_main_formal_exhaustive_closure_generated_at_utc" != "$main_formal_exhaustive_closure_generated_at_utc" ]]; then
    echo "error: main family formal-closure generated_at_utc mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_main_formal_exhaustive_primary_unmet" != "$main_formal_exhaustive_closure_primary_unmet_closure_criterion" ]]; then
    echo "error: main family formal-closure primary unmet mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_main_formal_exhaustive_unmet_count" != "$main_formal_exhaustive_closure_unmet_closure_criteria_count" ]]; then
    echo "error: main family formal-closure unmet-count mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_main_formal_exhaustive_closure_state_dir" != "$main_formal_exhaustive_closure_state_dir" || "$summary_main_formal_exhaustive_closure_summary_txt" != "$main_formal_exhaustive_closure_summary_txt" || "$summary_main_formal_exhaustive_closure_summary_json" != "$main_formal_exhaustive_closure_summary_json" ]]; then
    echo "error: main family formal-closure proof-surface path mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_syntax_closure_state_dir" != "$svpp_syntax_closure_state_dir" || "$summary_svpp_syntax_closure_summary_txt" != "$svpp_syntax_closure_summary_txt" || "$summary_svpp_syntax_closure_summary_json" != "$svpp_syntax_closure_summary_json" || "$summary_svpp_aggregate_state_dir" != "$svpp_aggregate_state_dir" || "$summary_svpp_aggregate_summary_txt" != "$svpp_aggregate_summary_txt" || "$summary_svpp_aggregate_summary_json" != "$svpp_aggregate_summary_json" || "$summary_svpp_reachability_state_dir" != "$svpp_reachability_state_dir" || "$summary_svpp_reachability_summary_txt" != "$svpp_reachability_summary_txt" || "$summary_svpp_formal_exhaustive_closure_state_dir" != "$svpp_formal_exhaustive_closure_state_dir" || "$summary_svpp_formal_exhaustive_closure_summary_txt" != "$svpp_formal_exhaustive_closure_summary_txt" || "$summary_svpp_formal_exhaustive_closure_summary_json" != "$svpp_formal_exhaustive_closure_summary_json" ]]; then
    echo "error: preprocessor family proof-surface path mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_formal_exhaustive_closure_gate" != "$svpp_formal_exhaustive_closure_gate" ]]; then
    echo "error: preprocessor family formal-closure gate mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_formal_exhaustive_closure_gate_version" != "$svpp_formal_exhaustive_closure_gate_version" ]]; then
    echo "error: preprocessor family formal-closure gate version mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_formal_exhaustive_closure_generated_at_utc" != "$svpp_formal_exhaustive_closure_generated_at_utc" ]]; then
    echo "error: preprocessor family formal-closure generated_at_utc mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_formal_exhaustive_primary_unmet" != "$svpp_formal_exhaustive_closure_primary_unmet_closure_criterion" ]]; then
    echo "error: preprocessor family formal-closure primary unmet mismatch between summary.txt and summary.json" >&2
    exit 1
fi
if [[ "$summary_svpp_formal_exhaustive_unmet_count" != "$svpp_formal_exhaustive_closure_unmet_closure_criteria_count" ]]; then
    echo "error: preprocessor family formal-closure unmet-count mismatch between summary.txt and summary.json" >&2
    exit 1
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "SV Parser Family Status Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "family_status_state_dir: $family_status_state_dir"
    echo "family_status_summary_json: $family_status_summary_json"
    echo "family_status_summary_txt: $family_status_summary_txt"
    echo "family_count: 2"
    echo "systemverilog_tracker_alignment_ok: $main_tracker_alignment_ok"
    echo "systemverilog_false_criteria_count: $main_false_criteria_count"
    echo "systemverilog_unmet_details_count: $main_details_count"
    echo "systemverilog_primary_unmet_detail_criterion: $main_primary_detail_criterion"
    echo "systemverilog_parser_aggregate_state_dir: $main_parser_aggregate_state_dir"
    echo "systemverilog_parser_aggregate_summary_txt: $main_parser_aggregate_summary_txt"
    echo "systemverilog_parser_aggregate_summary_json: $main_parser_aggregate_summary_json"
    echo "systemverilog_semantic_scope_contract_state_dir: $main_semantic_scope_contract_state_dir"
    echo "systemverilog_semantic_scope_contract_summary_txt: $main_semantic_scope_contract_summary_txt"
    echo "systemverilog_semantic_scope_contract_summary_json: $main_semantic_scope_contract_summary_json"
    echo "systemverilog_formal_exhaustive_closure_gate: $main_formal_exhaustive_closure_gate"
    echo "systemverilog_formal_exhaustive_closure_gate_version: $main_formal_exhaustive_closure_gate_version"
    echo "systemverilog_formal_exhaustive_closure_generated_at_utc: $main_formal_exhaustive_closure_generated_at_utc"
    echo "systemverilog_formal_exhaustive_closure_primary_unmet_closure_criterion: $main_formal_exhaustive_closure_primary_unmet_closure_criterion"
    echo "systemverilog_formal_exhaustive_closure_unmet_closure_criteria_count: $main_formal_exhaustive_closure_unmet_closure_criteria_count"
    echo "systemverilog_formal_exhaustive_closure_state_dir: $main_formal_exhaustive_closure_state_dir"
    echo "systemverilog_formal_exhaustive_closure_summary_txt: $main_formal_exhaustive_closure_summary_txt"
    echo "systemverilog_formal_exhaustive_closure_summary_json: $main_formal_exhaustive_closure_summary_json"
    echo "systemverilog_unmet_closure_criteria_json: $main_unmet_json"
    echo "systemverilog_unmet_closure_criteria_details_json: $main_details_json"
    echo "systemverilog_preprocessor_tracker_alignment_ok: $svpp_tracker_alignment_ok"
    echo "systemverilog_preprocessor_false_criteria_count: $svpp_false_criteria_count"
    echo "systemverilog_preprocessor_unmet_details_count: $svpp_details_count"
    echo "systemverilog_preprocessor_primary_unmet_detail_criterion: $svpp_primary_detail_criterion"
    echo "systemverilog_preprocessor_aggregate_state_dir: $svpp_aggregate_state_dir"
    echo "systemverilog_preprocessor_aggregate_summary_txt: $svpp_aggregate_summary_txt"
    echo "systemverilog_preprocessor_aggregate_summary_json: $svpp_aggregate_summary_json"
    echo "systemverilog_preprocessor_formal_exhaustive_closure_gate: $svpp_formal_exhaustive_closure_gate"
    echo "systemverilog_preprocessor_formal_exhaustive_closure_gate_version: $svpp_formal_exhaustive_closure_gate_version"
    echo "systemverilog_preprocessor_formal_exhaustive_closure_generated_at_utc: $svpp_formal_exhaustive_closure_generated_at_utc"
    echo "systemverilog_preprocessor_formal_exhaustive_closure_primary_unmet_closure_criterion: $svpp_formal_exhaustive_closure_primary_unmet_closure_criterion"
    echo "systemverilog_preprocessor_formal_exhaustive_closure_unmet_closure_criteria_count: $svpp_formal_exhaustive_closure_unmet_closure_criteria_count"
    echo "systemverilog_preprocessor_formal_exhaustive_closure_state_dir: $svpp_formal_exhaustive_closure_state_dir"
    echo "systemverilog_preprocessor_formal_exhaustive_closure_summary_txt: $svpp_formal_exhaustive_closure_summary_txt"
    echo "systemverilog_preprocessor_formal_exhaustive_closure_summary_json: $svpp_formal_exhaustive_closure_summary_json"
    echo "systemverilog_preprocessor_unmet_closure_criteria_json: $svpp_unmet_json"
    echo "systemverilog_preprocessor_unmet_closure_criteria_details_json: $svpp_details_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "sv_parser_family_status_contract_gate" \
    --argjson version 3 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg family_status_state_dir "$family_status_state_dir" \
    --arg family_status_summary_json "$family_status_summary_json" \
    --arg family_status_summary_txt "$family_status_summary_txt" \
    --argjson family_count 2 \
    --argjson systemverilog_tracker_alignment_ok "$main_tracker_alignment_ok" \
    --argjson systemverilog_false_criteria_count "$main_false_criteria_count" \
    --argjson systemverilog_unmet_details_count "$main_details_count" \
    --arg systemverilog_primary_unmet_detail_criterion "$main_primary_detail_criterion" \
    --arg systemverilog_parser_aggregate_state_dir "$main_parser_aggregate_state_dir" \
    --arg systemverilog_parser_aggregate_summary_txt "$main_parser_aggregate_summary_txt" \
    --arg systemverilog_parser_aggregate_summary_json "$main_parser_aggregate_summary_json" \
    --arg systemverilog_semantic_scope_contract_state_dir "$main_semantic_scope_contract_state_dir" \
    --arg systemverilog_semantic_scope_contract_summary_txt "$main_semantic_scope_contract_summary_txt" \
    --arg systemverilog_semantic_scope_contract_summary_json "$main_semantic_scope_contract_summary_json" \
    --arg systemverilog_formal_exhaustive_closure_gate "$main_formal_exhaustive_closure_gate" \
    --argjson systemverilog_formal_exhaustive_closure_gate_version "$main_formal_exhaustive_closure_gate_version" \
    --arg systemverilog_formal_exhaustive_closure_generated_at_utc "$main_formal_exhaustive_closure_generated_at_utc" \
    --arg systemverilog_formal_exhaustive_closure_primary_unmet_closure_criterion "$main_formal_exhaustive_closure_primary_unmet_closure_criterion" \
    --argjson systemverilog_formal_exhaustive_closure_unmet_closure_criteria_count "$main_formal_exhaustive_closure_unmet_closure_criteria_count" \
    --arg systemverilog_formal_exhaustive_closure_state_dir "$main_formal_exhaustive_closure_state_dir" \
    --arg systemverilog_formal_exhaustive_closure_summary_txt "$main_formal_exhaustive_closure_summary_txt" \
    --arg systemverilog_formal_exhaustive_closure_summary_json "$main_formal_exhaustive_closure_summary_json" \
    --argjson systemverilog_unmet_closure_criteria "$main_unmet_json" \
    --argjson systemverilog_unmet_closure_criteria_details "$main_details_json" \
    --argjson systemverilog_preprocessor_tracker_alignment_ok "$svpp_tracker_alignment_ok" \
    --argjson systemverilog_preprocessor_false_criteria_count "$svpp_false_criteria_count" \
    --argjson systemverilog_preprocessor_unmet_details_count "$svpp_details_count" \
    --arg systemverilog_preprocessor_primary_unmet_detail_criterion "$svpp_primary_detail_criterion" \
    --arg systemverilog_preprocessor_aggregate_state_dir "$svpp_aggregate_state_dir" \
    --arg systemverilog_preprocessor_aggregate_summary_txt "$svpp_aggregate_summary_txt" \
    --arg systemverilog_preprocessor_aggregate_summary_json "$svpp_aggregate_summary_json" \
    --arg systemverilog_preprocessor_formal_exhaustive_closure_gate "$svpp_formal_exhaustive_closure_gate" \
    --argjson systemverilog_preprocessor_formal_exhaustive_closure_gate_version "$svpp_formal_exhaustive_closure_gate_version" \
    --arg systemverilog_preprocessor_formal_exhaustive_closure_generated_at_utc "$svpp_formal_exhaustive_closure_generated_at_utc" \
    --arg systemverilog_preprocessor_formal_exhaustive_closure_primary_unmet_closure_criterion "$svpp_formal_exhaustive_closure_primary_unmet_closure_criterion" \
    --argjson systemverilog_preprocessor_formal_exhaustive_closure_unmet_closure_criteria_count "$svpp_formal_exhaustive_closure_unmet_closure_criteria_count" \
    --arg systemverilog_preprocessor_formal_exhaustive_closure_state_dir "$svpp_formal_exhaustive_closure_state_dir" \
    --arg systemverilog_preprocessor_formal_exhaustive_closure_summary_txt "$svpp_formal_exhaustive_closure_summary_txt" \
    --arg systemverilog_preprocessor_formal_exhaustive_closure_summary_json "$svpp_formal_exhaustive_closure_summary_json" \
    --argjson systemverilog_preprocessor_unmet_closure_criteria "$svpp_unmet_json" \
    --argjson systemverilog_preprocessor_unmet_closure_criteria_details "$svpp_details_json" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      family_status_state_dir: $family_status_state_dir,
      family_status_summary_json: $family_status_summary_json,
      family_status_summary_txt: $family_status_summary_txt,
      family_count: $family_count,
      families: [
        {
          family: "systemverilog",
          tracker_alignment_ok: $systemverilog_tracker_alignment_ok,
          false_criteria_count: $systemverilog_false_criteria_count,
          unmet_details_count: $systemverilog_unmet_details_count,
          primary_unmet_detail_criterion: $systemverilog_primary_unmet_detail_criterion,
          proof_surfaces: {
            parser_aggregate_state_dir: $systemverilog_parser_aggregate_state_dir,
            parser_aggregate_summary_txt: $systemverilog_parser_aggregate_summary_txt,
            parser_aggregate_summary_json: $systemverilog_parser_aggregate_summary_json,
            semantic_scope_contract_state_dir: $systemverilog_semantic_scope_contract_state_dir,
            semantic_scope_contract_summary_txt: $systemverilog_semantic_scope_contract_summary_txt,
            semantic_scope_contract_summary_json: $systemverilog_semantic_scope_contract_summary_json,
            formal_exhaustive_closure_state_dir: $systemverilog_formal_exhaustive_closure_state_dir,
            formal_exhaustive_closure_summary_txt: $systemverilog_formal_exhaustive_closure_summary_txt,
            formal_exhaustive_closure_summary_json: $systemverilog_formal_exhaustive_closure_summary_json
          },
          formal_exhaustive_closure: {
            gate: $systemverilog_formal_exhaustive_closure_gate,
            version: $systemverilog_formal_exhaustive_closure_gate_version,
            generated_at_utc: $systemverilog_formal_exhaustive_closure_generated_at_utc,
            primary_unmet_closure_criterion: $systemverilog_formal_exhaustive_closure_primary_unmet_closure_criterion,
            unmet_closure_criteria_count: $systemverilog_formal_exhaustive_closure_unmet_closure_criteria_count,
            state_dir: $systemverilog_formal_exhaustive_closure_state_dir,
            summary_txt: $systemverilog_formal_exhaustive_closure_summary_txt,
            summary_json: $systemverilog_formal_exhaustive_closure_summary_json
          },
          unmet_closure_criteria: $systemverilog_unmet_closure_criteria,
          unmet_closure_criteria_details: $systemverilog_unmet_closure_criteria_details
        },
        {
          family: "systemverilog_preprocessor",
          tracker_alignment_ok: $systemverilog_preprocessor_tracker_alignment_ok,
          false_criteria_count: $systemverilog_preprocessor_false_criteria_count,
          unmet_details_count: $systemverilog_preprocessor_unmet_details_count,
          primary_unmet_detail_criterion: $systemverilog_preprocessor_primary_unmet_detail_criterion,
          proof_surfaces: {
            aggregate_state_dir: $systemverilog_preprocessor_aggregate_state_dir,
            aggregate_summary_txt: $systemverilog_preprocessor_aggregate_summary_txt,
            aggregate_summary_json: $systemverilog_preprocessor_aggregate_summary_json,
            formal_exhaustive_closure_state_dir: $systemverilog_preprocessor_formal_exhaustive_closure_state_dir,
            formal_exhaustive_closure_summary_txt: $systemverilog_preprocessor_formal_exhaustive_closure_summary_txt,
            formal_exhaustive_closure_summary_json: $systemverilog_preprocessor_formal_exhaustive_closure_summary_json
          },
          formal_exhaustive_closure: {
            gate: $systemverilog_preprocessor_formal_exhaustive_closure_gate,
            version: $systemverilog_preprocessor_formal_exhaustive_closure_gate_version,
            generated_at_utc: $systemverilog_preprocessor_formal_exhaustive_closure_generated_at_utc,
            primary_unmet_closure_criterion: $systemverilog_preprocessor_formal_exhaustive_closure_primary_unmet_closure_criterion,
            unmet_closure_criteria_count: $systemverilog_preprocessor_formal_exhaustive_closure_unmet_closure_criteria_count,
            state_dir: $systemverilog_preprocessor_formal_exhaustive_closure_state_dir,
            summary_txt: $systemverilog_preprocessor_formal_exhaustive_closure_summary_txt,
            summary_json: $systemverilog_preprocessor_formal_exhaustive_closure_summary_json
          },
          unmet_closure_criteria: $systemverilog_preprocessor_unmet_closure_criteria,
          unmet_closure_criteria_details: $systemverilog_preprocessor_unmet_closure_criteria_details
        }
      ]
    }' >"$SUMMARY_JSON"

echo "✅ SV parser-family status contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
