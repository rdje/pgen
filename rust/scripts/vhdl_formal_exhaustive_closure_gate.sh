#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_VHDL_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR:-$RUST_DIR/target/vhdl_formal_exhaustive_closure_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

CONTRACT_FILE="${PGEN_VHDL_FORMAL_EXHAUSTIVE_CLOSURE_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/vhdl_formal_exhaustive_closure_contract.json}"
VHDL_FAMILY_CONTRACT_GATE="$RUST_DIR/scripts/vhdl_parser_family_contract_gate.sh"
VHDL_EXTERNAL_CORPUS_TRIAGE_GATE="$RUST_DIR/scripts/vhdl_external_corpus_triage_gate.sh"

EXISTING_FAMILY_CONTRACT_STATE_DIR="${PGEN_VHDL_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_FAMILY_CONTRACT_STATE_DIR:-}"
EXISTING_QUALITY_STATE_DIR="${PGEN_VHDL_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_QUALITY_STATE_DIR:-}"
EXISTING_STRICT_PROMOTION_STATE_DIR="${PGEN_VHDL_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_STRICT_PROMOTION_STATE_DIR:-}"
EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="${PGEN_VHDL_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-}"

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
require_file "$CONTRACT_FILE"
require_file "$VHDL_FAMILY_CONTRACT_GATE"
require_file "$VHDL_EXTERNAL_CORPUS_TRIAGE_GATE"

jq -e '
    .family == "vhdl"
    and ((.version | type) == "number")
    and ((.done_rule | type) == "string" and (.done_rule | length) > 0)
    and (.required_surface_key == "external_corpus_backed_proof_surface")
    and ((.required_surface_missing_detail | type) == "string" and (.required_surface_missing_detail | length) > 0)
' "$CONTRACT_FILE" >/dev/null

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

family_contract_state_dir="${EXISTING_FAMILY_CONTRACT_STATE_DIR:-$WORK_DIR/vhdl_parser_family_contract_gate}"

if [[ -z "$EXISTING_FAMILY_CONTRACT_STATE_DIR" ]]; then
    vhdl_family_contract_cmd=(
        env
        PGEN_VHDL_FAMILY_CONTRACT_STATE_DIR="$family_contract_state_dir"
    )
    if [[ -n "$EXISTING_QUALITY_STATE_DIR" ]]; then
        vhdl_family_contract_cmd+=(
            PGEN_VHDL_FAMILY_CONTRACT_EXISTING_QUALITY_STATE_DIR="$EXISTING_QUALITY_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_STRICT_PROMOTION_STATE_DIR" ]]; then
        vhdl_family_contract_cmd+=(
            PGEN_VHDL_FAMILY_CONTRACT_EXISTING_STRICT_PROMOTION_STATE_DIR="$EXISTING_STRICT_PROMOTION_STATE_DIR"
        )
    fi
    vhdl_family_contract_cmd+=("$VHDL_FAMILY_CONTRACT_GATE")
    run_logged "vhdl_parser_family_contract_gate" "${vhdl_family_contract_cmd[@]}"
fi

family_contract_summary_txt="$family_contract_state_dir/summary.txt"
family_contract_summary_json="$family_contract_state_dir/summary.json"

require_nonempty_file "$family_contract_summary_txt"
require_nonempty_file "$family_contract_summary_json"

family_contract_gate_name="$(jq -r '.gate' "$family_contract_summary_json")"
family_contract_gate_version="$(jq -r '.version' "$family_contract_summary_json")"
family_contract_generated_at_utc="$(jq -r '.generated_at_utc' "$family_contract_summary_json")"
family_contract_state_dir_from_json="$(jq -r '.state_dir' "$family_contract_summary_json")"
family_contract_summary_txt_from_json="$(jq -r '.summary_txt' "$family_contract_summary_json")"
family_contract_summary_json_from_json="$(jq -r '.summary_json' "$family_contract_summary_json")"

