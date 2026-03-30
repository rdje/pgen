#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR:-$RUST_DIR/target/sv_formal_exhaustive_closure_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

CONTRACT_FILE="${PGEN_SV_FORMAL_EXHAUSTIVE_CLOSURE_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_formal_exhaustive_closure_contract.json}"
SV_FAMILY_STATUS_GATE="$RUST_DIR/scripts/sv_parser_family_status_gate.sh"
SV_EXTERNAL_CORPUS_TRIAGE_GATE="$RUST_DIR/scripts/sv_external_corpus_triage_gate.sh"

EXISTING_FAMILY_STATUS_STATE_DIR="${PGEN_SV_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_FAMILY_STATUS_STATE_DIR:-}"
EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="${PGEN_SV_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-}"
SKIP_FAMILY_STATUS="${PGEN_SV_FORMAL_EXHAUSTIVE_CLOSURE_SKIP_FAMILY_STATUS:-0}"

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
require_file "$SV_FAMILY_STATUS_GATE"
require_file "$SV_EXTERNAL_CORPUS_TRIAGE_GATE"

jq -e '
    .family == "systemverilog"
    and ((.version | type) == "number")
    and ((.done_rule | type) == "string" and (.done_rule | length) > 0)
    and (.required_surface_key == "external_corpus_backed_proof_surface")
    and ((.required_surface_missing_detail | type) == "string" and (.required_surface_missing_detail | length) > 0)
' "$CONTRACT_FILE" >/dev/null

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

family_status_gate_name="sv_parser_family_status_gate"
family_status_gate_version=0
family_status_generated_at_utc="<skipped>"
family_status_state_dir="<skipped>"
family_status_summary_txt="<skipped>"
family_status_summary_json="<skipped>"
systemverilog_family_status="<skipped>"
systemverilog_family_status_tracker_alignment_ok=false
systemverilog_family_status_primary_unmet_closure_criterion="<skipped>"
systemverilog_family_status_closure_criteria_total_count=0
systemverilog_family_status_closure_criteria_satisfied_count=0
systemverilog_family_status_closure_criteria_unsatisfied_count=0

