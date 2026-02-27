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

GRAMMAR_NAME="systemverilog_preprocessor"
GRAMMAR_FILE="$GRAMMARS_DIR/${GRAMMAR_NAME}.ebnf"
GRAMMAR_JSON="$WORK_DIR/${GRAMMAR_NAME}.json"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"

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

run_logged_rust "build_generated_ast_pipeline" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

run_logged "frontend_ebnf_to_json" \
    perl "$EBNF_TO_JSON" --pretty --quiet "$GRAMMAR_FILE" -o "$GRAMMAR_JSON"

require_nonempty_file "$GRAMMAR_JSON"

parseability_enabled=0
parseability_effective="disabled"
parseability_note="parseability validation disabled by configuration"
declare -a parseability_args=()

probe_samples="$WORK_DIR/parseability_probe_samples.txt"
probe_log="$LOG_DIR/parseability_probe.log"

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
    "${parseability_args[@]}" \
    --gap-report-threshold "$GAP_THRESHOLD" \
    --output "$samples0a" \
    --coverage-output "$coverage0a" \
    --gap-report-json "$gap0a"

run_logged "stage0_baseline_b_replay" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-stimuli \
    --count "$SAMPLE_COUNT" \
    --seed "$SEED_BASE" \
    "${parseability_args[@]}" \
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

assert_same_text "$samples0a" "$samples0b" "stage0 sample corpus"
assert_same_json "$coverage0a" "$coverage0b" "stage0 coverage metrics"
assert_same_json "$gap0a" "$gap0b" "stage0 gap report"

assert_json "$coverage0a" ".grammar_name == \"$GRAMMAR_NAME\"" "coverage stage0 grammar_name mismatch"
assert_json "$coverage0a" ".total_rules > 0" "coverage stage0 must report total_rules > 0"
assert_json "$coverage0a" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage0 attempts consistency failed"
assert_json "$coverage0a" ".sample_successes >= $SAMPLE_COUNT" "coverage stage0 sample_successes below requested count"

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

run_logged "stage1_gap_priority" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-stimuli \
    --count "$SAMPLE_COUNT" \
    --seed "$((SEED_BASE + 1))" \
    "${parseability_args[@]}" \
    --coverage-input "$coverage0a" \
    --gap-priority-report-input "$gap0a" \
    --output "$samples1" \
    --coverage-output "$coverage1" \
    --gap-report-json "$gap1" \
    --gap-report-threshold "$GAP_THRESHOLD"

require_nonempty_file "$samples1"
require_nonempty_file "$coverage1"
require_nonempty_file "$gap1"

assert_json "$coverage1" ".grammar_name == \"$GRAMMAR_NAME\"" "coverage stage1 grammar_name mismatch"
assert_json "$coverage1" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage1 attempts consistency failed"
assert_json "$gap1" ".grammar_name == \"$GRAMMAR_NAME\"" "gap stage1 grammar_name mismatch"

attempts1="$(extract_json_u64 "$coverage1" ".sample_attempts")"
successes1="$(extract_json_u64 "$coverage1" ".sample_successes")"
covered_rules1="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage1")"
covered_branches1="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage1")"

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
    "${parseability_args[@]}" \
    --coverage-input "$coverage1" \
    --target-report-input "$gap0a" \
    --target-max-attempts "$TARGET_MAX_ATTEMPTS" \
    --output "$samples2" \
    --coverage-output "$coverage2"

require_file "$samples2"
require_nonempty_file "$coverage2"

assert_json "$coverage2" ".grammar_name == \"$GRAMMAR_NAME\"" "coverage stage2 grammar_name mismatch"
assert_json "$coverage2" ".sample_attempts == (.sample_successes + .sample_errors)" "coverage stage2 attempts consistency failed"

attempts2="$(extract_json_u64 "$coverage2" ".sample_attempts")"
successes2="$(extract_json_u64 "$coverage2" ".sample_successes")"
covered_rules2="$(jq -er '[.rule_success_hits[] | select(. > 0)] | length | numbers' "$coverage2")"
covered_branches2="$(jq -er '[.branch_groups[]?.success_counts[]? | select(. > 0)] | length | numbers' "$coverage2")"

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
    "${parseability_args[@]}" \
    --coverage-input "$coverage2" \
    --output "$samples3" \
    --coverage-output "$coverage3" \
    --gap-report-json "$gap3" \
    --gap-report-threshold "$GAP_THRESHOLD"

require_nonempty_file "$samples3"
require_nonempty_file "$coverage3"
require_nonempty_file "$gap3"

assert_json "$coverage3" ".grammar_name == \"$GRAMMAR_NAME\"" "coverage stage3 grammar_name mismatch"
assert_json "$gap3" ".grammar_name == \"$GRAMMAR_NAME\"" "gap stage3 grammar_name mismatch"

final_targets="$(jq -er '.targets | length | numbers' "$gap3")"
if (( final_targets > initial_targets )); then
    echo "error: final actionable targets regressed ($initial_targets -> $final_targets)" >&2
    exit 1
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

cat >"$SUMMARY_CSV" <<EOF
metric,value
grammar_name,$GRAMMAR_NAME
grammar_file,$GRAMMAR_FILE
parseability_mode_requested,$PARSEABILITY_MODE
parseability_mode_effective,$parseability_effective
sample_count,$SAMPLE_COUNT
gap_threshold,$GAP_THRESHOLD
target_max_attempts,$TARGET_MAX_ATTEMPTS
initial_targets,$initial_targets
resolved_targets,$resolved_targets
final_targets,$final_targets
target_attempts,$target_attempts
fuzz_rounds,$FUZZ_ROUNDS
fuzz_accepted,$fuzz_accepted
fuzz_rejected,$fuzz_rejected
fuzz_minimized,$fuzz_minimized
fuzz_parseability_counterexamples,$fuzz_parseability_counterexamples
fuzz_shrunk_counterexamples,$fuzz_shrunk_counterexamples
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
