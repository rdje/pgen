#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"
GRAMMARS_DIR="$ROOT_DIR/grammars"

STATE_DIR="${PGEN_SV_PREPROCESSOR_QUALITY_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_quality_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

SAMPLE_COUNT="${PGEN_SV_PREPROCESSOR_QUALITY_COUNT:-16}"
GAP_THRESHOLD="${PGEN_SV_PREPROCESSOR_QUALITY_GAP_THRESHOLD:-1}"
TARGET_MAX_ATTEMPTS="${PGEN_SV_PREPROCESSOR_QUALITY_TARGET_MAX_ATTEMPTS:-6000}"
SEED_BASE="${PGEN_SV_PREPROCESSOR_QUALITY_SEED_BASE:-9101}"
FUZZ_ROUNDS="${PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_ROUNDS:-8}"
FUZZ_SEED_START="${PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_SEED_START:-9201}"
PARSEABILITY_MODE="${PGEN_SV_PREPROCESSOR_QUALITY_VALIDATE_PARSEABILITY:-auto}"
DIFF_MODE="${PGEN_SV_PREPROCESSOR_DIFF_MODE:-auto}"
DIFF_MAX_SAMPLES="${PGEN_SV_PREPROCESSOR_DIFF_MAX_SAMPLES:-4}"
DIFF_REFERENCE_RUNNER="${PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER:-}"

GRAMMAR_NAME="systemverilog_preprocessor"
GRAMMAR_FILE="$GRAMMARS_DIR/${GRAMMAR_NAME}.ebnf"
GRAMMAR_JSON="$WORK_DIR/${GRAMMAR_NAME}.json"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSER_OUT="$WORK_DIR/${GRAMMAR_NAME}_parser.rs"

