#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
BUNDLE_DIR="$ROOT_DIR/regex_corpus_bundle"

STATE_DIR="${PGEN_REGEX_PCRE2_COMPILE_ORACLE_STATE_DIR:-$RUST_DIR/target/regex_pcre2_compile_oracle_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"
OBSERVATIONS_JSONL="$STATE_DIR/observations.jsonl"

NORMALIZED_JSONL="$WORK_DIR/pcre2_compile_oracle_cases.jsonl"
NORMALIZED_SUMMARY_JSON="$WORK_DIR/pcre2_compile_oracle_summary.json"
NORMALIZED_SKIP_JSONL="$WORK_DIR/pcre2_compile_oracle_skips.jsonl"

NORMALIZER="$BUNDLE_DIR/scripts/normalize_pcre2_compile_oracle.py"
LOCKFILE="$BUNDLE_DIR/manifests/upstreams.lock.json"
UPSTREAM_ROOT="$BUNDLE_DIR/third_party/upstream/pcre2"
PROBE_BIN="$RUST_DIR/target/debug/regex_corpus_probe"
BASELINE_ENV="${PGEN_REGEX_PCRE2_COMPILE_ORACLE_BASELINE_ENV:-$RUST_DIR/test_data/grammar_quality/regex_pcre2_compile_oracle_lightweight_v0.env}"

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

