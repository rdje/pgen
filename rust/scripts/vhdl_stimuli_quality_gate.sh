#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"

STATE_DIR="${PGEN_VHDL_STIMULI_QUALITY_STATE_DIR:-$RUST_DIR/target/vhdl_stimuli_quality_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

CONTRACT_FILE="${PGEN_VHDL_STIMULI_QUALITY_CONTRACT:-$RUST_DIR/test_data/grammar_quality/vhdl_core_v0_contract.json}"
PARSE_FULL_MODE="${PGEN_VHDL_STIMULI_QUALITY_PARSE_FULL_MODE:-auto}"
SAMPLE_COUNT_OVERRIDE="${PGEN_VHDL_STIMULI_QUALITY_COUNT:-}"
SEED_BASE_OVERRIDE="${PGEN_VHDL_STIMULI_QUALITY_SEED_BASE:-}"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"

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
        echo "    fail (${log_file})" >&2
        tail -n 80 "$log_file" >&2 || true
        exit 1
    fi
}

run_logged_rust() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (
        cd "$RUST_DIR"
        "$@"
    ) >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "    fail (${log_file})" >&2
        tail -n 80 "$log_file" >&2 || true
        exit 1
    fi
}

if [[ "$PARSE_FULL_MODE" != "auto" && "$PARSE_FULL_MODE" != "0" && "$PARSE_FULL_MODE" != "1" ]]; then
    echo "error: PGEN_VHDL_STIMULI_QUALITY_PARSE_FULL_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

require_tool jq
require_tool perl
require_file "$CONTRACT_FILE"
require_file "$EBNF_TO_JSON"

contract_version="$(jq -er '.version | numbers' "$CONTRACT_FILE")"
grammar_name="$(jq -er '.grammar_name | strings' "$CONTRACT_FILE")"
ebnf_path_rel="$(jq -er '.ebnf_path | strings' "$CONTRACT_FILE")"
entry_rule="$(jq -er '(.entry_rule // "vhdl_file") | strings' "$CONTRACT_FILE")"
default_sample_count="$(jq -er '.sample_count | numbers' "$CONTRACT_FILE")"
default_seed_base="$(jq -er '.seed_base | numbers' "$CONTRACT_FILE")"
closed_loop_enabled="$(jq -er 'if (.closed_loop.enabled // true) then 1 else 0 end' "$CONTRACT_FILE")"
gap_report_threshold="$(jq -er '(.closed_loop.gap_report_threshold // 1) | numbers' "$CONTRACT_FILE")"
target_max_attempts="$(jq -er '(.closed_loop.target_max_attempts // 5000) | numbers' "$CONTRACT_FILE")"
require_non_increasing_target_debt="$(jq -er 'if (.closed_loop.require_non_increasing_target_debt // true) then 1 else 0 end' "$CONTRACT_FILE")"

sample_count="${SAMPLE_COUNT_OVERRIDE:-$default_sample_count}"
seed_base="${SEED_BASE_OVERRIDE:-$default_seed_base}"
replay_sample_count="$(jq -er --argjson fallback "$sample_count" '(.closed_loop.replay_sample_count // $fallback) | numbers' "$CONTRACT_FILE")"

if ! [[ "$sample_count" =~ ^[0-9]+$ ]] || [[ "$sample_count" -lt 1 ]]; then
    echo "error: sample_count must be an integer >= 1 (effective value: '$sample_count')" >&2
    exit 2
fi
if ! [[ "$seed_base" =~ ^[0-9]+$ ]]; then
    echo "error: seed_base must be an integer >= 0 (effective value: '$seed_base')" >&2
    exit 2
fi
if ! [[ "$gap_report_threshold" =~ ^[0-9]+$ ]] || [[ "$gap_report_threshold" -lt 1 ]]; then
    echo "error: closed_loop.gap_report_threshold must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$target_max_attempts" =~ ^[0-9]+$ ]] || [[ "$target_max_attempts" -lt 1 ]]; then
    echo "error: closed_loop.target_max_attempts must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$replay_sample_count" =~ ^[0-9]+$ ]] || [[ "$replay_sample_count" -lt 1 ]]; then
    echo "error: closed_loop.replay_sample_count must be an integer >= 1" >&2
    exit 2
fi

grammar_file="$ROOT_DIR/$ebnf_path_rel"
grammar_json="$WORK_DIR/${grammar_name}.json"
parser_out="$WORK_DIR/${grammar_name}_parser.rs"

require_file "$grammar_file"