if ! [[ "$SAMPLE_COUNT" =~ ^[0-9]+$ ]] || [[ "$SAMPLE_COUNT" -lt 1 ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_QUALITY_COUNT must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$GAP_THRESHOLD" =~ ^[0-9]+$ ]] || [[ "$GAP_THRESHOLD" -lt 1 ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_QUALITY_GAP_THRESHOLD must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$TARGET_MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [[ "$TARGET_MAX_ATTEMPTS" -lt 1 ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_QUALITY_TARGET_MAX_ATTEMPTS must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SEED_BASE" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_QUALITY_SEED_BASE must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$FUZZ_ROUNDS" =~ ^[0-9]+$ ]] || [[ "$FUZZ_ROUNDS" -lt 1 ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_ROUNDS must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$FUZZ_SEED_START" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_QUALITY_FUZZ_SEED_START must be an integer >= 0" >&2
    exit 2
fi
if [[ "$PARSEABILITY_MODE" != "auto" && "$PARSEABILITY_MODE" != "0" && "$PARSEABILITY_MODE" != "1" ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_QUALITY_VALIDATE_PARSEABILITY must be one of: auto, 0, 1" >&2
    exit 2
fi
if [[ "$DIFF_MODE" != "auto" && "$DIFF_MODE" != "0" && "$DIFF_MODE" != "1" ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_DIFF_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$DIFF_MAX_SAMPLES" =~ ^[0-9]+$ ]] || [[ "$DIFF_MAX_SAMPLES" -lt 1 ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_DIFF_MAX_SAMPLES must be an integer >= 1" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

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

extract_json_u64() {
    local path="$1"
    local expr="$2"
    jq -er "$expr | numbers" "$path"
}

extract_rule_hit() {
    local path="$1"
    local rule_name="$2"
    jq -er --arg rule "$rule_name" '.rule_success_hits[$rule] // 0 | numbers' "$path"
}

parseability_summary_field_u64() {
    local path="$1"
    local field="$2"
    jq -er ".summary.${field} | numbers" "$path"
}

parseability_target_drive_field_u64() {
    local path="$1"
    local field="$2"
    jq -er "(.target_drive_validation.${field} // 0) | numbers" "$path"
}

parseability_acceptance_rate_percent() {
    local path="$1"
    local attempts accepted
    attempts="$(parseability_summary_field_u64 "$path" "attempts")"
    accepted="$(parseability_summary_field_u64 "$path" "accepted")"
    perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$accepted" "$attempts"
}

assert_json() {
    local path="$1"
    local expr="$2"
    local message="$3"
    if ! jq -e "$expr" "$path" >/dev/null; then
        echo "error: $message (file: $path, expr: $expr)" >&2
        exit 1
    fi
}

canonicalize_json() {
    local source="$1"
    local target="$2"
    jq -S . "$source" >"$target"
}

assert_same_text() {
    local left="$1"
    local right="$2"
    local context="$3"
    if ! cmp -s "$left" "$right"; then
        echo "error: deterministic replay mismatch for $context" >&2
        diff -u "$left" "$right" | head -n 80 >&2 || true
        exit 1
    fi
}

assert_same_json() {
    local left="$1"
    local right="$2"
    local context="$3"
    local left_norm="${left}.norm.json"
    local right_norm="${right}.norm.json"
    canonicalize_json "$left" "$left_norm"
    canonicalize_json "$right" "$right_norm"
    assert_same_text "$left_norm" "$right_norm" "$context"
}

normalize_text_for_diff_output() {
    local source="$1"
    local target="$2"
    tr -s '[:space:]' ' ' <"$source" | sed 's/^ *//; s/ *$//' >"$target"
}

parse_target_summary() {
    local log_path="$1"
    local line
    line="$(grep -E "Target-driven generation: resolved [0-9]+/[0-9]+ targets in [0-9]+ attempts" "$log_path" | tail -n 1 || true)"
    if [[ -z "$line" ]]; then
        echo "error: unable to locate target-driven summary in '$log_path'" >&2
        exit 1
    fi
    if [[ "$line" =~ resolved[[:space:]]+([0-9]+)/([0-9]+)[[:space:]]+targets[[:space:]]+in[[:space:]]+([0-9]+)[[:space:]]+attempts ]]; then
        echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]} ${BASH_REMATCH[3]}"
        return
    fi
    echo "error: failed to parse target-driven summary line '$line'" >&2
    exit 1
}

require_tool jq
require_tool perl
require_file "$GRAMMAR_FILE"

echo "==> SystemVerilog preprocessor quality gate"
echo "state_dir: $STATE_DIR"
echo "sample_count: $SAMPLE_COUNT"
echo "gap_threshold: $GAP_THRESHOLD"
echo "target_max_attempts: $TARGET_MAX_ATTEMPTS"
echo "seed_base: $SEED_BASE"
echo "fuzz_rounds: $FUZZ_ROUNDS"
echo "fuzz_seed_start: $FUZZ_SEED_START"
echo "parseability_mode: $PARSEABILITY_MODE"
echo "diff_mode: $DIFF_MODE"
echo "diff_max_samples: $DIFF_MAX_SAMPLES"

run_logged "frontend_ebnf_to_json" \
    perl "$EBNF_TO_JSON" --pretty --quiet "$GRAMMAR_FILE" -o "$GRAMMAR_JSON"

require_nonempty_file "$GRAMMAR_JSON"

run_logged_rust "build_ast_pipeline_for_preprocessor_generation" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

run_logged "generate_sv_preprocessor_parser" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-parser \
    --eliminate-left-recursion \
    --output "$PARSER_OUT"
require_nonempty_file "$PARSER_OUT"

run_logged_rust "build_ast_pipeline_with_sv_preprocessor_adapter" \
    env PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH="$PARSER_OUT" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after adapter build" >&2
    exit 1
fi

parseability_enabled=0
parseability_effective="disabled"
parseability_note="parseability validation disabled by configuration"
parseability_attempts_total=0
parseability_accepted_total=0
parseability_rejected_total=0
parseability_parser_rejections_total=0
parseability_generation_errors_total=0
parseability_empty_generations_total=0
parseability_acceptance_rate_percent_total="0.00"
target_drive_alternate_entry_attempts_total=0
target_drive_alternate_entry_accepted_outputs_total=0
target_drive_alternate_entry_rejected_outputs_total=0
parseability_report_json="n/a"
declare -a parseability_args=()
declare -a parseability_args_stage0a=()
declare -a parseability_args_stage0b=()
declare -a parseability_args_stage1=()
declare -a parseability_args_stage2=()
declare -a parseability_args_stage3=()

probe_samples="$WORK_DIR/parseability_probe_samples.txt"
probe_log="$LOG_DIR/parseability_probe.log"
stage0a_parseability_json="$WORK_DIR/${GRAMMAR_NAME}_parseability_stage0_a.json"
stage0b_parseability_json="$WORK_DIR/${GRAMMAR_NAME}_parseability_stage0_b.json"
stage1_parseability_json="$WORK_DIR/${GRAMMAR_NAME}_parseability_stage1.json"
stage2_parseability_json="$WORK_DIR/${GRAMMAR_NAME}_parseability_stage2.json"
stage3_parseability_json="$WORK_DIR/${GRAMMAR_NAME}_parseability_stage3.json"

if [[ "$PARSEABILITY_MODE" != "0" ]]; then
    if "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
        --generate-stimuli \
        --count 1 \
        --seed "$SEED_BASE" \
        --validate-parseability \
        --output "$probe_samples" >"$probe_log" 2>&1; then
        parseability_enabled=1
        parseability_effective="enabled"
        parseability_note="parseability validation enabled"
    else
        if grep -Eq "No matching generated parser validation path exists for grammar|No matching compiled generated parser is available for grammar" "$probe_log"; then
            if [[ "$PARSEABILITY_MODE" == "1" ]]; then
                echo "error: parseability mode is strict (1) but parser-registry path is unavailable for '$GRAMMAR_NAME'" >&2
                tail -n 40 "$probe_log" >&2 || true
                exit 1
            fi
            parseability_enabled=0
            parseability_effective="unsupported_adapter"
            parseability_note="parser-registry parseability adapter is not yet available for '$GRAMMAR_NAME'"
        else
            echo "error: parseability probe failed unexpectedly" >&2
            tail -n 80 "$probe_log" >&2 || true
            exit 1
        fi
    fi
fi

if [[ "$parseability_enabled" -eq 1 ]]; then
    parseability_args+=(--validate-parseability)
    parseability_args_stage0a=(--validate-parseability --parseability-report-json "$stage0a_parseability_json")
    parseability_args_stage0b=(--validate-parseability --parseability-report-json "$stage0b_parseability_json")
    parseability_args_stage1=(--validate-parseability --parseability-report-json "$stage1_parseability_json")
    parseability_args_stage2=(--validate-parseability --parseability-report-json "$stage2_parseability_json")
    parseability_args_stage3=(--validate-parseability --parseability-report-json "$stage3_parseability_json")
    parseability_report_json="$WORK_DIR/${GRAMMAR_NAME}_parseability_report.json"
fi

samples0a="$WORK_DIR/${GRAMMAR_NAME}_samples_stage0_a.txt"
samples0b="$WORK_DIR/${GRAMMAR_NAME}_samples_stage0_b.txt"
samples1="$WORK_DIR/${GRAMMAR_NAME}_samples_stage1.txt"
samples2="$WORK_DIR/${GRAMMAR_NAME}_samples_stage2.txt"
samples3="$WORK_DIR/${GRAMMAR_NAME}_samples_stage3.txt"
samples4a="$WORK_DIR/${GRAMMAR_NAME}_samples_stage4_fuzz_a.txt"
samples4b="$WORK_DIR/${GRAMMAR_NAME}_samples_stage4_fuzz_b.txt"

coverage0a="$WORK_DIR/${GRAMMAR_NAME}_coverage_stage0_a.json"
coverage0b="$WORK_DIR/${GRAMMAR_NAME}_coverage_stage0_b.json"
coverage1="$WORK_DIR/${GRAMMAR_NAME}_coverage_stage1.json"
coverage2="$WORK_DIR/${GRAMMAR_NAME}_coverage_stage2.json"
coverage3="$WORK_DIR/${GRAMMAR_NAME}_coverage_stage3.json"
coverage4a="$WORK_DIR/${GRAMMAR_NAME}_coverage_stage4_fuzz_a.json"
coverage4b="$WORK_DIR/${GRAMMAR_NAME}_coverage_stage4_fuzz_b.json"

gap0a="$WORK_DIR/${GRAMMAR_NAME}_gap_stage0_a.json"
gap0b="$WORK_DIR/${GRAMMAR_NAME}_gap_stage0_b.json"
gap1="$WORK_DIR/${GRAMMAR_NAME}_gap_stage1.json"
gap3="$WORK_DIR/${GRAMMAR_NAME}_gap_stage3.json"
gap4a="$WORK_DIR/${GRAMMAR_NAME}_gap_stage4_fuzz_a.json"
gap4b="$WORK_DIR/${GRAMMAR_NAME}_gap_stage4_fuzz_b.json"

fuzz_replay_a="$WORK_DIR/${GRAMMAR_NAME}_fuzz_replay_a.json"
fuzz_replay_b="$WORK_DIR/${GRAMMAR_NAME}_fuzz_replay_b.json"

run_logged "stage0_baseline_a" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-stimuli \
    --count "$SAMPLE_COUNT" \
    --seed "$SEED_BASE" \
    "${parseability_args_stage0a[@]}" \
    --gap-report-threshold "$GAP_THRESHOLD" \
    --output "$samples0a" \
    --coverage-output "$coverage0a" \
    --gap-report-json "$gap0a"

run_logged "stage0_baseline_b_replay" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-stimuli \
    --count "$SAMPLE_COUNT" \
    --seed "$SEED_BASE" \
    "${parseability_args_stage0b[@]}" \
    --gap-report-threshold "$GAP_THRESHOLD" \
    --output "$samples0b" \
    --coverage-output "$coverage0b" \
    --gap-report-json "$gap0b"

require_nonempty_file "$samples0a"
require_nonempty_file "$samples0b"
require_nonempty_file "$coverage0a"
require_nonempty_file "$coverage0b"
require_nonempty_file "$gap0a"
require_nonempty_file "$gap0b"
if [[ "$parseability_enabled" -eq 1 ]]; then
    require_nonempty_file "$stage0a_parseability_json"
    require_nonempty_file "$stage0b_parseability_json"
fi

assert_same_text "$samples0a" "$samples0b" "stage0 sample corpus"
assert_same_json "$coverage0a" "$coverage0b" "stage0 coverage metrics"
assert_same_json "$gap0a" "$gap0b" "stage0 gap report"
if [[ "$parseability_enabled" -eq 1 ]]; then
    assert_same_json "$stage0a_parseability_json" "$stage0b_parseability_json" "stage0 parseability report"
fi

assert_json "$coverage0a" ".grammar_name == \"$GRAMMAR_NAME\"" "coverage stage0 grammar_name mismatch"
assert_json "$coverage0a" ".total_rules > 0" "coverage stage0 must report total_rules > 0"
assert_json "$coverage0a" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage0 attempts consistency failed"
assert_json "$coverage0a" ".sample_successes >= $SAMPLE_COUNT" "coverage stage0 sample_successes below requested count"
if [[ "$parseability_enabled" -eq 1 ]]; then
    assert_json "$stage0a_parseability_json" ".grammar_name == \"$GRAMMAR_NAME\" and .summary.requested == $SAMPLE_COUNT and .summary.accepted == $SAMPLE_COUNT and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "stage0 parseability report contract mismatch"
fi

assert_json "$gap0a" ".grammar_name == \"$GRAMMAR_NAME\"" "gap stage0 grammar_name mismatch"
assert_json "$gap0a" ".summary.required_successes_per_target == $GAP_THRESHOLD" "gap stage0 threshold mismatch"
assert_json "$gap0a" "all(.targets[]?; .reachable == true)" "gap stage0 contains non-reachable target"
assert_json "$gap0a" ".summary.total_rules >= (.summary.reachable_rules + .summary.unreachable_rules)" "gap stage0 rule summary invariants failed"
assert_json "$gap0a" ".summary.total_branches >= (.summary.reachable_branches + .summary.unreachable_branches)" "gap stage0 branch summary invariants failed"

attempts0="$(extract_json_u64 "$coverage0a" ".sample_attempts")"
successes0="$(extract_json_u64 "$coverage0a" ".sample_successes")"
covered_rules0="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage0a")"
covered_branches0="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage0a")"
initial_targets="$(jq -er '.targets | length | numbers' "$gap0a")"
if [[ "$parseability_enabled" -eq 1 ]]; then
    stage0_parseability_attempts="$(parseability_summary_field_u64 "$stage0a_parseability_json" "attempts")"
    stage0_parseability_accepted="$(parseability_summary_field_u64 "$stage0a_parseability_json" "accepted")"
    stage0_parseability_rejected="$(parseability_summary_field_u64 "$stage0a_parseability_json" "rejected")"
    stage0_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage0a_parseability_json" "parser_rejections")"
    stage0_parseability_generation_errors="$(parseability_summary_field_u64 "$stage0a_parseability_json" "generation_errors")"
    stage0_parseability_empty_generations="$(parseability_summary_field_u64 "$stage0a_parseability_json" "empty_generations")"
else
    stage0_parseability_attempts=0
    stage0_parseability_accepted=0
    stage0_parseability_rejected=0
    stage0_parseability_parser_rejections=0
    stage0_parseability_generation_errors=0
    stage0_parseability_empty_generations=0
fi

run_logged "stage1_gap_priority" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-stimuli \
    --count "$SAMPLE_COUNT" \
    --seed "$((SEED_BASE + 1))" \
    "${parseability_args_stage1[@]}" \
    --coverage-input "$coverage0a" \
    --gap-priority-report-input "$gap0a" \
    --output "$samples1" \
    --coverage-output "$coverage1" \
    --gap-report-json "$gap1" \
    --gap-report-threshold "$GAP_THRESHOLD"

require_nonempty_file "$samples1"
require_nonempty_file "$coverage1"
require_nonempty_file "$gap1"
if [[ "$parseability_enabled" -eq 1 ]]; then
    require_nonempty_file "$stage1_parseability_json"
fi

assert_json "$coverage1" ".grammar_name == \"$GRAMMAR_NAME\"" "coverage stage1 grammar_name mismatch"
assert_json "$coverage1" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage1 attempts consistency failed"
assert_json "$gap1" ".grammar_name == \"$GRAMMAR_NAME\"" "gap stage1 grammar_name mismatch"
if [[ "$parseability_enabled" -eq 1 ]]; then
    assert_json "$stage1_parseability_json" ".grammar_name == \"$GRAMMAR_NAME\" and .summary.requested == $SAMPLE_COUNT and .summary.accepted == $SAMPLE_COUNT and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "stage1 parseability report contract mismatch"
fi

attempts1="$(extract_json_u64 "$coverage1" ".sample_attempts")"
successes1="$(extract_json_u64 "$coverage1" ".sample_successes")"
covered_rules1="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage1")"
covered_branches1="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage1")"
if [[ "$parseability_enabled" -eq 1 ]]; then
    stage1_parseability_attempts="$(parseability_summary_field_u64 "$stage1_parseability_json" "attempts")"
    stage1_parseability_accepted="$(parseability_summary_field_u64 "$stage1_parseability_json" "accepted")"
    stage1_parseability_rejected="$(parseability_summary_field_u64 "$stage1_parseability_json" "rejected")"
    stage1_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage1_parseability_json" "parser_rejections")"
    stage1_parseability_generation_errors="$(parseability_summary_field_u64 "$stage1_parseability_json" "generation_errors")"
    stage1_parseability_empty_generations="$(parseability_summary_field_u64 "$stage1_parseability_json" "empty_generations")"
