#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"

STATE_DIR="${PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_STATE_DIR:-$RUST_DIR/target/sv_preprocessor_template_differential_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"
REPORT_JSON="$WORK_DIR/systemverilog_preprocessor_template_differential_report.json"
CASES_JSONL="$WORK_DIR/systemverilog_preprocessor_template_cases.jsonl"

DIFF_MODE="${PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_MODE:-auto}"
CASE_COUNT="${PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_COUNT:-32}"
SEED_BASE="${PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_SEED_BASE:-13001}"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"

if [[ "$DIFF_MODE" != "auto" && "$DIFF_MODE" != "0" && "$DIFF_MODE" != "1" ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if ! [[ "$CASE_COUNT" =~ ^[0-9]+$ ]] || [[ "$CASE_COUNT" -lt 1 ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_COUNT must be an integer >= 1" >&2
    exit 2
fi
if ! [[ "$SEED_BASE" =~ ^[0-9]+$ ]]; then
    echo "error: PGEN_SV_PREPROCESSOR_TEMPLATE_DIFF_SEED_BASE must be an integer >= 0" >&2
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

build_template_case() {
    local seed="$1"
    local case_index="$2"
    local input_file="$3"
    local expected_output_file="$4"
    local expected_diag_file="$5"
    local template_name_out="$6"
    local case_label_out="$7"

    local template_id="$((seed % 4))"

    case "$template_id" in
        0)
            local width="$(((seed % 31) + 1))"
            local module_suffix="$(((seed * 7) % 10000))"
            cat >"$input_file" <<EOF_CASE
\`define WIDTH $width
module dyn_width_${module_suffix};
  logic [\`WIDTH-1:0] signal_a;
endmodule
EOF_CASE
            cat >"$expected_output_file" <<EOF_EXPECTED
module dyn_width_${module_suffix};
  logic [${width}-1:0] signal_a;
endmodule
EOF_EXPECTED
            echo "template_define_width" >"$template_name_out"
            echo "dyn_width_${module_suffix}" >"$case_label_out"
            ;;
        1)
            local symbol_suffix="$(((seed * 11) % 10000))"
            local symbol_name="USE_BRANCH_${symbol_suffix}"
            local define_enabled="$((seed % 2))"
            if [[ "$define_enabled" -eq 1 ]]; then
                cat >"$input_file" <<EOF_CASE
\`define ${symbol_name}
\`ifdef ${symbol_name}
module dyn_ifdef_true_${symbol_suffix};
endmodule
\`else
module dyn_ifdef_false_${symbol_suffix};
endmodule
\`endif
EOF_CASE
                cat >"$expected_output_file" <<EOF_EXPECTED
module dyn_ifdef_true_${symbol_suffix};
endmodule
EOF_EXPECTED
                echo "dyn_ifdef_true_${symbol_suffix}" >"$case_label_out"
            else
                cat >"$input_file" <<EOF_CASE
\`ifdef ${symbol_name}
module dyn_ifdef_true_${symbol_suffix};
endmodule
\`else
module dyn_ifdef_false_${symbol_suffix};
endmodule
\`endif
EOF_CASE
                cat >"$expected_output_file" <<EOF_EXPECTED
module dyn_ifdef_false_${symbol_suffix};
endmodule
EOF_EXPECTED
                echo "dyn_ifdef_false_${symbol_suffix}" >"$case_label_out"
            fi
            echo "template_ifdef_branch" >"$template_name_out"
            ;;
        2)
            local left_char_idx="$((seed % 26))"
            local right_char_idx="$(((seed / 26) % 26))"
            local left_char
            local right_char
            printf -v left_char "\\$(printf '%03o' $((97 + left_char_idx)))"
            printf -v right_char "\\$(printf '%03o' $((97 + right_char_idx)))"
            local module_token="${left_char}${right_char}$(((seed * 13) % 1000))"
            cat >"$input_file" <<EOF_CASE
\`define CAT(a,b) a\`\`b
module \`CAT(${left_char}${right_char},$(((seed * 13) % 1000)));
endmodule
EOF_CASE
            cat >"$expected_output_file" <<EOF_EXPECTED
module ${module_token};
endmodule
EOF_EXPECTED
            echo "template_token_paste" >"$template_name_out"
            echo "${module_token}" >"$case_label_out"
            ;;
        3)
            local macro_suffix="$(((seed * 17) % 10000))"
            local macro_name="TMP_${macro_suffix}"
            cat >"$input_file" <<EOF_CASE
\`define ${macro_name}
\`undef ${macro_name}
\`ifdef ${macro_name}
module dyn_undef_true_${macro_suffix};
endmodule
\`else
module dyn_undef_false_${macro_suffix};
endmodule
\`endif
EOF_CASE
            cat >"$expected_output_file" <<EOF_EXPECTED
module dyn_undef_false_${macro_suffix};
endmodule
EOF_EXPECTED
            echo "template_define_undef_ifdef" >"$template_name_out"
            echo "dyn_undef_false_${macro_suffix}" >"$case_label_out"
            ;;
        *)
            echo "error: unsupported template id '$template_id' for seed '$seed'" >&2
            exit 1
            ;;
    esac

    printf '[]\n' >"$expected_diag_file"
}

require_tool jq

echo "==> SV preprocessor template differential gate"
echo "state_dir: $STATE_DIR"
echo "diff_mode: $DIFF_MODE"
echo "case_count: $CASE_COUNT"
echo "seed_base: $SEED_BASE"

run_logged_rust "build_ast_pipeline_for_template_preprocessor_diff" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi

diff_effective_mode="disabled"
diff_note="template differential disabled by configuration"
cases_declared="$CASE_COUNT"
cases_checked=0

expected_match_count=0
expected_mismatch_count=0
bug_mismatch_count=0

taxonomy_match_count=0
taxonomy_diagnostics_mismatch_count=0
taxonomy_whitespace_only_output_mismatch_count=0
taxonomy_output_mismatch_count=0
taxonomy_rust_failed_expected_passed_count=0
taxonomy_reference_artifact_missing_count=0

template_define_width_count=0
template_ifdef_branch_count=0
template_token_paste_count=0
template_define_undef_ifdef_count=0

: >"$CASES_JSONL"

if [[ "$DIFF_MODE" != "0" ]]; then
    diff_effective_mode="enabled"
    diff_note="template differential classification enabled using deterministic template predictor"

    for ((i = 0; i < CASE_COUNT; i++)); do
        seed="$((SEED_BASE + i))"
        case_index="$i"

        input_file="$WORK_DIR/template_case_${case_index}.input.sv"
        expected_output_file="$WORK_DIR/template_case_${case_index}.expected.sv"
        expected_diag_file="$WORK_DIR/template_case_${case_index}.expected.diag.json"
        rust_output_file="$WORK_DIR/template_case_${case_index}.rust.out.sv"
        rust_diag_file="$WORK_DIR/template_case_${case_index}.rust.diag.json"
        rust_log_file="$LOG_DIR/template_case_${case_index}.rust.log"
        template_name_file="$WORK_DIR/template_case_${case_index}.template_name.txt"
        case_label_file="$WORK_DIR/template_case_${case_index}.case_label.txt"

        rust_output_norm="$WORK_DIR/template_case_${case_index}.rust.norm.txt"
        expected_output_norm="$WORK_DIR/template_case_${case_index}.expected.norm.txt"
        rust_diag_norm="$WORK_DIR/template_case_${case_index}.rust.diag.norm.json"
        expected_diag_norm="$WORK_DIR/template_case_${case_index}.expected.diag.norm.json"

        build_template_case \
            "$seed" \
            "$case_index" \
            "$input_file" \
            "$expected_output_file" \
            "$expected_diag_file" \
            "$template_name_file" \
            "$case_label_file"

        template_name="$(cat "$template_name_file")"
        case_label="$(cat "$case_label_file")"
        expected_categories='["match","whitespace_only_output_mismatch"]'

        case "$template_name" in
            template_define_width)
                template_define_width_count=$((template_define_width_count + 1))
                ;;
            template_ifdef_branch)
                template_ifdef_branch_count=$((template_ifdef_branch_count + 1))
                ;;
            template_token_paste)
                template_token_paste_count=$((template_token_paste_count + 1))
                ;;
            template_define_undef_ifdef)
                template_define_undef_ifdef_count=$((template_define_undef_ifdef_count + 1))
                ;;
            *)
                echo "error: unknown generated template name '$template_name'" >&2
                exit 1
                ;;
        esac

        rust_exit=0
        if "$AST_PIPELINE_BIN" "$input_file" \
            --preprocess-systemverilog \
            --output "$rust_output_file" \
            --sv-diagnostics-json "$rust_diag_file" >"$rust_log_file" 2>&1; then
            rust_exit=0
        else
            rust_exit=$?
        fi

        rust_warnings=0
        rust_errors=0
        if [[ -s "$rust_diag_file" ]] && jq -e 'type == "array"' "$rust_diag_file" >/dev/null 2>&1; then
            rust_warnings="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$rust_diag_file")"
            rust_errors="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$rust_diag_file")"
        fi

        category=""
        if (( rust_exit != 0 )); then
            category="rust_failed_expected_passed"
        elif [[ ! -f "$rust_output_file" || ! -f "$rust_diag_file" ]]; then
            category="reference_artifact_missing"
        else
            output_exact_match=0
            output_whitespace_match=0
            diagnostics_match=0

            if cmp -s "$rust_output_file" "$expected_output_file"; then
                output_exact_match=1
            fi

            normalize_text_for_diff_output "$rust_output_file" "$rust_output_norm"
            normalize_text_for_diff_output "$expected_output_file" "$expected_output_norm"
            if cmp -s "$rust_output_norm" "$expected_output_norm"; then
                output_whitespace_match=1
            fi

            canonicalize_json "$rust_diag_file" "$rust_diag_norm"
            canonicalize_json "$expected_diag_file" "$expected_diag_norm"
            if cmp -s "$rust_diag_norm" "$expected_diag_norm"; then
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
                taxonomy_match_count=$((taxonomy_match_count + 1))
                ;;
            diagnostics_mismatch)
                taxonomy_diagnostics_mismatch_count=$((taxonomy_diagnostics_mismatch_count + 1))
                ;;
            whitespace_only_output_mismatch)
                taxonomy_whitespace_only_output_mismatch_count=$((taxonomy_whitespace_only_output_mismatch_count + 1))
                ;;
            output_mismatch)
                taxonomy_output_mismatch_count=$((taxonomy_output_mismatch_count + 1))
                ;;
            rust_failed_expected_passed)
                taxonomy_rust_failed_expected_passed_count=$((taxonomy_rust_failed_expected_passed_count + 1))
                ;;
            reference_artifact_missing)
                taxonomy_reference_artifact_missing_count=$((taxonomy_reference_artifact_missing_count + 1))
                ;;
            *)
                echo "error: unknown template differential taxonomy category '$category'" >&2
                exit 1
                ;;
        esac

        classification=""
        if jq -e --arg category "$category" 'index($category) != null' <<<"$expected_categories" >/dev/null 2>&1; then
            if [[ "$category" == "match" ]]; then
                classification="expected_match"
                expected_match_count=$((expected_match_count + 1))
            else
                classification="expected_mismatch"
                expected_mismatch_count=$((expected_mismatch_count + 1))
            fi
        else
            classification="bug_mismatch"
            bug_mismatch_count=$((bug_mismatch_count + 1))
        fi

        jq -n \
            --argjson index "$case_index" \
            --argjson seed "$seed" \
            --arg template_name "$template_name" \
            --arg case_label "$case_label" \
            --arg input_file "$input_file" \
            --arg expected_output "$expected_output_file" \
            --arg expected_diagnostics "$expected_diag_file" \
            --argjson expected_categories "$expected_categories" \
            --arg observed_category "$category" \
            --arg classification "$classification" \
            --arg rust_output "$rust_output_file" \
            --arg rust_diag "$rust_diag_file" \
            --arg rust_log "$rust_log_file" \
            --argjson rust_exit "$rust_exit" \
            --argjson rust_warnings "$rust_warnings" \
            --argjson rust_errors "$rust_errors" \
            '{
                index: $index,
                seed: $seed,
                template_name: $template_name,
                case_label: $case_label,
                input_file: $input_file,
                expected_output: $expected_output,
                expected_diagnostics: $expected_diagnostics,
                expected_categories: $expected_categories,
                observed_category: $observed_category,
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

        cases_checked=$((cases_checked + 1))
    done
fi

if [[ "$DIFF_MODE" == "1" && "$diff_effective_mode" == "enabled" && "$bug_mismatch_count" -gt 0 ]]; then
    strict_failure=1
else
    strict_failure=0
fi

if [[ -s "$CASES_JSONL" ]]; then
    cases_json="$(jq -s '.' "$CASES_JSONL")"
else
    cases_json='[]'
fi

jq -n \
    --arg grammar_name "systemverilog_preprocessor" \
    --arg requested_mode "$DIFF_MODE" \
    --arg effective_mode "$diff_effective_mode" \
    --arg note "$diff_note" \
    --argjson cases_declared "$cases_declared" \
    --argjson cases_checked "$cases_checked" \
    --argjson case_count "$CASE_COUNT" \
    --argjson seed_base "$SEED_BASE" \
    --argjson expected_match_count "$expected_match_count" \
    --argjson expected_mismatch_count "$expected_mismatch_count" \
    --argjson bug_mismatch_count "$bug_mismatch_count" \
    --argjson taxonomy_match_count "$taxonomy_match_count" \
    --argjson taxonomy_diagnostics_mismatch_count "$taxonomy_diagnostics_mismatch_count" \
    --argjson taxonomy_whitespace_only_output_mismatch_count "$taxonomy_whitespace_only_output_mismatch_count" \
    --argjson taxonomy_output_mismatch_count "$taxonomy_output_mismatch_count" \
    --argjson taxonomy_rust_failed_expected_passed_count "$taxonomy_rust_failed_expected_passed_count" \
    --argjson taxonomy_reference_artifact_missing_count "$taxonomy_reference_artifact_missing_count" \
    --argjson template_define_width_count "$template_define_width_count" \
    --argjson template_ifdef_branch_count "$template_ifdef_branch_count" \
    --argjson template_token_paste_count "$template_token_paste_count" \
    --argjson template_define_undef_ifdef_count "$template_define_undef_ifdef_count" \
    --argjson strict_failure "$strict_failure" \
    --argjson cases "$cases_json" \
    '{
        grammar_name: $grammar_name,
        requested_mode: $requested_mode,
        effective_mode: $effective_mode,
        note: $note,
        generator: {
            case_count: $case_count,
            seed_base: $seed_base,
            templates: {
                template_define_width: $template_define_width_count,
                template_ifdef_branch: $template_ifdef_branch_count,
                template_token_paste: $template_token_paste_count,
                template_define_undef_ifdef: $template_define_undef_ifdef_count
            }
        },
        cases_declared: $cases_declared,
        cases_checked: $cases_checked,
        classification_counts: {
            expected_match: $expected_match_count,
            expected_mismatch: $expected_mismatch_count,
            bug_mismatch: $bug_mismatch_count
        },
        taxonomy_counts: {
            match: $taxonomy_match_count,
            diagnostics_mismatch: $taxonomy_diagnostics_mismatch_count,
            whitespace_only_output_mismatch: $taxonomy_whitespace_only_output_mismatch_count,
            output_mismatch: $taxonomy_output_mismatch_count,
            rust_failed_expected_passed: $taxonomy_rust_failed_expected_passed_count,
            reference_artifact_missing: $taxonomy_reference_artifact_missing_count
        },
        taxonomy_deltas: {
            non_primary_expected_count: $expected_mismatch_count,
            unexpected_category_count: $bug_mismatch_count
        },
        strict_failure: ($strict_failure == 1),
        cases: $cases
    }' >"$REPORT_JSON"

cat >"$SUMMARY_CSV" <<EOF_SUMMARY
grammar_name,systemverilog_preprocessor
diff_mode_requested,$DIFF_MODE
diff_mode_effective,$diff_effective_mode
case_count,$CASE_COUNT
seed_base,$SEED_BASE
cases_declared,$cases_declared
cases_checked,$cases_checked
template_define_width_count,$template_define_width_count
template_ifdef_branch_count,$template_ifdef_branch_count
template_token_paste_count,$template_token_paste_count
template_define_undef_ifdef_count,$template_define_undef_ifdef_count
classification_expected_match,$expected_match_count
classification_expected_mismatch,$expected_mismatch_count
classification_bug_mismatch,$bug_mismatch_count
diff_taxonomy_match,$taxonomy_match_count
diff_taxonomy_diagnostics_mismatch,$taxonomy_diagnostics_mismatch_count
diff_taxonomy_whitespace_only_output_mismatch,$taxonomy_whitespace_only_output_mismatch_count
diff_taxonomy_output_mismatch,$taxonomy_output_mismatch_count
diff_taxonomy_rust_failed_expected_passed,$taxonomy_rust_failed_expected_passed_count
diff_taxonomy_reference_artifact_missing,$taxonomy_reference_artifact_missing_count
report_json,$REPORT_JSON
EOF_SUMMARY

{
    echo "SV Preprocessor Template Differential Gate Summary"
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

if [[ "$strict_failure" -eq 1 ]]; then
    echo "error: strict template differential mode detected bug mismatches ($bug_mismatch_count)" >&2
    exit 1
fi

echo "✅ SV preprocessor template differential gate passed."
