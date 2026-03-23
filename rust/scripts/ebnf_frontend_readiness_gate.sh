#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"
GRAMMARS_DIR="$ROOT_DIR/grammars"

STATE_DIR="${PGEN_EBNF_FRONTEND_STATE_DIR:-$RUST_DIR/target/ebnf_frontend_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

STRICT="${PGEN_EBNF_FRONTEND_STRICT:-0}"
STIMULI_COUNT="${PGEN_EBNF_FRONTEND_STIMULI_COUNT:-8}"
STIMULI_SEED="${PGEN_EBNF_FRONTEND_STIMULI_SEED:-1337}"
FRONTEND_IMPL="${PGEN_EBNF_FRONTEND_IMPL:-rust}"

GRAMMARS=("ebnf" "json" "regex")
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"

if ! [[ "$STRICT" =~ ^[01]$ ]]; then
    echo "error: PGEN_EBNF_FRONTEND_STRICT must be 0 or 1" >&2
    exit 2
fi

if ! [[ "$STIMULI_COUNT" =~ ^[0-9]+$ ]] || [[ "$STIMULI_COUNT" -lt 1 ]]; then
    echo "error: PGEN_EBNF_FRONTEND_STIMULI_COUNT must be an integer >= 1" >&2
    exit 2
fi
if [[ "$FRONTEND_IMPL" != "perl" && "$FRONTEND_IMPL" != "rust" ]]; then
    echo "error: PGEN_EBNF_FRONTEND_IMPL must be 'perl' or 'rust'" >&2
    exit 2
fi

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
    awk -v accepted="$accepted" -v attempts="$attempts" 'BEGIN { if (attempts == 0) { printf "0.00" } else { printf "%.2f", (accepted * 100.0) / attempts } }'
}

build_frontend_binaries() {
    (
        cd "$RUST_DIR"
        env "$@" cargo build \
            --features "generated_parsers ebnf_dual_run" \
            --bin ast_pipeline \
            --bin parseability_probe \
            >/dev/null
    )
}

parseability_build_env_for_grammar() {
    local grammar="$1"
    local parser_path="$2"
    case "$grammar" in
        ebnf)
            printf 'PGEN_EBNF_PARSER_PATH=%s\n' "$parser_path"
            ;;
        json)
            printf 'PGEN_JSON_PARSER_PATH=%s\n' "$parser_path"
            ;;
        regex)
            printf 'PGEN_REGEX_PARSER_PATH=%s\n' "$parser_path"
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

echo "grammar,frontend_to_json,json_to_parser,json_to_stimuli,parser_registry_support,parseability,parseability_attempts,parseability_accepted,parseability_rejected,parseability_acceptance_rate_percent,parseability_report_json,overall,notes" >"$SUMMARY_CSV"
{
    echo "PGEN EBNF Frontend Readiness Summary"
    echo "state_dir: $STATE_DIR"
    echo "strict_mode: $STRICT"
    echo "frontend_impl: $FRONTEND_IMPL"
    echo "stimuli_count: $STIMULI_COUNT"
    echo "stimuli_seed: $STIMULI_SEED"
    echo
} >"$SUMMARY_TXT"

failures=0

run_frontend_to_json() {
    local grammar_file="$1"
    local json_out="$2"
    if [[ "$FRONTEND_IMPL" == "perl" ]]; then
        perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$grammar_file" -o "$json_out"
    else
        "$AST_PIPELINE_BIN" "$grammar_file" --emit-raw-ast-json "$json_out"
    fi
}