else
    stage1_parseability_attempts=0
    stage1_parseability_accepted=0
    stage1_parseability_rejected=0
    stage1_parseability_parser_rejections=0
    stage1_parseability_generation_errors=0
    stage1_parseability_empty_generations=0
fi

if (( attempts1 <= attempts0 )); then
    echo "error: stage1 sample_attempts did not increase ($attempts0 -> $attempts1)" >&2
    exit 1
fi
if (( successes1 <= successes0 )); then
    echo "error: stage1 sample_successes did not increase ($successes0 -> $successes1)" >&2
    exit 1
fi
if (( covered_rules1 < covered_rules0 )); then
    echo "error: stage1 covered_rules regressed ($covered_rules0 -> $covered_rules1)" >&2
    exit 1
fi
if (( covered_branches1 < covered_branches0 )); then
    echo "error: stage1 covered_branches regressed ($covered_branches0 -> $covered_branches1)" >&2
    exit 1
fi

run_logged "stage2_target_drive" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-stimuli \
    --seed "$((SEED_BASE + 2))" \
    "${parseability_args_stage2[@]}" \
    --coverage-input "$coverage1" \
    --target-report-input "$gap0a" \
    --target-max-attempts "$TARGET_MAX_ATTEMPTS" \
    --output "$samples2" \
    --coverage-output "$coverage2"

