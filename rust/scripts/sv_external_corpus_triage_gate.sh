#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"

STATE_DIR="${PGEN_SV_EXTERNAL_CORPUS_TRIAGE_STATE_DIR:-$RUST_DIR/target/sv_external_corpus_triage_gate}"
WORK_DIR="$STATE_DIR/work"
LOG_DIR="$STATE_DIR/logs"
SUMMARY_TXT="$STATE_DIR/summary.txt"
SUMMARY_JSON="$STATE_DIR/summary.json"
REPORT_JSON="$WORK_DIR/systemverilog_external_corpus_triage_report.json"
CASES_JSONL="$WORK_DIR/systemverilog_external_corpus_triage_cases.jsonl"

MANIFEST_FILE="${PGEN_SV_EXTERNAL_CORPUS_TRIAGE_MANIFEST:-$RUST_DIR/test_data/grammar_quality/systemverilog_external_corpus_triage_v0.json}"
MAX_CASES="${PGEN_SV_EXTERNAL_CORPUS_TRIAGE_MAX_CASES:-0}"
INCLUDE_MAX_DEPTH="${PGEN_SV_EXTERNAL_CORPUS_TRIAGE_INCLUDE_MAX_DEPTH:-64}"
INCLUDE_PATH_POLICY="${PGEN_SV_EXTERNAL_CORPUS_TRIAGE_INCLUDE_PATH_POLICY:-allow_absolute}"
MACRO_REDEFINE_POLICY="${PGEN_SV_EXTERNAL_CORPUS_TRIAGE_MACRO_REDEFINE_POLICY:-allow}"
CONDITIONAL_SYMBOL_POLICY="${PGEN_SV_EXTERNAL_CORPUS_TRIAGE_CONDITIONAL_SYMBOL_POLICY:-assume_false_silent}"
CONDITIONAL_EXPR_POLICY="${PGEN_SV_EXTERNAL_CORPUS_TRIAGE_CONDITIONAL_EXPR_POLICY:-identifier_or_defined}"
STRICT_WARNING_CODES="${PGEN_SV_EXTERNAL_CORPUS_TRIAGE_STRICT_WARNING_CODES:-none}"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"
GRAMMAR_FILE="$ROOT_DIR/grammars/systemverilog.ebnf"
GRAMMAR_JSON="$WORK_DIR/systemverilog_external_corpus_triage_grammar.json"
PARSER_OUT="$WORK_DIR/systemverilog_external_corpus_triage_parser.rs"

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

bootstrap_veer_default_snapshot() {
    local repo_root="$1"
    local required_file="$2"
    local label="$3"
    local log_file="$LOG_DIR/${label}.log"
    echo "==> ${label}"
    if (
        cd "$repo_root"
        export RV_ROOT="$repo_root"
        perl configs/veer.config -target=default -snapshot=default
    ) >"$log_file" 2>&1; then
        if [[ -f "$required_file" ]]; then
            echo "    ok (${log_file})"
            return 0
        fi
        echo "    soft-fail (${log_file})"
        return 1
    fi
    echo "    soft-fail (${log_file})"
    return 1
}

