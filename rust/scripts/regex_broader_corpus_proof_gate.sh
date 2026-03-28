#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_REGEX_BROADER_CORPUS_PROOF_STATE_DIR:-$RUST_DIR/target/regex_broader_corpus_proof_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
CASE_INPUT_DIR="$WORK_DIR/cases"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

MANIFEST_FILE="${PGEN_REGEX_BROADER_CORPUS_PROOF_MANIFEST:-$RUST_DIR/test_data/grammar_quality/regex_broader_corpus_v0.json}"
MAX_CASES="${PGEN_REGEX_BROADER_CORPUS_PROOF_MAX_CASES:-0}"

PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"
GENERATED_REGEX_PARSER="$ROOT_DIR/generated/regex_parser.rs"
CASES_JSONL="$WORK_DIR/regex_broader_corpus_cases.jsonl"

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
    echo "error: PGEN_REGEX_BROADER_CORPUS_PROOF_MAX_CASES must be an integer >= 0" >&2
    exit 2
fi

require_tool jq
require_tool perl
require_file "$MANIFEST_FILE"
require_file "$GENERATED_REGEX_PARSER"

jq -e '
    ((.version | type) == "number")
    and ((.description | type) == "string" and (.description | length) > 0)
    and ((.source_file | type) == "string" and (.source_file | length) > 0)
    and (.selection == "all_cases")
    and ((.expected_case_count | type) == "number")
    and (.expected_parser_type == "regex")
    and (.expected_normalizer == "text")
' "$MANIFEST_FILE" >/dev/null

mkdir -p "$STATE_DIR" "$WORK_DIR" "$LOG_DIR" "$CASE_INPUT_DIR"
: >"$SUMMARY_TXT"
: >"$CASES_JSONL"

manifest_version="$(jq -er '.version | numbers' "$MANIFEST_FILE")"
source_file_rel="$(jq -er '.source_file | strings' "$MANIFEST_FILE")"
source_file="$(resolve_path "$source_file_rel")"
expected_case_count="$(jq -er '.expected_case_count | numbers' "$MANIFEST_FILE")"
expected_parser_type="$(jq -er '.expected_parser_type | strings' "$MANIFEST_FILE")"
expected_normalizer="$(jq -er '.expected_normalizer | strings' "$MANIFEST_FILE")"

require_nonempty_file "$source_file"