require_file "$samples2"
require_nonempty_file "$coverage2"
if [[ "$parseability_enabled" -eq 1 ]]; then
    require_nonempty_file "$stage2_parseability_json"
fi

assert_json "$coverage2" ".grammar_name == \"$GRAMMAR_NAME\"" "coverage stage2 grammar_name mismatch"
assert_json "$coverage2" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage2 attempts consistency failed"
if [[ "$parseability_enabled" -eq 1 ]]; then
    assert_json "$stage2_parseability_json" ".grammar_name == \"$GRAMMAR_NAME\" and .summary.attempts == .summary.requested and .summary.accepted <= .summary.requested and .summary.rejected == (.summary.attempts - .summary.accepted)" "stage2 parseability report contract mismatch"
fi

attempts2="$(extract_json_u64 "$coverage2" ".sample_attempts")"
successes2="$(extract_json_u64 "$coverage2" ".sample_successes")"
covered_rules2="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage2")"
covered_branches2="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage2")"
if [[ "$parseability_enabled" -eq 1 ]]; then
    stage2_parseability_attempts="$(parseability_summary_field_u64 "$stage2_parseability_json" "attempts")"
    stage2_parseability_accepted="$(parseability_summary_field_u64 "$stage2_parseability_json" "accepted")"
    stage2_parseability_rejected="$(parseability_summary_field_u64 "$stage2_parseability_json" "rejected")"
    stage2_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage2_parseability_json" "parser_rejections")"
    stage2_parseability_generation_errors="$(parseability_summary_field_u64 "$stage2_parseability_json" "generation_errors")"
    stage2_parseability_empty_generations="$(parseability_summary_field_u64 "$stage2_parseability_json" "empty_generations")"
    target_drive_alternate_entry_attempts_total="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "alternate_entry_attempts")"
    target_drive_alternate_entry_accepted_outputs_total="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "alternate_entry_accepted_outputs")"
    target_drive_alternate_entry_rejected_outputs_total="$(parseability_target_drive_field_u64 "$stage2_parseability_json" "alternate_entry_rejected_outputs")"
else
    stage2_parseability_attempts=0
    stage2_parseability_accepted=0
    stage2_parseability_rejected=0
    stage2_parseability_parser_rejections=0
    stage2_parseability_generation_errors=0
    stage2_parseability_empty_generations=0
fi

if (( attempts2 < attempts1 )); then
    echo "error: stage2 sample_attempts regressed ($attempts1 -> $attempts2)" >&2
    exit 1
fi
if (( successes2 < successes1 )); then
    echo "error: stage2 sample_successes regressed ($successes1 -> $successes2)" >&2
    exit 1
fi
if (( covered_rules2 < covered_rules1 )); then
    echo "error: stage2 covered_rules regressed ($covered_rules1 -> $covered_rules2)" >&2
    exit 1
fi
if (( covered_branches2 < covered_branches1 )); then
    echo "error: stage2 covered_branches regressed ($covered_branches1 -> $covered_branches2)" >&2
    exit 1
fi

stage2_log="$LOG_DIR/stage2_target_drive.log"
read -r resolved_targets total_targets target_attempts < <(parse_target_summary "$stage2_log")

if (( total_targets != initial_targets )); then
    echo "error: stage2 target summary total ($total_targets) does not match stage0 targets ($initial_targets)" >&2
    exit 1
fi
if (( resolved_targets > total_targets )); then
    echo "error: stage2 resolved targets exceeds total ($resolved_targets > $total_targets)" >&2
    exit 1
fi

run_logged "stage3_recompute_gap" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-stimuli \
    --count 1 \
    --seed "$((SEED_BASE + 3))" \
    "${parseability_args_stage3[@]}" \
    --coverage-input "$coverage2" \
    --output "$samples3" \
    --coverage-output "$coverage3" \
    --gap-report-json "$gap3" \
    --gap-report-threshold "$GAP_THRESHOLD"

require_nonempty_file "$samples3"
require_nonempty_file "$coverage3"
require_nonempty_file "$gap3"
if [[ "$parseability_enabled" -eq 1 ]]; then
    require_nonempty_file "$stage3_parseability_json"
fi

assert_json "$coverage3" ".grammar_name == \"$GRAMMAR_NAME\"" "coverage stage3 grammar_name mismatch"
assert_json "$gap3" ".grammar_name == \"$GRAMMAR_NAME\"" "gap stage3 grammar_name mismatch"
if [[ "$parseability_enabled" -eq 1 ]]; then
    assert_json "$stage3_parseability_json" ".grammar_name == \"$GRAMMAR_NAME\" and .summary.requested == 1 and .summary.accepted == 1 and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "stage3 parseability report contract mismatch"
