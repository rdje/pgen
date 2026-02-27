#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"

STATE_DIR="${PGEN_SV_STIMULI_QUALITY_STATE_DIR:-$RUST_DIR/target/sv_stimuli_quality_gate}"
LOG_DIR="$STATE_DIR/logs"
WORK_DIR="$STATE_DIR/work"
SUMMARY_CSV="$STATE_DIR/summary.csv"
SUMMARY_TXT="$STATE_DIR/summary.txt"

CONTRACT_FILE="${PGEN_SV_STIMULI_QUALITY_CONTRACT:-$RUST_DIR/test_data/grammar_quality/systemverilog_core_v0_contract.json}"
PARSE_FULL_MODE="${PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE:-auto}"
SAMPLE_COUNT_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_COUNT:-}"
SEED_BASE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_SEED_BASE:-}"
LRM_PROFILE_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_LRM_PROFILE:-}"
LRM_PROFILES_OVERRIDE="${PGEN_SV_STIMULI_QUALITY_LRM_PROFILES:-}"

AST_PIPELINE_BIN="$RUST_DIR/target/debug/ast_pipeline"
PARSE_PROBE_BIN="$RUST_DIR/target/debug/parseability_probe"
EBNF_TO_JSON="$TOOLS_DIR/ebnf_to_json.pl"

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

