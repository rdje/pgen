#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"

STATE_DIR="${PGEN_STIMULI_MODULE_PARITY_STATE_DIR:-$RUST_DIR/target/stimuli_module_parity_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

SAMPLE_COUNT="${PGEN_STIMULI_MODULE_PARITY_COUNT:-16}"
GAP_THRESHOLD="${PGEN_STIMULI_MODULE_PARITY_GAP_THRESHOLD:-1}"
MAX_DEPTH="${PGEN_STIMULI_MODULE_PARITY_MAX_DEPTH:-24}"
MAX_REPEAT="${PGEN_STIMULI_MODULE_PARITY_MAX_REPEAT:-4}"
CONTRACT_FILE="${PGEN_STIMULI_MODULE_PARITY_CONTRACT:-$RUST_DIR/test_data/grammar_quality/stimuli_module_parity_contract.json}"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"

if ! [[ "$SAMPLE_COUNT" =~ ^[0-9]+$ ]] || [[ "$SAMPLE_COUNT" -lt 1 ]]; then
    echo "error: PGEN_STIMULI_MODULE_PARITY_COUNT must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$GAP_THRESHOLD" =~ ^[0-9]+$ ]] || [[ "$GAP_THRESHOLD" -lt 1 ]]; then
    echo "error: PGEN_STIMULI_MODULE_PARITY_GAP_THRESHOLD must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$MAX_DEPTH" =~ ^[0-9]+$ ]] || [[ "$MAX_DEPTH" -lt 1 ]]; then
    echo "error: PGEN_STIMULI_MODULE_PARITY_MAX_DEPTH must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$MAX_REPEAT" =~ ^[0-9]+$ ]] || [[ "$MAX_REPEAT" -lt 1 ]]; then
    echo "error: PGEN_STIMULI_MODULE_PARITY_MAX_REPEAT must be an integer >= 1" >&2
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
    local input_path="$1"
    local output_path="$2"
    jq -S . "$input_path" >"$output_path"
}

build_samples_to_literals_helper() {
    local helper_src="$WORK_DIR/samples_to_rust_literals.rs"
    local helper_bin="$WORK_DIR/samples_to_rust_literals"
    local log_file="$LOG_DIR/build_samples_to_literals_helper.log"

    echo "==> build_samples_to_literals_helper"
    if {
        cat >"$helper_src" <<'EOF'
use std::{env, fs};

fn main() {
    let path = env::args()
        .nth(1)
        .expect("usage: samples_to_rust_literals <samples.txt>");
    let input = fs::read_to_string(path).expect("failed to read samples file");
    for sample in input.lines() {
        println!("{:?},", sample);
    }
}
EOF
        rustc --edition 2021 "$helper_src" -o "$helper_bin"
    } >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
    else
        echo "    fail (${log_file})" >&2
        tail -n 80 "$log_file" >&2 || true
        exit 1
    fi
}

emit_samples_as_rust_literals() {
    local samples_path="$1"
    local output_path="$2"
    local helper_bin="$WORK_DIR/samples_to_rust_literals"
    "$helper_bin" "$samples_path" >"$output_path"
}

extract_module_rust_literals() {
    local module_path="$1"
    local output_path="$2"
    awk '
        /^pub const STIMULI: \[&str; [0-9]+\] = \[$/ { capture=1; next }
        capture && /^\];$/ { capture=0; exit }
        capture {
            sub(/^[[:space:]]+/, "", $0)
            print $0
        }
    ' "$module_path" >"$output_path"
}

assert_file_equals() {
    local left="$1"
    local right="$2"
    local diff_output="$3"
    local description="$4"
    if ! cmp -s "$left" "$right"; then
        diff -u "$left" "$right" >"$diff_output" || true
        echo "error: ${description} mismatch (diff: $diff_output)" >&2
        exit 1
    fi
}