fi

final_targets="$(jq -er '.targets | length | numbers' "$gap3")"
if [[ "$parseability_enabled" -eq 1 ]]; then
    stage3_parseability_attempts="$(parseability_summary_field_u64 "$stage3_parseability_json" "attempts")"
    stage3_parseability_accepted="$(parseability_summary_field_u64 "$stage3_parseability_json" "accepted")"
    stage3_parseability_rejected="$(parseability_summary_field_u64 "$stage3_parseability_json" "rejected")"
    stage3_parseability_parser_rejections="$(parseability_summary_field_u64 "$stage3_parseability_json" "parser_rejections")"
    stage3_parseability_generation_errors="$(parseability_summary_field_u64 "$stage3_parseability_json" "generation_errors")"
    stage3_parseability_empty_generations="$(parseability_summary_field_u64 "$stage3_parseability_json" "empty_generations")"
else
    stage3_parseability_attempts=0
    stage3_parseability_accepted=0
    stage3_parseability_rejected=0
    stage3_parseability_parser_rejections=0
    stage3_parseability_generation_errors=0
    stage3_parseability_empty_generations=0
fi
if (( final_targets > initial_targets )); then
    echo "error: final actionable targets regressed ($initial_targets -> $final_targets)" >&2
    exit 1
fi

if [[ "$parseability_enabled" -eq 1 ]]; then
    parseability_attempts_total=$((stage0_parseability_attempts + stage1_parseability_attempts + stage2_parseability_attempts + stage3_parseability_attempts))
    parseability_accepted_total=$((stage0_parseability_accepted + stage1_parseability_accepted + stage2_parseability_accepted + stage3_parseability_accepted))
    parseability_rejected_total=$((stage0_parseability_rejected + stage1_parseability_rejected + stage2_parseability_rejected + stage3_parseability_rejected))
    parseability_parser_rejections_total=$((stage0_parseability_parser_rejections + stage1_parseability_parser_rejections + stage2_parseability_parser_rejections + stage3_parseability_parser_rejections))
    parseability_generation_errors_total=$((stage0_parseability_generation_errors + stage1_parseability_generation_errors + stage2_parseability_generation_errors + stage3_parseability_generation_errors))
    parseability_empty_generations_total=$((stage0_parseability_empty_generations + stage1_parseability_empty_generations + stage2_parseability_empty_generations + stage3_parseability_empty_generations))
    parseability_acceptance_rate_percent_total="$(perl -e 'my ($accepted, $attempts) = @ARGV; if ($attempts == 0) { printf "0.00" } else { printf "%.2f", ($accepted * 100.0) / $attempts }' "$parseability_accepted_total" "$parseability_attempts_total")"

    jq -n \
        --arg grammar_name "$GRAMMAR_NAME" \
        --arg requested_mode "$PARSEABILITY_MODE" \
        --arg effective_mode "$parseability_effective" \
        --arg note "$parseability_note" \
        --argjson attempts_total "$parseability_attempts_total" \
        --argjson accepted_total "$parseability_accepted_total" \
        --argjson rejected_total "$parseability_rejected_total" \
        --argjson parser_rejections_total "$parseability_parser_rejections_total" \
        --argjson generation_errors_total "$parseability_generation_errors_total" \
        --argjson empty_generations_total "$parseability_empty_generations_total" \
        --argjson acceptance_rate_percent "$parseability_acceptance_rate_percent_total" \
        --argjson alternate_entry_attempts_total "$target_drive_alternate_entry_attempts_total" \
        --argjson alternate_entry_accepted_outputs_total "$target_drive_alternate_entry_accepted_outputs_total" \
        --argjson alternate_entry_rejected_outputs_total "$target_drive_alternate_entry_rejected_outputs_total" \
        --slurpfile stage0 "$stage0a_parseability_json" \
        --slurpfile stage1 "$stage1_parseability_json" \
        --slurpfile stage2 "$stage2_parseability_json" \
        --slurpfile stage3 "$stage3_parseability_json" \
        '{
            grammar_name: $grammar_name,
            requested_mode: $requested_mode,
            effective_mode: $effective_mode,
            note: $note,
            summary: {
                attempts: $attempts_total,
                accepted: $accepted_total,
                rejected: $rejected_total,
                parser_rejections: $parser_rejections_total,
                generation_errors: $generation_errors_total,
                empty_generations: $empty_generations_total,
                acceptance_rate_percent: $acceptance_rate_percent
            },
            target_drive_validation: {
                alternate_entry_attempts_total: $alternate_entry_attempts_total,
                alternate_entry_accepted_outputs_total: $alternate_entry_accepted_outputs_total,
                alternate_entry_rejected_outputs_total: $alternate_entry_rejected_outputs_total
            },
            stages: {
                stage0_baseline: $stage0[0],
                stage1_gap_priority: $stage1[0],
                stage2_target_drive: $stage2[0],
                stage3_recompute_gap: $stage3[0]
            }
        }' >"$parseability_report_json"
    require_nonempty_file "$parseability_report_json"
fi

for key_rule in \
    pp_include \
    pp_define \
    pp_conditional \
    pp_if_branch \
    pp_elsif_branch \
    pp_else_branch \
    macro_formals \
    macro_body_fragment \
    macro_token_paste \
    macro_stringize; do
    hit_count="$(extract_rule_hit "$coverage3" "$key_rule")"
    if (( hit_count <= 0 )); then
        echo "error: expected non-zero final hit count for key preprocessor rule '$key_rule'" >&2
        exit 1
    fi
done

assert_json "$coverage3" '.branch_groups["include_path::root"].success_counts | length >= 2 and all(.[]; . > 0)' \
    "include_path branch family not fully exercised in final coverage"
assert_json "$coverage3" '.branch_groups["pp_if_branch::root/s0"].success_counts | length >= 2 and all(.[]; . > 0)' \
    "pp_if_branch conditional family not fully exercised in final coverage"

run_logged "stage4_fuzz_replay_a" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-stimuli \
    --count 1 \
    --seed "$FUZZ_SEED_START" \
    "${parseability_args[@]}" \
    --coverage-input "$coverage3" \
    --coverage-guided-fuzz-rounds "$FUZZ_ROUNDS" \
    --coverage-guided-fuzz-seed-start "$FUZZ_SEED_START" \
    --coverage-guided-fuzz-replay-output "$fuzz_replay_a" \
    --output "$samples4a" \
    --coverage-output "$coverage4a" \
    --gap-report-json "$gap4a" \
    --gap-report-threshold "$GAP_THRESHOLD"