if ! [[ "$MAX_CASES" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_SV_EXTERNAL_CORPUS_TRIAGE_MAX_CASES must be an integer >= 0" >&2
    exit 2
fi
if ! [[ "$INCLUDE_MAX_DEPTH" =~ ^[0-9]+$ ]] || [[ "$INCLUDE_MAX_DEPTH" -lt 1 ]]; then
    echo "error: PGEN_SV_EXTERNAL_CORPUS_TRIAGE_INCLUDE_MAX_DEPTH must be an integer >= 1" >&2
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

run_logged_rust "build_ast_pipeline_for_sv_external_corpus_triage" \
    cargo build --features generated_parsers --bin ast_pipeline

run_logged "frontend_ebnf_to_json" \
    perl "$EBNF_TO_JSON" --pretty --quiet "$GRAMMAR_FILE" -o "$GRAMMAR_JSON"
require_nonempty_file "$GRAMMAR_JSON"

run_logged "generate_systemverilog_parser" \
    "$AST_PIPELINE_BIN" "$GRAMMAR_JSON" \
    --generate-parser \
    --eliminate-left-recursion \
    --output "$PARSER_OUT"
require_nonempty_file "$PARSER_OUT"

run_logged_rust "build_sv_external_corpus_triage_binaries" \
    env PGEN_SYSTEMVERILOG_PARSER_PATH="$PARSER_OUT" \
    cargo build --features generated_parsers --bin ast_pipeline --bin parseability_probe

require_file "$AST_PIPELINE_BIN"
require_file "$PARSE_PROBE_BIN"

mapfile -t case_rows < <(jq -c '.cases[]?' "$MANIFEST_FILE")
cases_declared="${#case_rows[@]}"
if (( cases_declared == 0 )); then
    echo "error: manifest has zero cases: $MANIFEST_FILE" >&2
    exit 1
fi

cases_executed=0
cases_blocked_total=0
preprocess_pass_total=0
preprocess_fail_total=0
parse_pass_total=0
parse_fail_total=0
parse_skipped_total=0
preprocess_warning_total=0
preprocess_error_total=0
sample_bytes_max=0
preprocessed_bytes_max=0
preprocess_total_ms=0
parse_total_ms=0
preprocess_max_ms=0
parse_max_ms=0
primary_preprocess_failure_case="<none>"
primary_preprocess_failure_profile="<none>"
primary_preprocess_failure_corpus="<none>"
primary_blocked_case="<none>"
primary_blocked_profile="<none>"
primary_blocked_corpus="<none>"
primary_blocked_reason="<none>"
primary_parse_failure_case="<none>"
primary_parse_failure_profile="<none>"
primary_parse_failure_corpus="<none>"

case_manifest_idx=0
for case_json in "${case_rows[@]}"; do
    if (( MAX_CASES > 0 && case_manifest_idx >= MAX_CASES )); then
        break
    fi
    case_manifest_idx=$((case_manifest_idx + 1))

    case_name="$(jq -er '.name | strings' <<<"$case_json")"
    case_corpus="$(jq -er '(.corpus // "uncategorized") | strings' <<<"$case_json")"
    case_mode="$(jq -er '(.mode // "preprocess_then_parse_full") | strings' <<<"$case_json")"
    case_source_rel="$(jq -er '.path | strings' <<<"$case_json")"
    case_source_path="$(resolve_path "$case_source_rel")"
    case_source_dir="$(dirname "$case_source_path")"
    case_blocked_reason="$(jq -r 'if (.blocked_reason | type) == "string" then .blocked_reason else "" end' <<<"$case_json")"
    case_bootstrap_kind="$(jq -r 'if (.bootstrap_kind | type) == "string" then .bootstrap_kind else "" end' <<<"$case_json")"
    case_bootstrap_root_rel="$(jq -r 'if (.bootstrap_root | type) == "string" then .bootstrap_root else "" end' <<<"$case_json")"
    case_bootstrap_required_rel="$(jq -r 'if (.bootstrap_required_file | type) == "string" then .bootstrap_required_file else "" end' <<<"$case_json")"
    require_file "$case_source_path"

    if [[ "$case_mode" != "preprocess_then_parse_full" ]]; then
        echo "error: unsupported SystemVerilog external triage mode '$case_mode' for case '$case_name'" >&2
        exit 1
    fi

    mapfile -t case_profiles < <(jq -r '.profiles[]? | select(type=="string")' <<<"$case_json")
    if [[ "${#case_profiles[@]}" -eq 0 ]]; then
        case_profiles=("2017" "2023")
    fi

    mapfile -t case_include_dirs_raw < <(jq -r '.include_dirs[]? | select(type=="string")' <<<"$case_json")
    mapfile -t case_blocked_profiles < <(jq -r '.blocked_profiles[]? | select(type=="string")' <<<"$case_json")

    if [[ "$case_bootstrap_kind" == "veer_default_snapshot" ]]; then
        if [[ -z "$case_bootstrap_root_rel" || -z "$case_bootstrap_required_rel" ]]; then
            echo "error: case '$case_name' requires bootstrap_root and bootstrap_required_file for bootstrap_kind '$case_bootstrap_kind'" >&2
            exit 1
        fi
        case_bootstrap_root="$(resolve_path "$case_bootstrap_root_rel")"
        case_bootstrap_required="$(resolve_path "$case_bootstrap_required_rel")"
        if [[ ! -f "$case_bootstrap_required" ]]; then
            bootstrap_label="case_$(printf '%s' "$case_name" | tr -c 'A-Za-z0-9_' '_')_bootstrap"
            if ! bootstrap_veer_default_snapshot "$case_bootstrap_root" "$case_bootstrap_required" "$bootstrap_label"; then
                case_blocked_reason="failed to bootstrap VeeR default snapshot via configs/veer.config"
                if [[ "${#case_blocked_profiles[@]}" -eq 0 ]]; then
                    case_blocked_profiles=("${case_profiles[@]}")
                fi
            fi
        fi
    fi

    case_name_key="$(printf '%s' "$case_name" | tr -c 'A-Za-z0-9_' '_')"
    for case_profile in "${case_profiles[@]}"; do
        case_profile_key="$(printf '%s' "$case_profile" | tr -c 'A-Za-z0-9_' '_')"
        case_input_file="$WORK_DIR/case_${case_name_key}_${case_profile_key}.sv"
        case_preprocessed_file="$WORK_DIR/case_${case_name_key}_${case_profile_key}.preprocessed.sv"
        case_diagnostics_json="$WORK_DIR/case_${case_name_key}_${case_profile_key}.diagnostics.json"
        case_preprocess_label="case_${case_name_key}_${case_profile_key}_preprocess"
        case_parse_label="case_${case_name_key}_${case_profile_key}_parse_full"
        case_preprocess_log="$LOG_DIR/${case_preprocess_label}.log"
        case_parse_log="$LOG_DIR/${case_parse_label}.log"

        cp "$case_source_path" "$case_input_file"

        case_sample_bytes="$(file_size_bytes "$case_input_file")"
        if (( case_sample_bytes > sample_bytes_max )); then
            sample_bytes_max="$case_sample_bytes"
        fi

        case_profile_blocked=0
        if [[ -n "$case_blocked_reason" ]]; then
            if [[ "${#case_blocked_profiles[@]}" -eq 0 ]]; then
                case_profile_blocked=1
            else
                for blocked_profile in "${case_blocked_profiles[@]}"; do
                    if [[ "$blocked_profile" == "$case_profile" ]]; then
                        case_profile_blocked=1
                        break
                    fi
                done
            fi
        fi

        preprocess_args=(
            "$AST_PIPELINE_BIN" "$case_input_file"
            --preprocess-systemverilog
            --output "$case_preprocessed_file"
            --sv-diagnostics-json "$case_diagnostics_json"
            --sv-include-dir "$case_source_dir"
            --sv-include-max-depth "$INCLUDE_MAX_DEPTH"
            --sv-include-path-policy "$INCLUDE_PATH_POLICY"
            --sv-macro-redefine-policy "$MACRO_REDEFINE_POLICY"
            --sv-conditional-symbol-policy "$CONDITIONAL_SYMBOL_POLICY"
            --sv-conditional-expr-policy "$CONDITIONAL_EXPR_POLICY"
            --sv-strict-warning-codes "$STRICT_WARNING_CODES"
        )
        for include_dir_raw in "${case_include_dirs_raw[@]}"; do
            include_dir_resolved="$(resolve_path "$include_dir_raw")"
            preprocess_args+=(--sv-include-dir "$include_dir_resolved")
        done

        case_warning_count=0
        case_error_count=0
        case_preprocess_elapsed_ms=0
        case_preprocess_status="blocked"
        case_parse_status="blocked"
        case_parse_elapsed_ms=0
        case_preprocessed_bytes=0
        case_status="blocked_external_dependency"
        case_note="$case_blocked_reason"

        if (( case_profile_blocked == 1 )); then
            printf 'blocked: %s\n' "$case_blocked_reason" >"$case_preprocess_log"
            printf 'blocked: %s\n' "$case_blocked_reason" >"$case_parse_log"
            cases_blocked_total=$((cases_blocked_total + 1))
            if [[ "$primary_blocked_case" == "<none>" ]]; then
                primary_blocked_case="$case_name"
                primary_blocked_profile="$case_profile"
                primary_blocked_corpus="$case_corpus"
                primary_blocked_reason="$case_blocked_reason"
            fi
        else
            case_preprocess_started_ms="$(now_ms)"
            if run_optional_logged "$case_preprocess_label" "${preprocess_args[@]}"; then
                case_preprocess_status="pass"
                preprocess_pass_total=$((preprocess_pass_total + 1))
            else
                case_preprocess_status="fail"
                preprocess_fail_total=$((preprocess_fail_total + 1))
                case_parse_status="skipped"
                case_status="preprocess_fail"
                case_note="preprocess failed"
                if [[ "$primary_preprocess_failure_case" == "<none>" ]]; then
                    primary_preprocess_failure_case="$case_name"
                    primary_preprocess_failure_profile="$case_profile"
                    primary_preprocess_failure_corpus="$case_corpus"
                fi
            fi
            case_preprocess_elapsed_ms=$(( $(now_ms) - case_preprocess_started_ms ))
            preprocess_total_ms=$((preprocess_total_ms + case_preprocess_elapsed_ms))
            if (( case_preprocess_elapsed_ms > preprocess_max_ms )); then
                preprocess_max_ms="$case_preprocess_elapsed_ms"
            fi

            if [[ -s "$case_diagnostics_json" ]]; then
                case_warning_count="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$case_diagnostics_json")"
                case_error_count="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$case_diagnostics_json")"
            fi
            preprocess_warning_total=$((preprocess_warning_total + case_warning_count))
            preprocess_error_total=$((preprocess_error_total + case_error_count))
        fi

        if [[ "$case_preprocess_status" == "pass" ]]; then
            case_preprocessed_bytes="$(file_size_bytes "$case_preprocessed_file")"
            if (( case_preprocessed_bytes > preprocessed_bytes_max )); then
                preprocessed_bytes_max="$case_preprocessed_bytes"
            fi

            case_parse_started_ms="$(now_ms)"
            if run_optional_logged "$case_parse_label" \
                "$PARSE_PROBE_BIN" --parse systemverilog "$case_preprocessed_file" --profile "$case_profile"; then
                case_parse_status="pass"
                parse_pass_total=$((parse_pass_total + 1))
                case_status="pass"
                case_note="preprocess and parse_full both passed"
            else
                case_parse_status="fail"
                parse_fail_total=$((parse_fail_total + 1))
                case_status="parse_fail"
                case_note="parse_full rejected preprocessed sample"
                if [[ "$primary_parse_failure_case" == "<none>" ]]; then
                    primary_parse_failure_case="$case_name"
                    primary_parse_failure_profile="$case_profile"
                    primary_parse_failure_corpus="$case_corpus"
                fi
            fi
            case_parse_elapsed_ms=$(( $(now_ms) - case_parse_started_ms ))
            parse_total_ms=$((parse_total_ms + case_parse_elapsed_ms))
            if (( case_parse_elapsed_ms > parse_max_ms )); then
                parse_max_ms="$case_parse_elapsed_ms"
            fi
        elif [[ "$case_preprocess_status" == "fail" ]]; then
            parse_skipped_total=$((parse_skipped_total + 1))
        fi

        if (( case_profile_blocked == 0 )); then
            cases_executed=$((cases_executed + 1))
        fi

        include_dirs_json="$(jq -c '(.include_dirs // []) | map(select(type=="string"))' <<<"$case_json")"
        jq -n \
            --arg case_name "$case_name" \
            --arg corpus "$case_corpus" \
            --arg profile "$case_profile" \
            --arg mode "$case_mode" \
            --arg source_file "$case_source_path" \
            --arg sample_file "$case_input_file" \
            --arg preprocessed_file "$case_preprocessed_file" \
            --arg diagnostics_file "$case_diagnostics_json" \
            --arg preprocess_log_file "$case_preprocess_log" \
            --arg parse_log_file "$case_parse_log" \
            --arg preprocess_status "$case_preprocess_status" \
            --arg parse_status "$case_parse_status" \
            --arg status "$case_status" \
            --arg note "$case_note" \
            --arg blocked_reason "$case_blocked_reason" \
            --argjson sample_bytes "$case_sample_bytes" \
            --argjson preprocessed_bytes "$case_preprocessed_bytes" \
            --argjson preprocess_ms "$case_preprocess_elapsed_ms" \
            --argjson parse_full_ms "$case_parse_elapsed_ms" \
            --argjson preprocess_warnings "$case_warning_count" \
            --argjson preprocess_errors "$case_error_count" \
            --argjson include_dirs "$include_dirs_json" \
            '{
                case_name: $case_name,
                corpus: $corpus,
                profile: $profile,
                mode: $mode,
                source_file: $source_file,
                sample_file: $sample_file,
                preprocessed_file: $preprocessed_file,
                diagnostics_file: $diagnostics_file,
                preprocess_log_file: $preprocess_log_file,
                parse_log_file: $parse_log_file,
                include_dirs: $include_dirs,
                status: $status,
                note: $note,
                blocked_reason: (if $blocked_reason == "" then null else $blocked_reason end),
                observed: {
                    preprocess_status: $preprocess_status,
                    parse_status: $parse_status,
                    sample_bytes: $sample_bytes,
                    preprocessed_bytes: $preprocessed_bytes,
                    preprocess_ms: $preprocess_ms,
                    parse_full_ms: $parse_full_ms,
                    preprocess_warnings: $preprocess_warnings,
                    preprocess_errors: $preprocess_errors
                }
            }' >>"$CASES_JSONL"
    done
