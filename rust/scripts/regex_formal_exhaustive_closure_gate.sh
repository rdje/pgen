#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_REGEX_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR:-$RUST_DIR/target/regex_formal_exhaustive_closure_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

CONTRACT_FILE="${PGEN_REGEX_FORMAL_EXHAUSTIVE_CLOSURE_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/regex_formal_exhaustive_closure_contract.json}"
REGEX_FAMILY_CONTRACT_GATE="$RUST_DIR/scripts/regex_parser_family_contract_gate.sh"

EXISTING_FAMILY_CONTRACT_STATE_DIR="${PGEN_REGEX_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_FAMILY_CONTRACT_STATE_DIR:-}"
EXISTING_FRONTEND_STATE_DIR="${PGEN_REGEX_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_FRONTEND_STATE_DIR:-}"
EXISTING_DUAL_RUN_STATE_DIR="${PGEN_REGEX_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_DUAL_RUN_STATE_DIR:-}"
EXISTING_STIMULI_STATE_DIR="${PGEN_REGEX_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_STIMULI_STATE_DIR:-}"
EXISTING_BROADER_CORPUS_PROOF_STATE_DIR="${PGEN_REGEX_FORMAL_EXHAUSTIVE_CLOSURE_EXISTING_BROADER_CORPUS_PROOF_STATE_DIR:-}"

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
require_file "$REGEX_FAMILY_CONTRACT_GATE"

jq -e '
    .family == "regex"
    and ((.version | type) == "number")
    and ((.done_rule | type) == "string" and (.done_rule | length) > 0)
    and (.required_surface_key == "broader_corpus_backed_proof_surface")
    and ((.required_surface_missing_detail | type) == "string" and (.required_surface_missing_detail | length) > 0)
' "$CONTRACT_FILE" >/dev/null

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

family_contract_state_dir="${EXISTING_FAMILY_CONTRACT_STATE_DIR:-$WORK_DIR/regex_parser_family_contract_gate}"

if [[ -z "$EXISTING_FAMILY_CONTRACT_STATE_DIR" ]]; then
    regex_family_contract_cmd=(
        env
        PGEN_REGEX_FAMILY_CONTRACT_STATE_DIR="$family_contract_state_dir"
    )
    if [[ -n "$EXISTING_FRONTEND_STATE_DIR" ]]; then
        regex_family_contract_cmd+=(
            PGEN_REGEX_FAMILY_CONTRACT_EXISTING_FRONTEND_STATE_DIR="$EXISTING_FRONTEND_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_DUAL_RUN_STATE_DIR" ]]; then
        regex_family_contract_cmd+=(
            PGEN_REGEX_FAMILY_CONTRACT_EXISTING_DUAL_RUN_STATE_DIR="$EXISTING_DUAL_RUN_STATE_DIR"
        )
    fi
    if [[ -n "$EXISTING_STIMULI_STATE_DIR" ]]; then
        regex_family_contract_cmd+=(
            PGEN_REGEX_FAMILY_CONTRACT_EXISTING_STIMULI_STATE_DIR="$EXISTING_STIMULI_STATE_DIR"
        )
    fi
    regex_family_contract_cmd+=("$REGEX_FAMILY_CONTRACT_GATE")
    run_logged "regex_parser_family_contract_gate" "${regex_family_contract_cmd[@]}"
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

if [[ "$family_contract_gate_name" != "regex_parser_family_contract_gate" ]]; then
    echo "error: unexpected regex family-contract gate identity '$family_contract_gate_name'" >&2
    exit 1
fi
if [[ ! "$family_contract_gate_version" =~ ^[0-9]+$ ]]; then
    echo "error: regex family-contract gate version is not numeric: '$family_contract_gate_version'" >&2
    exit 1
fi
if [[ -z "$family_contract_generated_at_utc" ]]; then
    echo "error: regex family-contract generated_at_utc is empty" >&2
    exit 1
fi
if [[ "$summary_family_contract_state_dir" != "$family_contract_state_dir" ]]; then
    echo "error: regex family-contract state_dir mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$family_contract_state_dir_from_json" != "$family_contract_state_dir" ]]; then
    echo "error: regex family-contract state_dir mismatch in summary.json" >&2
    exit 1