run_logged "stage4_fuzz_replay_b_replay" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-stimuli \
    --count 1 \
    --seed "$FUZZ_SEED_START" \
    "${parseability_args[@]}" \
    --coverage-input "$coverage3" \
    --coverage-guided-fuzz-rounds "$FUZZ_ROUNDS" \
    --coverage-guided-fuzz-seed-start "$FUZZ_SEED_START" \
    --coverage-guided-fuzz-replay-output "$fuzz_replay_b" \
    --output "$samples4b" \
    --coverage-output "$coverage4b" \
    --gap-report-json "$gap4b" \
    --gap-report-threshold "$GAP_THRESHOLD"

require_nonempty_file "$samples4a"
require_nonempty_file "$samples4b"
require_nonempty_file "$coverage4a"
require_nonempty_file "$coverage4b"
require_nonempty_file "$gap4a"
require_nonempty_file "$gap4b"
require_nonempty_file "$fuzz_replay_a"
require_nonempty_file "$fuzz_replay_b"

assert_same_text "$samples4a" "$samples4b" "fuzz replay sample corpus"
assert_same_json "$coverage4a" "$coverage4b" "fuzz replay coverage metrics"
assert_same_json "$gap4a" "$gap4b" "fuzz replay gap report"
assert_same_json "$fuzz_replay_a" "$fuzz_replay_b" "fuzz replay metadata"

assert_json "$fuzz_replay_a" ".rounds == $FUZZ_ROUNDS" "fuzz replay rounds mismatch"
assert_json "$fuzz_replay_a" ".cases | length == $FUZZ_ROUNDS" "fuzz replay case count mismatch"
assert_json "$fuzz_replay_a" ".accepted_cases + .rejected_cases == .rounds" "fuzz replay accepted/rejected invariant failed"
assert_json "$fuzz_replay_a" ".minimized_cases <= .rounds" "fuzz replay minimized_cases invariant failed"
assert_json "$fuzz_replay_a" ".shrunk_counterexamples <= .parseability_counterexamples" "fuzz replay shrink invariant failed"

fuzz_accepted="$(extract_json_u64 "$fuzz_replay_a" ".accepted_cases")"
fuzz_rejected="$(extract_json_u64 "$fuzz_replay_a" ".rejected_cases")"
fuzz_minimized="$(extract_json_u64 "$fuzz_replay_a" ".minimized_cases")"
fuzz_parseability_counterexamples="$(extract_json_u64 "$fuzz_replay_a" ".parseability_counterexamples")"
fuzz_shrunk_counterexamples="$(extract_json_u64 "$fuzz_replay_a" ".shrunk_counterexamples")"

diff_report_json="$WORK_DIR/${GRAMMAR_NAME}_differential_report.json"
diff_cases_jsonl="$WORK_DIR/${GRAMMAR_NAME}_differential_cases.jsonl"
diff_effective_mode="disabled"
diff_note="trusted-reference differential disabled by configuration"
diff_reference_runner="$DIFF_REFERENCE_RUNNER"
diff_total_samples_seen=0
diff_samples_checked=0
diff_mismatch_count=0
diff_match_count=0
diff_diagnostics_mismatch_count=0
diff_whitespace_only_output_mismatch_count=0
diff_output_mismatch_count=0
diff_rust_failed_reference_passed_count=0
diff_reference_failed_rust_passed_count=0
diff_both_failed_count=0
diff_reference_artifact_missing_count=0

