#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_REGEX_EMBEDDED_CODE_BLOCK_CONTRACT_STATE_DIR:-$RUST_DIR/target/regex_embedded_code_block_contract_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
CASE_INPUT_DIR="$WORK_DIR/cases"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"

MANIFEST_FILE="${PGEN_REGEX_EMBEDDED_CODE_BLOCK_CONTRACT_MANIFEST:-$RUST_DIR/test_data/grammar_quality/regex_embedded_code_block_contract_v0.json}"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"
GENERATED_REGEX_PARSER="$ROOT_DIR/generated/regex_parser.rs"
CASE_RESULTS_JSONL="$WORK_DIR/case_results.jsonl"

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
        tail -n 80 "$log_file" >&2 || true
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

require_tool jq
require_file "$MANIFEST_FILE"
require_file "$GENERATED_REGEX_PARSER"

mkdir -p "$STATE_DIR" "$WORK_DIR" "$LOG_DIR" "$CASE_INPUT_DIR"
: >"$SUMMARY_TXT"
: >"$CASE_RESULTS_JSONL"

jq -e '
    ((.version | type) == "number")
    and ((.description | type) == "string" and (.description | length) > 0)
    and (.expected_parser_type == "regex")
    and (.expected_profile == "regex_default")
    and (.cases | type == "array" and length > 0)
    and ((.cases | map(.name) | length) == (.cases | map(.name) | unique | length))
    and all(
        .cases[];
        (.name | type) == "string"
        and (.name | length) > 0
        and (.description | type) == "string"
        and (.input | type) == "string"
        and (.expect_parse | type) == "boolean"
        and ((.tags // []) | type) == "array"
        and ((.required_rule_names // []) | type) == "array"
        and ((.forbidden_rule_names // []) | type) == "array"
        and all((.required_rule_names // [])[]; (type == "string") and (length > 0))
        and all((.forbidden_rule_names // [])[]; (type == "string") and (length > 0))
    )
' "$MANIFEST_FILE" >/dev/null

manifest_version="$(jq -er '.version | numbers' "$MANIFEST_FILE")"
expected_parser_type="$(jq -er '.expected_parser_type | strings' "$MANIFEST_FILE")"
expected_profile="$(jq -er '.expected_profile | strings' "$MANIFEST_FILE")"

run_logged_rust "build_parseability_probe_for_regex_embedded_code_block_contract" \
    cargo build --features generated_parsers --bin parseability_probe

require_file "$PARSE_PROBE_BIN"

run_logged_rust "parseability_probe_supports_regex_generated_backend" \
    "$PARSE_PROBE_BIN" --supports regex

mapfile -t case_rows < <(jq -c '.cases[]' "$MANIFEST_FILE")

cases_declared="${#case_rows[@]}"
cases_executed=0
expected_pass_total=0
expected_fail_total=0
observed_pass_total=0
observed_fail_total=0
mismatched_cases_total=0
primary_mismatch_case="<none>"
primary_mismatch_expected="<none>"
primary_mismatch_observed="<none>"

for case_json in "${case_rows[@]}"; do
    case_name="$(jq -er '.name | strings' <<<"$case_json")"
    case_description="$(jq -er '.description | strings' <<<"$case_json")"
    case_input="$(jq -er '.input | strings' <<<"$case_json")"
    case_expect_parse="$(jq -r '.expect_parse' <<<"$case_json")"
    case_tags_json="$(jq -c '(.tags // [])' <<<"$case_json")"
    case_required_rule_names_json="$(jq -c '(.required_rule_names // [])' <<<"$case_json")"
    case_forbidden_rule_names_json="$(jq -c '(.forbidden_rule_names // [])' <<<"$case_json")"

    if [[ "$case_expect_parse" == "true" ]]; then
        expected_pass_total=$((expected_pass_total + 1))
    else
        expected_fail_total=$((expected_fail_total + 1))
    fi

    case_name_key="$(printf '%s' "$case_name" | tr -c 'A-Za-z0-9_' '_')"
    case_input_file="$CASE_INPUT_DIR/${case_name_key}.regex"
    printf '%s' "$case_input" >"$case_input_file"

    case_label="case_${case_name_key}_parse"
    case_log="$LOG_DIR/${case_label}.log"
    parser_error=""
    set +e
    run_optional_logged "$case_label" \
        "$PARSE_PROBE_BIN" --parse "$expected_parser_type" "$case_input_file" --profile "$expected_profile"
    case_rc=$?
    set -e

    if (( case_rc == 0 )); then
        observed_status="pass"
        observed_pass_total=$((observed_pass_total + 1))
    else
        observed_status="fail"
        observed_fail_total=$((observed_fail_total + 1))
        parser_error="$(
            awk '
                /^Error: parse_full rejected sample for grammar '\''regex'\'' on '\''/ {
                    sub(/^Error: parse_full rejected sample for grammar '\''regex'\'' on '\''.*'\'': /, "")
                    print
                    exit
                }
            ' "$case_log"
        )"
        if [[ -z "$parser_error" ]]; then
            parser_error="parseability_probe failure"
        fi
    fi

    expected_status="fail"
    if [[ "$case_expect_parse" == "true" ]]; then
        expected_status="pass"
    fi

    status_matches=true
    if [[ "$expected_status" != "$observed_status" ]]; then
        status_matches=false
    fi

    ast_contract_matches=true
    ast_contract_detail=""
    ast_dump_file=""
    ast_dump_log=""

    required_rule_count="$(jq -r 'length' <<<"$case_required_rule_names_json")"
    forbidden_rule_count="$(jq -r 'length' <<<"$case_forbidden_rule_names_json")"

    if [[ "$observed_status" == "pass" ]] && (( required_rule_count > 0 || forbidden_rule_count > 0 )); then
        ast_dump_file="$WORK_DIR/${case_name_key}.ast.json"
        ast_label="case_${case_name_key}_ast_dump"
        ast_dump_log="$LOG_DIR/${ast_label}.log"
        set +e
        run_optional_logged "$ast_label" \
            "$PARSE_PROBE_BIN" --parse-dump-ast-pretty "$expected_parser_type" "$case_input_file" "$ast_dump_file" --profile "$expected_profile"
        ast_rc=$?
        set -e

        if (( ast_rc != 0 )); then
            ast_contract_matches=false
            ast_contract_detail="ast dump generation failed"
        else
            missing_required_rules="$(
                jq -r --argjson required "$case_required_rule_names_json" '
                    def rule_names: [.. | objects | .rule_name? // empty];
                    (rule_names) as $rules
                    | [$required[]? as $rule | select(($rules | index($rule)) == null) | $rule]
                    | join(",")
                ' "$ast_dump_file"
            )"
            present_forbidden_rules="$(
                jq -r --argjson forbidden "$case_forbidden_rule_names_json" '
                    def rule_names: [.. | objects | .rule_name? // empty];
                    (rule_names) as $rules
                    | [$forbidden[]? as $rule | select(($rules | index($rule)) != null) | $rule]
                    | join(",")
                ' "$ast_dump_file"
            )"

            if [[ -n "$missing_required_rules" || -n "$present_forbidden_rules" ]]; then
                ast_contract_matches=false
                ast_contract_detail="missing_required_rules=${missing_required_rules:-<none>}; present_forbidden_rules=${present_forbidden_rules:-<none>}"
            fi
        fi
    fi

    case_matches=true
    if [[ "$status_matches" != "true" || "$ast_contract_matches" != "true" ]]; then
        case_matches=false
        mismatched_cases_total=$((mismatched_cases_total + 1))
        if [[ "$primary_mismatch_case" == "<none>" ]]; then
            primary_mismatch_case="$case_name"
            if [[ "$status_matches" != "true" ]]; then
                primary_mismatch_expected="$expected_status"
                primary_mismatch_observed="$observed_status"
            else
                primary_mismatch_expected="ast_contract"
                primary_mismatch_observed="$ast_contract_detail"
            fi
        fi
    fi

    cases_executed=$((cases_executed + 1))

    jq -n \
        --arg name "$case_name" \
        --arg description "$case_description" \
        --arg input "$case_input" \
        --argjson tags "$case_tags_json" \
        --arg expected_status "$expected_status" \
        --arg observed_status "$observed_status" \
        --argjson status_matches "$status_matches" \
        --argjson required_rule_names "$case_required_rule_names_json" \
        --argjson forbidden_rule_names "$case_forbidden_rule_names_json" \
        --argjson ast_contract_matches "$ast_contract_matches" \
        --arg ast_contract_detail "$ast_contract_detail" \
        --argjson case_matches "$case_matches" \
        --arg parser_error "$parser_error" \
        --arg input_file "$case_input_file" \
        --arg parse_log "$case_log" \
        --arg ast_dump_file "$ast_dump_file" \
        --arg ast_dump_log "$ast_dump_log" \
        '{
            name: $name,
            description: $description,
            input: $input,
            tags: $tags,
            expected_status: $expected_status,
            observed_status: $observed_status,
            status_matches: $status_matches,
            required_rule_names: $required_rule_names,
            forbidden_rule_names: $forbidden_rule_names,
            ast_contract_matches: $ast_contract_matches,
            ast_contract_detail: (if $ast_contract_detail == "" then null else $ast_contract_detail end),
            case_matches: $case_matches,
            parser_error: (if $parser_error == "" then null else $parser_error end),
            input_file: $input_file,
            parse_log: $parse_log,
            ast_dump_file: (if $ast_dump_file == "" then null else $ast_dump_file end),
            ast_dump_log: (if $ast_dump_log == "" then null else $ast_dump_log end)
        }' >>"$CASE_RESULTS_JSONL"
done

if (( mismatched_cases_total > 0 )); then
    echo "error: regex embedded code-block contract mismatches detected" >&2
fi

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cat >"$SUMMARY_TXT" <<EOF
gate: regex_embedded_code_block_contract_gate
manifest_file: $MANIFEST_FILE
manifest_version: $manifest_version
generated_at_utc: $generated_at_utc
expected_parser_type: $expected_parser_type
expected_profile: $expected_profile
cases_declared: $cases_declared
cases_executed: $cases_executed
expected_pass_total: $expected_pass_total
expected_fail_total: $expected_fail_total
observed_pass_total: $observed_pass_total
observed_fail_total: $observed_fail_total
mismatched_cases_total: $mismatched_cases_total
primary_mismatch_case: $primary_mismatch_case
primary_mismatch_expected: $primary_mismatch_expected
primary_mismatch_observed: $primary_mismatch_observed
case_results_jsonl: $CASE_RESULTS_JSONL
EOF

jq -n \
    --arg gate "regex_embedded_code_block_contract_gate" \
    --arg manifest_file "$MANIFEST_FILE" \
    --argjson manifest_version "$manifest_version" \
    --arg generated_at_utc "$generated_at_utc" \
    --arg expected_parser_type "$expected_parser_type" \
    --arg expected_profile "$expected_profile" \
    --argjson cases_declared "$cases_declared" \
    --argjson cases_executed "$cases_executed" \
    --argjson expected_pass_total "$expected_pass_total" \
    --argjson expected_fail_total "$expected_fail_total" \
    --argjson observed_pass_total "$observed_pass_total" \
    --argjson observed_fail_total "$observed_fail_total" \
    --argjson mismatched_cases_total "$mismatched_cases_total" \
    --arg primary_mismatch_case "$primary_mismatch_case" \
    --arg primary_mismatch_expected "$primary_mismatch_expected" \
    --arg primary_mismatch_observed "$primary_mismatch_observed" \
    --arg case_results_jsonl "$CASE_RESULTS_JSONL" \
    '{
        gate: $gate,
        manifest_file: $manifest_file,
        manifest_version: $manifest_version,
        generated_at_utc: $generated_at_utc,
        expected_parser_type: $expected_parser_type,
        expected_profile: $expected_profile,
        cases_declared: $cases_declared,
        cases_executed: $cases_executed,
        expected_pass_total: $expected_pass_total,
        expected_fail_total: $expected_fail_total,
        observed_pass_total: $observed_pass_total,
        observed_fail_total: $observed_fail_total,
        mismatched_cases_total: $mismatched_cases_total,
        primary_mismatch_case: (if $primary_mismatch_case == "<none>" then null else $primary_mismatch_case end),
        primary_mismatch_expected: (if $primary_mismatch_expected == "<none>" then null else $primary_mismatch_expected end),
        primary_mismatch_observed: (if $primary_mismatch_observed == "<none>" then null else $primary_mismatch_observed end),
        case_results_jsonl: $case_results_jsonl
    }' >"$SUMMARY_JSON"

if (( mismatched_cases_total > 0 )); then
    jq -c 'select(.case_matches == false)' "$CASE_RESULTS_JSONL" >&2 || true
    exit 1
fi

cat <<EOF
✅ regex embedded code-block contract gate passed.
Summary TXT: $SUMMARY_TXT
Summary JSON: $SUMMARY_JSON
EOF
