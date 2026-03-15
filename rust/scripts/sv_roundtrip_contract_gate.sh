#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_ROUNDTRIP_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_roundtrip_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"

SV_CONTRACT_FILE="${PGEN_SV_ROUNDTRIP_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_failure_context_v0_contract.json}"
SV_GATE_SCRIPT="$RUST_DIR/scripts/sv_stimuli_quality_gate.sh"
SV_PARSER_AGGREGATE_SCRIPT="$RUST_DIR/scripts/sv_parser_aggregate_contract_gate.sh"
SVPP_QUALITY_GATE_SCRIPT="$RUST_DIR/scripts/sv_preprocessor_quality_gate.sh"
SVPP_AGGREGATE_SCRIPT="$RUST_DIR/scripts/sv_preprocessor_aggregate_contract_gate.sh"

EXISTING_SV_STIMULI_QUALITY_STATE_DIR="${PGEN_SV_ROUNDTRIP_EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-}"
EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="${PGEN_SV_ROUNDTRIP_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-}"

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

extract_summary_number() {
    local path="$1"
    local key="$2"
    awk -F': ' -v key="$key" '$1 == key { print $2; found = 1 } END { if (!found) exit 1 }' "$path"
}

require_tool awk
require_file "$SV_CONTRACT_FILE"
require_file "$SV_GATE_SCRIPT"
require_file "$SV_PARSER_AGGREGATE_SCRIPT"
require_file "$SVPP_QUALITY_GATE_SCRIPT"
require_file "$SVPP_AGGREGATE_SCRIPT"

mkdir -p "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

sv_quality_state_dir="$WORK_DIR/systemverilog_roundtrip_quality_state"
if [[ -n "$EXISTING_SV_STIMULI_QUALITY_STATE_DIR" ]]; then
    sv_quality_state_dir="$EXISTING_SV_STIMULI_QUALITY_STATE_DIR"
else
    run_logged "systemverilog_roundtrip_quality_gate" env \
        PGEN_SV_STIMULI_QUALITY_CONTRACT="$SV_CONTRACT_FILE" \
        PGEN_SV_STIMULI_QUALITY_STATE_DIR="$sv_quality_state_dir" \
        "$SV_GATE_SCRIPT"
fi

sv_parser_aggregate_state_dir="$WORK_DIR/sv_parser_aggregate_contract_gate"
run_logged "systemverilog_roundtrip_aggregate_contract_gate" env \
    PGEN_SV_PARSER_AGGREGATE_CONTRACT_STATE_DIR="$sv_parser_aggregate_state_dir" \
    PGEN_SV_PARSER_AGGREGATE_CONTRACT_EXISTING_SV_STIMULI_QUALITY_STATE_DIR="$sv_quality_state_dir" \
    "$SV_PARSER_AGGREGATE_SCRIPT"

sv_parser_summary_txt="$sv_parser_aggregate_state_dir/summary.txt"
require_nonempty_file "$sv_parser_summary_txt"

sv_initial_targets="$(extract_summary_number "$sv_parser_summary_txt" "focused_initial_target_count")"
sv_replay_targets="$(extract_summary_number "$sv_parser_summary_txt" "focused_replay_target_count")"
sv_initial_rules="$(extract_summary_number "$sv_parser_summary_txt" "focused_initial_covered_reachable_rules")"
sv_replay_rules="$(extract_summary_number "$sv_parser_summary_txt" "focused_replay_covered_reachable_rules")"
sv_initial_branches="$(extract_summary_number "$sv_parser_summary_txt" "focused_initial_covered_reachable_branches")"
sv_replay_branches="$(extract_summary_number "$sv_parser_summary_txt" "focused_replay_covered_reachable_branches")"

if [[ "$sv_replay_targets" -gt "$sv_initial_targets" ]]; then
    echo "error: main SV replay target debt increased ($sv_initial_targets -> $sv_replay_targets)" >&2
    exit 1
fi
if [[ "$sv_replay_rules" -lt "$sv_initial_rules" ]]; then
    echo "error: main SV reachable-rule coverage regressed ($sv_initial_rules -> $sv_replay_rules)" >&2
    exit 1
fi
if [[ "$sv_replay_branches" -lt "$sv_initial_branches" ]]; then
    echo "error: main SV reachable-branch coverage regressed ($sv_initial_branches -> $sv_replay_branches)" >&2
    exit 1
fi

svpp_quality_state_dir="$WORK_DIR/systemverilog_preprocessor_roundtrip_quality_state"
if [[ -n "$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR" ]]; then
    svpp_quality_state_dir="$EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR"
else
    run_logged "systemverilog_preprocessor_roundtrip_quality_gate" env \
        PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR="$svpp_quality_state_dir" \
        "$SVPP_QUALITY_GATE_SCRIPT"
fi

