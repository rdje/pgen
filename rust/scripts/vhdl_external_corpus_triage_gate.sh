#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"

STATE_DIR="${PGEN_VHDL_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-$RUST_DIR/target/vhdl_external_corpus_triage_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"
REPORT_JSON="$WORK_DIR/vhdl_external_corpus_triage_report.json"
CASES_JSONL="$WORK_DIR/vhdl_external_corpus_triage_cases.jsonl"

MANIFEST_FILE="${PGEN_VHDL_EXTERNAL_CORPUS_TRIAGE_MANIFEST:-$RUST_DIR/test_data/grammar_quality/vhdl_external_corpus_triage_v0.json}"
MAX_CASES="${PGEN_VHDL_EXTERNAL_CORPUS_TRIAGE_MAX_CASES:-0}"

PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"
AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"
GRAMMAR_FILE="$ROOT_DIR/grammars/vhdl.ebnf"
GRAMMAR_JSON="$WORK_DIR/vhdl_external_corpus_triage_grammar.json"
PARSER_OUT="$WORK_DIR/vhdl_external_corpus_triage_parser.rs"

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

resolve_path() {
    local raw="$1"
    if [[ "$raw" == /* ]]; then
        printf '%s\n' "$raw"
    else
        printf '%s\n' "$ROOT_DIR/$raw"
    fi
}

now_ms() {
    perl -MTime::HiRes=time -e 'printf "%.0f\n", time()*1000'
}

file_size_bytes() {
    perl -e 'my $f = shift; my $s = -s $f; print defined($s) ? $s : 0;' "$1"
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
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
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
        echo "error: stage '$label' failed (log: $log_file)" >&2
        tail -n 120 "$log_file" >&2 || true
        exit 1
    fi
}

run_optional_logged() {
    local label="$1"
    shift
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok (${log_file})"
        return 0
    fi
    echo "    soft-fail (${log_file})"
    return 1
}

if ! [[ "$MAX_CASES" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_VHDL_EXTERNAL_CORPUS_TRIAGE_MAX_CASES must be an integer >= 0" >&2
    exit 2
fi

require_tool jq
require_tool perl
require_file "$MANIFEST_FILE"
require_file "$EBNF_TO_JSON"
require_file "$GRAMMAR_FILE"

mkdir -p "$STATE_DIR" "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"
: >"$CASES_JSONL"

manifest_version="$(jq -er '.version | numbers' "$MANIFEST_FILE")"
corpus_count="$(jq -er '[.cases[]?.corpus // "uncategorized"] | unique | length' "$MANIFEST_FILE")"

run_logged_rust "build_ast_pipeline_for_vhdl_external_corpus_triage" \
    cargo build --features generated_parsers --bin ast_pipeline

run_logged "frontend_ebnf_to_json" \
    perl "$EBNF_TO_JSON" --pretty --quiet "$GRAMMAR_FILE" -o "$GRAMMAR_JSON"
require_nonempty_file "$GRAMMAR_JSON"

run_logged "generate_vhdl_parser" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-parser \
    --eliminate-left-recursion \
    --output "$PARSER_OUT"
require_nonempty_file "$PARSER_OUT"

run_logged_rust "build_vhdl_external_corpus_triage_binaries" \
    env PGEN_VHDL_PARSER_PATH="$PARSER_OUT" \
    cargo build --features generated_parsers --bin ast_pipeline --bin parseability_probe

require_file "$PARSE_PROBE_BIN"

mapfile -t case_rows < <(jq -c '.cases[]?' "$MANIFEST_FILE")
cases_declared="${#case_rows[@]}"
if (( cases_declared == 0 )); then
    echo "error: manifest has zero cases: $MANIFEST_FILE" >&2
    exit 1
fi

cases_executed=0
parse_pass_total=0
parse_fail_total=0
sample_bytes_max=0
parse_total_ms=0
parse_max_ms=0
primary_parse_failure_case="<none>"
primary_parse_failure_corpus="<none>"

case_manifest_idx=0
for case_json in "${case_rows[@]}"; do
    if (( MAX_CASES > 0 && case_manifest_idx >= MAX_CASES )); then
        break
    fi
    case_manifest_idx=$((case_manifest_idx + 1))

    case_name="$(jq -er '.name | strings' <<<"$case_json")"
    case_corpus="$(jq -er '(.corpus // "uncategorized") | strings' <<<"$case_json")"
    case_source_rel="$(jq -er '.path | strings' <<<"$case_json")"
    case_source_path="$(resolve_path "$case_source_rel")"
    require_file "$case_source_path"

    case_name_key="$(printf '%s' "$case_name" | tr -c 'A-Za-z0-9_' '_')"
    case_parse_label="case_${case_name_key}_parse_full"
    case_parse_log="$LOG_DIR/${case_parse_label}.log"
    case_sample_bytes="$(file_size_bytes "$case_source_path")"
    if (( case_sample_bytes > sample_bytes_max )); then
        sample_bytes_max="$case_sample_bytes"
    fi

    case_parse_started_ms="$(now_ms)"
    if run_optional_logged "$case_parse_label" \
        "$PARSE_PROBE_BIN" --parse vhdl "$case_source_path"; then
        case_parse_status="pass"
        parse_pass_total=$((parse_pass_total + 1))
        case_status="pass"
        case_note="parse_full passed"
    else
        case_parse_status="fail"
        parse_fail_total=$((parse_fail_total + 1))
        case_status="parse_fail"
        case_note="parse_full rejected sample"
        if [[ "$primary_parse_failure_case" == "<none>" ]]; then
            primary_parse_failure_case="$case_name"
            primary_parse_failure_corpus="$case_corpus"
        fi
    fi
    case_parse_elapsed_ms=$(( $(now_ms) - case_parse_started_ms ))
    parse_total_ms=$((parse_total_ms + case_parse_elapsed_ms))
    if (( case_parse_elapsed_ms > parse_max_ms )); then
        parse_max_ms="$case_parse_elapsed_ms"
    fi

    cases_executed=$((cases_executed + 1))

    jq -n \
        --arg case_name "$case_name" \
        --arg corpus "$case_corpus" \
        --arg source_file "$case_source_path" \
        --arg parse_log_file "$case_parse_log" \
        --arg status "$case_status" \
        --arg note "$case_note" \
        --argjson sample_bytes "$case_sample_bytes" \
        --argjson parse_full_ms "$case_parse_elapsed_ms" \
        --arg parse_status "$case_parse_status" \
        '{
            case_name: $case_name,
            corpus: $corpus,
            source_file: $source_file,
            parse_log_file: $parse_log_file,
            status: $status,
            note: $note,
            observed: {
                parse_status: $parse_status,
                sample_bytes: $sample_bytes,
                parse_full_ms: $parse_full_ms
            }
        }' >>"$CASES_JSONL"
done

if (( cases_executed == 0 )); then
    echo "error: no VHDL external triage cases executed" >&2
    exit 1
fi

cases_json="$(jq -s '.' "$CASES_JSONL")"
generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

jq -n \
    --arg gate "vhdl_external_corpus_triage_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg manifest_file "$MANIFEST_FILE" \
    --argjson manifest_version "$manifest_version" \
    --argjson corpus_count "$corpus_count" \
    --argjson cases_declared "$cases_declared" \
    --argjson cases_executed "$cases_executed" \
    --argjson parse_pass_total "$parse_pass_total" \
    --argjson parse_fail_total "$parse_fail_total" \
    --argjson sample_bytes_max "$sample_bytes_max" \
    --argjson parse_total_ms "$parse_total_ms" \
    --argjson parse_max_ms "$parse_max_ms" \
    --arg primary_parse_failure_case "$primary_parse_failure_case" \
    --arg primary_parse_failure_corpus "$primary_parse_failure_corpus" \
    --argjson cases "$cases_json" \
    '{
        gate: $gate,
        version: $version,
        generated_at_utc: $generated_at_utc,
        manifest_file: $manifest_file,
        manifest_version: $manifest_version,
        totals: {
            corpus_count: $corpus_count,
            cases_declared: $cases_declared,
            cases_executed: $cases_executed,
            parse_pass_total: $parse_pass_total,
            parse_fail_total: $parse_fail_total,
            sample_bytes_max: $sample_bytes_max,
            parse_total_ms: $parse_total_ms,
            parse_max_ms: $parse_max_ms
        },
        primary_parse_failure: {
            case_name: $primary_parse_failure_case,
            corpus: $primary_parse_failure_corpus
        },
        cases: $cases
    }' >"$REPORT_JSON"

cp "$REPORT_JSON" "$SUMMARY_JSON"

{
    echo "VHDL External Corpus Triage Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "manifest_file: $MANIFEST_FILE"
    echo "manifest_version: $manifest_version"
    echo "corpus_count: $corpus_count"
    echo "cases_declared: $cases_declared"
    echo "cases_executed: $cases_executed"
    echo "parse_pass_total: $parse_pass_total"
    echo "parse_fail_total: $parse_fail_total"
    echo "sample_bytes_max: $sample_bytes_max"
    echo "parse_total_ms: $parse_total_ms"
    echo "parse_max_ms: $parse_max_ms"
    echo "primary_parse_failure_case: $primary_parse_failure_case"
    echo "primary_parse_failure_corpus: $primary_parse_failure_corpus"
    echo "report_json: $REPORT_JSON"
    echo "summary_json: $SUMMARY_JSON"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"