if [[ "$DIFF_MODE" != "0" ]]; then
    if [[ -z "$DIFF_REFERENCE_RUNNER" ]]; then
        if [[ "$DIFF_MODE" == "1" ]]; then
            echo "error: strict differential mode requires PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER" >&2
            exit 1
        fi
        diff_effective_mode="unsupported_reference_runner"
        diff_note="trusted-reference runner not configured; set PGEN_SV_PREPROCESSOR_REFERENCE_RUNNER"
    elif [[ ! -x "$DIFF_REFERENCE_RUNNER" ]]; then
        if [[ "$DIFF_MODE" == "1" ]]; then
            echo "error: strict differential mode requires executable trusted-reference runner at '$DIFF_REFERENCE_RUNNER'" >&2
            exit 1
        fi
        diff_effective_mode="unsupported_reference_runner"
        diff_note="trusted-reference runner path is not executable: $DIFF_REFERENCE_RUNNER"
    else
        diff_runner_ready=1
        diff_probe_help_log="$LOG_DIR/diff_reference_probe_help.log"
        diff_probe_log="$LOG_DIR/diff_reference_probe.log"
        if "$DIFF_REFERENCE_RUNNER" --help >"$diff_probe_help_log" 2>&1 && grep -q -- "--probe" "$diff_probe_help_log"; then
            if "$DIFF_REFERENCE_RUNNER" --probe >"$diff_probe_log" 2>&1; then
                diff_note="trusted-reference differential classification enabled (runner probe succeeded)"
            else
                if [[ "$DIFF_MODE" == "1" ]]; then
                    echo "error: strict differential mode requires an available trusted-reference backend" >&2
                    echo "probe log: $diff_probe_log" >&2
                    exit 1
                fi
                diff_effective_mode="unsupported_reference_runner"
                diff_note="trusted-reference backend unavailable; probe failed (see $diff_probe_log)"
                diff_runner_ready=0
            fi
        else
            diff_note="trusted-reference differential classification enabled (runner probe unsupported)"
        fi

        if (( diff_runner_ready == 1 )); then
            diff_effective_mode="enabled"
            : >"$diff_cases_jsonl"
            declare -a diff_raw_samples=()
            mapfile -t diff_raw_samples < "$samples0a"
            diff_total_samples_seen="${#diff_raw_samples[@]}"
            diff_case_index=0
            for diff_sample_text in "${diff_raw_samples[@]}"; do
                if (( diff_samples_checked >= DIFF_MAX_SAMPLES )); then
                    break
                fi
                if [[ -z "$diff_sample_text" ]]; then
                    continue
                fi

                diff_sample_file="$WORK_DIR/diff_sample_${diff_case_index}.sv"
                diff_rust_output="$WORK_DIR/diff_sample_${diff_case_index}.rust.out.sv"
                diff_ref_output="$WORK_DIR/diff_sample_${diff_case_index}.reference.out.sv"
                diff_rust_diag="$WORK_DIR/diff_sample_${diff_case_index}.rust.diag.json"
                diff_ref_diag="$WORK_DIR/diff_sample_${diff_case_index}.reference.diag.json"
                diff_rust_log="$LOG_DIR/diff_sample_${diff_case_index}.rust.log"
                diff_ref_log="$LOG_DIR/diff_sample_${diff_case_index}.reference.log"
                diff_rust_norm="$WORK_DIR/diff_sample_${diff_case_index}.rust.norm.txt"
                diff_ref_norm="$WORK_DIR/diff_sample_${diff_case_index}.reference.norm.txt"

            printf '%s\n' "$diff_sample_text" >"$diff_sample_file"

            rust_exit=0
            if "$AST_PIPELINE_BIN" "$diff_sample_file" \
                --preprocess-systemverilog \
                --output "$diff_rust_output" \
                --sv-diagnostics-json "$diff_rust_diag" >"$diff_rust_log" 2>&1; then
                rust_exit=0
            else
                rust_exit=$?
            fi

            ref_exit=0
            if "$DIFF_REFERENCE_RUNNER" "$diff_sample_file" "$diff_ref_output" "$diff_ref_diag" >"$diff_ref_log" 2>&1; then
                ref_exit=0
            else
                ref_exit=$?
            fi

            rust_warnings=0
            rust_errors=0
            ref_warnings=0
            ref_errors=0
            rust_diag_available=0
            ref_diag_available=0

            if [[ -s "$diff_rust_diag" ]] && jq -e 'type == "array"' "$diff_rust_diag" >/dev/null 2>&1; then
                rust_diag_available=1
                rust_warnings="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$diff_rust_diag")"
                rust_errors="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$diff_rust_diag")"
            fi
            if [[ -s "$diff_ref_diag" ]] && jq -e 'type == "array"' "$diff_ref_diag" >/dev/null 2>&1; then
                ref_diag_available=1
                ref_warnings="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$diff_ref_diag")"
                ref_errors="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$diff_ref_diag")"
            fi

            category=""
            if (( rust_exit == 0 && ref_exit == 0 )); then
                if [[ ! -f "$diff_rust_output" || ! -f "$diff_ref_output" ]]; then
                    category="reference_artifact_missing"
                elif cmp -s "$diff_rust_output" "$diff_ref_output"; then
                    if (( rust_diag_available == 1 && ref_diag_available == 1 )) && (( rust_warnings != ref_warnings || rust_errors != ref_errors )); then
                        category="diagnostics_mismatch"
                    else
                        category="match"
                    fi
                else
                    normalize_text_for_diff_output "$diff_rust_output" "$diff_rust_norm"
                    normalize_text_for_diff_output "$diff_ref_output" "$diff_ref_norm"
                    if cmp -s "$diff_rust_norm" "$diff_ref_norm"; then
                        category="whitespace_only_output_mismatch"
                    else
                        category="output_mismatch"
                    fi
                fi
            elif (( rust_exit != 0 && ref_exit == 0 )); then
                category="rust_failed_reference_passed"
            elif (( rust_exit == 0 && ref_exit != 0 )); then
                category="reference_failed_rust_passed"
            else
                category="both_failed"
            fi

            case "$category" in
                match)
                    diff_match_count=$((diff_match_count + 1))
                    ;;
                diagnostics_mismatch)
                    diff_diagnostics_mismatch_count=$((diff_diagnostics_mismatch_count + 1))
                    diff_mismatch_count=$((diff_mismatch_count + 1))
                    ;;
                whitespace_only_output_mismatch)
                    diff_whitespace_only_output_mismatch_count=$((diff_whitespace_only_output_mismatch_count + 1))
                    diff_mismatch_count=$((diff_mismatch_count + 1))
                    ;;
                output_mismatch)
                    diff_output_mismatch_count=$((diff_output_mismatch_count + 1))
                    diff_mismatch_count=$((diff_mismatch_count + 1))
                    ;;
                rust_failed_reference_passed)
                    diff_rust_failed_reference_passed_count=$((diff_rust_failed_reference_passed_count + 1))
                    diff_mismatch_count=$((diff_mismatch_count + 1))
                    ;;
                reference_failed_rust_passed)
                    diff_reference_failed_rust_passed_count=$((diff_reference_failed_rust_passed_count + 1))
                    diff_mismatch_count=$((diff_mismatch_count + 1))
                    ;;
                both_failed)
                    diff_both_failed_count=$((diff_both_failed_count + 1))
                    diff_mismatch_count=$((diff_mismatch_count + 1))
                    ;;
                reference_artifact_missing)
                    diff_reference_artifact_missing_count=$((diff_reference_artifact_missing_count + 1))
                    diff_mismatch_count=$((diff_mismatch_count + 1))
                    ;;
                *)
                    echo "error: unknown differential mismatch category '$category'" >&2
                    exit 1
                    ;;
            esac

            jq -n \
                --argjson index "$diff_case_index" \
                --arg category "$category" \
                --arg sample_file "$diff_sample_file" \
                --arg rust_output "$diff_rust_output" \
                --arg reference_output "$diff_ref_output" \
                --arg rust_log "$diff_rust_log" \
                --arg reference_log "$diff_ref_log" \
                --argjson rust_exit "$rust_exit" \
                --argjson reference_exit "$ref_exit" \
                --argjson rust_warnings "$rust_warnings" \
                --argjson rust_errors "$rust_errors" \
                --argjson reference_warnings "$ref_warnings" \
                --argjson reference_errors "$ref_errors" \
                '{
                    index: $index,
                    category: $category,
                    sample_file: $sample_file,
                    rust: {
                        output_file: $rust_output,
                        log_file: $rust_log,
                        exit_code: $rust_exit,
                        warning_count: $rust_warnings,
                        error_count: $rust_errors
                    },
                    reference: {
                        output_file: $reference_output,
                        log_file: $reference_log,
                        exit_code: $reference_exit,
                        warning_count: $reference_warnings,
                        error_count: $reference_errors
                    }
                }' >>"$diff_cases_jsonl"

                diff_samples_checked=$((diff_samples_checked + 1))
                diff_case_index=$((diff_case_index + 1))
            done
        fi
    fi
fi

if [[ -s "$diff_cases_jsonl" ]]; then
    diff_cases_json="$(jq -s '.' "$diff_cases_jsonl")"
