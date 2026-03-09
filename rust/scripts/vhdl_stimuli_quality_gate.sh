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
PARSEABILITY_MAX_ATTEMPTS_OVERRIDE="${PGEN_VHDL_STIMULI_QUALITY_PARSEABILITY_MAX_ATTEMPTS:-}"
REALISTIC_CORPUS_MODE="${PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MODE:-auto}"
REALISTIC_CORPUS_OVERRIDE="${PGEN_VHDL_STIMULI_REALISTIC_CORPUS:-}"
REALISTIC_CORPUS_MAX_CASES="${PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MAX_CASES:-0}"

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

parseability_summary_field_u64() {
    local path="$1"
    local field="$2"
    jq -er ".summary.${field} | numbers" "$path"
}

parseability_acceptance_rate_percent() {
    local path="$1"
    local attempts accepted
    attempts="$(parseability_summary_field_u64 "$path" "attempts")"
    accepted="$(parseability_summary_field_u64 "$path" "accepted")"
    perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$accepted" "$attempts"
}

now_ms() {
    perl -MTime::HiRes=time -e 'printf "%.0f\n", time()*1000'
}

file_size_bytes() {
    perl -e 'my $f = shift; my $s = -s $f; print defined($s) ? $s : 0;' "$1"
}

enforce_threshold_le() {
    local metric="$1"
    local value="$2"
    local max_allowed="$3"
    local context="$4"
    if [[ "$max_allowed" -gt 0 && "$value" -gt "$max_allowed" ]]; then
        echo "error: ${metric} budget exceeded for ${context} (${value} > ${max_allowed})" >&2
        exit 1
    fi
}