if [[ "$SKIP_FAMILY_STATUS" != "1" ]]; then
    family_status_state_dir="${EXISTING_FAMILY_STATUS_STATE_DIR:-$WORK_DIR/sv_parser_family_status_gate}"
    if [[ -z "$EXISTING_FAMILY_STATUS_STATE_DIR" ]]; then
        run_logged "sv_parser_family_status_gate" \
            env \
                PGEN_SV_FAMILY_STATUS_STATE_DIR="$family_status_state_dir" \
                "$SV_FAMILY_STATUS_GATE"
    else
        family_status_state_dir="$(cd "$family_status_state_dir" && pwd)"
    fi

    family_status_summary_txt="$family_status_state_dir/summary.txt"
    family_status_summary_json="$family_status_state_dir/summary.json"
    require_nonempty_file "$family_status_summary_txt"
    require_nonempty_file "$family_status_summary_json"

    family_status_gate_name="$(jq -r '.gate' "$family_status_summary_json")"
    family_status_gate_version="$(jq -r '.version' "$family_status_summary_json")"
    family_status_generated_at_utc="$(jq -r '.generated_at_utc' "$family_status_summary_json")"
    family_status_state_dir_from_json="$(jq -r '.state_dir' "$family_status_summary_json")"
    family_status_summary_txt_from_json="$(jq -r '.summary_txt' "$family_status_summary_json")"
    family_status_summary_json_from_json="$(jq -r '.summary_json' "$family_status_summary_json")"

    summary_family_status_state_dir="$(top_level_summary_value_from_txt "state_dir" "$family_status_summary_txt")"
    summary_family_status_generated_at_utc="$(top_level_summary_value_from_txt "generated_at_utc" "$family_status_summary_txt")"
    summary_family_status_summary_json="$(top_level_summary_value_from_txt "summary_json" "$family_status_summary_txt")"

    if [[ "$family_status_gate_name" != "sv_parser_family_status_gate" ]]; then
        echo "error: unexpected SV family-status gate identity '$family_status_gate_name'" >&2
        exit 1
    fi
    if [[ ! "$family_status_gate_version" =~ ^[0-9]+$ ]]; then
        echo "error: SV family-status gate version is not numeric: '$family_status_gate_version'" >&2
        exit 1
    fi
    if [[ -z "$family_status_generated_at_utc" ]]; then
        echo "error: SV family-status generated_at_utc is empty" >&2
        exit 1
    fi
    if [[ "$summary_family_status_state_dir" != "$family_status_state_dir" ]]; then
        echo "error: SV family-status state_dir mismatch in summary.txt" >&2
        exit 1
    fi
    if [[ "$family_status_state_dir_from_json" != "$family_status_state_dir" ]]; then
        echo "error: SV family-status state_dir mismatch in summary.json" >&2
        exit 1
    fi
    if [[ "$family_status_summary_txt_from_json" != "$family_status_summary_txt" ]]; then
        echo "error: SV family-status summary_txt mismatch in summary.json" >&2
        exit 1
    fi
    if [[ "$summary_family_status_summary_json" != "$family_status_summary_json" ]]; then
        echo "error: SV family-status summary_json mismatch in summary.txt" >&2
        exit 1
    fi
    if [[ "$family_status_summary_json_from_json" != "$family_status_summary_json" ]]; then
        echo "error: SV family-status summary_json mismatch in summary.json" >&2
        exit 1
    fi
    if [[ "$summary_family_status_generated_at_utc" != "$family_status_generated_at_utc" ]]; then
        echo "error: SV family-status generated_at_utc mismatch between summary.txt and summary.json" >&2
        exit 1
    fi

    systemverilog_family_status="$(summary_value_from_txt "systemverilog_status" "$family_status_summary_txt")"
    systemverilog_family_status_tracker_alignment_ok="$(summary_value_from_txt "systemverilog_tracker_alignment_ok" "$family_status_summary_txt")"
    systemverilog_family_status_primary_unmet_closure_criterion="$(summary_value_from_txt "systemverilog_primary_unmet_closure_criterion" "$family_status_summary_txt")"
    systemverilog_family_status_closure_criteria_total_count="$(summary_value_from_txt "systemverilog_closure_criteria_total_count" "$family_status_summary_txt")"
    systemverilog_family_status_closure_criteria_satisfied_count="$(summary_value_from_txt "systemverilog_closure_criteria_satisfied_count" "$family_status_summary_txt")"
    systemverilog_family_status_closure_criteria_unsatisfied_count="$(summary_value_from_txt "systemverilog_closure_criteria_unsatisfied_count" "$family_status_summary_txt")"
fi

external_corpus_backed_proof_state_dir="${EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-$WORK_DIR/sv_external_corpus_triage_gate}"
if [[ -z "$EXISTING_EXTERNAL_CORPUS_TRIAGE_STATE_DIR" ]]; then
    run_logged "sv_external_corpus_triage_gate" \
        env \
            PGEN_SV_EXTERNAL_CORPUS_TRIAGE_STATE_DIR="$external_corpus_backed_proof_state_dir" \
            "$SV_EXTERNAL_CORPUS_TRIAGE_GATE"
else
    external_corpus_backed_proof_state_dir="$(cd "$external_corpus_backed_proof_state_dir" && pwd)"
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

if [[ "$external_corpus_backed_proof_gate_name" != "sv_external_corpus_triage_gate" ]]; then
    echo "error: unexpected SV external-corpus gate identity '$external_corpus_backed_proof_gate_name'" >&2
    exit 1
fi
if [[ ! "$external_corpus_backed_proof_gate_version" =~ ^[0-9]+$ ]]; then
    echo "error: SV external-corpus gate version is not numeric: '$external_corpus_backed_proof_gate_version'" >&2
    exit 1
fi
if [[ -z "$external_corpus_backed_proof_generated_at_utc" ]]; then
    echo "error: SV external-corpus generated_at_utc is empty" >&2
    exit 1
