#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_AST_DUMP_CONTRACT_STATE_DIR:-$RUST_DIR/target/ast_dump_contract_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_TXT="$STATE_DIR/summary.txt"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSEABILITY_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"

mkdir -p "$LOG_DIR" "$WORK_DIR"

require_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool '$tool' is not available in PATH" >&2
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

run_expected_fail() {
    local label="$1"
    local expected_pattern="$2"
    shift 2
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then
        echo "error: '${label}' unexpectedly succeeded" >&2
        tail -n 80 "$log_file" >&2 || true
        exit 1
    fi
    if ! grep -Fq "$expected_pattern" "$log_file"; then
        echo "error: '${label}' failure log does not contain expected text: $expected_pattern" >&2
        tail -n 80 "$log_file" >&2 || true
        exit 1
    fi
    echo "    ok (${log_file})"
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

assert_same_file() {
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

require_tool jq
require_tool cmp
require_tool diff
require_tool cargo

echo "==> AST dump contract gate"
echo "state_dir: $STATE_DIR"

GEN_GRAMMAR_JSON="$WORK_DIR/mini_raw_ast.json"
GEN_GRAMMAR_LARGE_JSON="$WORK_DIR/mini_large_raw_ast.json"
GEN_SAMPLES_A="$WORK_DIR/gen_samples_a.txt"
GEN_SAMPLES_B="$WORK_DIR/gen_samples_b.txt"
GEN_SAMPLES_TRUNC="$WORK_DIR/gen_samples_trunc.txt"
GEN_DUMP_A="$WORK_DIR/gen_ast_a.json"
GEN_DUMP_B="$WORK_DIR/gen_ast_b.json"
GEN_DUMP_TRUNC="$WORK_DIR/gen_ast_trunc.json"
GEN_DUMP_DIFF="$WORK_DIR/gen_ast_determinism.diff"

PARSER_SMALL_INPUT="$WORK_DIR/parser_input_small.txt"
PARSER_LARGE_INPUT="$WORK_DIR/parser_input_large.txt"
PARSER_DUMP_A="$WORK_DIR/parser_ast_a.json"
PARSER_DUMP_B="$WORK_DIR/parser_ast_b.json"
PARSER_DUMP_TRUNC="$WORK_DIR/parser_ast_trunc.json"
PARSER_DUMP_DIFF="$WORK_DIR/parser_ast_determinism.diff"

cat >"$GEN_GRAMMAR_JSON" <<'EOF'
{
  "grammar_name": "mini",
  "raw_ast": [
    [
      ["rule", "start"],
      ["quoted_string", "a"]
    ]
  ]
}
EOF

large_token="$(printf 'a%.0s' {1..600})"
jq -n --arg tok "$large_token" '
{
  grammar_name: "mini_large",
  raw_ast: [
    [
      ["rule", "start"],
      ["quoted_string", $tok]
    ]
  ]
}
' >"$GEN_GRAMMAR_LARGE_JSON"

printf '%s\n' '@priority: [9, 1]' >"$PARSER_SMALL_INPUT"
large_left="$(printf '9%.0s' {1..400})"
large_right="$(printf '8%.0s' {1..400})"
printf '@priority: [%s, %s]\n' "$large_left" "$large_right" >"$PARSER_LARGE_INPUT"

run_logged_rust "build_ast_pipeline" cargo build --bin ast_pipeline
run_logged_rust "build_parseability_probe" cargo build --features generated_parsers --bin parseability_probe

run_logged "generation_dump_a" \
    "$AST_PIPELINE_BIN" "$GEN_GRAMMAR_JSON" \
    --generate-stimuli \
    --count 6 \
    --seed 424242 \
    --output "$GEN_SAMPLES_A" \
    --dump-gen-ast "$GEN_DUMP_A"

run_logged "generation_dump_b_replay" \
    "$AST_PIPELINE_BIN" "$GEN_GRAMMAR_JSON" \
    --generate-stimuli \
    --count 6 \
    --seed 424242 \
    --output "$GEN_SAMPLES_B" \
    --dump-gen-ast "$GEN_DUMP_B"

assert_same_file "$GEN_DUMP_A" "$GEN_DUMP_B" "$GEN_DUMP_DIFF" "generation AST dump determinism"
assert_same_file "$GEN_SAMPLES_A" "$GEN_SAMPLES_B" "$WORK_DIR/gen_stimuli_determinism.diff" "generation stimuli replay determinism"
assert_json "$GEN_DUMP_A" '.grammar_name == "mini"' "generation dump grammar name mismatch"
assert_json "$GEN_DUMP_A" 'has("kind") | not' "generation dump unexpectedly emitted truncation envelope"

run_logged "generation_dump_truncation" \
    "$AST_PIPELINE_BIN" "$GEN_GRAMMAR_LARGE_JSON" \
    --generate-stimuli \
    --count 1 \
    --seed 99 \
    --output "$GEN_SAMPLES_TRUNC" \
    --dump-gen-ast "$GEN_DUMP_TRUNC" \
    --dump-gen-ast-max-bytes 256

assert_json "$GEN_DUMP_TRUNC" '.kind == "pgen_ast_dump_truncation"' "generation truncation kind mismatch"
assert_json "$GEN_DUMP_TRUNC" '.dump_kind == "generation_input_ast"' "generation truncation dump_kind mismatch"
assert_json "$GEN_DUMP_TRUNC" '.truncated == true' "generation truncation flag mismatch"
assert_json "$GEN_DUMP_TRUNC" '.full_bytes > .max_bytes' "generation truncation full_bytes/max_bytes invariant failed"

GEN_NEG_DIR="$WORK_DIR/generation_negative_path"
mkdir -p "$GEN_NEG_DIR"
run_expected_fail "generation_dump_negative_path" \
    "failed to write generation-input AST JSON" \
    "$AST_PIPELINE_BIN" "$GEN_GRAMMAR_JSON" \
    --generate-parser \
    --output "$WORK_DIR/mini_parser.rs" \
    --dump-gen-ast "$GEN_NEG_DIR"

run_logged "parser_dump_a" \
    "$PARSEABILITY_PROBE_BIN" \
    --parse-dump-ast builtin_semantic_annotation "$PARSER_SMALL_INPUT" "$PARSER_DUMP_A"

run_logged "parser_dump_b_replay" \
    "$PARSEABILITY_PROBE_BIN" \
    --parse-dump-ast builtin_semantic_annotation "$PARSER_SMALL_INPUT" "$PARSER_DUMP_B"

assert_same_file "$PARSER_DUMP_A" "$PARSER_DUMP_B" "$PARSER_DUMP_DIFF" "parser AST dump determinism"
assert_json "$PARSER_DUMP_A" 'has("kind") | not' "parser dump unexpectedly emitted truncation envelope"

run_logged "parser_dump_truncation" \
    "$PARSEABILITY_PROBE_BIN" \
    --parse-dump-ast builtin_semantic_annotation "$PARSER_LARGE_INPUT" "$PARSER_DUMP_TRUNC" \
    --max-bytes 256

assert_json "$PARSER_DUMP_TRUNC" '.kind == "pgen_ast_dump_truncation"' "parser truncation kind mismatch"
assert_json "$PARSER_DUMP_TRUNC" '.dump_kind == "parser_return_ast"' "parser truncation dump_kind mismatch"
assert_json "$PARSER_DUMP_TRUNC" '.truncated == true' "parser truncation flag mismatch"
assert_json "$PARSER_DUMP_TRUNC" '.full_bytes > .max_bytes' "parser truncation full_bytes/max_bytes invariant failed"

PARSER_NEG_DIR="$WORK_DIR/parser_negative_path"
mkdir -p "$PARSER_NEG_DIR"
run_expected_fail "parser_dump_negative_path" \
    "failed to write parser AST log" \
    "$PARSEABILITY_PROBE_BIN" \
    --parse-dump-ast builtin_semantic_annotation "$PARSER_SMALL_INPUT" "$PARSER_NEG_DIR"

cat >"$SUMMARY_TXT" <<EOF
AST Dump Contract Gate Summary
state_dir: $STATE_DIR
generation_dump_deterministic: 1
generation_dump_truncation_envelope: 1
generation_dump_negative_path: 1
parser_dump_deterministic: 1
parser_dump_truncation_envelope: 1
parser_dump_negative_path: 1
logs: $LOG_DIR
work: $WORK_DIR
EOF

cat "$SUMMARY_TXT"
echo "✅ AST dump contract gate passed."