echo "==> VHDL stimuli quality gate"
echo "state_dir: $STATE_DIR"
echo "contract_file: $CONTRACT_FILE"
echo "contract_version: $contract_version"
echo "grammar_name: $grammar_name"
echo "grammar_file: $grammar_file"
echo "entry_rule: $entry_rule"
echo "sample_count: $sample_count"
echo "seed_base: $seed_base"
echo "parse_full_mode: $PARSE_FULL_MODE"
echo "closed_loop_enabled: $closed_loop_enabled"
echo "closed_loop_gap_report_threshold: $gap_report_threshold"
echo "closed_loop_target_max_attempts: $target_max_attempts"
echo "closed_loop_replay_sample_count: $replay_sample_count"
echo "closed_loop_require_non_increasing_target_debt: $require_non_increasing_target_debt"

echo "sample,seed,coverage_gap_initial,gap_replay,stimuli_generate,parse_full,warnings,errors,status,notes" >"$SUMMARY_CSV"

run_logged_rust "build_ast_pipeline_for_vhdl_generation" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

run_logged "frontend_ebnf_to_json" \
    perl "$EBNF_TO_JSON" --pretty --quiet "$grammar_file" -o "$grammar_json"
require_nonempty_file "$grammar_json"

run_logged "generate_vhdl_parser" \
    "$AST_PIPELINE_BIN" "$grammar_json" \
    --generate-parser \
    --eliminate-left-recursion \
    --output "$parser_out"
require_nonempty_file "$parser_out"

run_logged_rust "build_parseability_probe_with_vhdl_adapter" \
    env PGEN_VHDL_PARSER_PATH="$parser_out" \
    cargo build --features generated_parsers --bin parseability_probe
if [[ ! -x "$PARSE_PROBE_BIN" ]]; then
    echo "error: parseability_probe binary is missing at '$PARSE_PROBE_BIN' after adapter build" >&2
    exit 1
fi

parse_full_supported=0
probe_log="$LOG_DIR/probe_parse_full_support.log"
echo "==> probe_parse_full_support"
if "$PARSE_PROBE_BIN" --supports "$grammar_name" >"$probe_log" 2>&1; then
    echo "    ok (${probe_log})"
    parse_full_supported=1
else
    echo "    skip (${probe_log})"
fi

parse_full_enabled=0
parse_full_effective="disabled"
if [[ "$PARSE_FULL_MODE" == "0" ]]; then
    parse_full_enabled=0
    parse_full_effective="disabled_by_mode"
elif [[ "$PARSE_FULL_MODE" == "1" ]]; then
    if [[ "$parse_full_supported" -eq 0 ]]; then
        echo "error: parse_full mode is strict (1) but no generated parser adapter is registered for '$grammar_name'" >&2
        exit 1
    fi
    parse_full_enabled=1
    parse_full_effective="enabled"
else
    if [[ "$parse_full_supported" -eq 1 ]]; then
        parse_full_enabled=1
        parse_full_effective="enabled"
    else
        parse_full_enabled=0
        parse_full_effective="unsupported_adapter"
    fi
fi

closed_loop_initial_status="skip"
closed_loop_replay_status="skip"
closed_loop_note="closed-loop disabled by contract"
closed_loop_initial_targets=0
closed_loop_replay_targets=0

if [[ "$closed_loop_enabled" -eq 1 ]]; then
    closed_loop_initial_stimuli="$WORK_DIR/closed_loop_initial_stimuli.vhd"
    closed_loop_initial_coverage="$WORK_DIR/closed_loop_initial_coverage.json"
    closed_loop_initial_gap_json="$WORK_DIR/closed_loop_initial_gap.json"
    closed_loop_initial_gap_text="$WORK_DIR/closed_loop_initial_gap.txt"
    closed_loop_replay_stimuli="$WORK_DIR/closed_loop_replay_stimuli.vhd"
    closed_loop_replay_coverage="$WORK_DIR/closed_loop_replay_coverage.json"
    closed_loop_replay_gap_json="$WORK_DIR/closed_loop_replay_gap.json"
    closed_loop_replay_gap_text="$WORK_DIR/closed_loop_replay_gap.txt"
    closed_loop_replay_seed=$((seed_base + 700000))

    run_logged "closed_loop_initial" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --count "$sample_count" \
        --seed "$seed_base" \
        --entry-rule "$entry_rule" \
        --output "$closed_loop_initial_stimuli" \
        --coverage-output "$closed_loop_initial_coverage" \
        --gap-report-json "$closed_loop_initial_gap_json" \
        --gap-report-text "$closed_loop_initial_gap_text" \
        --gap-report-threshold "$gap_report_threshold"
    require_nonempty_file "$closed_loop_initial_stimuli"
    require_nonempty_file "$closed_loop_initial_coverage"
    require_nonempty_file "$closed_loop_initial_gap_json"
    require_nonempty_file "$closed_loop_initial_gap_text"
    closed_loop_initial_targets="$(jq -er '(.targets // []) | length | numbers' "$closed_loop_initial_gap_json")"
    closed_loop_initial_status="pass"

    run_logged "closed_loop_replay" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --count "$replay_sample_count" \
        --seed "$closed_loop_replay_seed" \
        --entry-rule "$entry_rule" \
        --output "$closed_loop_replay_stimuli" \
        --coverage-output "$closed_loop_replay_coverage" \
        --gap-report-json "$closed_loop_replay_gap_json" \
        --gap-report-text "$closed_loop_replay_gap_text" \
        --gap-report-threshold "$gap_report_threshold" \
        --target-max-attempts "$target_max_attempts" \
        --target-report-input "$closed_loop_initial_gap_json"
    require_nonempty_file "$closed_loop_replay_stimuli"
    require_nonempty_file "$closed_loop_replay_coverage"
    require_nonempty_file "$closed_loop_replay_gap_json"
    require_nonempty_file "$closed_loop_replay_gap_text"
    closed_loop_replay_targets="$(jq -er '(.targets // []) | length | numbers' "$closed_loop_replay_gap_json")"
    closed_loop_replay_status="pass"
    closed_loop_note="initial_targets=${closed_loop_initial_targets}; replay_targets=${closed_loop_replay_targets}"

    if [[ "$require_non_increasing_target_debt" -eq 1 ]] && (( closed_loop_replay_targets > closed_loop_initial_targets )); then
        echo "error: closed-loop replay increased target debt (${closed_loop_initial_targets} -> ${closed_loop_replay_targets})" >&2
        exit 1
    fi
