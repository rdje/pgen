#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_aggregate_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

QUALITY_GATE_SCRIPT="$RUST_DIR/scripts/sv_preprocessor_quality_gate.sh"
EXISTING_QUALITY_STATE_DIR="${PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_EXISTING_QUALITY_STATE_DIR:-}"

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

extract_json_number() {
    local path="$1"
    local expr="$2"
    jq -er "$expr | numbers" "$path"
}

require_tool jq
if [[ -z "$EXISTING_QUALITY_STATE_DIR" ]]; then
    require_file "$QUALITY_GATE_SCRIPT"
fi

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

quality_state_dir="${EXISTING_QUALITY_STATE_DIR:-$WORK_DIR/quality_state}"

if [[ -z "$EXISTING_QUALITY_STATE_DIR" ]]; then
    run_logged "preprocessor_quality_probe" \
        env \
            PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR="$quality_state_dir" \
            PGEN_SV_PREPROCESSOR_DIFF_MODE=0 \
            "$QUALITY_GATE_SCRIPT"
fi

parseability_report_json="$quality_state_dir/work/systemverilog_preprocessor_parseability_report.json"
gap_stage3_json="$quality_state_dir/work/systemverilog_preprocessor_gap_stage3.json"

require_nonempty_file "$parseability_report_json"
require_nonempty_file "$gap_stage3_json"

if ! jq -e '
    .grammar_name == "systemverilog_preprocessor"
    and .effective_mode == "enabled"
    and (.counterexamples | type == "array")
    and ((.counterexamples | length) <= 20)
    and (
        if (.summary.parser_rejections // 0) > 0
        then (.counterexamples | length) > 0
        else true
        end
    )
    and (.stages.stage0_baseline.summary | type == "object")
    and (.stages.stage1_gap_priority.summary | type == "object")
    and (.stages.stage2_target_drive.summary | type == "object")
    and (.stages.stage3_recompute_gap.summary | type == "object")
    and all(
        .counterexamples[]?;
        has("stage")
        and has("sample")
        and has("shrunk_sample")
    )
' "$parseability_report_json" >/dev/null; then
    echo "error: preprocessor aggregate parseability report contract failed: $parseability_report_json" >&2
    cat "$parseability_report_json" >&2
    exit 1
fi

if ! jq -e '
    .grammar_name == "systemverilog_preprocessor"
    and (.targets | type == "array")
    and ((.targets | length) == 0)
    and (.summary.covered_reachable_rules == .summary.reachable_rules)
    and (.summary.covered_reachable_branches == .summary.reachable_branches)
' "$gap_stage3_json" >/dev/null; then
    echo "error: preprocessor final gap contract failed: $gap_stage3_json" >&2
    cat "$gap_stage3_json" >&2
    exit 1
fi

parseability_attempts_total="$(extract_json_number "$parseability_report_json" '.summary.attempts')"
parseability_accepted_total="$(extract_json_number "$parseability_report_json" '.summary.accepted')"
parseability_rejected_total="$(extract_json_number "$parseability_report_json" '.summary.rejected')"
parseability_parser_rejections_total="$(extract_json_number "$parseability_report_json" '.summary.parser_rejections')"
parseability_counterexamples_captured_total="$(extract_json_number "$parseability_report_json" '((.counterexamples // []) | length)')"
final_targets="$(extract_json_number "$gap_stage3_json" '(.targets | length)')"
covered_reachable_rules="$(extract_json_number "$gap_stage3_json" '.summary.covered_reachable_rules')"
reachable_rules="$(extract_json_number "$gap_stage3_json" '.summary.reachable_rules')"
covered_reachable_branches="$(extract_json_number "$gap_stage3_json" '.summary.covered_reachable_branches')"
reachable_branches="$(extract_json_number "$gap_stage3_json" '.summary.reachable_branches')"

{
    echo "SV Preprocessor Aggregate Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "existing_quality_state_dir: ${EXISTING_QUALITY_STATE_DIR:-<unset>}"
    echo "quality_state_dir: $quality_state_dir"
    echo "parseability_report_json: $parseability_report_json"
    echo "gap_stage3_json: $gap_stage3_json"
    echo "parseability_attempts_total: $parseability_attempts_total"
    echo "parseability_accepted_total: $parseability_accepted_total"
    echo "parseability_rejected_total: $parseability_rejected_total"
    echo "parseability_parser_rejections_total: $parseability_parser_rejections_total"
    echo "parseability_counterexamples_captured_total: $parseability_counterexamples_captured_total"
    echo "final_targets: $final_targets"
    echo "covered_reachable_rules: $covered_reachable_rules/$reachable_rules"
    echo "covered_reachable_branches: $covered_reachable_branches/$reachable_branches"
} | tee "$SUMMARY_TXT"

echo "✅ SV preprocessor aggregate contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
