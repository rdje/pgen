#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
BUNDLE_DIR="$ROOT_DIR/regex_corpus_bundle"

STATE_DIR="${PGEN_REGEX_PCRE2_TEXTSAFE_CORPUS_STATE_DIR:-$RUST_DIR/target/regex_pcre2_textsafe_corpus_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"
FAILURES_JSONL="$STATE_DIR/failures.jsonl"

NORMALIZED_JSONL="$WORK_DIR/pcre2_textsafe_cases.jsonl"
NORMALIZED_SUMMARY_JSON="$WORK_DIR/pcre2_textsafe_summary.json"
NORMALIZED_SKIP_JSONL="$WORK_DIR/pcre2_textsafe_skips.jsonl"

NORMALIZER="$BUNDLE_DIR/scripts/normalize_pcre2_testdata.py"
LOCKFILE="$BUNDLE_DIR/manifests/upstreams.lock.json"
UPSTREAM_ROOT="$BUNDLE_DIR/third_party/upstream/pcre2"
PROBE_BIN="$RUST_DIR/target/debug/regex_corpus_probe"

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

pcre2_ref="$(jq -er '.upstreams[] | select(.name == "pcre2") | .ref' "$LOCKFILE")"
require_dir "$UPSTREAM_ROOT/$pcre2_ref"

run_logged "normalize_pcre2_testdata" \
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
    "$FAILURES_JSONL"

normalize_cases_emitted="$(jq -er '.cases_emitted | numbers' "$NORMALIZED_SUMMARY_JSON")"
normalize_pattern_specs_detected="$(jq -er '.pattern_specs_detected | numbers' "$NORMALIZED_SUMMARY_JSON")"
normalize_utf8_skips="$(jq -er '.skip_reason_counts.utf8_decode_failure // 0' "$NORMALIZED_SUMMARY_JSON")"
normalize_unsupported_skips="$(jq -er '.skip_reason_counts.unsupported_suffix_token // 0' "$NORMALIZED_SUMMARY_JSON")"

cases_executed="$(jq -er '.cases_executed | numbers' "$SUMMARY_JSON")"
parse_pass_total="$(jq -er '.parse_pass_total | numbers' "$SUMMARY_JSON")"
parse_fail_total="$(jq -er '.parse_fail_total | numbers' "$SUMMARY_JSON")"
acceptance_rate_percent="$(jq -er '.acceptance_rate_percent | strings' "$SUMMARY_JSON")"
primary_failure_case="$(jq -er '.primary_failure_case | strings' "$SUMMARY_JSON")"
primary_failure_parser_error="$(jq -er '.primary_failure_parser_error | strings' "$SUMMARY_JSON")"
unique_parser_error_count="$(jq -er '.unique_parser_error_count | numbers' "$SUMMARY_JSON")"

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cat >"$SUMMARY_TXT" <<EOF
gate: regex_pcre2_textsafe_corpus_gate
version: 1
generated_at_utc: $generated_at_utc
pcre2_ref: $pcre2_ref
normalized_cases_emitted: $normalize_cases_emitted
normalized_pattern_specs_detected: $normalize_pattern_specs_detected
normalized_utf8_decode_skips: $normalize_utf8_skips
normalized_unsupported_suffix_skips: $normalize_unsupported_skips
cases_executed: $cases_executed
parse_pass_total: $parse_pass_total
parse_fail_total: $parse_fail_total
acceptance_rate_percent: $acceptance_rate_percent
unique_parser_error_count: $unique_parser_error_count
primary_failure_case: $primary_failure_case
primary_failure_parser_error: $primary_failure_parser_error
normalized_jsonl: $NORMALIZED_JSONL
normalized_summary_json: $NORMALIZED_SUMMARY_JSON
normalized_skip_jsonl: $NORMALIZED_SKIP_JSONL
failures_jsonl: $FAILURES_JSONL
EOF