summary_family_contract_state_dir="$(top_level_summary_value_from_txt "state_dir" "$family_contract_summary_txt")"
summary_family_contract_generated_at_utc="$(top_level_summary_value_from_txt "generated_at_utc" "$family_contract_summary_txt")"
summary_family_contract_summary_json="$(top_level_summary_value_from_txt "summary_json" "$family_contract_summary_txt")"

if [[ "$family_contract_gate_name" != "vhdl_parser_family_contract_gate" ]]; then
    echo "error: unexpected VHDL family-contract gate identity '$family_contract_gate_name'" >&2
    exit 1
fi
if [[ ! "$family_contract_gate_version" =~ ^[0-9]+$ ]]; then
    echo "error: VHDL family-contract gate version is not numeric: '$family_contract_gate_version'" >&2
    exit 1
fi
if [[ -z "$family_contract_generated_at_utc" ]]; then
    echo "error: VHDL family-contract generated_at_utc is empty" >&2
    exit 1
fi
if [[ "$summary_family_contract_state_dir" != "$family_contract_state_dir" ]]; then
    echo "error: VHDL family-contract state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$family_contract_state_dir_from_json" != "$family_contract_state_dir" ]]; then
    echo "error: VHDL family-contract state_dir mismatch in summary.json" >&2
    exit 1
fi
if [[ "$family_contract_summary_txt_from_json" != "$family_contract_summary_txt" ]]; then
    echo "error: VHDL family-contract summary_txt mismatch in summary.json" >&2
    exit 1
fi
if [[ "$summary_family_contract_summary_json" != "$family_contract_summary_json" ]]; then
    echo "error: VHDL family-contract summary_json mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$family_contract_summary_json_from_json" != "$family_contract_summary_json" ]]; then
    echo "error: VHDL family-contract summary_json mismatch in summary.json" >&2
    exit 1
fi
if [[ "$summary_family_contract_generated_at_utc" != "$family_contract_generated_at_utc" ]]; then
    echo "error: VHDL family-contract generated_at_utc mismatch between summary.txt and summary.json" >&2
    exit 1
fi

required_surface_key="$(jq -r '.required_surface_key' "$CONTRACT_FILE")"
required_surface_missing_detail="$(jq -r '.required_surface_missing_detail' "$CONTRACT_FILE")"
done_rule="$(jq -r '.done_rule' "$CONTRACT_FILE")"

vhdl_family_contract_green=true
vhdl_quality_closed_loop_replay_targets="$(summary_value_from_txt "quality_closed_loop_replay_targets" "$family_contract_summary_txt")"
vhdl_quality_parseability_generation_parser_rejections_total="$(summary_value_from_txt "quality_parseability_generation_parser_rejections_total" "$family_contract_summary_txt")"
vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total="$(summary_value_from_txt "quality_closed_loop_parseability_shadow_parser_rejections_total" "$family_contract_summary_txt")"
vhdl_strict_promotion_recommendation="$(summary_value_from_txt "strict_promotion_recommendation" "$family_contract_summary_txt")"
vhdl_strict_promotion_eligible_for_required_strict_mode="$(summary_value_from_txt "strict_promotion_eligible_for_required_strict_mode" "$family_contract_summary_txt")"
vhdl_strict_promotion_primary_blocker="$(summary_value_from_txt "strict_promotion_primary_blocker" "$family_contract_summary_txt")"

external_corpus_backed_proof_state_dir="${EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-$WORK_DIR/vhdl_external_corpus_triage_gate}"
if [[ -z "$EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR" ]]; then
    vhdl_external_corpus_triage_cmd=(
        env
        PGEN_VHDL_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="$external_corpus_backed_proof_state_dir"
        "$VHDL_EXTERNAL_CORPUS_TRIAGE_GATE"
    )
    run_logged "vhdl_external_corpus_triage_gate" "${vhdl_external_corpus_triage_cmd[@]}"
fi

external_corpus_backed_proof_summary_txt="$external_corpus_backed_proof_state_dir/summary.txt"
external_corpus_backed_proof_summary_json="$external_corpus_backed_proof_state_dir/summary.json"