check_balanced_structural_keywords() {
    local file="$1"
    local pair
    for pair in \
        "module:endmodule" \
        "interface:endinterface" \
        "program:endprogram" \
        "package:endpackage" \
        "class:endclass" \
        "begin:end"; do
        local open_kw="${pair%%:*}"
        local close_kw="${pair##*:}"
        local open_count
        local close_count
        open_count="$(perl -ne "while (/\\b\\Q${open_kw}\\E\\b/g) { ++\$c } END { print(\$c // 0) }" "$file")"
        close_count="$(perl -ne "while (/\\b\\Q${close_kw}\\E\\b/g) { ++\$c } END { print(\$c // 0) }" "$file")"
        if [[ "$open_count" != "$close_count" ]]; then
            echo "unbalanced structural keywords: ${open_kw}=${open_count}, ${close_kw}=${close_count}"
            return 1
        fi
    done
    return 0
}

check_unique_named_port_bindings() {
    local file="$1"
    local out
    if ! out="$(
        perl -0777 -e '
            use strict;
            use warnings;
            local $/;
            my $text = <>;
            my @stmts = split /;/, $text;
            my $stmt_idx = 0;
            for my $stmt (@stmts) {
                $stmt_idx++;
                my @names = ($stmt =~ /\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(/g);
                next unless @names > 1;
                my %seen;
                for my $name (@names) {
                    if ($seen{$name}++) {
                        print "duplicate named port binding \"$name\" in statement $stmt_idx\n";
                        exit 1;
                    }
                }
            }
            exit 0;
        ' "$file" 2>&1
    )"; then
        echo "$out"
        return 1
    fi
    return 0
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

if [[ "$PARSE_FULL_MODE" != "auto" && "$PARSE_FULL_MODE" != "0" && "$PARSE_FULL_MODE" != "1" ]]; then
    echo "error: PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE must be one of: auto, 0, 1" >&2
    exit 2
fi
if [[ -n "$LRM_PROFILE_OVERRIDE" && -n "$LRM_PROFILES_OVERRIDE" ]]; then
    echo "error: set either PGEN_SV_STIMULI_QUALITY_LRM_PROFILE or PGEN_SV_STIMULI_QUALITY_LRM_PROFILES, not both" >&2
    exit 2
fi

mkdir -p "$LOG_DIR" "$WORK_DIR"

require_tool jq
require_tool perl
require_file "$CONTRACT_FILE"
require_file "$EBNF_TO_JSON"

contract_version="$(jq -er '.version | numbers' "$CONTRACT_FILE")"
grammar_name="$(jq -er '.grammar_name | strings' "$CONTRACT_FILE")"
ebnf_path_rel="$(jq -er '.ebnf_path | strings' "$CONTRACT_FILE")"
default_sample_count="$(jq -er '.sample_count | numbers' "$CONTRACT_FILE")"
default_seed_base="$(jq -er '.seed_base | numbers' "$CONTRACT_FILE")"
default_lrm_profile="$(jq -er '(.lrm_profiles.default_profile // "2017") | strings' "$CONTRACT_FILE")"
supported_lrm_profiles_csv="$(jq -er '(.lrm_profiles.supported_profiles // ["2017","2023"]) | map(select(type=="string")) | join(",")' "$CONTRACT_FILE")"
required_lrm_profiles_csv="$(jq -er '(.lrm_profiles.required_profiles // [(.lrm_profiles.default_profile // "2017")]) | map(select(type=="string")) | join(",")' "$CONTRACT_FILE")"

sample_count="${SAMPLE_COUNT_OVERRIDE:-$default_sample_count}"
seed_base="${SEED_BASE_OVERRIDE:-$default_seed_base}"

if ! [[ "$sample_count" =~ ^[0-9]+$ ]] || [[ "$sample_count" -lt 1 ]]; then
    echo "error: sample_count must be an integer >= 1 (effective value: '$sample_count')" >&2
    exit 2
fi
if ! [[ "$seed_base" =~ ^[0-9]+$ ]]; then
    echo "error: seed_base must be an integer >= 0 (effective value: '$seed_base')" >&2
    exit 2
fi

include_max_depth="$(jq -er '.preprocess.include_max_depth | numbers' "$CONTRACT_FILE")"
include_path_policy="$(jq -er '.preprocess.include_path_policy | strings' "$CONTRACT_FILE")"
macro_redefine_policy="$(jq -er '.preprocess.macro_redefine_policy | strings' "$CONTRACT_FILE")"
conditional_symbol_policy="$(jq -er '.preprocess.conditional_symbol_policy | strings' "$CONTRACT_FILE")"
conditional_expr_policy="$(jq -er '.preprocess.conditional_expr_policy | strings' "$CONTRACT_FILE")"
strict_warning_codes="$(jq -er '.preprocess.strict_warning_codes | strings' "$CONTRACT_FILE")"

require_nonempty_preprocessed_output="$(jq -er 'if .semantic_baseline.require_nonempty_preprocessed_output then 1 else 0 end' "$CONTRACT_FILE")"
require_no_preprocess_errors="$(jq -er 'if .semantic_baseline.require_no_preprocess_errors then 1 else 0 end' "$CONTRACT_FILE")"
require_balanced_structural_keywords="$(jq -er 'if .semantic_baseline.require_balanced_structural_keywords then 1 else 0 end' "$CONTRACT_FILE")"
require_unique_named_port_bindings="$(jq -er 'if .semantic_baseline.require_unique_named_port_bindings then 1 else 0 end' "$CONTRACT_FILE")"

if ! [[ "$include_max_depth" =~ ^[0-9]+$ ]] || [[ "$include_max_depth" -lt 1 ]]; then
    echo "error: preprocess.include_max_depth must be an integer >= 1" >&2
    exit 2
fi

grammar_file="$ROOT_DIR/$ebnf_path_rel"
grammar_json="$WORK_DIR/${grammar_name}.json"
parser_out="$WORK_DIR/${grammar_name}_parser.rs"

require_file "$grammar_file"

profiles_csv="$required_lrm_profiles_csv"
if [[ -n "$LRM_PROFILE_OVERRIDE" ]]; then
    profiles_csv="$LRM_PROFILE_OVERRIDE"
elif [[ -n "$LRM_PROFILES_OVERRIDE" ]]; then
    profiles_csv="$LRM_PROFILES_OVERRIDE"
fi

declare -a supported_profiles=()
declare -A supported_profiles_map=()
IFS=',' read -r -a _supported_raw <<< "$supported_lrm_profiles_csv"
for _p in "${_supported_raw[@]}"; do
    p="$(echo "$_p" | tr -d '[:space:]')"
    if [[ -n "$p" ]]; then
        supported_profiles+=("$p")
        supported_profiles_map["$p"]=1
    fi
done
if [[ "${#supported_profiles[@]}" -eq 0 ]]; then
    echo "error: no supported lrm profiles defined in contract" >&2
    exit 2
fi

declare -a run_profiles=()
declare -A run_profiles_map=()
IFS=',' read -r -a _run_raw <<< "$profiles_csv"
for _p in "${_run_raw[@]}"; do
    p="$(echo "$_p" | tr -d '[:space:]')"
    [[ -n "$p" ]] || continue
    if [[ -n "${run_profiles_map[$p]:-}" ]]; then
        continue
    fi
    if [[ -z "${supported_profiles_map[$p]:-}" ]]; then
        echo "error: unsupported lrm profile '$p' (supported: ${supported_profiles[*]})" >&2
        exit 2
    fi
    run_profiles+=("$p")
    run_profiles_map["$p"]=1
done
if [[ "${#run_profiles[@]}" -eq 0 ]]; then
    echo "error: no runnable lrm profile selected" >&2
    exit 2
fi

echo "==> SystemVerilog stimuli quality gate (skeleton)"
echo "state_dir: $STATE_DIR"
echo "contract_file: $CONTRACT_FILE"
echo "contract_version: $contract_version"
echo "grammar_name: $grammar_name"
echo "grammar_file: $grammar_file"
echo "sample_count: $sample_count"
echo "seed_base: $seed_base"
echo "parse_full_mode: $PARSE_FULL_MODE"
echo "lrm_default_profile: $default_lrm_profile"
echo "lrm_supported_profiles: ${supported_profiles[*]}"
echo "lrm_run_profiles: ${run_profiles[*]}"

echo "profile,sample,seed,stimuli_generate,preprocess,semantic_validate,parse_full,warnings,errors,status,notes" >"$SUMMARY_CSV"

run_logged_rust "build_ast_pipeline_for_sv_generation" \
    cargo build --features generated_parsers --bin ast_pipeline

if [[ ! -x "$AST_PIPELINE_BIN" ]]; then
    echo "error: ast_pipeline binary is missing at '$AST_PIPELINE_BIN' after build" >&2
    exit 1
fi
run_logged "frontend_ebnf_to_json" \
    perl "$EBNF_TO_JSON" --pretty --quiet "$grammar_file" -o "$grammar_json"
require_nonempty_file "$grammar_json"

run_logged "generate_sv_parser" \
    "$AST_PIPELINE_BIN" "$grammar_json" \
    --generate-parser \
    --eliminate-left-recursion \
    --output "$parser_out"
require_nonempty_file "$parser_out"

run_logged_rust "build_parseability_probe_with_systemverilog_adapter" \
    env PGEN_SYSTEMVERILOG_PARSER_PATH="$parser_out" \
    cargo build --features generated_parsers --bin parseability_probe
if [[ ! -x "$PARSE_PROBE_BIN" ]]; then
    echo "error: parseability_probe binary is missing at '$PARSE_PROBE_BIN' after adapter build" >&2
    exit 1
fi

parse_full_supported=0
probe_log="$LOG_DIR/probe_parse_full_support.log"
echo "==> probe_parse_full_support"
if "$PARSE_PROBE_BIN" --supports "$grammar_name" >"$probe_log" 2>&1; then
    echo "    ok (${probe_log})"
    parse_full_supported=1
else
    echo "    skip (${probe_log})"
fi

parse_full_enabled=0
parse_full_effective="disabled"
if [[ "$PARSE_FULL_MODE" == "0" ]]; then
    parse_full_enabled=0
    parse_full_effective="disabled_by_mode"
elif [[ "$PARSE_FULL_MODE" == "1" ]]; then
    if [[ "$parse_full_supported" -eq 0 ]]; then
        echo "error: parse_full mode is strict (1) but no generated parser adapter is registered for '$grammar_name'" >&2
        exit 1
    fi
    parse_full_enabled=1
    parse_full_effective="enabled"
else
    if [[ "$parse_full_supported" -eq 1 ]]; then
        parse_full_enabled=1
        parse_full_effective="enabled"
    else
        parse_full_enabled=0
        parse_full_effective="unsupported_adapter"
    fi
fi

semantic_pass_count=0
parse_full_pass_count=0
parse_full_skip_count=0
parse_full_fail_count=0
total_warnings=0
total_errors=0
profile_count="${#run_profiles[@]}"
total_samples=$((sample_count * profile_count))

for profile_idx in "${!run_profiles[@]}"; do
    lrm_profile="${run_profiles[$profile_idx]}"
    profile_seed_base=$((seed_base + (profile_idx * 1000000)))
    profile_key="${lrm_profile//[^A-Za-z0-9_]/_}"

    for ((idx = 0; idx < sample_count; idx++)); do
        seed=$((profile_seed_base + idx))
        sample_file="$WORK_DIR/sample_${profile_key}_${idx}.sv"
        preprocessed_file="$WORK_DIR/sample_${profile_key}_${idx}.preprocessed.sv"
        diagnostics_json="$WORK_DIR/sample_${profile_key}_${idx}.diagnostics.json"

        run_logged "sample_${profile_key}_${idx}_generate_stimulus" \
            "$AST_PIPELINE_BIN" "$grammar_json" \
            --generate-stimuli \
            --count 1 \
            --seed "$seed" \
            --output "$sample_file"
        require_nonempty_file "$sample_file"

        run_logged "sample_${profile_key}_${idx}_preprocess" \
            "$AST_PIPELINE_BIN" "$sample_file" \
            --preprocess-systemverilog \
            --output "$preprocessed_file" \
            --sv-diagnostics-json "$diagnostics_json" \
            --sv-include-max-depth "$include_max_depth" \
            --sv-include-path-policy "$include_path_policy" \
            --sv-macro-redefine-policy "$macro_redefine_policy" \
            --sv-conditional-symbol-policy "$conditional_symbol_policy" \
            --sv-conditional-expr-policy "$conditional_expr_policy" \
            --sv-strict-warning-codes "$strict_warning_codes"

        require_file "$diagnostics_json"
        warning_count="$(jq -er '[.[] | select(.severity == "warning")] | length | numbers' "$diagnostics_json")"
        error_count="$(jq -er '[.[] | select(.severity == "error")] | length | numbers' "$diagnostics_json")"
        total_warnings=$((total_warnings + warning_count))
        total_errors=$((total_errors + error_count))

        semantic_status="pass"
        semantic_note="baseline semantic validation passed"

        if [[ "$require_nonempty_preprocessed_output" -eq 1 ]] && [[ ! -s "$preprocessed_file" ]]; then
            semantic_status="fail"
            semantic_note="preprocessed output is empty"
        fi
        if [[ "$require_no_preprocess_errors" -eq 1 ]] && (( error_count > 0 )); then
            semantic_status="fail"
            semantic_note="preprocessor diagnostics contain error severity entries"
        fi
        if [[ "$semantic_status" == "pass" ]] && [[ "$require_balanced_structural_keywords" -eq 1 ]]; then
            if ! semantic_note="$(check_balanced_structural_keywords "$preprocessed_file")"; then
                semantic_status="fail"
            fi
        fi
        if [[ "$semantic_status" == "pass" ]] && [[ "$require_unique_named_port_bindings" -eq 1 ]]; then
            if ! semantic_note="$(check_unique_named_port_bindings "$preprocessed_file")"; then
                semantic_status="fail"
            fi
        fi

        if [[ "$semantic_status" != "pass" ]]; then
            echo "${lrm_profile},${idx},${seed},pass,pass,fail,skip,${warning_count},${error_count},fail,${semantic_note}" >>"$SUMMARY_CSV"
            echo "error: semantic baseline validation failed for profile '${lrm_profile}' sample_${idx}: ${semantic_note}" >&2
            exit 1
        fi
        semantic_pass_count=$((semantic_pass_count + 1))

        parse_status="skip"
        parse_note="parse_full stage skipped"
        if [[ "$parse_full_enabled" -eq 1 ]]; then
            parse_log="$LOG_DIR/sample_${profile_key}_${idx}_parse_full.log"
            echo "==> sample_${profile_key}_${idx}_parse_full"
            if "$PARSE_PROBE_BIN" --parse "$grammar_name" "$preprocessed_file" >"$parse_log" 2>&1; then
                echo "    ok (${parse_log})"
                parse_status="pass"
                parse_note="parse_full accepted preprocessed sample"
                parse_full_pass_count=$((parse_full_pass_count + 1))
            else
                parse_status="fail"
                parse_note="parse_full rejected preprocessed sample"
                parse_full_fail_count=$((parse_full_fail_count + 1))
                if [[ "$PARSE_FULL_MODE" == "1" ]]; then
                    echo "    fail (${parse_log})" >&2
                    tail -n 80 "$parse_log" >&2 || true
                    echo "error: strict parse_full mode requires all samples to pass parse_full" >&2
                    exit 1
                fi
                echo "    soft-fail (${parse_log})"
            fi
        else
            parse_full_skip_count=$((parse_full_skip_count + 1))
            parse_note="parse_full unavailable (${parse_full_effective})"
        fi

        echo "${lrm_profile},${idx},${seed},pass,pass,${semantic_status},${parse_status},${warning_count},${error_count},pass,${parse_note}" >>"$SUMMARY_CSV"
    done
done

{
    echo "PGEN SV Stimuli Quality Gate Summary (Skeleton)"
    echo "state_dir: $STATE_DIR"
    echo "contract_file: $CONTRACT_FILE"
    echo "grammar_name: $grammar_name"
    echo "sample_count: $sample_count"
    echo "profile_count: $profile_count"
    echo "run_profiles: ${run_profiles[*]}"
    echo "seed_base: $seed_base"
    echo "parse_full_mode: $PARSE_FULL_MODE"
    echo "parse_full_effective: $parse_full_effective"
    echo "semantic_baseline_passes: $semantic_pass_count/$total_samples"
    echo "parse_full_passes: $parse_full_pass_count/$total_samples"
    echo "parse_full_failures: $parse_full_fail_count"
    echo "parse_full_skips: $parse_full_skip_count"
    echo "total_warnings: $total_warnings"
    echo "total_errors: $total_errors"
    echo
    if command -v column >/dev/null 2>&1; then
        column -s, -t "$SUMMARY_CSV"
    else
        cat "$SUMMARY_CSV"
    fi
} >"$SUMMARY_TXT"

cat "$SUMMARY_TXT"

cat <<EOF
✅ SV stimuli quality gate skeleton passed.
Logs: $LOG_DIR
Artifacts: $WORK_DIR
EOF