resolve_path() {
    local raw="$1"
    if [[ "$raw" == /* ]]; then
        printf '%s\n' "$raw"
    else
        printf '%s\n' "$ROOT_DIR/$raw"
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
if [[ "$REALISTIC_CORPUS_MODE" != "auto" && "$REALISTIC_CORPUS_MODE" != "0" && "$REALISTIC_CORPUS_MODE" != "1" ]]; then
    echo "error: PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$REALISTIC_CORPUS_MAX_CASES" =~ ^[0-9]+$ ]] || [[ "$REALISTIC_CORPUS_MAX_CASES" -lt 0 ]]; then
    echo "error: PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MAX_CASES must be an integer >= 0" >&2
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
parseability_generation_contract_enabled="$(jq -er 'if (.parseability_generation.enabled // false) then 1 else 0 end' "$CONTRACT_FILE")"
default_parseability_max_attempts_per_sample="$(jq -er '(.parseability_generation.max_attempts_per_sample // 50) | numbers' "$CONTRACT_FILE")"
realistic_corpus_contract_enforced="$(jq -er 'if (.realistic_corpus.enforce // false) then 1 else 0 end' "$CONTRACT_FILE")"
realistic_corpus_rel_default="$(jq -er '(.realistic_corpus.cases_path // "") | strings' "$CONTRACT_FILE")"
realistic_max_parse_full_ms_per_case="$(jq -er '(.realistic_corpus.max_parse_full_ms_per_case // 0) | numbers' "$CONTRACT_FILE")"
realistic_max_sample_bytes="$(jq -er '(.realistic_corpus.max_sample_bytes // 0) | numbers' "$CONTRACT_FILE")"

sample_count="${SAMPLE_COUNT_OVERRIDE:-$default_sample_count}"
seed_base="${SEED_BASE_OVERRIDE:-$default_seed_base}"
parseability_max_attempts_per_sample="${PARSEABILITY_MAX_ATTEMPTS_OVERRIDE:-$default_parseability_max_attempts_per_sample}"
replay_sample_count="$(jq -er --argjson fallback "$sample_count" '(.closed_loop.replay_sample_count // $fallback) | numbers' "$CONTRACT_FILE")"
realistic_corpus_rel="$realistic_corpus_rel_default"
if [[ -n "$REALISTIC_CORPUS_OVERRIDE" ]]; then
    realistic_corpus_rel="$REALISTIC_CORPUS_OVERRIDE"
fi
realistic_corpus_path=""
if [[ -n "$realistic_corpus_rel" ]]; then
    realistic_corpus_path="$(resolve_path "$realistic_corpus_rel")"
fi

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
if ! [[ "$parseability_max_attempts_per_sample" =~ ^[0-9]+$ ]] || [[ "$parseability_max_attempts_per_sample" -lt 1 ]]; then
    echo "error: parseability_generation.max_attempts_per_sample must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$realistic_max_parse_full_ms_per_case" =~ ^[0-9]+$ ]]; then
    echo "error: realistic_corpus.max_parse_full_ms_per_case must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$realistic_max_sample_bytes" =~ ^[0-9]+$ ]]; then
    echo "error: realistic_corpus.max_sample_bytes must be an integer >= 0" >&2
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
echo "realistic_corpus_mode: $REALISTIC_CORPUS_MODE"
echo "realistic_corpus_contract_enforced: $realistic_corpus_contract_enforced"
echo "realistic_corpus_path: ${realistic_corpus_path:-<unset>}"
echo "realistic_corpus_max_cases: $REALISTIC_CORPUS_MAX_CASES"
echo "realistic_corpus_max_parse_full_ms_per_case: $realistic_max_parse_full_ms_per_case"
echo "realistic_corpus_max_sample_bytes: $realistic_max_sample_bytes"
echo "closed_loop_enabled: $closed_loop_enabled"
echo "closed_loop_gap_report_threshold: $gap_report_threshold"
echo "closed_loop_target_max_attempts: $target_max_attempts"
echo "closed_loop_replay_sample_count: $replay_sample_count"
echo "closed_loop_require_non_increasing_target_debt: $require_non_increasing_target_debt"
echo "parseability_generation_contract_enabled: $parseability_generation_contract_enabled"
echo "parseability_generation_max_attempts_per_sample: $parseability_max_attempts_per_sample"

echo "sample,seed,coverage_gap_initial,gap_replay,stimuli_generate,parseability_attempts,parseability_accepted,parseability_rejected,parseability_parser_rejections,parseability_generation_errors,parseability_empty_generations,parseability_acceptance_rate_percent,parse_full,warnings,errors,status,notes" >"$SUMMARY_CSV"

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

run_logged_rust "build_ast_pipeline_and_parseability_probe_with_vhdl_adapter" \
    env PGEN_VHDL_PARSER_PATH="$parser_out" \
    cargo build --features generated_parsers --bin ast_pipeline --bin parseability_probe
if [[ ! -x "$PARSE_PROBE_BIN" ]]; then
    echo "error: parseability_probe binary is missing at '$PARSE_PROBE_BIN' after adapter build" >&2
    exit 1
fi
if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after adapter build" >&2
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

realistic_corpus_enabled=0
realistic_corpus_effective="disabled_by_mode"
realistic_corpus_note="realistic corpus validation disabled by mode"
if [[ "$REALISTIC_CORPUS_MODE" == "1" ]]; then
    realistic_corpus_enabled=1
    realistic_corpus_effective="enabled"
    realistic_corpus_note="realistic corpus validation enabled by strict mode"
elif [[ "$REALISTIC_CORPUS_MODE" == "auto" ]]; then
    if [[ "$realistic_corpus_contract_enforced" -eq 1 ]]; then
        realistic_corpus_enabled=1
        realistic_corpus_effective="enabled"
        realistic_corpus_note="realistic corpus validation enabled by contract"
    else
        realistic_corpus_enabled=0
        realistic_corpus_effective="disabled_by_contract"
        realistic_corpus_note="realistic corpus validation disabled by contract"
    fi
fi

if [[ "$realistic_corpus_enabled" -eq 1 ]]; then
    if [[ "$parse_full_supported" -ne 1 ]]; then
        echo "error: realistic corpus validation requires generated parser adapter for '$grammar_name'" >&2
        exit 1
    fi
    if [[ -z "$realistic_corpus_path" ]]; then
        echo "error: realistic corpus validation is enabled but no corpus path is configured" >&2
        exit 1
    fi
    require_file "$realistic_corpus_path"
fi

parseability_generation_enabled=0
parseability_generation_effective="disabled_by_contract"
if [[ "$parse_full_enabled" -eq 1 && "$parseability_generation_contract_enabled" -eq 1 ]]; then
    parseability_generation_enabled=1
    parseability_generation_effective="enabled"
elif [[ "$parse_full_enabled" -ne 1 ]]; then
    parseability_generation_effective="disabled_by_parse_full_mode"
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
parseability_generation_requested_total=0
parseability_generation_attempts_total=0
parseability_generation_accepted_total=0
parseability_generation_rejected_total=0
parseability_generation_parser_rejections_total=0
parseability_generation_errors_total=0
parseability_generation_empty_generations_total=0
parseability_generation_acceptance_rate_percent="0.00"
parseability_generation_report_json="$WORK_DIR/${grammar_name}_parseability_generation_report.json"
total_warnings=0
total_errors=0
realistic_cases_declared=0
realistic_cases_executed=0
realistic_expected_pass_total=0
realistic_expected_fail_total=0
realistic_parse_pass_total=0
realistic_parse_fail_total=0
realistic_expected_fail_parse_pass_total=0
realistic_parse_total_ms=0
realistic_parse_max_ms=0
realistic_sample_bytes_max=0
realistic_report_json="$WORK_DIR/${grammar_name}_realistic_corpus_report.json"
realistic_cases_jsonl="$WORK_DIR/${grammar_name}_realistic_corpus_cases.jsonl"

for ((idx = 0; idx < sample_count; idx++)); do
    seed=$((seed_base + idx))
    sample_file="$WORK_DIR/sample_${idx}.vhd"
    parseability_report_json="$WORK_DIR/sample_${idx}.parseability_generation.json"
    parseability_attempts=0
    parseability_accepted=0
    parseability_rejected=0
    parseability_parser_rejections=0
    parseability_generation_errors=0
    parseability_empty_generations=0
    parseability_acceptance_rate_percent="0.00"
    parseability_args=()
    if [[ "$parseability_generation_enabled" -eq 1 ]]; then
        parseability_args=(
            --validate-parseability
            --parseability-max-attempts "$parseability_max_attempts_per_sample"
            --parseability-report-json "$parseability_report_json"
        )
    fi

    run_logged "sample_${idx}_generate_stimulus" \
        "$AST_PIPELINE_BIN" "$grammar_json" \
        --generate-stimuli \
        --count 1 \
        --seed "$seed" \
        --entry-rule "$entry_rule" \
        --output "$sample_file" \
        "${parseability_args[@]}"
    require_nonempty_file "$sample_file"

    if [[ "$parseability_generation_enabled" -eq 1 ]]; then
        require_nonempty_file "$parseability_report_json"
        if ! jq -e ".grammar_name == \"$grammar_name\" and .summary.requested == 1 and .summary.accepted == 1 and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "$parseability_report_json" >/dev/null; then
            echo "error: parseability report validation failed for sample ${idx}: $parseability_report_json" >&2
            exit 1
        fi
        parseability_requested="$(parseability_summary_field_u64 "$parseability_report_json" "requested")"
        parseability_attempts="$(parseability_summary_field_u64 "$parseability_report_json" "attempts")"
        parseability_accepted="$(parseability_summary_field_u64 "$parseability_report_json" "accepted")"
        parseability_rejected="$(parseability_summary_field_u64 "$parseability_report_json" "rejected")"
        parseability_parser_rejections="$(parseability_summary_field_u64 "$parseability_report_json" "parser_rejections")"
        parseability_generation_errors="$(parseability_summary_field_u64 "$parseability_report_json" "generation_errors")"
        parseability_empty_generations="$(parseability_summary_field_u64 "$parseability_report_json" "empty_generations")"
        parseability_acceptance_rate_percent="$(parseability_acceptance_rate_percent "$parseability_report_json")"
        parseability_generation_requested_total=$((parseability_generation_requested_total + parseability_requested))
        parseability_generation_attempts_total=$((parseability_generation_attempts_total + parseability_attempts))
        parseability_generation_accepted_total=$((parseability_generation_accepted_total + parseability_accepted))
        parseability_generation_rejected_total=$((parseability_generation_rejected_total + parseability_rejected))
        parseability_generation_parser_rejections_total=$((parseability_generation_parser_rejections_total + parseability_parser_rejections))
        parseability_generation_errors_total=$((parseability_generation_errors_total + parseability_generation_errors))
        parseability_generation_empty_generations_total=$((parseability_generation_empty_generations_total + parseability_empty_generations))
    fi

    parse_status="skip"
    parse_note="parse_full stage skipped"
    if [[ "$parse_full_enabled" -eq 1 ]]; then
        parse_log="$LOG_DIR/sample_${idx}_parse_full.log"
        echo "==> sample_${idx}_parse_full"
        if "$PARSE_PROBE_BIN" --parse "$grammar_name" "$sample_file" >"$parse_log" 2>&1; then
            echo "    ok (${parse_log})"
            parse_status="pass"
            if [[ "$parseability_generation_enabled" -eq 1 ]]; then
                parse_note="parse_full accepted parser-backed generated sample"
            else
                parse_note="parse_full accepted generated sample"
            fi
            parse_full_pass_count=$((parse_full_pass_count + 1))
        else
            parse_status="fail"
            if [[ "$parseability_generation_enabled" -eq 1 ]]; then
                parse_note="parse_full rejected parser-backed generated sample"
            else
                parse_note="parse_full rejected generated sample"
            fi
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

    echo "${idx},${seed},${closed_loop_initial_status},${closed_loop_replay_status},pass,${parseability_attempts},${parseability_accepted},${parseability_rejected},${parseability_parser_rejections},${parseability_generation_errors},${parseability_empty_generations},${parseability_acceptance_rate_percent},${parse_status},0,0,pass,${parse_note}" >>"$SUMMARY_CSV"
done

parseability_generation_acceptance_rate_percent="$(perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$parseability_generation_accepted_total" "$parseability_generation_attempts_total")"

jq -n \
    --arg grammar_name "$grammar_name" \
    --arg effective_mode "$parseability_generation_effective" \
    --argjson enabled "$parseability_generation_enabled" \
    --argjson requested_total "$parseability_generation_requested_total" \
    --argjson attempts_total "$parseability_generation_attempts_total" \
    --argjson accepted_total "$parseability_generation_accepted_total" \
    --argjson rejected_total "$parseability_generation_rejected_total" \
    --argjson parser_rejections_total "$parseability_generation_parser_rejections_total" \
    --argjson generation_errors_total "$parseability_generation_errors_total" \
    --argjson empty_generations_total "$parseability_generation_empty_generations_total" \
    --argjson max_attempts_per_sample "$parseability_max_attempts_per_sample" \
    --arg acceptance_rate_percent "$parseability_generation_acceptance_rate_percent" \
    '{
        grammar_name: $grammar_name,
        effective_mode: $effective_mode,
        enabled: ($enabled == 1),
        max_attempts_per_sample: $max_attempts_per_sample,
        totals: {
            requested_total: $requested_total,
            attempts_total: $attempts_total,
            accepted_total: $accepted_total,
            rejected_total: $rejected_total,
            parser_rejections_total: $parser_rejections_total,
            generation_errors_total: $generation_errors_total,
            empty_generations_total: $empty_generations_total,
            acceptance_rate_percent: ($acceptance_rate_percent | tonumber)
        }
    }' >"$parseability_generation_report_json"

realistic_cases_json='[]'
if [[ "$realistic_corpus_enabled" -eq 1 ]]; then
    : >"$realistic_cases_jsonl"
    mapfile -t realistic_case_rows < <(jq -c '.cases[]?' "$realistic_corpus_path")
    realistic_cases_declared="${#realistic_case_rows[@]}"
    if (( realistic_cases_declared == 0 )); then
        echo "error: realistic corpus has zero cases: $realistic_corpus_path" >&2
        exit 1
    fi

    realistic_case_manifest_idx=0
    for case_json in "${realistic_case_rows[@]}"; do
        if (( REALISTIC_CORPUS_MAX_CASES > 0 && realistic_case_manifest_idx >= REALISTIC_CORPUS_MAX_CASES )); then
            break
        fi
        realistic_case_manifest_idx=$((realistic_case_manifest_idx + 1))

        case_name="$(jq -er '.name | strings' <<<"$case_json")"
        case_source_rel="$(jq -er '.path | strings' <<<"$case_json")"
        case_expect_parse_full_pass="$(jq -er 'if has("expect_parse_full_pass") then (if .expect_parse_full_pass then 1 else 0 end) else 1 end' <<<"$case_json")"
        case_source_path="$(resolve_path "$case_source_rel")"
        require_file "$case_source_path"

        if [[ "$case_expect_parse_full_pass" -eq 1 ]]; then
            realistic_expected_pass_total=$((realistic_expected_pass_total + 1))
        else
            realistic_expected_fail_total=$((realistic_expected_fail_total + 1))
        fi

        case_name_key="$(printf '%s' "$case_name" | tr -c 'A-Za-z0-9_' '_')"
        case_log="$LOG_DIR/realistic_case_${case_name_key}_parse_full.log"
        case_sample_bytes="$(file_size_bytes "$case_source_path")"
        if (( case_sample_bytes > realistic_sample_bytes_max )); then
            realistic_sample_bytes_max="$case_sample_bytes"
        fi
        enforce_threshold_le "realistic_sample_bytes" "$case_sample_bytes" "$realistic_max_sample_bytes" "case=${case_name}"

        case_parse_started_ms="$(now_ms)"
        if "$PARSE_PROBE_BIN" --parse "$grammar_name" "$case_source_path" >"$case_log" 2>&1; then
            case_parse_status="pass"
            realistic_parse_pass_total=$((realistic_parse_pass_total + 1))
        else
            case_parse_status="fail"
            realistic_parse_fail_total=$((realistic_parse_fail_total + 1))
        fi
        case_parse_elapsed_ms=$(( $(now_ms) - case_parse_started_ms ))
        realistic_parse_total_ms=$((realistic_parse_total_ms + case_parse_elapsed_ms))
        if (( case_parse_elapsed_ms > realistic_parse_max_ms )); then
            realistic_parse_max_ms="$case_parse_elapsed_ms"
        fi
        enforce_threshold_le "realistic_parse_full_ms_per_case" "$case_parse_elapsed_ms" "$realistic_max_parse_full_ms_per_case" "case=${case_name}"

        case_status="pass"
        case_note="parse_full status '${case_parse_status}' matched minimum expectation"
        if [[ "$case_expect_parse_full_pass" -eq 1 && "$case_parse_status" != "pass" ]]; then
            case_status="fail"
            case_note="expected parse_full pass but observed fail"
            echo "error: realistic corpus case '$case_name' failed required parse_full pass" >&2
            tail -n 80 "$case_log" >&2 || true
            exit 1
        elif [[ "$case_expect_parse_full_pass" -eq 0 && "$case_parse_status" == "pass" ]]; then
            realistic_expected_fail_parse_pass_total=$((realistic_expected_fail_parse_pass_total + 1))
            case_note="parse_full passed on expected-fail case (improvement signal)"
        fi

        jq -n \
            --arg case_name "$case_name" \
            --arg source_file "$case_source_path" \
            --arg parse_log_file "$case_log" \
            --arg parse_status "$case_parse_status" \
            --arg status "$case_status" \
            --arg note "$case_note" \
            --argjson expect_parse_full_pass "$case_expect_parse_full_pass" \
            --argjson parse_full_ms "$case_parse_elapsed_ms" \
            --argjson sample_bytes "$case_sample_bytes" \
            '{
                case_name: $case_name,
                source_file: $source_file,
                parse_log_file: $parse_log_file,
                expect_parse_full_pass: ($expect_parse_full_pass == 1),
                parse_status: $parse_status,
                status: $status,
                note: $note,
                observed: {
                    parse_full_ms: $parse_full_ms,
                    sample_bytes: $sample_bytes
                }
            }' >>"$realistic_cases_jsonl"

        realistic_cases_executed=$((realistic_cases_executed + 1))
    done

    if (( realistic_cases_executed == 0 )); then
        echo "error: realistic corpus validation is enabled but no cases executed" >&2
        exit 1
    fi
fi

if [[ -s "$realistic_cases_jsonl" ]]; then
    realistic_cases_json="$(jq -s '.' "$realistic_cases_jsonl")"
fi

jq -n \
    --arg grammar_name "$grammar_name" \
    --arg requested_mode "$REALISTIC_CORPUS_MODE" \
    --arg effective_mode "$realistic_corpus_effective" \
    --arg note "$realistic_corpus_note" \
    --arg corpus_path "${realistic_corpus_path:-}" \
    --argjson max_cases "$REALISTIC_CORPUS_MAX_CASES" \
    --argjson enabled "$realistic_corpus_enabled" \
    --argjson contract_enforced "$realistic_corpus_contract_enforced" \
    --argjson cases_declared "$realistic_cases_declared" \
    --argjson cases_executed "$realistic_cases_executed" \
    --argjson expected_pass_total "$realistic_expected_pass_total" \
    --argjson expected_fail_total "$realistic_expected_fail_total" \
    --argjson observed_parse_pass_total "$realistic_parse_pass_total" \
    --argjson observed_parse_fail_total "$realistic_parse_fail_total" \
    --argjson expected_fail_parse_pass_total "$realistic_expected_fail_parse_pass_total" \
    --argjson parse_total_ms "$realistic_parse_total_ms" \
    --argjson parse_max_ms "$realistic_parse_max_ms" \
    --argjson sample_bytes_max "$realistic_sample_bytes_max" \
    --argjson max_parse_full_ms_per_case "$realistic_max_parse_full_ms_per_case" \
    --argjson max_sample_bytes "$realistic_max_sample_bytes" \
    --argjson cases "$realistic_cases_json" \
    '{
        grammar_name: $grammar_name,
        requested_mode: $requested_mode,
        effective_mode: $effective_mode,
        note: $note,
        corpus_path: $corpus_path,
        max_cases: $max_cases,
        enabled: $enabled,
        contract_enforced: $contract_enforced,
        thresholds: {
            max_parse_full_ms_per_case: $max_parse_full_ms_per_case,
            max_sample_bytes: $max_sample_bytes
        },
        totals: {
            cases_declared: $cases_declared,
            cases_executed: $cases_executed,
            expected_pass_total: $expected_pass_total,
            expected_fail_total: $expected_fail_total,
            observed_parse_pass_total: $observed_parse_pass_total,
            observed_parse_fail_total: $observed_parse_fail_total,
            expected_fail_parse_pass_total: $expected_fail_parse_pass_total,
            parse_total_ms: $parse_total_ms,
            parse_max_ms: $parse_max_ms,
            sample_bytes_max: $sample_bytes_max
        },
        cases: $cases
    }' >"$realistic_report_json"

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
    echo "parseability_generation_enabled: $parseability_generation_enabled"
    echo "parseability_generation_effective: $parseability_generation_effective"
    echo "parseability_generation_max_attempts_per_sample: $parseability_max_attempts_per_sample"
    echo "parseability_generation_requested_total: $parseability_generation_requested_total"
    echo "parseability_generation_attempts_total: $parseability_generation_attempts_total"
    echo "parseability_generation_accepted_total: $parseability_generation_accepted_total"
    echo "parseability_generation_rejected_total: $parseability_generation_rejected_total"
    echo "parseability_generation_parser_rejections_total: $parseability_generation_parser_rejections_total"
    echo "parseability_generation_errors_total: $parseability_generation_errors_total"
    echo "parseability_generation_empty_generations_total: $parseability_generation_empty_generations_total"
    echo "parseability_generation_acceptance_rate_percent: $parseability_generation_acceptance_rate_percent"
    echo "parseability_generation_report_json: $parseability_generation_report_json"
    echo "parse_full_passes: $parse_full_pass_count/$sample_count"
    echo "parse_full_failures: $parse_full_fail_count"
    echo "parse_full_skips: $parse_full_skip_count"
    echo "realistic_corpus_effective: $realistic_corpus_effective"
    echo "realistic_corpus_note: $realistic_corpus_note"
    echo "realistic_corpus_path: ${realistic_corpus_path:-}"
    echo "realistic_corpus_cases_declared: $realistic_cases_declared"
    echo "realistic_corpus_cases_executed: $realistic_cases_executed"
    echo "realistic_corpus_expected_pass_total: $realistic_expected_pass_total"
    echo "realistic_corpus_expected_fail_total: $realistic_expected_fail_total"
    echo "realistic_corpus_observed_parse_pass_total: $realistic_parse_pass_total"
    echo "realistic_corpus_observed_parse_fail_total: $realistic_parse_fail_total"
    echo "realistic_corpus_expected_fail_parse_pass_total: $realistic_expected_fail_parse_pass_total"
    echo "realistic_corpus_parse_total_ms: $realistic_parse_total_ms"
    echo "realistic_corpus_parse_max_ms: $realistic_parse_max_ms"
    echo "realistic_corpus_sample_bytes_max: $realistic_sample_bytes_max"
    echo "realistic_corpus_report_json: $realistic_report_json"
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