else
    diff_cases_json='[]'
fi

jq -n \
    --arg grammar_name "$GRAMMAR_NAME" \
    --arg requested_mode "$DIFF_MODE" \
    --arg effective_mode "$diff_effective_mode" \
    --arg note "$diff_note" \
    --arg reference_runner "$diff_reference_runner" \
    --argjson total_samples_seen "$diff_total_samples_seen" \
    --argjson samples_checked "$diff_samples_checked" \
    --argjson max_samples "$DIFF_MAX_SAMPLES" \
    --argjson mismatch_count "$diff_mismatch_count" \
    --argjson match_count "$diff_match_count" \
    --argjson diagnostics_mismatch_count "$diff_diagnostics_mismatch_count" \
    --argjson whitespace_only_output_mismatch_count "$diff_whitespace_only_output_mismatch_count" \
    --argjson output_mismatch_count "$diff_output_mismatch_count" \
    --argjson rust_failed_reference_passed_count "$diff_rust_failed_reference_passed_count" \
    --argjson reference_failed_rust_passed_count "$diff_reference_failed_rust_passed_count" \
    --argjson both_failed_count "$diff_both_failed_count" \
    --argjson reference_artifact_missing_count "$diff_reference_artifact_missing_count" \
    --argjson cases "$diff_cases_json" \
    '{
        grammar_name: $grammar_name,
        requested_mode: $requested_mode,
        effective_mode: $effective_mode,
        note: $note,
        reference_runner: $reference_runner,
        total_samples_seen: $total_samples_seen,
        max_samples: $max_samples,
        samples_checked: $samples_checked,
        mismatch_count: $mismatch_count,
        taxonomy_counts: {
            match: $match_count,
            diagnostics_mismatch: $diagnostics_mismatch_count,
            whitespace_only_output_mismatch: $whitespace_only_output_mismatch_count,
            output_mismatch: $output_mismatch_count,
            rust_failed_reference_passed: $rust_failed_reference_passed_count,
            reference_failed_rust_passed: $reference_failed_rust_passed_count,
            both_failed: $both_failed_count,
            reference_artifact_missing: $reference_artifact_missing_count
        },
        cases: $cases
    }' >"$diff_report_json"

if [[ "$DIFF_MODE" == "1" && "$diff_effective_mode" == "enabled" && "$diff_mismatch_count" -gt 0 ]]; then
    echo "error: strict differential mode detected mismatches ($diff_mismatch_count)" >&2
    cat "$diff_report_json" >&2
    exit 1
fi

cat >"$SUMMARY_CSV" <<EOF
metric,value
grammar_name,$GRAMMAR_NAME
grammar_file,$GRAMMAR_FILE
parseability_mode_requested,$PARSEABILITY_MODE
parseability_mode_effective,$parseability_effective
parseability_attempts_total,$parseability_attempts_total
parseability_accepted_total,$parseability_accepted_total
parseability_rejected_total,$parseability_rejected_total
parseability_parser_rejections_total,$parseability_parser_rejections_total
parseability_generation_errors_total,$parseability_generation_errors_total
parseability_empty_generations_total,$parseability_empty_generations_total
parseability_acceptance_rate_percent,$parseability_acceptance_rate_percent_total
parseability_report_json,$parseability_report_json
sample_count,$SAMPLE_COUNT
gap_threshold,$GAP_THRESHOLD
target_max_attempts,$TARGET_MAX_ATTEMPTS
initial_targets,$initial_targets
resolved_targets,$resolved_targets
final_targets,$final_targets
target_attempts,$target_attempts
target_drive_alternate_entry_attempts_total,$target_drive_alternate_entry_attempts_total
target_drive_alternate_entry_accepted_outputs_total,$target_drive_alternate_entry_accepted_outputs_total
target_drive_alternate_entry_rejected_outputs_total,$target_drive_alternate_entry_rejected_outputs_total
fuzz_rounds,$FUZZ_ROUNDS
fuzz_accepted,$fuzz_accepted
fuzz_rejected,$fuzz_rejected
fuzz_minimized,$fuzz_minimized
fuzz_parseability_counterexamples,$fuzz_parseability_counterexamples
fuzz_shrunk_counterexamples,$fuzz_shrunk_counterexamples
diff_mode_requested,$DIFF_MODE
diff_mode_effective,$diff_effective_mode
diff_reference_runner,$diff_reference_runner
diff_samples_checked,$diff_samples_checked
diff_mismatch_count,$diff_mismatch_count
diff_taxonomy_match,$diff_match_count
diff_taxonomy_diagnostics_mismatch,$diff_diagnostics_mismatch_count
diff_taxonomy_whitespace_only_output_mismatch,$diff_whitespace_only_output_mismatch_count
diff_taxonomy_output_mismatch,$diff_output_mismatch_count
diff_taxonomy_rust_failed_reference_passed,$diff_rust_failed_reference_passed_count
diff_taxonomy_reference_failed_rust_passed,$diff_reference_failed_rust_passed_count
diff_taxonomy_both_failed,$diff_both_failed_count
diff_taxonomy_reference_artifact_missing,$diff_reference_artifact_missing_count
key_hit_pp_include,$(extract_rule_hit "$coverage3" "pp_include")
key_hit_pp_define,$(extract_rule_hit "$coverage3" "pp_define")
key_hit_pp_conditional,$(extract_rule_hit "$coverage3" "pp_conditional")
key_hit_pp_if_branch,$(extract_rule_hit "$coverage3" "pp_if_branch")
key_hit_pp_elsif_branch,$(extract_rule_hit "$coverage3" "pp_elsif_branch")
key_hit_pp_else_branch,$(extract_rule_hit "$coverage3" "pp_else_branch")
key_hit_macro_formals,$(extract_rule_hit "$coverage3" "macro_formals")
key_hit_macro_body_fragment,$(extract_rule_hit "$coverage3" "macro_body_fragment")
key_hit_macro_token_paste,$(extract_rule_hit "$coverage3" "macro_token_paste")
key_hit_macro_stringize,$(extract_rule_hit "$coverage3" "macro_stringize")
EOF

{
    echo "SV Preprocessor Quality Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "parseability: $parseability_note"
    echo "parseability_report: $parseability_report_json"
    echo "differential: $diff_note"
    echo "differential_report: $diff_report_json"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
    echo
    echo "Logs: $LOG_DIR"
    echo "Artifacts: $WORK_DIR"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

echo "✅ SV preprocessor quality gate passed."
