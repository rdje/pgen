#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PREPROCESSOR_REACHABILITY_CLOSURE_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_reachability_closure_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

QUALITY_GATE_SCRIPT="$RUST_DIR/scripts/sv_preprocessor_quality_gate.sh"
POLICY_ENV_FILE="${PGEN_SV_PREPROCESSOR_REACHABILITY_CLOSURE_POLICY_ENV_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_preprocessor_lightweight_v0.env}"
EXISTING_QUALITY_STATE_DIR="${PGEN_SV_PREPROCESSOR_REACHABILITY_CLOSURE_EXISTING_QUALITY_STATE_DIR:-}"

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
    jq -er "$expr | numbers" "$path"
}

require_file "$POLICY_ENV_FILE"
if [[ -z "$EXISTING_QUALITY_STATE_DIR" ]]; then
    require_file "$QUALITY_GATE_SCRIPT"
fi

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

quality_state_dir="${EXISTING_QUALITY_STATE_DIR:-$WORK_DIR/quality_state}"
if [[ -z "$EXISTING_QUALITY_STATE_DIR" ]]; then
    run_logged_with_env_file "preprocessor_quality_gate" "$POLICY_ENV_FILE" \
        env PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR="$quality_state_dir" "$QUALITY_GATE_SCRIPT"
fi

parseability_report_json="$quality_state_dir/work/systemverilog_preprocessor_parseability_report.json"
gap_stage0_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage0_a.json"
gap_stage1_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage1.json"
gap_stage3_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage3.json"
gap_stage4_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage4_fuzz_a.json"

require_nonempty_file "$parseability_report_json"
require_nonempty_file "$gap_stage0_json"
require_nonempty_file "$gap_stage1_json"
require_nonempty_file "$gap_stage3_json"
require_nonempty_file "$gap_stage4_json"

if ! jq -e '
    .grammar_name == "systemverilog_preprocessor"
    and (.targets | type == "array")
    and ((.targets | length) == 0)
    and (.summary.covered_reachable_rules == .summary.reachable_rules)
    and (.summary.covered_reachable_branches == .summary.reachable_branches)
' "$gap_stage3_json" >/dev/null; then
    echo "error: stage3 preprocessor reachability closure contract failed: $gap_stage3_json" >&2
    cat "$gap_stage3_json" >&2
    exit 1
fi

if ! jq -e '
    .grammar_name == "systemverilog_preprocessor"
    and (.targets | type == "array")
    and ((.targets | length) == 0)
    and (.summary.covered_reachable_rules == .summary.reachable_rules)
    and (.summary.covered_reachable_branches == .summary.reachable_branches)
' "$gap_stage4_json" >/dev/null; then
    echo "error: stage4 preprocessor reachability closure contract failed: $gap_stage4_json" >&2
    cat "$gap_stage4_json" >&2
    exit 1
fi

stage0_targets="$(extract_json_number "$gap_stage0_json" '(.targets | length)')"
stage1_targets="$(extract_json_number "$gap_stage1_json" '(.targets | length)')"
stage3_targets="$(extract_json_number "$gap_stage3_json" '(.targets | length)')"
stage4_targets="$(extract_json_number "$gap_stage4_json" '(.targets | length)')"

stage0_rules="$(extract_json_number "$gap_stage0_json" '.summary.covered_reachable_rules')"
stage1_rules="$(extract_json_number "$gap_stage1_json" '.summary.covered_reachable_rules')"
stage3_rules="$(extract_json_number "$gap_stage3_json" '.summary.covered_reachable_rules')"
stage4_rules="$(extract_json_number "$gap_stage4_json" '.summary.covered_reachable_rules')"
stage3_reachable_rules="$(extract_json_number "$gap_stage3_json" '.summary.reachable_rules')"
stage4_reachable_rules="$(extract_json_number "$gap_stage4_json" '.summary.reachable_rules')"

stage0_branches="$(extract_json_number "$gap_stage0_json" '.summary.covered_reachable_branches')"
stage1_branches="$(extract_json_number "$gap_stage1_json" '.summary.covered_reachable_branches')"
stage3_branches="$(extract_json_number "$gap_stage3_json" '.summary.covered_reachable_branches')"
stage4_branches="$(extract_json_number "$gap_stage4_json" '.summary.covered_reachable_branches')"
stage3_reachable_branches="$(extract_json_number "$gap_stage3_json" '.summary.reachable_branches')"
stage4_reachable_branches="$(extract_json_number "$gap_stage4_json" '.summary.reachable_branches')"

parseability_attempts="$(extract_json_number "$parseability_report_json" '.summary.attempts')"
parseability_accepted="$(extract_json_number "$parseability_report_json" '.summary.accepted')"
parseability_rejected="$(extract_json_number "$parseability_report_json" '.summary.rejected')"
parser_rejections="$(extract_json_number "$parseability_report_json" '.summary.parser_rejections')"
generation_errors="$(extract_json_number "$parseability_report_json" '.summary.generation_errors')"
empty_generations="$(extract_json_number "$parseability_report_json" '.summary.empty_generations')"

{
    echo "SV Preprocessor Reachability Closure Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "policy_env_file: $POLICY_ENV_FILE"
    echo "existing_quality_state_dir: ${EXISTING_QUALITY_STATE_DIR:-<unset>}"
    echo "quality_state_dir: $quality_state_dir"
    echo "stage0_targets: $stage0_targets"
    echo "stage1_targets: $stage1_targets"
    echo "stage3_targets: $stage3_targets"
    echo "stage4_targets: $stage4_targets"
    echo "stage0_covered_reachable_rules: $stage0_rules"
    echo "stage1_covered_reachable_rules: $stage1_rules"
    echo "stage3_covered_reachable_rules: $stage3_rules/$stage3_reachable_rules"
    echo "stage4_covered_reachable_rules: $stage4_rules/$stage4_reachable_rules"
    echo "stage0_covered_reachable_branches: $stage0_branches"
    echo "stage1_covered_reachable_branches: $stage1_branches"
    echo "stage3_covered_reachable_branches: $stage3_branches/$stage3_reachable_branches"
    echo "stage4_covered_reachable_branches: $stage4_branches/$stage4_reachable_branches"
    echo "parseability_attempts: $parseability_attempts"
    echo "parseability_accepted: $parseability_accepted"
    echo "parseability_rejected: $parseability_rejected"
    echo "parser_rejections: $parser_rejections"
    echo "generation_errors: $generation_errors"
    echo "empty_generations: $empty_generations"
} >"$SUMMARY_TXT"

require_nonempty_file "$SUMMARY_TXT"
cat "$SUMMARY_TXT"
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