jq -e \
    --arg expected_parser_type "$expected_parser_type" \
    --arg expected_normalizer "$expected_normalizer" \
    --argjson expected_case_count "$expected_case_count" \
    '
    type == "array"
    and length == $expected_case_count
    and (map(.name) | length == (unique | length))
    and all(
        .[];
        (.name | type) == "string"
        and (.name | length) > 0
        and (.description | type) == "string"
        and (.input | type) == "string"
        and (.expected_round_trip | type) == "string"
        and (.parser_type == $expected_parser_type)
        and (.normalizer == $expected_normalizer)
        and ((.tags // []) | type) == "array"
    )
    ' "$source_file" >/dev/null

source_case_count="$(jq -er 'length' "$source_file")"
source_unique_tag_count="$(jq -er '[.[].tags[]?] | unique | length' "$source_file")"

run_logged_rust "build_parseability_probe_for_regex_broader_corpus" \
    cargo build --features generated_parsers --bin parseability_probe

require_file "$PARSE_PROBE_BIN"

run_logged "parseability_probe_supports_regex" \
    "$PARSE_PROBE_BIN" --supports regex

mapfile -t case_rows < <(jq -c '.[]' "$source_file")
cases_declared="${#case_rows[@]}"
if (( cases_declared == 0 )); then
    echo "error: broader regex corpus source has zero cases: $source_file" >&2
    exit 1
fi

cases_executed=0
parse_pass_total=0
parse_fail_total=0
sample_bytes_max=0
parse_total_ms=0
parse_max_ms=0
primary_parse_failure_case="<none>"
primary_parse_failure_parser_error="<none>"

case_manifest_idx=0
for case_json in "${case_rows[@]}"; do
    if (( MAX_CASES > 0 && case_manifest_idx >= MAX_CASES )); then
        break
    fi
    case_manifest_idx=$((case_manifest_idx + 1))

    case_name="$(jq -er '.name | strings' <<<"$case_json")"
    case_description="$(jq -er '.description | strings' <<<"$case_json")"
    case_input="$(jq -er '.input | strings' <<<"$case_json")"
    case_expected_round_trip="$(jq -er '.expected_round_trip | strings' <<<"$case_json")"
    case_tags_json="$(jq -c '(.tags // [])' <<<"$case_json")"
    case_round_trip_matches_input=false
    if [[ "$case_input" == "$case_expected_round_trip" ]]; then
        case_round_trip_matches_input=true
    fi

    case_name_key="$(printf '%s' "$case_name" | tr -c 'A-Za-z0-9_' '_')"
    case_input_file="$CASE_INPUT_DIR/${case_name_key}.regex"
    printf '%s' "$case_input" >"$case_input_file"

    case_parse_label="case_${case_name_key}_parse_full"
    case_parse_log="$LOG_DIR/${case_parse_label}.log"
    case_sample_bytes="$(file_size_bytes "$case_input_file")"
    if (( case_sample_bytes > sample_bytes_max )); then
        sample_bytes_max="$case_sample_bytes"
    fi

    case_parse_started_ms="$(now_ms)"
    case_parser_error=""
    if run_optional_logged "$case_parse_label" \
        "$PARSE_PROBE_BIN" --parse regex "$case_input_file"; then
        case_parse_status="pass"
        parse_pass_total=$((parse_pass_total + 1))
        case_status="pass"
        case_note="parse_full passed"
    else
        case_parse_status="fail"
        parse_fail_total=$((parse_fail_total + 1))
        case_status="parse_fail"
        case_note="parse_full rejected sample"
        case_parser_error="$(sed -n "s/^Error: parse_full rejected sample for grammar 'regex' on '.*': //p" "$case_parse_log" | head -n 1)"
        if [[ -z "$case_parser_error" ]]; then
            case_parser_error="parseability_probe failure"
        fi
        if [[ "$primary_parse_failure_case" == "<none>" ]]; then
            primary_parse_failure_case="$case_name"
            primary_parse_failure_parser_error="$case_parser_error"
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
        --arg description "$case_description" \
        --arg input "$case_input" \
        --arg expected_round_trip "$case_expected_round_trip" \
        --argjson tags "$case_tags_json" \
        --arg source_file "$case_input_file" \
        --arg parse_log_file "$case_parse_log" \
        --arg status "$case_status" \
        --arg note "$case_note" \
        --arg parser_error "$case_parser_error" \
        --argjson sample_bytes "$case_sample_bytes" \
        --argjson parse_full_ms "$case_parse_elapsed_ms" \
        --argjson round_trip_matches_input "$case_round_trip_matches_input" \
        --arg parse_status "$case_parse_status" \
        '{
            case_name: $case_name,
            description: $description,
            input: $input,
            expected_round_trip: $expected_round_trip,
            tags: $tags,
            source_file: $source_file,
            parse_log_file: $parse_log_file,
            status: $status,
            note: $note,
            observed: {
                parse_status: $parse_status,
                parser_error: (if $parser_error == "" then null else $parser_error end),
                sample_bytes: $sample_bytes,
                parse_full_ms: $parse_full_ms,
                expected_round_trip_matches_input: $round_trip_matches_input
            }
        }' >>"$CASES_JSONL"
done

if (( cases_executed == 0 )); then
    echo "error: no broader regex corpus cases executed" >&2
    exit 1
fi

cases_json="$(jq -s '.' "$CASES_JSONL")"
generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

jq -n \
    --arg gate "regex_broader_corpus_proof_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg state_dir "$STATE_DIR" \
    --arg summary_txt "$SUMMARY_TXT" \
    --arg summary_json "$SUMMARY_JSON" \
    --arg manifest_file "$MANIFEST_FILE" \
    --arg source_file "$source_file" \
    --arg generated_parser_file "$GENERATED_REGEX_PARSER" \
    --arg parseability_probe_bin "$PARSE_PROBE_BIN" \
    --argjson manifest_version "$manifest_version" \
    --argjson expected_case_count "$expected_case_count" \
    --argjson source_case_count "$source_case_count" \
    --argjson source_unique_tag_count "$source_unique_tag_count" \
    --argjson cases_declared "$cases_declared" \
    --argjson cases_executed "$cases_executed" \
    --argjson parse_pass_total "$parse_pass_total" \
    --argjson parse_fail_total "$parse_fail_total" \
    --argjson sample_bytes_max "$sample_bytes_max" \
    --argjson parse_total_ms "$parse_total_ms" \
    --argjson parse_max_ms "$parse_max_ms" \
    --arg primary_parse_failure_case "$primary_parse_failure_case" \
    --arg primary_parse_failure_parser_error "$primary_parse_failure_parser_error" \
    --argjson cases "$cases_json" \
    '{
        gate: $gate,
        version: $version,
        generated_at_utc: $generated_at_utc,
        state_dir: $state_dir,
        summary_txt: $summary_txt,
        summary_json: $summary_json,
        manifest_file: $manifest_file,
        source_file: $source_file,
        generated_parser_file: $generated_parser_file,
        parseability_probe_bin: $parseability_probe_bin,
        manifest_version: $manifest_version,
        expected_case_count: $expected_case_count,
        source_case_count: $source_case_count,
        source_unique_tag_count: $source_unique_tag_count,
        totals: {
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
            parser_error: $primary_parse_failure_parser_error
        },
        cases: $cases
    }' >"$SUMMARY_JSON"

require_nonempty_file "$SUMMARY_JSON"

{
    echo "Regex Broader Corpus Proof Gate Summary"
    echo "gate: regex_broader_corpus_proof_gate"
    echo "version: 1"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "summary_json: $SUMMARY_JSON"
    echo "manifest_file: $MANIFEST_FILE"
    echo "source_file: $source_file"
    echo "generated_parser_file: $GENERATED_REGEX_PARSER"
    echo "parseability_probe_bin: $PARSE_PROBE_BIN"
    echo "manifest_version: $manifest_version"
    echo "expected_case_count: $expected_case_count"
    echo "source_case_count: $source_case_count"
    echo "source_unique_tag_count: $source_unique_tag_count"
    echo "cases_declared: $cases_declared"
    echo "cases_executed: $cases_executed"
    echo "parse_pass_total: $parse_pass_total"
    echo "parse_fail_total: $parse_fail_total"
    echo "sample_bytes_max: $sample_bytes_max"
    echo "parse_total_ms: $parse_total_ms"
    echo "parse_max_ms: $parse_max_ms"
    echo "primary_parse_failure_case: $primary_parse_failure_case"
    echo "primary_parse_failure_parser_error: $primary_parse_failure_parser_error"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"
