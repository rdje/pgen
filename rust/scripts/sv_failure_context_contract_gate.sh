#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_failure_context_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

SV_CONTRACT_FILE="${PGEN_SV_FAILURE_CONTEXT_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_failure_context_v0_contract.json}"
SVPP_POLICY_ENV_FILE="${PGEN_SV_FAILURE_CONTEXT_SVPP_POLICY_ENV_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_preprocessor_lightweight_v0.env}"
SV_GATE_SCRIPT="$RUST_DIR/scripts/sv_stimuli_quality_gate.sh"
SV_PARSER_AGGREGATE_SCRIPT="$RUST_DIR/scripts/sv_parser_aggregate_contract_gate.sh"
SVPP_QUALITY_GATE_SCRIPT="$RUST_DIR/scripts/sv_preprocessor_quality_gate.sh"
SVPP_AGGREGATE_SCRIPT="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"

EXISTING_SV_STIMULI_QUALITY_STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-}"
EXISTING_SV_PARSER_AGGREGATE_STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_EXISTING_SV_PARSER_AGGREGATE_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR="${PGEN_SV_FAILURE_CONTEXT_EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR:-}"

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

run_logged_with_env_file() {
    local label="$1"
    local env_file="$2"
    shift 2
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (
        set -a
        # shellcheck disable=SC1090
        source "$env_file"
        set +a
        "$@"
    ) >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        exit 1
    fi
}

extract_json_number() {
    local path="$1"
    local expr="$2"
    jq -er "${expr} | numbers" "$path"
}

extract_json_string() {
    local path="$1"
    local expr="$2"
    jq -er "${expr} | strings" "$path"
}

require_tool jq
require_file "$SV_CONTRACT_FILE"
require_file "$SVPP_POLICY_ENV_FILE"
require_file "$SV_GATE_SCRIPT"
require_file "$SV_PARSER_AGGREGATE_SCRIPT"
require_file "$SVPP_QUALITY_GATE_SCRIPT"
require_file "$SVPP_AGGREGATE_SCRIPT"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

sv_quality_state_dir="$WORK_DIR/systemverilog_failure_context_quality_state"
if [[ -n "$EXISTING_SV_STIMULI_QUALITY_STATE_DIR" ]]; then
    sv_quality_state_dir="$EXISTING_SV_STIMULI_QUALITY_STATE_DIR"
else
    run_logged "systemverilog_failure_context_quality_gate" env \
        PGEN_SV_STIMULI_QUALITY_CONTRACT="$SV_CONTRACT_FILE" \
        PGEN_SV_STIMULI_QUALITY_STATE_DIR="$sv_quality_state_dir" \
        "$SV_GATE_SCRIPT"
fi

sv_parser_aggregate_state_dir="$WORK_DIR/sv_parser_aggregate_contract_gate"
if [[ -n "$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR" ]]; then
    sv_parser_aggregate_state_dir="$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR"
else
    run_logged "systemverilog_failure_context_aggregate_contract_gate" env \
        PGEN_SV_PARSER_AGGREGATE_CONTRACT_STATE_DIR="$sv_parser_aggregate_state_dir" \
        PGEN_SV_PARSER_AGGREGATE_CONTRACT_EXISTING_SV_STIMULI_QUALITY_STATE_DIR="$sv_quality_state_dir" \
        "$SV_PARSER_AGGREGATE_SCRIPT"
fi

sv_generation_triage_json="$sv_parser_aggregate_state_dir/work/systemverilog_parseability_generation_counterexample_triage.json"
sv_shadow_triage_json="$sv_parser_aggregate_state_dir/work/systemverilog_closed_loop_parseability_shadow_counterexample_triage.json"
require_nonempty_file "$sv_generation_triage_json"
require_nonempty_file "$sv_shadow_triage_json"