fi
if [[ "$summary_external_corpus_backed_proof_state_dir" != "$external_corpus_backed_proof_state_dir" ]]; then
    echo "error: SV external-corpus state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$summary_external_corpus_backed_proof_summary_json" != "$external_corpus_backed_proof_summary_json" ]]; then
    echo "error: SV external-corpus summary_json mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$summary_external_corpus_backed_proof_generated_at_utc" != "$external_corpus_backed_proof_generated_at_utc" ]]; then
    echo "error: SV external-corpus generated_at_utc mismatch between summary.txt and summary.json" >&2
    exit 1
fi

external_corpus_backed_proof_cases_declared="$(summary_value_from_txt "cases_declared" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_cases_executed="$(summary_value_from_txt "cases_executed" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_cases_blocked_total="$(summary_value_from_txt "cases_blocked_total" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_preprocess_pass_total="$(summary_value_from_txt "preprocess_pass_total" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_preprocess_fail_total="$(summary_value_from_txt "preprocess_fail_total" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_parse_pass_total="$(summary_value_from_txt "parse_pass_total" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_parse_fail_total="$(summary_value_from_txt "parse_fail_total" "$external_corpus_backed_proof_summary_txt")"
external_corpus_backed_proof_primary_parse_failure_case="$(summary_value_from_txt "primary_parse_failure_case" "$external_corpus_backed_proof_summary_txt")"

required_surface_key="$(jq -r '.required_surface_key' "$CONTRACT_FILE")"
required_surface_missing_detail="$(jq -r '.required_surface_missing_detail' "$CONTRACT_FILE")"
done_rule="$(jq -r '.done_rule' "$CONTRACT_FILE")"

external_corpus_backed_proof_surface_present=true
systemverilog_formal_exhaustive_closure_surface_green=false
systemverilog_closure_criteria_total_count=1
systemverilog_closure_criteria_satisfied_count=0
if [[ "$external_corpus_backed_proof_surface_present" == true ]]; then
    systemverilog_closure_criteria_satisfied_count=1
    systemverilog_formal_exhaustive_closure_surface_green=true
fi
systemverilog_closure_criteria_unsatisfied_count=$((systemverilog_closure_criteria_total_count - systemverilog_closure_criteria_satisfied_count))

declare -a systemverilog_unmet=()
declare -a systemverilog_unmet_details=()
if [[ "$external_corpus_backed_proof_surface_present" != true ]]; then
    systemverilog_unmet+=("${required_surface_key}=missing")
    systemverilog_unmet_details+=("{\"criterion\":\"external_corpus_backed_proof_surface_present\",\"evidence_key\":\"${required_surface_key}\",\"observed\":\"missing\",\"expected\":\"present\",\"detail\":\"${required_surface_missing_detail}\"}")
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
systemverilog_unmet_count="${#systemverilog_unmet[@]}"
systemverilog_primary_unmet_closure_criterion="<none>"
if [[ "$systemverilog_unmet_count" -gt 0 ]]; then
    systemverilog_primary_unmet_closure_criterion="${systemverilog_unmet[0]}"
fi
systemverilog_unmet_json="$(printf '%s\n' "${systemverilog_unmet[@]:-}" | jq -R . | jq -sc 'map(select(length > 0))')"
systemverilog_unmet_details_json="$(printf '%s\n' "${systemverilog_unmet_details[@]:-}" | jq -R . | jq -sc 'map(select(length > 0) | fromjson)')"

{
    echo "SV Formal Exhaustive Closure Gate Summary"
    echo "gate: sv_formal_exhaustive_closure_gate"
    echo "version: 1"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "contract_file: $CONTRACT_FILE"
    echo "status_rule_done: $done_rule"
    echo "systemverilog_formal_exhaustive_required_surface_key: $required_surface_key"
    echo "systemverilog_formal_exhaustive_closure_surface_green: $systemverilog_formal_exhaustive_closure_surface_green"
    echo "systemverilog_unmet_closure_criteria_count: $systemverilog_unmet_count"
    echo "systemverilog_primary_unmet_closure_criterion: $systemverilog_primary_unmet_closure_criterion"
    echo "systemverilog_unmet_closure_criteria_json: $systemverilog_unmet_json"
    echo "systemverilog_unmet_closure_criteria_details_json: $systemverilog_unmet_details_json"
    echo "systemverilog_closure_criteria_total_count: $systemverilog_closure_criteria_total_count"
    echo "systemverilog_closure_criteria_satisfied_count: $systemverilog_closure_criteria_satisfied_count"
    echo "systemverilog_closure_criteria_unsatisfied_count: $systemverilog_closure_criteria_unsatisfied_count"
    echo "systemverilog_external_corpus_backed_proof_surface_present: $external_corpus_backed_proof_surface_present"
    echo "systemverilog_family_status: $systemverilog_family_status"
    echo "systemverilog_family_status_tracker_alignment_ok: $systemverilog_family_status_tracker_alignment_ok"
    echo "systemverilog_family_status_closure_criteria_total_count: $systemverilog_family_status_closure_criteria_total_count"
    echo "systemverilog_family_status_closure_criteria_satisfied_count: $systemverilog_family_status_closure_criteria_satisfied_count"
    echo "systemverilog_family_status_closure_criteria_unsatisfied_count: $systemverilog_family_status_closure_criteria_unsatisfied_count"
    echo "systemverilog_family_status_primary_unmet_closure_criterion: $systemverilog_family_status_primary_unmet_closure_criterion"
    echo "systemverilog_family_status_gate: $family_status_gate_name"
    echo "systemverilog_family_status_gate_version: $family_status_gate_version"
    echo "systemverilog_family_status_generated_at_utc: $family_status_generated_at_utc"
    echo "systemverilog_family_status_state_dir: $family_status_state_dir"
    echo "systemverilog_family_status_summary_txt: $family_status_summary_txt"
    echo "systemverilog_family_status_summary_json: $family_status_summary_json"
    echo "systemverilog_external_corpus_backed_proof_state_dir: $external_corpus_backed_proof_state_dir"
    echo "systemverilog_external_corpus_backed_proof_summary_txt: $external_corpus_backed_proof_summary_txt"
    echo "systemverilog_external_corpus_backed_proof_summary_json: $external_corpus_backed_proof_summary_json"
    echo "systemverilog_external_corpus_backed_proof_gate: $external_corpus_backed_proof_gate_name"
    echo "systemverilog_external_corpus_backed_proof_gate_version: $external_corpus_backed_proof_gate_version"
    echo "systemverilog_external_corpus_backed_proof_generated_at_utc: $external_corpus_backed_proof_generated_at_utc"
    echo "systemverilog_external_corpus_backed_proof_cases_declared: $external_corpus_backed_proof_cases_declared"
    echo "systemverilog_external_corpus_backed_proof_cases_executed: $external_corpus_backed_proof_cases_executed"
    echo "systemverilog_external_corpus_backed_proof_cases_blocked_total: $external_corpus_backed_proof_cases_blocked_total"
    echo "systemverilog_external_corpus_backed_proof_preprocess_pass_total: $external_corpus_backed_proof_preprocess_pass_total"
    echo "systemverilog_external_corpus_backed_proof_preprocess_fail_total: $external_corpus_backed_proof_preprocess_fail_total"
    echo "systemverilog_external_corpus_backed_proof_parse_pass_total: $external_corpus_backed_proof_parse_pass_total"
    echo "systemverilog_external_corpus_backed_proof_parse_fail_total: $external_corpus_backed_proof_parse_fail_total"
    echo "systemverilog_external_corpus_backed_proof_primary_parse_failure_case: $external_corpus_backed_proof_primary_parse_failure_case"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "sv_formal_exhaustive_closure_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg contract_file "$CONTRACT_FILE" \
    --arg status_rule_done "$done_rule" \
    --arg systemverilog_formal_exhaustive_required_surface_key "$required_surface_key" \
    --argjson systemverilog_formal_exhaustive_closure_surface_green "$systemverilog_formal_exhaustive_closure_surface_green" \
    --arg systemverilog_primary_unmet_closure_criterion "$systemverilog_primary_unmet_closure_criterion" \
    --argjson systemverilog_unmet_closure_criteria_count "$systemverilog_unmet_count" \
    --argjson systemverilog_unmet_closure_criteria "$systemverilog_unmet_json" \
    --argjson systemverilog_unmet_closure_criteria_details "$systemverilog_unmet_details_json" \
    --argjson systemverilog_closure_criteria_total_count "$systemverilog_closure_criteria_total_count" \
    --argjson systemverilog_closure_criteria_satisfied_count "$systemverilog_closure_criteria_satisfied_count" \
    --argjson systemverilog_closure_criteria_unsatisfied_count "$systemverilog_closure_criteria_unsatisfied_count" \
    --argjson systemverilog_external_corpus_backed_proof_surface_present "$external_corpus_backed_proof_surface_present" \
    --arg systemverilog_family_status "$systemverilog_family_status" \
    --argjson systemverilog_family_status_tracker_alignment_ok "$systemverilog_family_status_tracker_alignment_ok" \
    --argjson systemverilog_family_status_closure_criteria_total_count "$systemverilog_family_status_closure_criteria_total_count" \
    --argjson systemverilog_family_status_closure_criteria_satisfied_count "$systemverilog_family_status_closure_criteria_satisfied_count" \
    --argjson systemverilog_family_status_closure_criteria_unsatisfied_count "$systemverilog_family_status_closure_criteria_unsatisfied_count" \
    --arg systemverilog_family_status_primary_unmet_closure_criterion "$systemverilog_family_status_primary_unmet_closure_criterion" \
    --arg systemverilog_family_status_gate "$family_status_gate_name" \
    --argjson systemverilog_family_status_gate_version "$family_status_gate_version" \
    --arg systemverilog_family_status_generated_at_utc "$family_status_generated_at_utc" \
    --arg systemverilog_family_status_state_dir "$family_status_state_dir" \
    --arg systemverilog_family_status_summary_txt "$family_status_summary_txt" \
    --arg systemverilog_family_status_summary_json "$family_status_summary_json" \
    --arg systemverilog_external_corpus_backed_proof_state_dir "$external_corpus_backed_proof_state_dir" \
    --arg systemverilog_external_corpus_backed_proof_summary_txt "$external_corpus_backed_proof_summary_txt" \
    --arg systemverilog_external_corpus_backed_proof_summary_json "$external_corpus_backed_proof_summary_json" \
    --arg systemverilog_external_corpus_backed_proof_gate "$external_corpus_backed_proof_gate_name" \
    --argjson systemverilog_external_corpus_backed_proof_gate_version "$external_corpus_backed_proof_gate_version" \
    --arg systemverilog_external_corpus_backed_proof_generated_at_utc "$external_corpus_backed_proof_generated_at_utc" \
    --argjson systemverilog_external_corpus_backed_proof_cases_declared "$external_corpus_backed_proof_cases_declared" \
    --argjson systemverilog_external_corpus_backed_proof_cases_executed "$external_corpus_backed_proof_cases_executed" \
    --argjson systemverilog_external_corpus_backed_proof_cases_blocked_total "$external_corpus_backed_proof_cases_blocked_total" \
    --argjson systemverilog_external_corpus_backed_proof_preprocess_pass_total "$external_corpus_backed_proof_preprocess_pass_total" \
    --argjson systemverilog_external_corpus_backed_proof_preprocess_fail_total "$external_corpus_backed_proof_preprocess_fail_total" \
    --argjson systemverilog_external_corpus_backed_proof_parse_pass_total "$external_corpus_backed_proof_parse_pass_total" \
    --argjson systemverilog_external_corpus_backed_proof_parse_fail_total "$external_corpus_backed_proof_parse_fail_total" \
    --arg systemverilog_external_corpus_backed_proof_primary_parse_failure_case "$external_corpus_backed_proof_primary_parse_failure_case" \
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
          family: "systemverilog",
          formal_exhaustive_closure_surface_green: $systemverilog_formal_exhaustive_closure_surface_green,
          required_surface_key: $systemverilog_formal_exhaustive_required_surface_key,
          primary_unmet_closure_criterion: $systemverilog_primary_unmet_closure_criterion,
          unmet_closure_criteria_count: $systemverilog_unmet_closure_criteria_count,
          unmet_closure_criteria: $systemverilog_unmet_closure_criteria,
          unmet_closure_criteria_details: $systemverilog_unmet_closure_criteria_details,
          closure_criteria_total_count: $systemverilog_closure_criteria_total_count,
          closure_criteria_satisfied_count: $systemverilog_closure_criteria_satisfied_count,
          closure_criteria_unsatisfied_count: $systemverilog_closure_criteria_unsatisfied_count,
          criteria: {
            external_corpus_backed_proof_surface_present: $systemverilog_external_corpus_backed_proof_surface_present
          },
          metrics: {
            family_status: $systemverilog_family_status,
            family_status_tracker_alignment_ok: $systemverilog_family_status_tracker_alignment_ok,
            family_status_closure_criteria_total_count: $systemverilog_family_status_closure_criteria_total_count,
            family_status_closure_criteria_satisfied_count: $systemverilog_family_status_closure_criteria_satisfied_count,
            family_status_closure_criteria_unsatisfied_count: $systemverilog_family_status_closure_criteria_unsatisfied_count,
            family_status_primary_unmet_closure_criterion: $systemverilog_family_status_primary_unmet_closure_criterion,
            family_status_gate: $systemverilog_family_status_gate,
            family_status_gate_version: $systemverilog_family_status_gate_version,
            family_status_generated_at_utc: $systemverilog_family_status_generated_at_utc,
            external_corpus_backed_proof_gate: $systemverilog_external_corpus_backed_proof_gate,
            external_corpus_backed_proof_gate_version: $systemverilog_external_corpus_backed_proof_gate_version,
            external_corpus_backed_proof_generated_at_utc: $systemverilog_external_corpus_backed_proof_generated_at_utc,
            external_corpus_backed_proof_cases_declared: $systemverilog_external_corpus_backed_proof_cases_declared,
            external_corpus_backed_proof_cases_executed: $systemverilog_external_corpus_backed_proof_cases_executed,
            external_corpus_backed_proof_cases_blocked_total: $systemverilog_external_corpus_backed_proof_cases_blocked_total,
            external_corpus_backed_proof_preprocess_pass_total: $systemverilog_external_corpus_backed_proof_preprocess_pass_total,
            external_corpus_backed_proof_preprocess_fail_total: $systemverilog_external_corpus_backed_proof_preprocess_fail_total,
            external_corpus_backed_proof_parse_pass_total: $systemverilog_external_corpus_backed_proof_parse_pass_total,
            external_corpus_backed_proof_parse_fail_total: $systemverilog_external_corpus_backed_proof_parse_fail_total,
            external_corpus_backed_proof_primary_parse_failure_case: $systemverilog_external_corpus_backed_proof_primary_parse_failure_case
          },
          proof_surfaces: {
            family_status_state_dir: $systemverilog_family_status_state_dir,
            family_status_summary_txt: $systemverilog_family_status_summary_txt,
            family_status_summary_json: $systemverilog_family_status_summary_json,
            external_corpus_backed_proof_state_dir: $systemverilog_external_corpus_backed_proof_state_dir,
            external_corpus_backed_proof_summary_txt: $systemverilog_external_corpus_backed_proof_summary_txt,
            external_corpus_backed_proof_summary_json: $systemverilog_external_corpus_backed_proof_summary_json
          }
        }
      ]
    }' >"$SUMMARY_JSON"

require_nonempty_file "$SUMMARY_JSON"

echo "✅ SV formal exhaustive closure gate passed."