require_dir() {
    local path="$1"
    if [[ ! -d "$path" ]]; then
        echo "error: missing required directory '$path'" >&2
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

require_tool jq
require_tool python3

mkdir -p "$STATE_DIR" "$WORK_DIR" "$LOG_DIR"
: >"$SUMMARY_TXT"

require_file "$NORMALIZER"
require_file "$LOCKFILE"
require_dir "$UPSTREAM_ROOT"
require_file "$BASELINE_ENV"
source "$BASELINE_ENV"

pcre2_ref="$(jq -er '.upstreams[] | select(.name == "pcre2") | .ref' "$LOCKFILE")"
require_dir "$UPSTREAM_ROOT/$pcre2_ref"

run_logged "normalize_pcre2_compile_oracle" \
    python3 "$NORMALIZER" \
        --output-jsonl "$NORMALIZED_JSONL" \
        --summary-json "$NORMALIZED_SUMMARY_JSON" \
        --skip-jsonl "$NORMALIZED_SKIP_JSONL"

run_logged_rust "build_regex_corpus_probe" \
    cargo build --features generated_parsers --bin regex_corpus_probe

require_file "$PROBE_BIN"

run_logged "run_regex_corpus_probe" \
    "$PROBE_BIN" \
    "$NORMALIZED_JSONL" \
    "$SUMMARY_JSON" \
    "$OBSERVATIONS_JSONL"

normalize_cases_emitted="$(jq -er '.cases_emitted | numbers' "$NORMALIZED_SUMMARY_JSON")"
expected_parse_ok_total="$(jq -er '.expected_parse_counts.ok // 0' "$NORMALIZED_SUMMARY_JSON")"
expected_parse_fail_total="$(jq -er '.expected_parse_counts.fail // 0' "$NORMALIZED_SUMMARY_JSON")"
normalize_utf8_skips="$(jq -er '.skip_reason_counts.utf8_decode_failure // 0' "$NORMALIZED_SUMMARY_JSON")"
normalize_unsupported_skips="$(jq -er '.skip_reason_counts.unsupported_suffix_token // 0' "$NORMALIZED_SUMMARY_JSON")"

cases_executed="$(jq -er '.cases_executed | numbers' "$SUMMARY_JSON")"
parse_pass_total="$(jq -er '.parse_pass_total | numbers' "$SUMMARY_JSON")"
parse_fail_total="$(jq -er '.parse_fail_total | numbers' "$SUMMARY_JSON")"
match_total="$(jq -er '.parse_expectation_match_total | numbers' "$SUMMARY_JSON")"
mismatch_total="$(jq -er '.parse_expectation_mismatch_total | numbers' "$SUMMARY_JSON")"
false_accept_total="$(jq -er '.false_accept_total | numbers' "$SUMMARY_JSON")"
false_reject_total="$(jq -er '.false_reject_total | numbers' "$SUMMARY_JSON")"
primary_mismatch_case="$(jq -er '.primary_mismatch_case | strings' "$SUMMARY_JSON")"
primary_mismatch_expected_parse="$(jq -er '.primary_mismatch_expected_parse | strings' "$SUMMARY_JSON")"
primary_mismatch_actual_parse="$(jq -er '.primary_mismatch_actual_parse | strings' "$SUMMARY_JSON")"
primary_mismatch_parser_error="$(jq -er '.primary_mismatch_parser_error // "<none>"' "$SUMMARY_JSON")"

[[ "$pcre2_ref" == "$PGEN_REGEX_PCRE2_COMPILE_ORACLE_REF" ]] || {
    echo "error: expected PCRE2 ref '$PGEN_REGEX_PCRE2_COMPILE_ORACLE_REF' but found '$pcre2_ref'" >&2
    exit 1
}

[[ "$normalize_cases_emitted" -eq "$PGEN_REGEX_PCRE2_COMPILE_ORACLE_EXPECTED_CASES" ]] || {
    echo "error: expected $PGEN_REGEX_PCRE2_COMPILE_ORACLE_EXPECTED_CASES oracle cases but observed $normalize_cases_emitted" >&2
    exit 1
}
[[ "$cases_executed" -eq "$PGEN_REGEX_PCRE2_COMPILE_ORACLE_EXPECTED_CASES" ]] || {
    echo "error: expected $PGEN_REGEX_PCRE2_COMPILE_ORACLE_EXPECTED_CASES executed cases but observed $cases_executed" >&2
    exit 1
}
[[ "$expected_parse_ok_total" -eq "$PGEN_REGEX_PCRE2_COMPILE_ORACLE_EXPECTED_PARSE_OK_TOTAL" ]] || {
    echo "error: expected $PGEN_REGEX_PCRE2_COMPILE_ORACLE_EXPECTED_PARSE_OK_TOTAL compile-ok cases but observed $expected_parse_ok_total" >&2
    exit 1
}
[[ "$expected_parse_fail_total" -eq "$PGEN_REGEX_PCRE2_COMPILE_ORACLE_EXPECTED_PARSE_FAIL_TOTAL" ]] || {
    echo "error: expected $PGEN_REGEX_PCRE2_COMPILE_ORACLE_EXPECTED_PARSE_FAIL_TOTAL compile-fail cases but observed $expected_parse_fail_total" >&2
    exit 1
}
[[ "$match_total" -ge "$PGEN_REGEX_PCRE2_COMPILE_ORACLE_MIN_MATCH_TOTAL" ]] || {
    echo "error: expected at least $PGEN_REGEX_PCRE2_COMPILE_ORACLE_MIN_MATCH_TOTAL oracle matches but observed $match_total" >&2
    exit 1
}
[[ "$mismatch_total" -le "$PGEN_REGEX_PCRE2_COMPILE_ORACLE_MAX_MISMATCH_TOTAL" ]] || {
    echo "error: expected at most $PGEN_REGEX_PCRE2_COMPILE_ORACLE_MAX_MISMATCH_TOTAL oracle mismatches but observed $mismatch_total" >&2
    exit 1
}
[[ "$false_accept_total" -le "$PGEN_REGEX_PCRE2_COMPILE_ORACLE_MAX_FALSE_ACCEPT_TOTAL" ]] || {
    echo "error: expected at most $PGEN_REGEX_PCRE2_COMPILE_ORACLE_MAX_FALSE_ACCEPT_TOTAL false accepts but observed $false_accept_total" >&2
    exit 1
}
[[ "$false_reject_total" -le "$PGEN_REGEX_PCRE2_COMPILE_ORACLE_MAX_FALSE_REJECT_TOTAL" ]] || {
    echo "error: expected at most $PGEN_REGEX_PCRE2_COMPILE_ORACLE_MAX_FALSE_REJECT_TOTAL false rejects but observed $false_reject_total" >&2
    exit 1
}

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cat >"$SUMMARY_TXT" <<EOF
gate: regex_pcre2_compile_oracle_gate
version: 1
generated_at_utc: $generated_at_utc
pcre2_ref: $pcre2_ref
baseline_env: $BASELINE_ENV
normalized_cases_emitted: $normalize_cases_emitted
normalized_utf8_decode_skips: $normalize_utf8_skips
normalized_unsupported_suffix_skips: $normalize_unsupported_skips
expected_parse_ok_total: $expected_parse_ok_total
expected_parse_fail_total: $expected_parse_fail_total
cases_executed: $cases_executed
parse_pass_total: $parse_pass_total
parse_fail_total: $parse_fail_total
parse_expectation_match_total: $match_total
parse_expectation_mismatch_total: $mismatch_total
false_accept_total: $false_accept_total
false_reject_total: $false_reject_total
primary_mismatch_case: $primary_mismatch_case
primary_mismatch_expected_parse: $primary_mismatch_expected_parse
primary_mismatch_actual_parse: $primary_mismatch_actual_parse
primary_mismatch_parser_error: $primary_mismatch_parser_error
normalized_jsonl: $NORMALIZED_JSONL
normalized_summary_json: $NORMALIZED_SUMMARY_JSON
normalized_skip_jsonl: $NORMALIZED_SKIP_JSONL
observations_jsonl: $OBSERVATIONS_JSONL
EOF