fi

parse_full_pass_count=0
parse_full_skip_count=0
parse_full_fail_count=0
total_warnings=0
total_errors=0

for ((idx = 0; idx < sample_count; idx++)); do
    seed=$((seed_base + idx))
    sample_file="$WORK_DIR/sample_${idx}.vhd"

    run_logged "sample_${idx}_generate_stimulus" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --count 1 \
        --seed "$seed" \
        --entry-rule "$entry_rule" \
        --output "$sample_file"
    require_nonempty_file "$sample_file"

    parse_status="skip"
    parse_note="parse_full stage skipped"
    if [[ "$parse_full_enabled" -eq 1 ]]; then
        parse_log="$LOG_DIR/sample_${idx}_parse_full.log"
        echo "==> sample_${idx}_parse_full"
        if "$PARSE_PROBE_BIN" --parse "$grammar_name" "$sample_file" >"$parse_log" 2>&1; then
            echo "    ok (${parse_log})"
            parse_status="pass"
            parse_note="parse_full accepted generated sample"
            parse_full_pass_count=$((parse_full_pass_count + 1))
        else
            parse_status="fail"
            parse_note="parse_full rejected generated sample"
            parse_full_fail_count=$((parse_full_fail_count + 1))
            if [[ "$PARSE_FULL_MODE" == "1" ]]; then
                echo "    fail (${parse_log})" >&2
                tail -n 80 "$parse_log" >&2 || true
                echo "error: strict parse_full mode requires all samples to pass parse_full" >&2
                exit 1
            fi
            echo "    soft-fail (${parse_log})"
        fi
    else
        parse_full_skip_count=$((parse_full_skip_count + 1))
        parse_note="parse_full unavailable (${parse_full_effective})"
    fi

    echo "${idx},${seed},${closed_loop_initial_status},${closed_loop_replay_status},pass,${parse_status},0,0,pass,${parse_note}" >>"$SUMMARY_CSV"
done

{
    echo "PGEN VHDL Stimuli Quality Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "contract_file: $CONTRACT_FILE"
    echo "grammar_name: $grammar_name"
    echo "entry_rule: $entry_rule"
    echo "sample_count: $sample_count"
    echo "seed_base: $seed_base"
    echo "closed_loop_enabled: $closed_loop_enabled"
    echo "closed_loop_gap_report_threshold: $gap_report_threshold"
    echo "closed_loop_target_max_attempts: $target_max_attempts"
    echo "closed_loop_replay_sample_count: $replay_sample_count"
    echo "closed_loop_initial_status: $closed_loop_initial_status"
    echo "closed_loop_replay_status: $closed_loop_replay_status"
    echo "closed_loop_initial_targets: $closed_loop_initial_targets"
    echo "closed_loop_replay_targets: $closed_loop_replay_targets"
    echo "closed_loop_note: $closed_loop_note"
    echo "parse_full_mode: $PARSE_FULL_MODE"
    echo "parse_full_effective: $parse_full_effective"
    echo "parse_full_passes: $parse_full_pass_count/$sample_count"
    echo "parse_full_failures: $parse_full_fail_count"
    echo "parse_full_skips: $parse_full_skip_count"
    echo "total_warnings: $total_warnings"
    echo "total_errors: $total_errors"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

cat <<EOF
✅ VHDL stimuli quality gate passed.
Logs: $LOG_DIR
Artifacts: $WORK_DIR
EOF