sv_generation_failure_context_count="$(extract_json_number "$sv_generation_triage_json" '(.by_failure_context_excerpt | length)')"
sv_shadow_failure_context_count="$(extract_json_number "$sv_shadow_triage_json" '(.by_failure_context_excerpt | length)')"
if [[ "$sv_generation_failure_context_count" -lt 1 ]]; then
    echo "error: expected at least one generation failure-context excerpt" >&2
    exit 1
fi
if [[ "$sv_shadow_failure_context_count" -lt 1 ]]; then
    echo "error: expected at least one replay-shadow failure-context excerpt" >&2
    exit 1
fi

sv_generation_failure_context_example="$(extract_json_string "$sv_generation_triage_json" '.sample_previews[0].failure_context_excerpt')"
sv_shadow_failure_context_example="$(extract_json_string "$sv_shadow_triage_json" '.sample_previews[0].failure_context_excerpt')"

svpp_quality_state_dir="$WORK_DIR/systemverilog_preprocessor_failure_context_quality_state"
if [[ -n "$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR" ]]; then
    svpp_quality_state_dir="$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR"
else
    run_logged_with_env_file "systemverilog_preprocessor_failure_context_quality_gate" "$SVPP_POLICY_ENV_FILE" \
        env PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR="$svpp_quality_state_dir" "$SVPP_QUALITY_GATE_SCRIPT"
fi

svpp_aggregate_state_dir="$WORK_DIR/sv_preprocessor_aggregate_contract_gate"
if [[ -n "$EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR" ]]; then
    svpp_aggregate_state_dir="$EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR"
else
    run_logged "systemverilog_preprocessor_failure_context_aggregate_contract_gate" env \
        PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_STATE_DIR="$svpp_aggregate_state_dir" \
        PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_EXISTING_QUALITY_STATE_DIR="$svpp_quality_state_dir" \
        "$SVPP_AGGREGATE_SCRIPT"
fi

svpp_triage_json="$svpp_aggregate_state_dir/work/systemverilog_preprocessor_parseability_counterexample_triage.json"
require_nonempty_file "$svpp_triage_json"

svpp_failure_context_count="$(extract_json_number "$svpp_triage_json" '(.by_failure_context_excerpt | length)')"
if [[ "$svpp_failure_context_count" -lt 1 ]]; then
    echo "error: expected at least one preprocessor failure-context excerpt" >&2
    exit 1
fi

svpp_failure_context_example="$(extract_json_string "$svpp_triage_json" '.sample_previews[0].failure_context_excerpt')"

{
    echo "SV Failure Context Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "sv_contract_file: $SV_CONTRACT_FILE"
    echo "svpp_policy_env_file: $SVPP_POLICY_ENV_FILE"
    echo "existing_sv_stimuli_quality_state_dir: ${EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-<unset>}"
    echo "existing_sv_preprocessor_quality_state_dir: ${EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-<unset>}"
    echo "systemverilog_failure_context_quality_state_dir: $sv_quality_state_dir"
    echo "systemverilog_parser_aggregate_state_dir: $sv_parser_aggregate_state_dir"
    echo "systemverilog_generation_failure_context_excerpts: $sv_generation_failure_context_count"
    echo "systemverilog_shadow_failure_context_excerpts: $sv_shadow_failure_context_count"
    echo "systemverilog_generation_failure_context_example: $sv_generation_failure_context_example"
    echo "systemverilog_shadow_failure_context_example: $sv_shadow_failure_context_example"
    echo "systemverilog_preprocessor_failure_context_quality_state_dir: $svpp_quality_state_dir"
    echo "systemverilog_preprocessor_aggregate_state_dir: $svpp_aggregate_state_dir"
    echo "systemverilog_preprocessor_failure_context_excerpts: $svpp_failure_context_count"
    echo "systemverilog_preprocessor_failure_context_example: $svpp_failure_context_example"
} >"$SUMMARY_TXT"

