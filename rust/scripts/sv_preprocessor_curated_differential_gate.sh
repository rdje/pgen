#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PREPROCESSOR_CURATED_DIFF_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_curated_differential_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"
REPORT_JSON="$WORK_DIR/systemverilog_preprocessor_curated_differential_report.json"
CASES_JSONL="$WORK_DIR/systemverilog_preprocessor_curated_cases.jsonl"

CURATED_CORPUS="${PGEN_SV_PREPROCESSOR_CURATED_DIFF_CORPUS:-$RUST_DIR/test_data/grammar_quality/systemverilog_preprocessor_curated_differential_corpus.json}"
DIFF_MODE="${PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE:-auto}"
DIFF_MAX_CASES="${PGEN_SV_PREPROCESSOR_CURATED_DIFF_MAX_CASES:-0}"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"

if [[ "$DIFF_MODE" != "auto" && "$DIFF_MODE" != "0" && "$DIFF_MODE" != "1" ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_CURATED_DIFF_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$DIFF_MAX_CASES" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_CURATED_DIFF_MAX_CASES must be an integer >= 0" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

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
        echo "    fail (${log_file})" >&2
        tail -n 80 "$log_file" >&2 || true
        exit 1
    fi
}

normalize_text_for_diff_output() {
    local source="$1"
    local target="$2"
    tr -s '[:space:]' ' ' <"$source" | sed 's/^ *//; s/ *$//' >"$target"
}

canonicalize_json() {
    local source="$1"
    local target="$2"
    jq -S . "$source" >"$target"
}

resolve_case_path() {
    local raw_path="$1"
    if [[ "$raw_path" = /* ]]; then
        printf '%s\n' "$raw_path"
    else
        printf '%s\n' "$ROOT_DIR/$raw_path"
    fi
}

require_tool jq
require_file "$CURATED_CORPUS"

if ! jq -e '.cases | type == "array"' "$CURATED_CORPUS" >/dev/null 2>&1; then
    echo "error: curated corpus must contain an array field '.cases'" >&2
    exit 1
fi

echo "==> SV preprocessor curated differential gate"
echo "state_dir: $STATE_DIR"
echo "curated_corpus: $CURATED_CORPUS"
echo "diff_mode: $DIFF_MODE"
echo "diff_max_cases: $DIFF_MAX_CASES"

run_logged_rust "build_ast_pipeline_for_curated_preprocessor_diff" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

diff_effective_mode="disabled"
diff_note="curated differential disabled by configuration"
diff_cases_declared="$(jq -er '.cases | length | numbers' "$CURATED_CORPUS")"
diff_cases_checked=0

expected_match_count=0
expected_mismatch_count=0
bug_mismatch_count=0

diff_match_count=0
diff_diagnostics_mismatch_count=0
diff_whitespace_only_output_mismatch_count=0
diff_output_mismatch_count=0
diff_rust_failed_expected_passed_count=0
diff_reference_artifact_missing_count=0

if [[ "$DIFF_MODE" != "0" ]]; then
    diff_effective_mode="enabled"
    diff_note="curated differential classification enabled using checked-in expected artifacts"

    : >"$CASES_JSONL"

    while IFS= read -r case_entry; do
        if [[ "$DIFF_MAX_CASES" -gt 0 && "$diff_cases_checked" -ge "$DIFF_MAX_CASES" ]]; then
            break
        fi

        case_name="$(jq -er '.name' <<<"$case_entry")"
        case_input_raw="$(jq -er '.input' <<<"$case_entry")"
        case_expected_output_raw="$(jq -er '.expected_output' <<<"$case_entry")"
        case_expected_diag_raw="$(jq -er '.expected_diagnostics' <<<"$case_entry")"
        case_note="$(jq -er '.note // ""' <<<"$case_entry")"
        case_expected_categories="$(jq -c '.expected_categories // ["match"]' <<<"$case_entry")"

        case_input_file="$(resolve_case_path "$case_input_raw")"
        case_expected_output="$(resolve_case_path "$case_expected_output_raw")"
        case_expected_diag="$(resolve_case_path "$case_expected_diag_raw")"

        require_file "$case_input_file"
        require_file "$case_expected_output"
        require_file "$case_expected_diag"

        case_index="$diff_cases_checked"
        case_rust_output="$WORK_DIR/curated_case_${case_index}.rust.out.sv"
        case_rust_diag="$WORK_DIR/curated_case_${case_index}.rust.diag.json"
        case_rust_log="$LOG_DIR/curated_case_${case_index}.rust.log"

        case_rust_norm="$WORK_DIR/curated_case_${case_index}.rust.norm.txt"
        case_expected_norm="$WORK_DIR/curated_case_${case_index}.expected.norm.txt"

        case_rust_diag_norm="$WORK_DIR/curated_case_${case_index}.rust.diag.norm.json"
        case_expected_diag_norm="$WORK_DIR/curated_case_${case_index}.expected.diag.norm.json"

        rust_exit=0
        if "$AST_PIPELINE_BIN" "$case_input_file" \
            --preprocess-systemverilog \
            --output "$case_rust_output" \
            --sv-diagnostics-json "$case_rust_diag" >"$case_rust_log" 2>&1; then
            rust_exit=0
        else
            rust_exit=$?
        fi

        rust_warnings=0
        rust_errors=0
        rust_diag_available=0
        if [[ -s "$case_rust_diag" ]] && jq -e 'type == "array"' "$case_rust_diag" >/dev/null 2>&1; then
            rust_diag_available=1
            rust_warnings="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$case_rust_diag")"
            rust_errors="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$case_rust_diag")"
        fi

        category=""
        if (( rust_exit != 0 )); then
            category="rust_failed_expected_passed"
        elif [[ ! -f "$case_rust_output" || ! -f "$case_rust_diag" ]]; then
            category="reference_artifact_missing"
        else
            output_exact_match=0
            output_whitespace_match=0
            diagnostics_match=0

            if cmp -s "$case_rust_output" "$case_expected_output"; then
                output_exact_match=1
            fi

            normalize_text_for_diff_output "$case_rust_output" "$case_rust_norm"
            normalize_text_for_diff_output "$case_expected_output" "$case_expected_norm"
            if cmp -s "$case_rust_norm" "$case_expected_norm"; then
                output_whitespace_match=1
            fi

            canonicalize_json "$case_rust_diag" "$case_rust_diag_norm"
            canonicalize_json "$case_expected_diag" "$case_expected_diag_norm"
            if cmp -s "$case_rust_diag_norm" "$case_expected_diag_norm"; then
                diagnostics_match=1
            fi

            if (( output_exact_match == 1 && diagnostics_match == 1 )); then
                category="match"
            elif (( output_whitespace_match == 1 && diagnostics_match == 1 )); then
                category="whitespace_only_output_mismatch"
            elif (( output_whitespace_match == 1 && diagnostics_match == 0 )); then
                category="diagnostics_mismatch"
            else
                category="output_mismatch"
            fi
        fi

        case "$category" in
            match)
                diff_match_count=$((diff_match_count + 1))
                ;;
            diagnostics_mismatch)
                diff_diagnostics_mismatch_count=$((diff_diagnostics_mismatch_count + 1))
                ;;
            whitespace_only_output_mismatch)
                diff_whitespace_only_output_mismatch_count=$((diff_whitespace_only_output_mismatch_count + 1))
                ;;
            output_mismatch)
                diff_output_mismatch_count=$((diff_output_mismatch_count + 1))
                ;;
            rust_failed_expected_passed)
                diff_rust_failed_expected_passed_count=$((diff_rust_failed_expected_passed_count + 1))
                ;;
            reference_artifact_missing)
                diff_reference_artifact_missing_count=$((diff_reference_artifact_missing_count + 1))
                ;;
            *)
                echo "error: unknown curated differential taxonomy category '$category'" >&2
                exit 1
                ;;
        esac

        case_classification=""
        if jq -e --arg category "$category" 'index($category) != null' <<<"$case_expected_categories" >/dev/null 2>&1; then
            if [[ "$category" == "match" ]]; then
                case_classification="expected_match"
                expected_match_count=$((expected_match_count + 1))
            else
                case_classification="expected_mismatch"
                expected_mismatch_count=$((expected_mismatch_count + 1))
            fi
        else
            case_classification="bug_mismatch"
            bug_mismatch_count=$((bug_mismatch_count + 1))
        fi

        jq -n \
            --arg index "$case_index" \
            --arg name "$case_name" \
            --arg input_file "$case_input_file" \
            --arg expected_output "$case_expected_output" \
            --arg expected_diagnostics "$case_expected_diag" \
            --arg note "$case_note" \
            --arg category "$category" \
            --arg classification "$case_classification" \
            --argjson expected_categories "$case_expected_categories" \
            --arg rust_output "$case_rust_output" \
            --arg rust_diag "$case_rust_diag" \
            --arg rust_log "$case_rust_log" \
            --argjson rust_exit "$rust_exit" \
            --argjson rust_warnings "$rust_warnings" \
            --argjson rust_errors "$rust_errors" \
            '{
                index: ($index | tonumber),
                name: $name,
                input_file: $input_file,
                expected_output: $expected_output,
                expected_diagnostics: $expected_diagnostics,
                note: $note,
                expected_categories: $expected_categories,
                observed_category: $category,
                classification: $classification,
                rust: {
                    output_file: $rust_output,
                    diagnostics_file: $rust_diag,
                    log_file: $rust_log,
                    exit_code: $rust_exit,
                    warning_count: $rust_warnings,
                    error_count: $rust_errors
                }
            }' >>"$CASES_JSONL"

        diff_cases_checked=$((diff_cases_checked + 1))
    done < <(jq -c '.cases[]' "$CURATED_CORPUS")

    if [[ "$DIFF_MODE" == "1" && "$diff_cases_checked" -eq 0 ]]; then
        echo "error: strict curated differential mode checked zero cases" >&2
        exit 1
    fi
fi

if [[ -s "$CASES_JSONL" ]]; then
    curated_cases_json="$(jq -s '.' "$CASES_JSONL")"
else
    curated_cases_json='[]'
fi

jq -n \
    --arg grammar_name "systemverilog_preprocessor" \
    --arg corpus "$CURATED_CORPUS" \
    --arg requested_mode "$DIFF_MODE" \
    --arg effective_mode "$diff_effective_mode" \
    --arg note "$diff_note" \
    --argjson cases_declared "$diff_cases_declared" \
    --argjson max_cases "$DIFF_MAX_CASES" \
    --argjson cases_checked "$diff_cases_checked" \
    --argjson expected_match_count "$expected_match_count" \
    --argjson expected_mismatch_count "$expected_mismatch_count" \
    --argjson bug_mismatch_count "$bug_mismatch_count" \
    --argjson match_count "$diff_match_count" \
    --argjson diagnostics_mismatch_count "$diff_diagnostics_mismatch_count" \
    --argjson whitespace_only_output_mismatch_count "$diff_whitespace_only_output_mismatch_count" \
    --argjson output_mismatch_count "$diff_output_mismatch_count" \
    --argjson rust_failed_expected_passed_count "$diff_rust_failed_expected_passed_count" \
    --argjson reference_artifact_missing_count "$diff_reference_artifact_missing_count" \
    --argjson cases "$curated_cases_json" \
    '{
        grammar_name: $grammar_name,
        curated_corpus: $corpus,
        requested_mode: $requested_mode,
        effective_mode: $effective_mode,
        note: $note,
        cases_declared: $cases_declared,
        max_cases: $max_cases,
        cases_checked: $cases_checked,
        classification_counts: {
            expected_match: $expected_match_count,
            expected_mismatch: $expected_mismatch_count,
            bug_mismatch: $bug_mismatch_count
        },
        taxonomy_counts: {
            match: $match_count,
            diagnostics_mismatch: $diagnostics_mismatch_count,
            whitespace_only_output_mismatch: $whitespace_only_output_mismatch_count,
            output_mismatch: $output_mismatch_count,
            rust_failed_expected_passed: $rust_failed_expected_passed_count,
            reference_artifact_missing: $reference_artifact_missing_count
        },
        taxonomy_deltas: {
            non_primary_expected_count: $expected_mismatch_count,
            unexpected_category_count: $bug_mismatch_count
        },
        cases: $cases
    }' >"$REPORT_JSON"

if [[ "$DIFF_MODE" == "1" && "$diff_effective_mode" == "enabled" && "$bug_mismatch_count" -gt 0 ]]; then
    echo "error: strict curated differential mode detected bug mismatches ($bug_mismatch_count)" >&2
    cat "$REPORT_JSON" >&2
    exit 1
fi

cat >"$SUMMARY_CSV" <<EOF_SUMMARY
grammar_name,systemverilog_preprocessor
curated_corpus,$CURATED_CORPUS
diff_mode_requested,$DIFF_MODE
diff_mode_effective,$diff_effective_mode
diff_cases_declared,$diff_cases_declared
diff_cases_checked,$diff_cases_checked
classification_expected_match,$expected_match_count
classification_expected_mismatch,$expected_mismatch_count
classification_bug_mismatch,$bug_mismatch_count
diff_taxonomy_match,$diff_match_count
diff_taxonomy_diagnostics_mismatch,$diff_diagnostics_mismatch_count
diff_taxonomy_whitespace_only_output_mismatch,$diff_whitespace_only_output_mismatch_count
diff_taxonomy_output_mismatch,$diff_output_mismatch_count
diff_taxonomy_rust_failed_expected_passed,$diff_rust_failed_expected_passed_count
diff_taxonomy_reference_artifact_missing,$diff_reference_artifact_missing_count
report_json,$REPORT_JSON
EOF_SUMMARY

{
    echo "SV Preprocessor Curated Differential Gate Summary"
    echo "state_dir: $STATE_DIR"
    echo "note: $diff_note"
    echo "report_json: $REPORT_JSON"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
    echo
    echo "Logs: $LOG_DIR"
    echo "Artifacts: $WORK_DIR"
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

echo "✅ SV preprocessor curated differential gate passed."
