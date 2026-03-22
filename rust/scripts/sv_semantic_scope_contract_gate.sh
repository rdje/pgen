#!/usr/bin/env bash
set -euo pipefail

RUST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_DIR="$(cd "$RUST_DIR/.." && pwd)"

STATE_DIR="${PGEN_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR:-$RUST_DIR/target/sv_semantic_scope_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUITE_FILE="${PGEN_SV_SEMANTIC_SCOPE_CONTRACT_FILE:-$RUST_DIR/test_data/grammar_quality/systemverilog_semantic_scope_contract_cases.json}"
GRAMMAR_FILE="$ROOT_DIR/grammars/systemverilog.ebnf"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"
PARSER_OUT="$WORK_DIR/systemverilog_semantic_scope_contract_parser.rs"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

mkdir -p "$WORK_DIR" "$LOG_DIR"

require_file() {
    local path="$1"
    if [[ ! -f "$path" ]]; then
        echo "error: required file not found: $path" >&2
        exit 1
    fi
}

require_nonempty_file() {
    local path="$1"
    require_file "$path"
    if [[ ! -s "$path" ]]; then
        echo "error: required file is empty: $path" >&2
        exit 1
    fi
}

csv_sanitize() {
    local text="$1"
    text="${text//$'\r'/ }"
    text="${text//$'\n'/ }"
    text="${text//,/;}"
    printf '%s' "$text"
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

require_file "$SUITE_FILE"
require_file "$GRAMMAR_FILE"

suite_version="$(jq -er '.version' "$SUITE_FILE")"
grammar_name="$(jq -er '.grammar_name | strings' "$SUITE_FILE")"
default_profile="$(jq -er '.default_profile | strings' "$SUITE_FILE")"
case_count="$(jq -er '.cases | length' "$SUITE_FILE")"

if [[ "$grammar_name" != "systemverilog" ]]; then
    echo "error: this gate only supports grammar_name=systemverilog (got '$grammar_name')" >&2
    exit 2
fi
if [[ "$case_count" -le 0 ]]; then
    echo "error: semantic scope contract suite has zero cases: $SUITE_FILE" >&2
    exit 2
fi

echo "==> SystemVerilog semantic-scope contract gate"
echo "state_dir: $STATE_DIR"
echo "suite_file: $SUITE_FILE"
echo "suite_version: $suite_version"
echo "grammar_file: $GRAMMAR_FILE"
echo "default_profile: $default_profile"
echo "case_count: $case_count"

run_logged_rust "build_ast_pipeline_with_ebnf_frontend" \
    cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline
require_file "$AST_PIPELINE_BIN"

run_logged "generate_systemverilog_parser" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_FILE" \
    --generate-parser \
    --eliminate-left-recursion \
    --output "$PARSER_OUT"
require_nonempty_file "$PARSER_OUT"

run_logged_rust "build_parseability_probe_with_contract_parser" \
    env PGEN_SYSTEMVERILOG_PARSER_PATH="$PARSER_OUT" \
    cargo build --features generated_parsers --bin parseability_probe
require_file "$PARSE_PROBE_BIN"

run_logged "probe_systemverilog_support" \
    "$PARSE_PROBE_BIN" --supports "$grammar_name"

echo "case,profile,expect,actual,status,notes" >"$SUMMARY_CSV"

total_cases=0
passed_cases=0
failed_cases=0

while IFS= read -r case_json; do
    total_cases=$((total_cases + 1))

    case_name="$(jq -er '.name | strings' <<<"$case_json")"
    case_expect_pass="$(jq -er 'if (.expect_pass // false) then 1 else 0 end' <<<"$case_json")"
    case_profile="$(jq -er --arg default_profile "$default_profile" '(.profile // $default_profile) | strings' <<<"$case_json")"
    case_input="$(jq -er '.input | strings' <<<"$case_json")"
    case_file="$WORK_DIR/${total_cases}_$(printf '%s' "$case_name" | tr -c 'A-Za-z0-9_-' '_').sv"
    case_log="$LOG_DIR/case_${total_cases}_$(printf '%s' "$case_name" | tr -c 'A-Za-z0-9_-' '_').log"

    printf '%s\n' "$case_input" >"$case_file"

    if "$PARSE_PROBE_BIN" --parse "$grammar_name" "$case_file" --profile "$case_profile" >"$case_log" 2>&1; then
        case_actual_pass=1
        case_note="parser accepted"
    else
        case_actual_pass=0
        case_note="$(tail -n 1 "$case_log" 2>/dev/null || true)"
        if [[ -z "$case_note" ]]; then
            case_note="parser rejected"
        fi
    fi

    if [[ "$case_expect_pass" -eq 1 ]]; then
        case_expected_label="pass"
    else
        case_expected_label="fail"
    fi
    if [[ "$case_actual_pass" -eq 1 ]]; then
        case_actual_label="pass"
    else
        case_actual_label="fail"
    fi

    if [[ "$case_expect_pass" -eq "$case_actual_pass" ]]; then
        case_status="pass"
        passed_cases=$((passed_cases + 1))
    else
        case_status="fail"
        failed_cases=$((failed_cases + 1))
        echo "semantic scope contract mismatch: case='${case_name}' profile='${case_profile}' expected=${case_expected_label} actual=${case_actual_label}" >&2
        tail -n 40 "$case_log" >&2 || true
    fi

    echo "${case_name},${case_profile},${case_expected_label},${case_actual_label},${case_status},$(csv_sanitize "$case_note")" >>"$SUMMARY_CSV"
done < <(jq -c '.cases[]' "$SUITE_FILE")

cat >"$SUMMARY_TXT" <<EOF
systemverilog_semantic_scope_contract_version=1
suite_file=$SUITE_FILE
grammar_file=$GRAMMAR_FILE
parser_out=$PARSER_OUT
default_profile=$default_profile
case_count=$total_cases
passed_count=$passed_cases
failed_count=$failed_cases
summary_csv=$SUMMARY_CSV
EOF

jq -n \
    --arg suite_file "$SUITE_FILE" \
    --arg grammar_file "$GRAMMAR_FILE" \
    --arg parser_out "$PARSER_OUT" \
    --arg default_profile "$default_profile" \
    --argjson case_count "$total_cases" \
    --argjson passed_count "$passed_cases" \
    --argjson failed_count "$failed_cases" \
    '{
        pgen_contract_version: 1,
        kind: "systemverilog_semantic_scope_contract_summary",
        suite_file: $suite_file,
        grammar_file: $grammar_file,
        parser_out: $parser_out,
        default_profile: $default_profile,
        case_count: $case_count,
        passed_count: $passed_count,
        failed_count: $failed_count
    }' >"$SUMMARY_JSON"

if (( failed_cases > 0 )); then
    echo "semantic scope contract suite failed: $failed_cases/$total_cases mismatches (summary: $SUMMARY_CSV)" >&2
    exit 1
fi

echo "==> semantic scope contract suite passed (${passed_cases}/${total_cases})"
echo "summary_txt: $SUMMARY_TXT"
echo "summary_json: $SUMMARY_JSON"
echo "summary_csv: $SUMMARY_CSV"