svpp_aggregate_state_dir="$WORK_DIR/sv_preprocessor_aggregate_contract_gate"
run_logged "systemverilog_preprocessor_roundtrip_aggregate_contract_gate" env \
    PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_STATE_DIR="$svpp_aggregate_state_dir" \
    PGEN_SV_PREPROCESSOR_AGGREGATE_CONTRACT_EXISTING_QUALITY_STATE_DIR="$svpp_quality_state_dir" \
    "$SVPP_AGGREGATE_SCRIPT"

svpp_summary_txt="$svpp_aggregate_state_dir/summary.txt"
require_nonempty_file "$svpp_summary_txt"

svpp_stage0_targets="$(extract_summary_number "$svpp_summary_txt" "stage0_target_count")"
svpp_stage1_targets="$(extract_summary_number "$svpp_summary_txt" "stage1_target_count")"
svpp_final_targets="$(extract_summary_number "$svpp_summary_txt" "final_targets")"
svpp_stage4_targets="$(extract_summary_number "$svpp_summary_txt" "stage4_target_count")"
svpp_stage0_rules="$(extract_summary_number "$svpp_summary_txt" "stage0_covered_reachable_rules")"
svpp_stage1_rules="$(extract_summary_number "$svpp_summary_txt" "stage1_covered_reachable_rules")"
svpp_stage4_rules="$(extract_summary_number "$svpp_summary_txt" "stage4_covered_reachable_rules")"
svpp_stage0_branches="$(extract_summary_number "$svpp_summary_txt" "stage0_covered_reachable_branches")"
svpp_stage1_branches="$(extract_summary_number "$svpp_summary_txt" "stage1_covered_reachable_branches")"
svpp_stage4_branches="$(extract_summary_number "$svpp_summary_txt" "stage4_covered_reachable_branches")"

if [[ "$svpp_stage1_targets" -gt "$svpp_stage0_targets" ]]; then
    echo "error: preprocessor stage1 target debt increased ($svpp_stage0_targets -> $svpp_stage1_targets)" >&2
    exit 1
fi
if [[ "$svpp_final_targets" -gt "$svpp_stage1_targets" ]]; then
    echo "error: preprocessor final target debt increased after stage1 ($svpp_stage1_targets -> $svpp_final_targets)" >&2
    exit 1
fi
if [[ "$svpp_stage4_targets" -gt "$svpp_final_targets" ]]; then
    echo "error: preprocessor stage4 target debt increased after final closure ($svpp_final_targets -> $svpp_stage4_targets)" >&2
    exit 1
fi

{
    echo "SV Roundtrip Contract Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "sv_contract_file: $SV_CONTRACT_FILE"
    echo "existing_sv_stimuli_quality_state_dir: ${EXISTING_SV_STIMULI_QUALITY_STATE_DIR:-<unset>}"
    echo "existing_sv_preprocessor_quality_state_dir: ${EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR:-<unset>}"
    echo "systemverilog_roundtrip_quality_state_dir: $sv_quality_state_dir"
    echo "systemverilog_roundtrip_aggregate_state_dir: $sv_parser_aggregate_state_dir"
    echo "systemverilog_roundtrip_initial_targets: $sv_initial_targets"
    echo "systemverilog_roundtrip_replay_targets: $sv_replay_targets"
    echo "systemverilog_roundtrip_initial_covered_reachable_rules: $sv_initial_rules"
    echo "systemverilog_roundtrip_replay_covered_reachable_rules: $sv_replay_rules"
    echo "systemverilog_roundtrip_initial_covered_reachable_branches: $sv_initial_branches"
    echo "systemverilog_roundtrip_replay_covered_reachable_branches: $sv_replay_branches"
    echo "systemverilog_preprocessor_roundtrip_quality_state_dir: $svpp_quality_state_dir"
    echo "systemverilog_preprocessor_roundtrip_aggregate_state_dir: $svpp_aggregate_state_dir"
    echo "systemverilog_preprocessor_roundtrip_stage0_targets: $svpp_stage0_targets"
    echo "systemverilog_preprocessor_roundtrip_stage1_targets: $svpp_stage1_targets"
    echo "systemverilog_preprocessor_roundtrip_final_targets: $svpp_final_targets"
    echo "systemverilog_preprocessor_roundtrip_stage4_targets: $svpp_stage4_targets"
    echo "systemverilog_preprocessor_roundtrip_stage0_covered_reachable_rules: $svpp_stage0_rules"
    echo "systemverilog_preprocessor_roundtrip_stage1_covered_reachable_rules: $svpp_stage1_rules"
    echo "systemverilog_preprocessor_roundtrip_stage4_covered_reachable_rules: $svpp_stage4_rules"
    echo "systemverilog_preprocessor_roundtrip_stage0_covered_reachable_branches: $svpp_stage0_branches"
    echo "systemverilog_preprocessor_roundtrip_stage1_covered_reachable_branches: $svpp_stage1_branches"
    echo "systemverilog_preprocessor_roundtrip_stage4_covered_reachable_branches: $svpp_stage4_branches"
} >"$SUMMARY_TXT"

require_nonempty_file "$SUMMARY_TXT"
cat "$SUMMARY_TXT"
echo "Logs: $LOG_DIR"
echo "Artifacts: $WORK_DIR"