fi
if [[ "$family_contract_summary_txt_from_json" != "$family_contract_summary_txt" ]]; then
    echo "error: regex family-contract summary_txt mismatch in summary.json" >&2
    exit 1
fi
if [[ "$summary_family_contract_summary_json" != "$family_contract_summary_json" ]]; then
    echo "error: regex family-contract summary_json mismatch in summary.txt" >&2
    exit 1
fi
if [[ "$family_contract_summary_json_from_json" != "$family_contract_summary_json" ]]; then
    echo "error: regex family-contract summary_json mismatch in summary.json" >&2
    exit 1
fi
if [[ "$summary_family_contract_generated_at_utc" != "$family_contract_generated_at_utc" ]]; then
    echo "error: regex family-contract generated_at_utc mismatch between summary.txt and summary.json" >&2
    exit 1
fi

required_surface_key="$(jq -r '.required_surface_key' "$CONTRACT_FILE")"
required_surface_missing_detail="$(jq -r '.required_surface_missing_detail' "$CONTRACT_FILE")"
done_rule="$(jq -r '.done_rule' "$CONTRACT_FILE")"

regex_family_contract_green=true
regex_stimuli_parseability_parser_rejections_total="$(summary_value_from_txt "stimuli_regex_parseability_parser_rejections_total" "$family_contract_summary_txt")"
regex_stimuli_final_targets="$(summary_value_from_txt "stimuli_regex_final_targets" "$family_contract_summary_txt")"
regex_stimuli_parseability_acceptance_rate_percent="$(summary_value_from_txt "stimuli_regex_parseability_acceptance_rate_percent" "$family_contract_summary_txt")"
regex_stimuli_parseability_counterexample_primary_parser_error="$(summary_value_from_txt "stimuli_regex_parseability_counterexample_primary_parser_error" "$family_contract_summary_txt")"
regex_stimuli_parseability_counterexample_primary_failure_location="$(summary_value_from_txt "stimuli_regex_parseability_counterexample_primary_failure_location" "$family_contract_summary_txt")"

broader_corpus_backed_proof_surface_present=false
broader_corpus_backed_proof_state_dir_txt="<missing>"
broader_corpus_backed_proof_summary_txt="<missing>"
broader_corpus_backed_proof_summary_json="<missing>"
broader_corpus_backed_proof_state_dir_json=""
broader_corpus_backed_proof_summary_txt_json=""
broader_corpus_backed_proof_summary_json_json=""

if [[ -n "$EXISTING_BROADER_CORPUS_PROOF_STATE_DIR" ]]; then
    broader_corpus_backed_proof_state_dir_txt="$EXISTING_BROADER_CORPUS_PROOF_STATE_DIR"
    broader_corpus_backed_proof_summary_txt="$EXISTING_BROADER_CORPUS_PROOF_STATE_DIR/summary.txt"
    broader_corpus_backed_proof_summary_json="$EXISTING_BROADER_CORPUS_PROOF_STATE_DIR/summary.json"
    require_nonempty_file "$broader_corpus_backed_proof_summary_txt"
    require_nonempty_file "$broader_corpus_backed_proof_summary_json"
    broader_corpus_backed_proof_state_dir_json="$broader_corpus_backed_proof_state_dir_txt"
    broader_corpus_backed_proof_summary_txt_json="$broader_corpus_backed_proof_summary_txt"
    broader_corpus_backed_proof_summary_json_json="$broader_corpus_backed_proof_summary_json"
    broader_corpus_backed_proof_surface_present=true
fi

regex_formal_exhaustive_closure_surface_green=false
regex_closure_criteria_total_count=1
regex_closure_criteria_satisfied_count=0
if [[ "$broader_corpus_backed_proof_surface_present" == true ]]; then
    regex_closure_criteria_satisfied_count=1
    regex_formal_exhaustive_closure_surface_green=true
fi
regex_closure_criteria_unsatisfied_count=$((regex_closure_criteria_total_count - regex_closure_criteria_satisfied_count))

