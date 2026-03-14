#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PARSER_AGGREGATE_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_parser_aggregate_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

BASE_CONTRACT_FILE="${PGEN_SV_PARSER_AGGREGATE_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_core_v0_contract.json}"
SV_GATE_SCRIPT="$RUST_DIR/scripts/sv_stimuli_quality_gate.sh"

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
    local name="$1"
    shift
    local log_file="$LOG_DIR/${name}.log"
    echo "==> $name"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok ($log_file)"
    else
        echo "error: stage '$name' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        exit 1
    fi
}

extract_json_number() {
    local path="$1"
    local expr="$2"
    jq -er "${expr} | numbers" "$path"
}

require_tool jq
require_file "$BASE_CONTRACT_FILE"
require_file "$SV_GATE_SCRIPT"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

generation_contract="$WORK_DIR/systemverilog_generation_only_contract.json"
shadow_contract="$WORK_DIR/systemverilog_shadow_enabled_contract.json"

jq '
    .closed_loop.enabled = true
    | .closed_loop.parseability_shadow_enabled = false
    | .nexsim_realistic_corpus.enforce = false
    | .performance_budgets.enforce = false
    | .parse_full_quality.enforce_min_pass_ratio = false
' "$BASE_CONTRACT_FILE" >"$generation_contract"

jq '
    .closed_loop.enabled = true
    | .closed_loop.parseability_shadow_enabled = true
    | .nexsim_realistic_corpus.enforce = false
    | .performance_budgets.enforce = false
    | .parse_full_quality.enforce_min_pass_ratio = false
' "$BASE_CONTRACT_FILE" >"$shadow_contract"

generation_state_dir="$WORK_DIR/generation_state"
shadow_state_dir="$WORK_DIR/shadow_state"

run_logged "generation_only_aggregate_probe" \
    env \
        PGEN_SV_STIMULI_QUALITY_CONTRACT="$generation_contract" \
        PGEN_SV_STIMULI_QUALITY_STATE_DIR="$generation_state_dir" \
        PGEN_SV_STIMULI_QUALITY_COUNT=1 \
        PGEN_SV_STIMULI_QUALITY_LRM_PROFILES=2017 \
        PGEN_SV_STIMULI_DIFF_MODE=0 \
        PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 \
        PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 \
        "$SV_GATE_SCRIPT"

run_logged "shadow_enabled_aggregate_probe" \
    env \
        PGEN_SV_STIMULI_QUALITY_CONTRACT="$shadow_contract" \
        PGEN_SV_STIMULI_QUALITY_STATE_DIR="$shadow_state_dir" \
        PGEN_SV_STIMULI_QUALITY_COUNT=1 \
        PGEN_SV_STIMULI_QUALITY_LRM_PROFILES=2017 \
        PGEN_SV_STIMULI_DIFF_MODE=0 \
        PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 \
        PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 \
        "$SV_GATE_SCRIPT"

generation_report_json="$generation_state_dir/work/systemverilog_parseability_generation_report.json"
shadow_report_json="$shadow_state_dir/work/systemverilog_closed_loop_parseability_shadow_report.json"

require_nonempty_file "$generation_report_json"
require_nonempty_file "$shadow_report_json"

if ! jq -e '
    .grammar_name == "systemverilog"
    and .enabled == true
    and (.counterexamples | type == "array")
    and ((.counterexamples | length) <= 20)
    and (
        if (.observed.parser_rejections_total // 0) > 0
        then (.counterexamples | length) > 0
        else true
        end
    )
    and all(
        .counterexamples[]?;
        has("stage")
        and has("sample")
        and has("shrunk_sample")
        and has("profile")
        and has("sample_index")
        and has("seed")
    )
' "$generation_report_json" >/dev/null; then
    echo "error: generation aggregate report contract failed: $generation_report_json" >&2
    cat "$generation_report_json" >&2
    exit 1
fi

if ! jq -e '
    .grammar_name == "systemverilog"
    and .enabled == true
    and .effective_mode == "enabled"
    and (.counterexamples | type == "array")
    and (.counterexamples_captured_total | numbers) >= (.counterexamples | length)
    and ((.counterexamples | length) <= 20)
    and (
        if (.observed.parser_rejections_total // 0) > 0
        then (.counterexamples | length) > 0
        else true
        end
    )
    and ((.profiles | length) >= 1)
    and all(.profiles[]; (.counterexamples_captured | numbers) >= 0)
    and all(
        .counterexamples[]?;
        has("stage")
        and has("sample")
        and has("shrunk_sample")
        and has("profile")
    )
' "$shadow_report_json" >/dev/null; then
    echo "error: replay-shadow aggregate report contract failed: $shadow_report_json" >&2
    cat "$shadow_report_json" >&2
    exit 1
fi

generation_parser_rejections="$(extract_json_number "$generation_report_json" '.observed.parser_rejections_total')"
generation_counterexamples_count="$(extract_json_number "$generation_report_json" '((.counterexamples // []) | length)')"
shadow_parser_rejections="$(extract_json_number "$shadow_report_json" '.observed.parser_rejections_total')"
shadow_counterexamples_count="$(extract_json_number "$shadow_report_json" '((.counterexamples // []) | length)')"
shadow_counterexamples_captured_total="$(extract_json_number "$shadow_report_json" '.counterexamples_captured_total')"

{
    echo "SV Parser Aggregate Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "base_contract_file: $BASE_CONTRACT_FILE"
    echo "generation_contract_file: $generation_contract"
    echo "shadow_contract_file: $shadow_contract"
    echo "generation_report_json: $generation_report_json"
    echo "shadow_report_json: $shadow_report_json"
    echo "generation_parser_rejections_total: $generation_parser_rejections"
    echo "generation_counterexamples_count: $generation_counterexamples_count"
    echo "shadow_parser_rejections_total: $shadow_parser_rejections"
    echo "shadow_counterexamples_count: $shadow_counterexamples_count"
    echo "shadow_counterexamples_captured_total: $shadow_counterexamples_captured_total"
} | tee "$SUMMARY_TXT"

echo "✅ SV parser aggregate contract gate passed."
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