require_nonempty_file "$SUMMARY_TXT"
jq -n \
  --arg gate "sv_failure_context_contract_gate" \
  --argjson version 1 \
  --arg generated_at_utc "$generated_at_utc" \
  --arg state_dir "$STATE_DIR" \
  --arg summary_txt "$SUMMARY_TXT" \
  --arg summary_json "$SUMMARY_JSON" \
  --arg sv_contract_file "$SV_CONTRACT_FILE" \
  --arg svpp_policy_env_file "$SVPP_POLICY_ENV_FILE" \
  --arg existing_sv_stimuli_quality_state_dir "${EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}" \
  --arg existing_sv_preprocessor_quality_state_dir "${EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-}" \
  --arg systemverilog_failure_context_quality_state_dir "$sv_quality_state_dir" \
  --arg systemverilog_parser_aggregate_state_dir "$sv_parser_aggregate_state_dir" \
  --arg systemverilog_generation_counterexample_triage_json "$sv_generation_triage_json" \
  --arg systemverilog_shadow_counterexample_triage_json "$sv_shadow_triage_json" \
  --arg systemverilog_preprocessor_failure_context_quality_state_dir "$svpp_quality_state_dir" \
  --arg systemverilog_preprocessor_aggregate_state_dir "$svpp_aggregate_state_dir" \
  --arg systemverilog_preprocessor_counterexample_triage_json "$svpp_triage_json" \
  --argjson systemverilog_generation_failure_context_excerpts "$sv_generation_failure_context_count" \
  --argjson systemverilog_shadow_failure_context_excerpts "$sv_shadow_failure_context_count" \
  --argjson systemverilog_preprocessor_failure_context_excerpts "$svpp_failure_context_count" \
  --arg systemverilog_generation_failure_context_example "$sv_generation_failure_context_example" \
  --arg systemverilog_shadow_failure_context_example "$sv_shadow_failure_context_example" \
  --arg systemverilog_preprocessor_failure_context_example "$svpp_failure_context_example" \
  '{
    gate: $gate,
    version: $version,
    generated_at_utc: $generated_at_utc,
    state_dir: $state_dir,
    summary_txt: $summary_txt,
    summary_json: $summary_json,
    sv_contract_file: $sv_contract_file,
    svpp_policy_env_file: $svpp_policy_env_file,
    existing_sv_stimuli_quality_state_dir: (if $existing_sv_stimuli_quality_state_dir == "" then null else $existing_sv_stimuli_quality_state_dir end),
    existing_sv_preprocessor_quality_state_dir: (if $existing_sv_preprocessor_quality_state_dir == "" then null else $existing_sv_preprocessor_quality_state_dir end),
    proof_surfaces: {
      systemverilog_failure_context_quality_state_dir: $systemverilog_failure_context_quality_state_dir,
      systemverilog_parser_aggregate_state_dir: $systemverilog_parser_aggregate_state_dir,
      systemverilog_generation_counterexample_triage_json: $systemverilog_generation_counterexample_triage_json,
      systemverilog_shadow_counterexample_triage_json: $systemverilog_shadow_counterexample_triage_json,
      systemverilog_preprocessor_failure_context_quality_state_dir: $systemverilog_preprocessor_failure_context_quality_state_dir,
      systemverilog_preprocessor_aggregate_state_dir: $systemverilog_preprocessor_aggregate_state_dir,
      systemverilog_preprocessor_counterexample_triage_json: $systemverilog_preprocessor_counterexample_triage_json
    },
    metrics: {
      systemverilog_generation_failure_context_excerpts: $systemverilog_generation_failure_context_excerpts,
      systemverilog_shadow_failure_context_excerpts: $systemverilog_shadow_failure_context_excerpts,
      systemverilog_preprocessor_failure_context_excerpts: $systemverilog_preprocessor_failure_context_excerpts
    },
    examples: {
      systemverilog_generation_failure_context_example: $systemverilog_generation_failure_context_example,
      systemverilog_shadow_failure_context_example: $systemverilog_shadow_failure_context_example,
      systemverilog_preprocessor_failure_context_example: $systemverilog_preprocessor_failure_context_example
    }
  }' >"$SUMMARY_JSON"
require_nonempty_file "$SUMMARY_JSON"
cat "$SUMMARY_TXT"
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