declare -a regex_unmet=()
declare -a regex_unmet_details=()
if [[ "$broader_corpus_backed_proof_surface_present" != true ]]; then
    regex_unmet+=("${required_surface_key}=missing")
    regex_unmet_details+=("{\"criterion\":\"broader_corpus_backed_proof_surface_present\",\"evidence_key\":\"${required_surface_key}\",\"observed\":\"missing\",\"expected\":\"present\",\"detail\":\"${required_surface_missing_detail}\"}")
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
    echo "Regex Formal Exhaustive Closure Gate Summary"
    echo "gate: regex_formal_exhaustive_closure_gate"
    echo "version: 1"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "contract_file: $CONTRACT_FILE"
    echo "status_rule_done: $done_rule"
    echo "regex_formal_exhaustive_required_surface_key: $required_surface_key"
    echo "regex_formal_exhaustive_closure_surface_green: $regex_formal_exhaustive_closure_surface_green"
    echo "regex_unmet_closure_criteria_count: $regex_unmet_count"
    echo "regex_primary_unmet_closure_criterion: $regex_primary_unmet_closure_criterion"
    echo "regex_unmet_closure_criteria_json: $regex_unmet_json"
    echo "regex_unmet_closure_criteria_details_json: $regex_unmet_details_json"
    echo "regex_closure_criteria_total_count: $regex_closure_criteria_total_count"
    echo "regex_closure_criteria_satisfied_count: $regex_closure_criteria_satisfied_count"
    echo "regex_closure_criteria_unsatisfied_count: $regex_closure_criteria_unsatisfied_count"
    echo "regex_broader_corpus_backed_proof_surface_present: $broader_corpus_backed_proof_surface_present"
    echo "regex_family_contract_green: $regex_family_contract_green"
    echo "regex_stimuli_parseability_parser_rejections_total: $regex_stimuli_parseability_parser_rejections_total"
    echo "regex_stimuli_final_targets: $regex_stimuli_final_targets"
    echo "regex_stimuli_parseability_acceptance_rate_percent: $regex_stimuli_parseability_acceptance_rate_percent"
    echo "regex_stimuli_parseability_counterexample_primary_parser_error: $regex_stimuli_parseability_counterexample_primary_parser_error"
    echo "regex_stimuli_parseability_counterexample_primary_failure_location: $regex_stimuli_parseability_counterexample_primary_failure_location"
    echo "regex_family_contract_gate: $family_contract_gate_name"
    echo "regex_family_contract_gate_version: $family_contract_gate_version"
    echo "regex_family_contract_generated_at_utc: $family_contract_generated_at_utc"
    echo "regex_family_contract_state_dir: $family_contract_state_dir"
    echo "regex_family_contract_summary_txt: $family_contract_summary_txt"
    echo "regex_family_contract_summary_json: $family_contract_summary_json"
    echo "regex_broader_corpus_backed_proof_state_dir: $broader_corpus_backed_proof_state_dir_txt"
    echo "regex_broader_corpus_backed_proof_summary_txt: $broader_corpus_backed_proof_summary_txt"
    echo "regex_broader_corpus_backed_proof_summary_json: $broader_corpus_backed_proof_summary_json"
} | tee "$SUMMARY_TXT"