done

if (( cases_executed == 0 && cases_blocked_total == 0 )); then
    echo "error: no SystemVerilog external triage cases executed" >&2
    exit 1
fi

cases_json="$(jq -s '.' "$CASES_JSONL")"
generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

jq -n \
    --arg gate "sv_external_corpus_triage_gate" \
    --argjson version 1 \
    --arg generated_at_utc "$generated_at_utc" \
    --arg manifest_file "$MANIFEST_FILE" \
    --argjson manifest_version "$manifest_version" \
    --arg report_summary_txt "$SUMMARY_TXT" \
    --argjson corpus_count "$corpus_count" \
    --argjson cases_declared "$cases_declared" \
    --argjson cases_executed "$cases_executed" \
    --argjson cases_blocked_total "$cases_blocked_total" \
    --argjson preprocess_pass_total "$preprocess_pass_total" \
    --argjson preprocess_fail_total "$preprocess_fail_total" \
    --argjson parse_pass_total "$parse_pass_total" \
    --argjson parse_fail_total "$parse_fail_total" \
    --argjson parse_skipped_total "$parse_skipped_total" \
    --argjson preprocess_warning_total "$preprocess_warning_total" \
    --argjson preprocess_error_total "$preprocess_error_total" \
    --argjson sample_bytes_max "$sample_bytes_max" \
    --argjson preprocessed_bytes_max "$preprocessed_bytes_max" \
    --argjson preprocess_total_ms "$preprocess_total_ms" \
    --argjson preprocess_max_ms "$preprocess_max_ms" \
    --argjson parse_total_ms "$parse_total_ms" \
    --argjson parse_max_ms "$parse_max_ms" \
    --arg primary_preprocess_failure_case "$primary_preprocess_failure_case" \
    --arg primary_preprocess_failure_profile "$primary_preprocess_failure_profile" \
    --arg primary_preprocess_failure_corpus "$primary_preprocess_failure_corpus" \
    --arg primary_blocked_case "$primary_blocked_case" \
    --arg primary_blocked_profile "$primary_blocked_profile" \
    --arg primary_blocked_corpus "$primary_blocked_corpus" \
    --arg primary_blocked_reason "$primary_blocked_reason" \
    --arg primary_parse_failure_case "$primary_parse_failure_case" \
    --arg primary_parse_failure_profile "$primary_parse_failure_profile" \
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
            cases_blocked_total: $cases_blocked_total,
            preprocess_pass_total: $preprocess_pass_total,
            preprocess_fail_total: $preprocess_fail_total,
            parse_pass_total: $parse_pass_total,
            parse_fail_total: $parse_fail_total,
            parse_skipped_total: $parse_skipped_total,
            preprocess_warning_total: $preprocess_warning_total,
            preprocess_error_total: $preprocess_error_total,
            sample_bytes_max: $sample_bytes_max,
            preprocessed_bytes_max: $preprocessed_bytes_max,
            preprocess_total_ms: $preprocess_total_ms,
            preprocess_max_ms: $preprocess_max_ms,
            parse_total_ms: $parse_total_ms,
            parse_max_ms: $parse_max_ms
        },
        primary_preprocess_failure: {
            case_name: $primary_preprocess_failure_case,
            profile: $primary_preprocess_failure_profile,
            corpus: $primary_preprocess_failure_corpus
        },
        primary_blocked_case: {
            case_name: $primary_blocked_case,
            profile: $primary_blocked_profile,
            corpus: $primary_blocked_corpus,
            reason: $primary_blocked_reason
        },
        primary_parse_failure: {
            case_name: $primary_parse_failure_case,
            profile: $primary_parse_failure_profile,
            corpus: $primary_parse_failure_corpus
        },
        cases: $cases
    }' >"$REPORT_JSON"