require_nonempty_file "$external_corpus_backed_proof_summary_txt"
require_nonempty_file "$external_corpus_backed_proof_summary_json"

external_corpus_backed_proof_gate_name="$(jq -r '.gate' "$external_corpus_backed_proof_summary_json")"
external_corpus_backed_proof_gate_version="$(jq -r '.version' "$external_corpus_backed_proof_summary_json")"
external_corpus_backed_proof_generated_at_utc="$(jq -r '.generated_at_utc' "$external_corpus_backed_proof_summary_json")"

summary_external_corpus_backed_proof_state_dir="$(top_level_summary_value_from_txt "state_dir" "$external_corpus_backed_proof_summary_txt")"
summary_external_corpus_backed_proof_generated_at_utc="$(top_level_summary_value_from_txt "generated_at_utc" "$external_corpus_backed_proof_summary_txt")"
summary_external_corpus_backed_proof_summary_json="$(top_level_summary_value_from_txt "summary_json" "$external_corpus_backed_proof_summary_txt")"

if [[ "$external_corpus_backed_proof_gate_name" != "vhdl_external_corpus_triage_gate" ]]; then
    echo "error: unexpected VHDL external-corpus gate identity '$external_corpus_backed_proof_gate_name'" >&2
    exit 1
fi
if [[ ! "$external_corpus_backed_proof_gate_version" =~ ^[0-9]+$ ]]; then
    echo "error: VHDL external-corpus gate version is not numeric: '$external_corpus_backed_proof_gate_version'" >&2
    exit 1
fi
if [[ -z "$external_corpus_backed_proof_generated_at_utc" ]]; then
    echo "error: VHDL external-corpus generated_at_utc is empty" >&2
    exit 1
fi
if [[ "$summary_external_corpus_backed_proof_state_dir" != "$external_corpus_backed_proof_state_dir" ]]; then
    echo "error: VHDL external-corpus state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$summary_external_corpus_backed_proof_summary_json" != "$external_corpus_backed_proof_summary_json" ]]; then
    echo "error: VHDL external-corpus summary_json mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$summary_external_corpus_backed_proof_generated_at_utc" != "$external_corpus_backed_proof_generated_at_utc" ]]; then
    echo "error: VHDL external-corpus generated_at_utc mismatch between summary.txt and summary.json" >&2
    exit 1
fi