for grammar in "${GRAMMARS[@]}"; do
    grammar_file="$GRAMMARS_DIR/${grammar}.ebnf"
    json_out="$WORK_DIR/${grammar}.json"
    parser_out="$WORK_DIR/${grammar}_parser.rs"
    stimuli_out="$WORK_DIR/${grammar}_stimuli.txt"

    frontend_to_json_status="skip"
    json_to_parser_status="skip"
    json_to_stimuli_status="skip"
    parser_registry_support_status="skip"
    parseability_status="skip"
    parseability_attempts="0"
    parseability_accepted="0"
    parseability_rejected="0"
    parseability_acceptance_rate="0.00"
    parseability_report_path="n/a"
    notes="ok"

    frontend_log="$LOG_DIR/${grammar}.frontend_to_json.log"
    parser_log="$LOG_DIR/${grammar}.json_to_parser.log"
    stimuli_log="$LOG_DIR/${grammar}.json_to_stimuli.log"
    probe_build_log="$LOG_DIR/${grammar}.parseability_probe_build.log"
    probe_support_log="$LOG_DIR/${grammar}.parseability_support.log"
    parseability_report_json="$WORK_DIR/${grammar}_parseability_report.json"

    if run_frontend_to_json "$grammar_file" "$json_out" >"$frontend_log" 2>&1; then
        frontend_to_json_status="pass"
        if "$AST_PIPELINE_BIN" "$json_out" --generate-parser --eliminate-left-recursion -o "$parser_out" >"$parser_log" 2>&1; then
            json_to_parser_status="pass"
            build_env_line="$(parseability_build_env_for_grammar "$grammar" "$parser_out")"
            echo "==> Rebuilding ast_pipeline and parseability_probe for ${grammar} parseability"
            if build_frontend_binaries "$build_env_line" >"$probe_build_log" 2>&1; then
                :
            else
                parser_registry_support_status="fail"
                parseability_status="fail"
                failures=$((failures + 1))
                notes="failed to rebuild parseability binaries (see logs/${grammar}.parseability_probe_build.log)"
            fi

            if [[ "$notes" == "ok" ]]; then
                if "$PARSE_PROBE_BIN" --supports "$grammar" >"$probe_support_log" 2>&1; then
                    parser_registry_support_status="pass"
                    if "$AST_PIPELINE_BIN" "$json_out" --generate-stimuli --count "$STIMULI_COUNT" --seed "$STIMULI_SEED" --validate-parseability --parseability-report-json "$parseability_report_json" --output "$stimuli_out" >"$stimuli_log" 2>&1; then
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
                        else
                            json_to_stimuli_status="fail"
                            parseability_status="fail"
                            failures=$((failures + 1))
                            notes="parseability report validation failed (see work/${grammar}_parseability_report.json)"
                        fi
                    else
                        json_to_stimuli_status="fail"
                        parseability_status="fail"
                        failures=$((failures + 1))
                        notes="parseability-backed json_to_stimuli failed (see logs/${grammar}.json_to_stimuli.log)"
                    fi
                elif grep -q "adapter unavailable for grammar" "$probe_support_log"; then
                    parser_registry_support_status="unavailable"
                    parseability_status="skip"
                    if "$AST_PIPELINE_BIN" "$json_out" --generate-stimuli --count "$STIMULI_COUNT" --seed "$STIMULI_SEED" --output "$stimuli_out" >"$stimuli_log" 2>&1 && [[ -s "$stimuli_out" ]]; then
                        json_to_stimuli_status="pass"
                        notes="ok (parser-backed parseability unavailable for ${grammar})"
                    else
                        json_to_stimuli_status="fail"
                        failures=$((failures + 1))
                        notes="json_to_stimuli failed (see logs/${grammar}.json_to_stimuli.log)"
                    fi
                else
                    parser_registry_support_status="fail"
                    parseability_status="fail"
                    failures=$((failures + 1))
                    notes="parser_registry support probe failed (see logs/${grammar}.parseability_support.log)"
                fi
            fi
        else
            json_to_parser_status="fail"
            failures=$((failures + 1))
            notes="json_to_parser failed (see logs/${grammar}.json_to_parser.log)"
        fi
    else
        frontend_to_json_status="fail"
        failures=$((failures + 1))
        notes="frontend_to_json failed (see logs/${grammar}.frontend_to_json.log)"
    fi

    overall="pass"
    if [[ "$frontend_to_json_status" == "fail" || "$json_to_parser_status" == "fail" || "$json_to_stimuli_status" == "fail" || "$parser_registry_support_status" == "fail" || "$parseability_status" == "fail" ]]; then
        overall="fail"
    fi

    echo "${grammar},${frontend_to_json_status},${json_to_parser_status},${json_to_stimuli_status},${parser_registry_support_status},${parseability_status},${parseability_attempts},${parseability_accepted},${parseability_rejected},${parseability_acceptance_rate},${parseability_report_path},${overall},${notes}" >>"$SUMMARY_CSV"
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
    echo "⚠️  EBNF frontend readiness has $failures failing grammar flow(s)." >&2
    if [[ "$STRICT" -eq 1 ]]; then
        echo "❌ strict mode enabled: failing." >&2
        exit 1
    fi
    echo "ℹ️  strict mode disabled: reporting only." >&2
else
    echo "✅ EBNF frontend readiness check passed for all tracked grammars."
fi
