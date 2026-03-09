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
PARSEABILITY_MAX_ATTEMPTS="${PGEN_HDL_FRONTEND_PARSEABILITY_MAX_ATTEMPTS:-50}"

GRAMMARS=("systemverilog" "vhdl")
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"

if ! [[ "$STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_HDL_FRONTEND_STRICT must be 0 or 1" >&2
    exit 2
fi

if ! [[ "$STIMULI_COUNT" =~ ^[0-9]+$ ]] || [[ "$STIMULI_COUNT" -lt 1 ]]; then
    echo "error: PGEN_HDL_FRONTEND_STIMULI_COUNT must be an integer >= 1" >&2
    exit 2
fi

if ! [[ "$PARSEABILITY_MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [[ "$PARSEABILITY_MAX_ATTEMPTS" -lt 1 ]]; then
    echo "error: PGEN_HDL_FRONTEND_PARSEABILITY_MAX_ATTEMPTS must be an integer >= 1" >&2
    exit 2
fi

PARSEABILITY_ATTEMPT_BUDGET=$((STIMULI_COUNT * PARSEABILITY_MAX_ATTEMPTS))

mkdir -p "$STATE_DIR" "$LOG_DIR" "$WORK_DIR"

require_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool '$tool' is not available in PATH" >&2
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

build_frontend_binaries() {
    (
        cd "$RUST_DIR"
        env "$@" cargo build \
            --features generated_parsers \
            --bin ast_pipeline \
            --bin parseability_probe \
            >/dev/null
    )
}

parseability_build_env_for_grammar() {
    local grammar="$1"
    local parser_path="$2"
    case "$grammar" in
        systemverilog)
            printf 'PGEN_SYSTEMVERILOG_PARSER_PATH=%s\n' "$parser_path"
            ;;
        vhdl)
            printf 'PGEN_VHDL_PARSER_PATH=%s\n' "$parser_path"
            ;;
    esac
}

require_tool jq

echo "==> Building ast_pipeline and parseability_probe binaries"
build_frontend_binaries

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary not found at '$AST_PIPELINE_BIN'" >&2
    exit 1
fi
if [[ ! -x "$PARSE_PROBE_BIN" ]]; then
    echo "error: parseability_probe binary not found at '$PARSE_PROBE_BIN'" >&2
    exit 1
fi

echo "grammar,grammar_file,ebnf_to_json,json_to_parser,json_to_stimuli,parser_registry_support,parseability,parseability_attempts,parseability_accepted,parseability_rejected,parseability_acceptance_rate_percent,parseability_report_json,overall,notes" >"$SUMMARY_CSV"
{
    echo "PGEN HDL Frontend Readiness Summary"
    echo "state_dir: $STATE_DIR"
    echo "strict_mode: $STRICT"
    echo "stimuli_count: $STIMULI_COUNT"
    echo "stimuli_seed: $STIMULI_SEED"
    echo "parseability_max_attempts_per_sample: $PARSEABILITY_MAX_ATTEMPTS"
    echo "parseability_attempt_budget: $PARSEABILITY_ATTEMPT_BUDGET"
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
    parser_registry_support_status="skip"
    parseability_status="skip"
    parseability_attempts="0"
    parseability_accepted="0"
    parseability_rejected="0"
    parseability_acceptance_rate="0.00"
    parseability_report_path="n/a"
    overall="not_ready"
    notes="grammar file missing (expected ${grammar}.ebnf)"

    ebnf_log="$LOG_DIR/${grammar}.ebnf_to_json.log"
    parser_log="$LOG_DIR/${grammar}.json_to_parser.log"
    stimuli_log="$LOG_DIR/${grammar}.json_to_stimuli.log"
    probe_build_log="$LOG_DIR/${grammar}.parseability_binaries_build.log"
    probe_support_log="$LOG_DIR/${grammar}.parseability_support.log"
    parseability_report_json="$WORK_DIR/${grammar}_parseability_report.json"

    if [[ -f "$grammar_file" ]]; then
        grammar_file_status="present"
        notes="ok"
        if perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$grammar_file" -o "$json_out" >"$ebnf_log" 2>&1; then
            ebnf_to_json_status="pass"
            if "$AST_PIPELINE_BIN" "$json_out" --generate-parser --eliminate-left-recursion -o "$parser_out" >"$parser_log" 2>&1; then
                json_to_parser_status="pass"
                build_env_line="$(parseability_build_env_for_grammar "$grammar" "$parser_out")"
                echo "==> Rebuilding ast_pipeline and parseability_probe for ${grammar} parseability"
                if build_frontend_binaries "$build_env_line" >"$probe_build_log" 2>&1; then
                    :
                else
                    parser_registry_support_status="fail"
                    parseability_status="fail"
                    overall="fail"
                    failures=$((failures + 1))
                    notes="failed to rebuild parseability binaries (see logs/${grammar}.parseability_binaries_build.log)"
                fi

                if [[ "$notes" == "ok" ]]; then
                    if "$PARSE_PROBE_BIN" --supports "$grammar" >"$probe_support_log" 2>&1; then
                        parser_registry_support_status="pass"
                        if "$AST_PIPELINE_BIN" "$json_out" --generate-stimuli --count "$STIMULI_COUNT" --seed "$STIMULI_SEED" --validate-parseability --parseability-max-attempts "$PARSEABILITY_ATTEMPT_BUDGET" --parseability-report-json "$parseability_report_json" --output "$stimuli_out" >"$stimuli_log" 2>&1; then
                            require_nonempty_file "$stimuli_out"
                            require_nonempty_file "$parseability_report_json"
                            if jq -e ".grammar_name == \"$grammar\" and .summary.requested == $STIMULI_COUNT and .summary.accepted == $STIMULI_COUNT and .summary.attempts >= .summary.accepted and .summary.rejected == (.summary.attempts - .summary.accepted)" "$parseability_report_json" >/dev/null; then
                                json_to_stimuli_status="pass"
                                parseability_status="pass"
                                parseability_attempts="$(parseability_summary_field_u64 "$parseability_report_json" "attempts")"
                                parseability_accepted="$(parseability_summary_field_u64 "$parseability_report_json" "accepted")"
                                parseability_rejected="$(parseability_summary_field_u64 "$parseability_report_json" "rejected")"
                                parseability_acceptance_rate="$(parseability_acceptance_rate_percent "$parseability_report_json")"
                                parseability_report_path="$parseability_report_json"
                                overall="pass"
                            else
                                json_to_stimuli_status="fail"
                                parseability_status="fail"
                                overall="fail"
                                failures=$((failures + 1))
                                notes="parseability report validation failed (see work/${grammar}_parseability_report.json)"
                            fi
                        else
                            json_to_stimuli_status="fail"
                            parseability_status="fail"
                            overall="fail"
                            failures=$((failures + 1))
                            notes="parseability-backed json_to_stimuli failed (see logs/${grammar}.json_to_stimuli.log)"
                        fi
                    else
                        parser_registry_support_status="fail"
                        parseability_status="fail"
                        overall="fail"
                        failures=$((failures + 1))
                        notes="parser_registry adapter unavailable (see logs/${grammar}.parseability_support.log)"
                    fi
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

    echo "${grammar},${grammar_file_status},${ebnf_to_json_status},${json_to_parser_status},${json_to_stimuli_status},${parser_registry_support_status},${parseability_status},${parseability_attempts},${parseability_accepted},${parseability_rejected},${parseability_acceptance_rate},${parseability_report_path},${overall},${notes}" >>"$SUMMARY_CSV"
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