cp "$REPORT_JSON" "$SUMMARY_JSON"

{
    echo "SystemVerilog External Corpus Triage Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "generated_at_utc: $generated_at_utc"
    echo "manifest_file: $MANIFEST_FILE"
    echo "manifest_version: $manifest_version"
    echo "corpus_count: $corpus_count"
    echo "cases_declared: $cases_declared"
    echo "cases_executed: $cases_executed"
    echo "cases_blocked_total: $cases_blocked_total"
    echo "preprocess_pass_total: $preprocess_pass_total"
    echo "preprocess_fail_total: $preprocess_fail_total"
    echo "parse_pass_total: $parse_pass_total"
    echo "parse_fail_total: $parse_fail_total"
    echo "parse_skipped_total: $parse_skipped_total"
    echo "preprocess_warning_total: $preprocess_warning_total"
    echo "preprocess_error_total: $preprocess_error_total"
    echo "sample_bytes_max: $sample_bytes_max"
    echo "preprocessed_bytes_max: $preprocessed_bytes_max"
    echo "preprocess_total_ms: $preprocess_total_ms"
    echo "preprocess_max_ms: $preprocess_max_ms"
    echo "parse_total_ms: $parse_total_ms"
    echo "parse_max_ms: $parse_max_ms"
    echo "primary_preprocess_failure_case: $primary_preprocess_failure_case"
    echo "primary_preprocess_failure_profile: $primary_preprocess_failure_profile"
    echo "primary_preprocess_failure_corpus: $primary_preprocess_failure_corpus"
    echo "primary_blocked_case: $primary_blocked_case"
    echo "primary_blocked_profile: $primary_blocked_profile"
    echo "primary_blocked_corpus: $primary_blocked_corpus"
    echo "primary_blocked_reason: $primary_blocked_reason"
    echo "primary_parse_failure_case: $primary_parse_failure_case"
    echo "primary_parse_failure_profile: $primary_parse_failure_profile"
    echo "primary_parse_failure_corpus: $primary_parse_failure_corpus"
    echo "report_json: $REPORT_JSON"
    echo "summary_json: $SUMMARY_JSON"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"