external_corpus_backed_proof_cases_declared="$(summary_value_from_txt "cases_declared" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_cases_executed="$(summary_value_from_txt "cases_executed" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_parse_pass_total="$(summary_value_from_txt "parse_pass_total" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_parse_fail_total="$(summary_value_from_txt "parse_fail_total" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_primary_parse_failure_case="$(summary_value_from_txt "primary_parse_failure_case" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_primary_parse_failure_corpus="$(summary_value_from_txt "primary_parse_failure_corpus" "$external_corpus_backed_proof_summary_txt")"

external_corpus_backed_proof_surface_present=true

vhdl_formal_exhaustive_closure_surface_green=false
vhdl_closure_criteria_total_count=1
vhdl_closure_criteria_satisfied_count=0
if [[ "$external_corpus_backed_proof_surface_present" == true ]]; then
    vhdl_closure_criteria_satisfied_count=1
    vhdl_formal_exhaustive_closure_surface_green=true
fi
vhdl_closure_criteria_unsatisfied_count=$((vhdl_closure_criteria_total_count - vhdl_closure_criteria_satisfied_count))

declare -a vhdl_unmet=()
declare -a vhdl_unmet_details=()
if [[ "$external_corpus_backed_proof_surface_present" != true ]]; then
    vhdl_unmet+=("${required_surface_key}=missing")
    vhdl_unmet_details+=("{\"criterion\":\"external_corpus_backed_proof_surface_present\",\"evidence_key\":\"${required_surface_key}\",\"observed\":\"missing\",\"expected\":\"present\",\"detail\":\"${required_surface_missing_detail}\"}")
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
    echo "VHDL Formal Exhaustive Closure Gate Summary"
    echo "gate: vhdl_formal_exhaustive_closure_gate"
    echo "version: 1"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "contract_file: $CONTRACT_FILE"
    echo "status_rule_done: $done_rule"
    echo "vhdl_formal_exhaustive_required_surface_key: $required_surface_key"
    echo "vhdl_formal_exhaustive_closure_surface_green: $vhdl_formal_exhaustive_closure_surface_green"
    echo "vhdl_unmet_closure_criteria_count: $vhdl_unmet_count"
    echo "vhdl_primary_unmet_closure_criterion: $vhdl_primary_unmet_closure_criterion"
    echo "vhdl_unmet_closure_criteria_json: $vhdl_unmet_json"
    echo "vhdl_unmet_closure_criteria_details_json: $vhdl_unmet_details_json"
    echo "vhdl_closure_criteria_total_count: $vhdl_closure_criteria_total_count"
    echo "vhdl_closure_criteria_satisfied_count: $vhdl_closure_criteria_satisfied_count"
    echo "vhdl_closure_criteria_unsatisfied_count: $vhdl_closure_criteria_unsatisfied_count"
    echo "vhdl_external_corpus_backed_proof_surface_present: $external_corpus_backed_proof_surface_present"
    echo "vhdl_family_contract_green: $vhdl_family_contract_green"
    echo "vhdl_quality_parseability_generation_parser_rejections_total: $vhdl_quality_parseability_generation_parser_rejections_total"
    echo "vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total: $vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total"
    echo "vhdl_quality_closed_loop_replay_targets: $vhdl_quality_closed_loop_replay_targets"
    echo "vhdl_strict_promotion_recommendation: $vhdl_strict_promotion_recommendation"
    echo "vhdl_strict_promotion_eligible_for_required_strict_mode: $vhdl_strict_promotion_eligible_for_required_strict_mode"
    echo "vhdl_strict_promotion_primary_blocker: $vhdl_strict_promotion_primary_blocker"
    echo "vhdl_family_contract_gate: $family_contract_gate_name"
    echo "vhdl_family_contract_gate_version: $family_contract_gate_version"
    echo "vhdl_family_contract_generated_at_utc: $family_contract_generated_at_utc"
    echo "vhdl_family_contract_state_dir: $family_contract_state_dir"
    echo "vhdl_family_contract_summary_txt: $family_contract_summary_txt"
    echo "vhdl_family_contract_summary_json: $family_contract_summary_json"
    echo "vhdl_external_corpus_backed_proof_state_dir: $external_corpus_backed_proof_state_dir"
    echo "vhdl_external_corpus_backed_proof_summary_txt: $external_corpus_backed_proof_summary_txt"
    echo "vhdl_external_corpus_backed_proof_summary_json: $external_corpus_backed_proof_summary_json"
    echo "vhdl_external_corpus_backed_proof_gate: $external_corpus_backed_proof_gate_name"
    echo "vhdl_external_corpus_backed_proof_gate_version: $external_corpus_backed_proof_gate_version"
    echo "vhdl_external_corpus_backed_proof_generated_at_utc: $external_corpus_backed_proof_generated_at_utc"
    echo "vhdl_external_corpus_backed_proof_cases_declared: $external_corpus_backed_proof_cases_declared"
    echo "vhdl_external_corpus_backed_proof_cases_executed: $external_corpus_backed_proof_cases_executed"
    echo "vhdl_external_corpus_backed_proof_parse_pass_total: $external_corpus_backed_proof_parse_pass_total"
    echo "vhdl_external_corpus_backed_proof_parse_fail_total: $external_corpus_backed_proof_parse_fail_total"
    echo "vhdl_external_corpus_backed_proof_primary_parse_failure_case: $external_corpus_backed_proof_primary_parse_failure_case"
    echo "vhdl_external_corpus_backed_proof_primary_parse_failure_corpus: $external_corpus_backed_proof_primary_parse_failure_corpus"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "vhdl_formal_exhaustive_closure_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg contract_file "$CONTRACT_FILE" \
    --arg status_rule_done "$done_rule" \
    --arg vhdl_formal_exhaustive_required_surface_key "$required_surface_key" \
    --argjson vhdl_formal_exhaustive_closure_surface_green "$vhdl_formal_exhaustive_closure_surface_green" \
    --arg vhdl_primary_unmet_closure_criterion "$vhdl_primary_unmet_closure_criterion" \
    --argjson vhdl_unmet_closure_criteria_count "$vhdl_unmet_count" \
    --argjson vhdl_unmet_closure_criteria "$vhdl_unmet_json" \
    --argjson vhdl_unmet_closure_criteria_details "$vhdl_unmet_details_json" \
    --argjson vhdl_closure_criteria_total_count "$vhdl_closure_criteria_total_count" \
    --argjson vhdl_closure_criteria_satisfied_count "$vhdl_closure_criteria_satisfied_count" \
    --argjson vhdl_closure_criteria_unsatisfied_count "$vhdl_closure_criteria_unsatisfied_count" \
    --argjson vhdl_external_corpus_backed_proof_surface_present "$external_corpus_backed_proof_surface_present" \
    --argjson vhdl_family_contract_green "$vhdl_family_contract_green" \
    --argjson vhdl_quality_parseability_generation_parser_rejections_total "$vhdl_quality_parseability_generation_parser_rejections_total" \
    --argjson vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total "$vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total" \
    --argjson vhdl_quality_closed_loop_replay_targets "$vhdl_quality_closed_loop_replay_targets" \
    --arg vhdl_strict_promotion_recommendation "$vhdl_strict_promotion_recommendation" \
    --argjson vhdl_strict_promotion_eligible_for_required_strict_mode "$vhdl_strict_promotion_eligible_for_required_strict_mode" \
    --arg vhdl_strict_promotion_primary_blocker "$vhdl_strict_promotion_primary_blocker" \
    --arg vhdl_family_contract_gate "$family_contract_gate_name" \
    --argjson vhdl_family_contract_gate_version "$family_contract_gate_version" \
    --arg vhdl_family_contract_generated_at_utc "$family_contract_generated_at_utc" \
    --arg vhdl_family_contract_state_dir "$family_contract_state_dir" \
    --arg vhdl_family_contract_summary_txt "$family_contract_summary_txt" \
    --arg vhdl_family_contract_summary_json "$family_contract_summary_json" \
    --arg vhdl_external_corpus_backed_proof_state_dir "$external_corpus_backed_proof_state_dir" \
    --arg vhdl_external_corpus_backed_proof_summary_txt "$external_corpus_backed_proof_summary_txt" \
    --arg vhdl_external_corpus_backed_proof_summary_json "$external_corpus_backed_proof_summary_json" \
    --arg vhdl_external_corpus_backed_proof_gate "$external_corpus_backed_proof_gate_name" \
    --argjson vhdl_external_corpus_backed_proof_gate_version "$external_corpus_backed_proof_gate_version" \
    --arg vhdl_external_corpus_backed_proof_generated_at_utc "$external_corpus_backed_proof_generated_at_utc" \
    --argjson vhdl_external_corpus_backed_proof_cases_declared "$external_corpus_backed_proof_cases_declared" \
    --argjson vhdl_external_corpus_backed_proof_cases_executed "$external_corpus_backed_proof_cases_executed" \
    --argjson vhdl_external_corpus_backed_proof_parse_pass_total "$external_corpus_backed_proof_parse_pass_total" \
    --argjson vhdl_external_corpus_backed_proof_parse_fail_total "$external_corpus_backed_proof_parse_fail_total" \
    --arg vhdl_external_corpus_backed_proof_primary_parse_failure_case "$external_corpus_backed_proof_primary_parse_failure_case" \
    --arg vhdl_external_corpus_backed_proof_primary_parse_failure_corpus "$external_corpus_backed_proof_primary_parse_failure_corpus" \
    '{
      gate: $gate,
      version: $version,
      generated_at_utc: $generated_at_utc,
      state_dir: $state_dir,
      summary_txt: $summary_txt,
      summary_json: $summary_json,
      contract_file: $contract_file,
      status_rule_done: $status_rule_done,
      families: [
        {
          family: "vhdl",
          formal_exhaustive_closure_surface_green: $vhdl_formal_exhaustive_closure_surface_green,
          required_surface_key: $vhdl_formal_exhaustive_required_surface_key,
          primary_unmet_closure_criterion: $vhdl_primary_unmet_closure_criterion,
          unmet_closure_criteria_count: $vhdl_unmet_closure_criteria_count,
          unmet_closure_criteria: $vhdl_unmet_closure_criteria,
          unmet_closure_criteria_details: $vhdl_unmet_closure_criteria_details,
          closure_criteria_total_count: $vhdl_closure_criteria_total_count,
          closure_criteria_satisfied_count: $vhdl_closure_criteria_satisfied_count,
          closure_criteria_unsatisfied_count: $vhdl_closure_criteria_unsatisfied_count,
          criteria: {
            external_corpus_backed_proof_surface_present: $vhdl_external_corpus_backed_proof_surface_present
          },
          metrics: {
            family_contract_green: $vhdl_family_contract_green,
            quality_parseability_generation_parser_rejections_total: $vhdl_quality_parseability_generation_parser_rejections_total,
            quality_closed_loop_parseability_shadow_parser_rejections_total: $vhdl_quality_closed_loop_parseability_shadow_parser_rejections_total,
            quality_closed_loop_replay_targets: $vhdl_quality_closed_loop_replay_targets,
            strict_promotion_recommendation: $vhdl_strict_promotion_recommendation,
            strict_promotion_eligible_for_required_strict_mode: $vhdl_strict_promotion_eligible_for_required_strict_mode,
            strict_promotion_primary_blocker: $vhdl_strict_promotion_primary_blocker,
            family_contract_gate: $vhdl_family_contract_gate,
            family_contract_gate_version: $vhdl_family_contract_gate_version,
            family_contract_generated_at_utc: $vhdl_family_contract_generated_at_utc,
            external_corpus_backed_proof_gate: $vhdl_external_corpus_backed_proof_gate,
            external_corpus_backed_proof_gate_version: $vhdl_external_corpus_backed_proof_gate_version,
            external_corpus_backed_proof_generated_at_utc: $vhdl_external_corpus_backed_proof_generated_at_utc,
            external_corpus_backed_proof_cases_declared: $vhdl_external_corpus_backed_proof_cases_declared,
            external_corpus_backed_proof_cases_executed: $vhdl_external_corpus_backed_proof_cases_executed,
            external_corpus_backed_proof_parse_pass_total: $vhdl_external_corpus_backed_proof_parse_pass_total,
            external_corpus_backed_proof_parse_fail_total: $vhdl_external_corpus_backed_proof_parse_fail_total,
            external_corpus_backed_proof_primary_parse_failure_case: $vhdl_external_corpus_backed_proof_primary_parse_failure_case,
            external_corpus_backed_proof_primary_parse_failure_corpus: $vhdl_external_corpus_backed_proof_primary_parse_failure_corpus
          },
          proof_surfaces: {
            family_contract_state_dir: $vhdl_family_contract_state_dir,
            family_contract_summary_txt: $vhdl_family_contract_summary_txt,
            family_contract_summary_json: $vhdl_family_contract_summary_json,
            external_corpus_backed_proof_state_dir: $vhdl_external_corpus_backed_proof_state_dir,
            external_corpus_backed_proof_summary_txt: $vhdl_external_corpus_backed_proof_summary_txt,
            external_corpus_backed_proof_summary_json: $vhdl_external_corpus_backed_proof_summary_json
          }
        }
      ]
    }' >"$SUMMARY_JSON"

require_nonempty_file "$SUMMARY_JSON"

echo "✅ VHDL formal exhaustive closure gate passed."