parity_for_grammar() {
    local label="$1"
    local grammar_name="$2"
    local grammar_json="$3"
    local seed="$4"
    local parseability_required="$5"
    local entry_rule="$6"

    local inmem_samples="$WORK_DIR/${label}_samples_inmemory.txt"
    local inmem_coverage="$WORK_DIR/${label}_coverage_inmemory.json"
    local inmem_gap="$WORK_DIR/${label}_gap_inmemory.json"

    local module_rs="$WORK_DIR/${label}_stimuli.rs"
    local inmem_literals="$WORK_DIR/${label}_samples_inmemory.literals"
    local module_literals="$WORK_DIR/${label}_samples_module.literals"
    local module_coverage="$WORK_DIR/${label}_coverage_module.json"
    local module_gap="$WORK_DIR/${label}_gap_module.json"

    local inmem_coverage_norm="$WORK_DIR/${label}_coverage_inmemory.normalized.json"
    local module_coverage_norm="$WORK_DIR/${label}_coverage_module.normalized.json"
    local inmem_gap_norm="$WORK_DIR/${label}_gap_inmemory.normalized.json"
    local module_gap_norm="$WORK_DIR/${label}_gap_module.normalized.json"

    local sample_diff="$WORK_DIR/${label}_sample_literals.diff"
    local coverage_diff="$WORK_DIR/${label}_coverage.diff"
    local gap_diff="$WORK_DIR/${label}_gap.diff"

    local -a base_args=(
        "$grammar_json"
        --count "$SAMPLE_COUNT"
        --seed "$seed"
        --max-depth "$MAX_DEPTH"
        --max-repeat "$MAX_REPEAT"
        --gap-report-threshold "$GAP_THRESHOLD"
    )
    if [[ -n "$entry_rule" ]]; then
        base_args+=(--entry-rule "$entry_rule")
    fi
    if [[ "$parseability_required" -eq 1 ]]; then
        base_args+=(--validate-parseability)
    fi

    run_logged "${label}_inmemory_generate" \
        "$AST_PIPELINE_BIN" "${base_args[@]}" \
        --generate-stimuli \
        --output "$inmem_samples" \
        --coverage-output "$inmem_coverage" \
        --gap-report-json "$inmem_gap"

    require_nonempty_file "$inmem_samples"
    require_nonempty_file "$inmem_coverage"
    require_nonempty_file "$inmem_gap"
    assert_json "$inmem_coverage" ".grammar_name == \"$grammar_name\"" "in-memory coverage grammar mismatch for '$label'"
    assert_json "$inmem_gap" ".grammar_name == \"$grammar_name\"" "in-memory gap grammar mismatch for '$label'"

    run_logged "${label}_module_generate" \
        "$AST_PIPELINE_BIN" "${base_args[@]}" \
        --generate-stimuli-module \
        --output "$module_rs" \
        --coverage-output "$module_coverage" \
        --gap-report-json "$module_gap"

    require_nonempty_file "$module_rs"
    require_nonempty_file "$module_coverage"
    require_nonempty_file "$module_gap"
    assert_json "$module_coverage" ".grammar_name == \"$grammar_name\"" "module coverage grammar mismatch for '$label'"
    assert_json "$module_gap" ".grammar_name == \"$grammar_name\"" "module gap grammar mismatch for '$label'"

    emit_samples_as_rust_literals "$inmem_samples" "$inmem_literals"
    extract_module_rust_literals "$module_rs" "$module_literals"
    require_nonempty_file "$inmem_literals"
    require_nonempty_file "$module_literals"

    if ! grep -Fq "pub const STIMULI_MODULE_API_VERSION: u32 = 1;" "$module_rs"; then
        echo "error: module '$module_rs' missing STIMULI_MODULE_API_VERSION constant" >&2
        exit 1
    fi
    if ! grep -Fq "pub const REQUESTED_SAMPLE_COUNT: usize = ${SAMPLE_COUNT};" "$module_rs"; then
        echo "error: module '$module_rs' REQUESTED_SAMPLE_COUNT does not match requested count" >&2
        exit 1
    fi
    if ! grep -Fq "pub const GENERATION_SEED: u64 = ${seed};" "$module_rs"; then
        echo "error: module '$module_rs' GENERATION_SEED does not match configured seed" >&2
        exit 1
    fi

    assert_file_equals "$inmem_literals" "$module_literals" "$sample_diff" "${label} sample corpus parity"

    canonicalize_json "$inmem_coverage" "$inmem_coverage_norm"
    canonicalize_json "$module_coverage" "$module_coverage_norm"
    assert_file_equals "$inmem_coverage_norm" "$module_coverage_norm" "$coverage_diff" "${label} coverage parity"

    canonicalize_json "$inmem_gap" "$inmem_gap_norm"
    canonicalize_json "$module_gap" "$module_gap_norm"
    assert_file_equals "$inmem_gap_norm" "$module_gap_norm" "$gap_diff" "${label} gap parity"

    local entry_rule_used
    entry_rule_used="$(jq -er '.entry_rule | strings' "$inmem_gap")"
    echo "    ${label} parity summary: parseability_required=${parseability_required} entry_rule=${entry_rule_used} samples=$SAMPLE_COUNT seed=$seed"
    echo "${label},${grammar_name},${parseability_required},${entry_rule_used},${SAMPLE_COUNT},${seed},pass" >>"$SUMMARY_CSV"
}