jq -n \
    --arg gate "regex_formal_exhaustive_closure_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg contract_file "$CONTRACT_FILE" \
    --arg status_rule_done "$done_rule" \
    --arg regex_formal_exhaustive_required_surface_key "$required_surface_key" \
    --argjson regex_formal_exhaustive_closure_surface_green "$regex_formal_exhaustive_closure_surface_green" \
    --arg regex_primary_unmet_closure_criterion "$regex_primary_unmet_closure_criterion" \
    --argjson regex_unmet_closure_criteria_count "$regex_unmet_count" \
    --argjson regex_unmet_closure_criteria "$regex_unmet_json" \
    --argjson regex_unmet_closure_criteria_details "$regex_unmet_details_json" \
    --argjson regex_closure_criteria_total_count "$regex_closure_criteria_total_count" \
    --argjson regex_closure_criteria_satisfied_count "$regex_closure_criteria_satisfied_count" \
    --argjson regex_closure_criteria_unsatisfied_count "$regex_closure_criteria_unsatisfied_count" \
    --argjson regex_broader_corpus_backed_proof_surface_present "$broader_corpus_backed_proof_surface_present" \
    --argjson regex_family_contract_green "$regex_family_contract_green" \
    --argjson regex_stimuli_parseability_parser_rejections_total "$regex_stimuli_parseability_parser_rejections_total" \
    --argjson regex_stimuli_final_targets "$regex_stimuli_final_targets" \
    --argjson regex_stimuli_parseability_acceptance_rate_percent "$regex_stimuli_parseability_acceptance_rate_percent" \
    --arg regex_stimuli_parseability_counterexample_primary_parser_error "$regex_stimuli_parseability_counterexample_primary_parser_error" \
    --arg regex_stimuli_parseability_counterexample_primary_failure_location "$regex_stimuli_parseability_counterexample_primary_failure_location" \
    --arg regex_family_contract_gate "$family_contract_gate_name" \
    --argjson regex_family_contract_gate_version "$family_contract_gate_version" \
    --arg regex_family_contract_generated_at_utc "$family_contract_generated_at_utc" \
    --arg regex_family_contract_state_dir "$family_contract_state_dir" \
    --arg regex_family_contract_summary_txt "$family_contract_summary_txt" \
    --arg regex_family_contract_summary_json "$family_contract_summary_json" \
    --arg regex_broader_corpus_backed_proof_state_dir "$broader_corpus_backed_proof_state_dir_json" \
    --arg regex_broader_corpus_backed_proof_summary_txt "$broader_corpus_backed_proof_summary_txt_json" \
    --arg regex_broader_corpus_backed_proof_summary_json "$broader_corpus_backed_proof_summary_json_json" \
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
          family: "regex",
          formal_exhaustive_closure_surface_green: $regex_formal_exhaustive_closure_surface_green,
          required_surface_key: $regex_formal_exhaustive_required_surface_key,
          primary_unmet_closure_criterion: $regex_primary_unmet_closure_criterion,
          unmet_closure_criteria_count: $regex_unmet_closure_criteria_count,
          unmet_closure_criteria: $regex_unmet_closure_criteria,
          unmet_closure_criteria_details: $regex_unmet_closure_criteria_details,
          closure_criteria_total_count: $regex_closure_criteria_total_count,
          closure_criteria_satisfied_count: $regex_closure_criteria_satisfied_count,
          closure_criteria_unsatisfied_count: $regex_closure_criteria_unsatisfied_count,
          criteria: {
            broader_corpus_backed_proof_surface_present: $regex_broader_corpus_backed_proof_surface_present
          },
          metrics: {
            family_contract_green: $regex_family_contract_green,
            stimuli_parseability_parser_rejections_total: $regex_stimuli_parseability_parser_rejections_total,
            stimuli_final_targets: $regex_stimuli_final_targets,
            stimuli_parseability_acceptance_rate_percent: $regex_stimuli_parseability_acceptance_rate_percent,
            stimuli_parseability_counterexample_primary_parser_error: $regex_stimuli_parseability_counterexample_primary_parser_error,
            stimuli_parseability_counterexample_primary_failure_location: $regex_stimuli_parseability_counterexample_primary_failure_location,
            family_contract_gate: $regex_family_contract_gate,
            family_contract_gate_version: $regex_family_contract_gate_version,
            family_contract_generated_at_utc: $regex_family_contract_generated_at_utc
          },
          proof_surfaces: {
            family_contract_state_dir: $regex_family_contract_state_dir,
            family_contract_summary_txt: $regex_family_contract_summary_txt,
            family_contract_summary_json: $regex_family_contract_summary_json,
            broader_corpus_backed_proof_state_dir: (if $regex_broader_corpus_backed_proof_state_dir == "" then null else $regex_broader_corpus_backed_proof_state_dir end),
            broader_corpus_backed_proof_summary_txt: (if $regex_broader_corpus_backed_proof_summary_txt == "" then null else $regex_broader_corpus_backed_proof_summary_txt end),
            broader_corpus_backed_proof_summary_json: (if $regex_broader_corpus_backed_proof_summary_json == "" then null else $regex_broader_corpus_backed_proof_summary_json end)
          }
        }
      ]
    }' >"$SUMMARY_JSON"

require_nonempty_file "$SUMMARY_JSON"

echo "✅ Regex formal exhaustive closure gate passed."
