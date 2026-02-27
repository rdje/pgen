#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"
GRAMMARS_DIR="$ROOT_DIR/grammars"

STATE_DIR="${PGEN_HDL_FRONTEND_STATE_DIR:-$RUST_DIR/target/hdl_frontend_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

STRICT="${PGEN_HDL_FRONTEND_STRICT:-0}"
STIMULI_COUNT="${PGEN_HDL_FRONTEND_STIMULI_COUNT:-8}"
STIMULI_SEED="${PGEN_HDL_FRONTEND_STIMULI_SEED:-1337}"

GRAMMARS=("systemverilog" "vhdl")
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"

if ! [[ "$STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_HDL_FRONTEND_STRICT must be 0 or 1" >&2
    exit 2
fi

if ! [[ "$STIMULI_COUNT" =~ ^[0-9]+$ ]] || [[ "$STIMULI_COUNT" -lt 1 ]]; then
    echo "error: PGEN_HDL_FRONTEND_STIMULI_COUNT must be an integer >= 1" >&2
    exit 2
fi

mkdir -p "$STATE_DIR" "$LOG_DIR" "$WORK_DIR"

echo "==> Building ast_pipeline binary"
(cd "$RUST_DIR" && cargo build --bin ast_pipeline >/dev/null)

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary not found at '$AST_PIPELINE_BIN'" >&2
    exit 1
fi

echo "grammar,grammar_file,ebnf_to_json,json_to_parser,json_to_stimuli,overall,notes" >"$SUMMARY_CSV"
{
    echo "PGEN HDL Frontend Readiness Summary"
    echo "state_dir: $STATE_DIR"
    echo "strict_mode: $STRICT"
    echo "stimuli_count: $STIMULI_COUNT"
    echo "stimuli_seed: $STIMULI_SEED"
    echo
} >"$SUMMARY_TXT"

failures=0

for grammar in "${GRAMMARS[@]}"; do
    grammar_file="$GRAMMARS_DIR/${grammar}.ebnf"
    json_out="$WORK_DIR/${grammar}.json"
    parser_out="$WORK_DIR/${grammar}_parser.rs"
    stimuli_out="$WORK_DIR/${grammar}_stimuli.txt"

    grammar_file_status="missing"
    ebnf_to_json_status="skip"
    json_to_parser_status="skip"
    json_to_stimuli_status="skip"
    overall="not_ready"
    notes="grammar file missing (expected ${grammar}.ebnf)"

    ebnf_log="$LOG_DIR/${grammar}.ebnf_to_json.log"
    parser_log="$LOG_DIR/${grammar}.json_to_parser.log"
    stimuli_log="$LOG_DIR/${grammar}.json_to_stimuli.log"

    if [[ -f "$grammar_file" ]]; then
        grammar_file_status="present"
        notes="ok"
        if perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$grammar_file" -o "$json_out" >"$ebnf_log" 2>&1; then
            ebnf_to_json_status="pass"
            if "$AST_PIPELINE_BIN" "$json_out" --generate-parser --eliminate-left-recursion -o "$parser_out" >"$parser_log" 2>&1; then
                json_to_parser_status="pass"
                if "$AST_PIPELINE_BIN" "$json_out" --generate-stimuli --count "$STIMULI_COUNT" --seed "$STIMULI_SEED" --output "$stimuli_out" >"$stimuli_log" 2>&1; then
                    json_to_stimuli_status="pass"
                    overall="pass"
                else
                    json_to_stimuli_status="fail"
                    overall="fail"
                    failures=$((failures + 1))
                    notes="json_to_stimuli failed (see logs/${grammar}.json_to_stimuli.log)"
                fi
            else
                json_to_parser_status="fail"
                overall="fail"
                failures=$((failures + 1))
                notes="json_to_parser failed (see logs/${grammar}.json_to_parser.log)"
            fi
        else
            ebnf_to_json_status="fail"
            overall="fail"
            failures=$((failures + 1))
            notes="ebnf_to_json failed (see logs/${grammar}.ebnf_to_json.log)"
        fi
    else
        failures=$((failures + 1))
    fi

    echo "${grammar},${grammar_file_status},${ebnf_to_json_status},${json_to_parser_status},${json_to_stimuli_status},${overall},${notes}" >>"$SUMMARY_CSV"
done

{
    echo "Results:"
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
    echo
    echo "Logs: $LOG_DIR"
    echo "Artifacts: $WORK_DIR"
} >>"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

if [[ "$failures" -ne 0 ]]; then
    echo "⚠️  HDL frontend readiness has $failures failing or missing grammar flow(s)." >&2
    if [[ "$STRICT" -eq 1 ]]; then
        echo "❌ strict mode enabled: failing." >&2
        exit 1
    fi
    echo "ℹ️  strict mode disabled: reporting only." >&2
else
    echo "✅ HDL frontend readiness check passed for all tracked grammars."
fi