require_tool jq
require_tool perl
require_tool rustc
require_tool base64
require_file "$EBNF_TO_JSON"
require_file "$CONTRACT_FILE"

grammar_count="$(jq -er '.grammars | length | numbers' "$CONTRACT_FILE")"
contract_version="$(jq -er '.version | numbers' "$CONTRACT_FILE")"
if [[ "$grammar_count" -lt 1 ]]; then
    echo "error: contract '$CONTRACT_FILE' must contain at least one grammar entry" >&2
    exit 1
fi

echo "==> Stimuli module parity gate"
echo "state_dir: $STATE_DIR"
echo "contract_file: $CONTRACT_FILE"
echo "contract_version: $contract_version"
echo "grammar_count: $grammar_count"
echo "sample_count: $SAMPLE_COUNT"
echo "gap_threshold: $GAP_THRESHOLD"
echo "max_depth: $MAX_DEPTH"
echo "max_repeat: $MAX_REPEAT"

echo "grammar,grammar_name,parseability_required,entry_rule,sample_count,seed,status" >"$SUMMARY_CSV"

run_logged_rust "build_generated_ast_pipeline" \
    cargo build --features generated_parsers --bin ast_pipeline

build_samples_to_literals_helper

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

mapfile -t grammar_rows < <(jq -r '.grammars[] | @base64' "$CONTRACT_FILE")

for encoded in "${grammar_rows[@]}"; do
    decoded="$(printf '%s' "$encoded" | base64 --decode)"
    label="$(printf '%s\n' "$decoded" | jq -er '.id | strings')"
    grammar_name="$(printf '%s\n' "$decoded" | jq -er '.grammar_name | strings')"
    ebnf_path_rel="$(printf '%s\n' "$decoded" | jq -er '.ebnf_path | strings')"
    seed="$(printf '%s\n' "$decoded" | jq -er '.seed | numbers')"
    parseability_required="$(printf '%s\n' "$decoded" | jq -er 'if .require_parseability then 1 else 0 end')"
    entry_rule="$(printf '%s\n' "$decoded" | jq -er '.entry_rule // ""')"

    ebnf_path="$ROOT_DIR/$ebnf_path_rel"
    grammar_json="$WORK_DIR/${label}.json"

    require_file "$ebnf_path"
    run_logged "${label}_frontend_ebnf_to_json" \
        "$EBNF_TO_JSON" --pretty --quiet "$ebnf_path" -o "$grammar_json"
    require_nonempty_file "$grammar_json"
    assert_json "$grammar_json" ".grammar_name == \"$grammar_name\"" "frontend grammar_name mismatch for contract entry '$label'"
    assert_json "$grammar_json" ".raw_ast | type == \"array\"" "frontend output raw_ast must be an array for contract entry '$label'"

    parity_for_grammar \
        "$label" \
        "$grammar_name" \
        "$grammar_json" \
        "$seed" \
        "$parseability_required" \
        "$entry_rule"
done

{
    echo "PGEN Stimuli Module Parity Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "contract_file: $CONTRACT_FILE"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

cat <<EOF
✅ Stimuli module parity gate passed.
Logs: $LOG_DIR
Artifacts: $WORK_DIR
EOF
