#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

REPORT_DIR="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REPORT_DIR:-${RUST_DIR}/target/stimuli_cross_family_platform_gate}"
SUMMARY_TXT="${REPORT_DIR}/summary.txt"
SUMMARY_JSON="${REPORT_DIR}/summary.json"

REGEX_STATE_DIR="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REGEX_STATE_DIR:-${REPORT_DIR}/regex}"
REGEX_CONTRACT_FILE="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REGEX_CONTRACT_FILE:-${RUST_DIR}/test_data/grammar_quality/regex_family_stimuli_contract.json}"
REGEX_COUNT="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REGEX_COUNT:-4}"
REGEX_TARGET_MAX_ATTEMPTS="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REGEX_TARGET_MAX_ATTEMPTS:-1000}"

VHDL_STATE_DIR="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_VHDL_STATE_DIR:-${REPORT_DIR}/vhdl}"
VHDL_CONTRACT_FILE="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_VHDL_CONTRACT_FILE:-${RUST_DIR}/test_data/grammar_quality/vhdl_stimuli_cross_family_platform_contract_v0.json}"
VHDL_TARGET_MAX_ATTEMPTS="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_VHDL_TARGET_MAX_ATTEMPTS:-200}"

SV_STATE_DIR="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_SV_STATE_DIR:-${REPORT_DIR}/systemverilog}"
SV_CONTRACT_FILE="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_SV_CONTRACT_FILE:-${RUST_DIR}/test_data/grammar_quality/systemverilog_stimuli_cross_family_platform_contract_v0.json}"
SV_TARGET_MAX_ATTEMPTS="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_SV_TARGET_MAX_ATTEMPTS:-50}"
SV_LRM_PROFILES="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_SV_LRM_PROFILES:-2017}"
CROSS_FAMILY_CARGO_BUILD_JOBS="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_CARGO_BUILD_JOBS:-1}"

require_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool '$tool' is not available in PATH" >&2
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

extract_summary_value() {
    local path="$1"
    local key="$2"
    awk -v key="$key" 'index($0, key ": ") == 1 { print substr($0, length(key) + 3); exit 0 }' "$path"
}

csv_field_from_first_data_row() {
    local path="$1"
    local field="$2"
    awk -F',' -v field="$field" '
        NR == 1 {
            for (i = 1; i <= NF; ++i) {
                if ($i == field) {
                    idx = i
                }
            }
            next
        }
        NR == 2 {
            if (idx < 1) {
                exit 2
            }
            print $idx
            exit 0
        }
    ' "$path"
}

if ! [[ "${CROSS_FAMILY_CARGO_BUILD_JOBS}" =~ ^[0-9]+$ ]] || [[ "${CROSS_FAMILY_CARGO_BUILD_JOBS}" -lt 1 ]]; then
    echo "error: PGEN_STIMULI_CROSS_FAMILY_PLATFORM_CARGO_BUILD_JOBS must be an integer >= 1" >&2
    exit 2
fi

mkdir -p "${REPORT_DIR}"
require_tool jq

generated_at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

run_stage() {
    local name="$1"
    shift
    local log_file="${REPORT_DIR}/${name}.log"
    echo "==> ${name}" | tee -a "${SUMMARY_TXT}"
    if (
        cd "${RUST_DIR}"
        "$@"
    ) >"${log_file}" 2>&1; then
        echo "pass: ${name} (${log_file})" | tee -a "${SUMMARY_TXT}"
        return 0
    fi

    echo "fail: ${name} (${log_file})" | tee -a "${SUMMARY_TXT}" >&2
    tail -n 80 "${log_file}" >&2 || true
    return 1
}

: >"${SUMMARY_TXT}"
echo "PGEN Stimuli Cross-Family Platform Gate" >>"${SUMMARY_TXT}"
echo "generated_at_utc: ${generated_at_utc}" >>"${SUMMARY_TXT}"
echo "report_dir: ${REPORT_DIR}" >>"${SUMMARY_TXT}"
echo "summary_json: ${SUMMARY_JSON}" >>"${SUMMARY_TXT}"
echo "regex_state_dir: ${REGEX_STATE_DIR}" >>"${SUMMARY_TXT}"
echo "regex_contract_file: ${REGEX_CONTRACT_FILE}" >>"${SUMMARY_TXT}"
echo "regex_count: ${REGEX_COUNT}" >>"${SUMMARY_TXT}"
echo "regex_target_max_attempts: ${REGEX_TARGET_MAX_ATTEMPTS}" >>"${SUMMARY_TXT}"
echo "vhdl_state_dir: ${VHDL_STATE_DIR}" >>"${SUMMARY_TXT}"
echo "vhdl_contract_file: ${VHDL_CONTRACT_FILE}" >>"${SUMMARY_TXT}"
echo "vhdl_target_max_attempts: ${VHDL_TARGET_MAX_ATTEMPTS}" >>"${SUMMARY_TXT}"
echo "sv_state_dir: ${SV_STATE_DIR}" >>"${SUMMARY_TXT}"
echo "sv_contract_file: ${SV_CONTRACT_FILE}" >>"${SUMMARY_TXT}"
echo "sv_target_max_attempts: ${SV_TARGET_MAX_ATTEMPTS}" >>"${SUMMARY_TXT}"
echo "sv_lrm_profiles: ${SV_LRM_PROFILES}" >>"${SUMMARY_TXT}"
echo "cargo_build_jobs: ${CROSS_FAMILY_CARGO_BUILD_JOBS}" >>"${SUMMARY_TXT}"
echo >>"${SUMMARY_TXT}"

run_stage "regex_family_stimuli_quality" \
    env \
    CARGO_BUILD_JOBS="${CROSS_FAMILY_CARGO_BUILD_JOBS}" \
    PGEN_EBNF_STIMULI_QUALITY_STATE_DIR="${REGEX_STATE_DIR}" \
    PGEN_EBNF_STIMULI_QUALITY_CONTRACT="${REGEX_CONTRACT_FILE}" \
    PGEN_EBNF_STIMULI_QUALITY_COUNT="${REGEX_COUNT}" \
    PGEN_EBNF_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${REGEX_TARGET_MAX_ATTEMPTS}" \
    bash ./scripts/ebnf_stimuli_quality_gate.sh

run_stage "vhdl_stimuli_quality_bounded" \
    env \
    CARGO_BUILD_JOBS="${CROSS_FAMILY_CARGO_BUILD_JOBS}" \
    PGEN_VHDL_STIMULI_QUALITY_STATE_DIR="${VHDL_STATE_DIR}" \
    PGEN_VHDL_STIMULI_QUALITY_CONTRACT="${VHDL_CONTRACT_FILE}" \
    PGEN_VHDL_STIMULI_CARGO_TARGET_DIR=target \
    PGEN_VHDL_STIMULI_QUALITY_PARSE_FULL_MODE=0 \
    PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MODE=0 \
    PGEN_VHDL_STIMULI_REALISTIC_CORPUS_MAX_CASES=0 \
    PGEN_VHDL_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${VHDL_TARGET_MAX_ATTEMPTS}" \
    bash ./scripts/vhdl_stimuli_quality_gate.sh

run_stage "sv_stimuli_quality_bounded" \
    env \
    CARGO_BUILD_JOBS="${CROSS_FAMILY_CARGO_BUILD_JOBS}" \
    PGEN_SV_STIMULI_QUALITY_STATE_DIR="${SV_STATE_DIR}" \
    PGEN_SV_STIMULI_QUALITY_CONTRACT="${SV_CONTRACT_FILE}" \
    PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 \
    PGEN_SV_STIMULI_QUALITY_LRM_PROFILES="${SV_LRM_PROFILES}" \
    PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 \
    PGEN_SV_STIMULI_DIFF_MODE=0 \
    PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 \
    PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE=0 \
    PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${SV_TARGET_MAX_ATTEMPTS}" \
    PGEN_SV_STIMULI_CARGO_BUILD_JOBS="${CROSS_FAMILY_CARGO_BUILD_JOBS}" \
    bash ./scripts/sv_stimuli_quality_gate.sh

REGEX_SUMMARY_TXT="${REGEX_STATE_DIR}/summary.txt"
REGEX_SUMMARY_CSV="${REGEX_STATE_DIR}/summary.csv"
VHDL_SUMMARY_TXT="${VHDL_STATE_DIR}/summary.txt"
VHDL_SUMMARY_CSV="${VHDL_STATE_DIR}/summary.csv"
SV_SUMMARY_TXT="${SV_STATE_DIR}/summary.txt"
SV_SUMMARY_CSV="${SV_STATE_DIR}/summary.csv"

require_nonempty_file "${REGEX_SUMMARY_TXT}"
require_nonempty_file "${REGEX_SUMMARY_CSV}"
require_nonempty_file "${VHDL_SUMMARY_TXT}"
require_nonempty_file "${VHDL_SUMMARY_CSV}"
require_nonempty_file "${SV_SUMMARY_TXT}"
require_nonempty_file "${SV_SUMMARY_CSV}"

REGEX_STAGE_LOG="${REPORT_DIR}/regex_family_stimuli_quality.log"
VHDL_STAGE_LOG="${REPORT_DIR}/vhdl_stimuli_quality_bounded.log"
SV_STAGE_LOG="${REPORT_DIR}/sv_stimuli_quality_bounded.log"

regex_grammar="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "grammar")"
regex_grammar_name="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "grammar_name")"
regex_parseability_attempts_total="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "parseability_attempts_total")"
regex_parseability_accepted_total="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "parseability_accepted_total")"
regex_parseability_rejected_total="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "parseability_rejected_total")"
regex_parseability_acceptance_rate_percent="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "parseability_acceptance_rate_percent")"
regex_initial_targets="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "initial_targets")"
regex_resolved_targets="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "resolved_targets")"
regex_final_targets="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "final_targets")"
regex_status="$(csv_field_from_first_data_row "${REGEX_SUMMARY_CSV}" "status")"

vhdl_grammar_name="$(extract_summary_value "${VHDL_SUMMARY_TXT}" "grammar_name")"
vhdl_entry_rule="$(extract_summary_value "${VHDL_SUMMARY_TXT}" "entry_rule")"
vhdl_sample_count="$(extract_summary_value "${VHDL_SUMMARY_TXT}" "sample_count")"
vhdl_seed_base="$(extract_summary_value "${VHDL_SUMMARY_TXT}" "seed_base")"
vhdl_closed_loop_initial_targets="$(extract_summary_value "${VHDL_SUMMARY_TXT}" "closed_loop_initial_targets")"
vhdl_closed_loop_replay_targets="$(extract_summary_value "${VHDL_SUMMARY_TXT}" "closed_loop_replay_targets")"
vhdl_closed_loop_parseability_shadow_enabled="$(extract_summary_value "${VHDL_SUMMARY_TXT}" "closed_loop_parseability_shadow_enabled")"
vhdl_closed_loop_parseability_shadow_acceptance_rate_percent="$(extract_summary_value "${VHDL_SUMMARY_TXT}" "closed_loop_parseability_shadow_acceptance_rate_percent")"
vhdl_parse_full_effective="$(extract_summary_value "${VHDL_SUMMARY_TXT}" "parse_full_effective")"
vhdl_status="pass"

sv_grammar_name="$(extract_summary_value "${SV_SUMMARY_TXT}" "grammar_name")"
sv_sample_count="$(extract_summary_value "${SV_SUMMARY_TXT}" "sample_count")"
sv_profile_count="$(extract_summary_value "${SV_SUMMARY_TXT}" "profile_count")"
sv_run_profiles="$(extract_summary_value "${SV_SUMMARY_TXT}" "run_profiles")"
sv_seed_base="$(extract_summary_value "${SV_SUMMARY_TXT}" "seed_base")"
sv_closed_loop_profiles_passed="$(extract_summary_value "${SV_SUMMARY_TXT}" "closed_loop_profiles_passed")"
sv_closed_loop_initial_targets_total="$(extract_summary_value "${SV_SUMMARY_TXT}" "closed_loop_initial_targets_total")"
sv_closed_loop_replay_targets_total="$(extract_summary_value "${SV_SUMMARY_TXT}" "closed_loop_replay_targets_total")"
sv_closed_loop_parseability_shadow_effective="$(extract_summary_value "${SV_SUMMARY_TXT}" "closed_loop_parseability_shadow_effective")"
sv_closed_loop_parseability_shadow_enabled="$(extract_summary_value "${SV_SUMMARY_TXT}" "closed_loop_parseability_shadow_enabled")"
sv_default_mode="$(jq -er '.stimuli_modes.default_mode | strings' "${SV_CONTRACT_FILE}")"
sv_status="pass"

jq -n \
    --arg gate "stimuli_cross_family_platform_gate" \
    --arg status "pass" \
    --arg generated_at_utc "${generated_at_utc}" \
    --arg report_dir "${REPORT_DIR}" \
    --arg summary_txt "${SUMMARY_TXT}" \
    --arg summary_json "${SUMMARY_JSON}" \
    --argjson cargo_build_jobs "${CROSS_FAMILY_CARGO_BUILD_JOBS}" \
    --arg regex_stage_log "${REGEX_STAGE_LOG}" \
    --arg vhdl_stage_log "${VHDL_STAGE_LOG}" \
    --arg sv_stage_log "${SV_STAGE_LOG}" \
    --arg regex_state_dir "${REGEX_STATE_DIR}" \
    --arg regex_summary_txt "${REGEX_SUMMARY_TXT}" \
    --arg regex_summary_csv "${REGEX_SUMMARY_CSV}" \
    --arg regex_contract_file "${REGEX_CONTRACT_FILE}" \
    --arg regex_grammar "${regex_grammar}" \
    --arg regex_grammar_name "${regex_grammar_name}" \
    --argjson regex_count "${REGEX_COUNT}" \
    --argjson regex_target_max_attempts "${REGEX_TARGET_MAX_ATTEMPTS}" \
    --argjson regex_parseability_attempts_total "${regex_parseability_attempts_total}" \
    --argjson regex_parseability_accepted_total "${regex_parseability_accepted_total}" \
    --argjson regex_parseability_rejected_total "${regex_parseability_rejected_total}" \
    --argjson regex_parseability_acceptance_rate_percent "${regex_parseability_acceptance_rate_percent}" \
    --argjson regex_initial_targets "${regex_initial_targets}" \
    --argjson regex_resolved_targets "${regex_resolved_targets}" \
    --argjson regex_final_targets "${regex_final_targets}" \
    --arg regex_status "${regex_status}" \
    --arg vhdl_state_dir "${VHDL_STATE_DIR}" \
    --arg vhdl_summary_txt "${VHDL_SUMMARY_TXT}" \
    --arg vhdl_summary_csv "${VHDL_SUMMARY_CSV}" \
    --arg vhdl_contract_file "${VHDL_CONTRACT_FILE}" \
    --arg vhdl_grammar_name "${vhdl_grammar_name}" \
    --arg vhdl_entry_rule "${vhdl_entry_rule}" \
    --argjson vhdl_target_max_attempts "${VHDL_TARGET_MAX_ATTEMPTS}" \
    --argjson vhdl_sample_count "${vhdl_sample_count}" \
    --argjson vhdl_seed_base "${vhdl_seed_base}" \
    --argjson vhdl_closed_loop_initial_targets "${vhdl_closed_loop_initial_targets}" \
    --argjson vhdl_closed_loop_replay_targets "${vhdl_closed_loop_replay_targets}" \
    --argjson vhdl_closed_loop_parseability_shadow_enabled "${vhdl_closed_loop_parseability_shadow_enabled}" \
    --argjson vhdl_closed_loop_parseability_shadow_acceptance_rate_percent "${vhdl_closed_loop_parseability_shadow_acceptance_rate_percent}" \
    --arg vhdl_parse_full_effective "${vhdl_parse_full_effective}" \
    --arg vhdl_status "${vhdl_status}" \
    --arg sv_state_dir "${SV_STATE_DIR}" \
    --arg sv_summary_txt "${SV_SUMMARY_TXT}" \
    --arg sv_summary_csv "${SV_SUMMARY_CSV}" \
    --arg sv_contract_file "${SV_CONTRACT_FILE}" \
    --arg sv_grammar_name "${sv_grammar_name}" \
    --arg sv_default_mode "${sv_default_mode}" \
    --arg sv_lrm_profiles "${SV_LRM_PROFILES}" \
    --argjson sv_target_max_attempts "${SV_TARGET_MAX_ATTEMPTS}" \
    --argjson sv_sample_count "${sv_sample_count}" \
    --argjson sv_profile_count "${sv_profile_count}" \
    --arg sv_run_profiles "${sv_run_profiles}" \
    --argjson sv_seed_base "${sv_seed_base}" \
    --arg sv_closed_loop_profiles_passed "${sv_closed_loop_profiles_passed}" \
    --argjson sv_closed_loop_initial_targets_total "${sv_closed_loop_initial_targets_total}" \
    --argjson sv_closed_loop_replay_targets_total "${sv_closed_loop_replay_targets_total}" \
    --argjson sv_closed_loop_parseability_shadow_enabled "${sv_closed_loop_parseability_shadow_enabled}" \
    --arg sv_closed_loop_parseability_shadow_effective "${sv_closed_loop_parseability_shadow_effective}" \
    --arg sv_status "${sv_status}" \
    '{
        gate: $gate,
        status: $status,
        generated_at_utc: $generated_at_utc,
        report_dir: $report_dir,
        summary_txt: $summary_txt,
        summary_json: $summary_json,
        config: {
            cargo_build_jobs: $cargo_build_jobs,
            regex: {
                count: $regex_count,
                target_max_attempts: $regex_target_max_attempts
            },
            vhdl: {
                target_max_attempts: $vhdl_target_max_attempts
            },
            systemverilog: {
                target_max_attempts: $sv_target_max_attempts,
                lrm_profiles: $sv_lrm_profiles,
                default_mode: $sv_default_mode
            }
        },
        stages: [
            {
                name: "regex_family_stimuli_quality",
                status: $regex_status,
                log: $regex_stage_log,
                state_dir: $regex_state_dir,
                summary_txt: $regex_summary_txt,
                summary_csv: $regex_summary_csv,
                contract_file: $regex_contract_file
            },
            {
                name: "vhdl_stimuli_quality_bounded",
                status: $vhdl_status,
                log: $vhdl_stage_log,
                state_dir: $vhdl_state_dir,
                summary_txt: $vhdl_summary_txt,
                summary_csv: $vhdl_summary_csv,
                contract_file: $vhdl_contract_file
            },
            {
                name: "sv_stimuli_quality_bounded",
                status: $sv_status,
                log: $sv_stage_log,
                state_dir: $sv_state_dir,
                summary_txt: $sv_summary_txt,
                summary_csv: $sv_summary_csv,
                contract_file: $sv_contract_file
            }
        ],
        families: {
            regex: {
                grammar: $regex_grammar,
                grammar_name: $regex_grammar_name,
                state_dir: $regex_state_dir,
                summary_txt: $regex_summary_txt,
                summary_csv: $regex_summary_csv,
                contract_file: $regex_contract_file,
                parseability_attempts_total: $regex_parseability_attempts_total,
                parseability_accepted_total: $regex_parseability_accepted_total,
                parseability_rejected_total: $regex_parseability_rejected_total,
                parseability_acceptance_rate_percent: $regex_parseability_acceptance_rate_percent,
                initial_targets: $regex_initial_targets,
                resolved_targets: $regex_resolved_targets,
                final_targets: $regex_final_targets
            },
            vhdl: {
                grammar_name: $vhdl_grammar_name,
                entry_rule: $vhdl_entry_rule,
                state_dir: $vhdl_state_dir,
                summary_txt: $vhdl_summary_txt,
                summary_csv: $vhdl_summary_csv,
                contract_file: $vhdl_contract_file,
                sample_count: $vhdl_sample_count,
                seed_base: $vhdl_seed_base,
                parse_full_effective: $vhdl_parse_full_effective,
                closed_loop_initial_targets: $vhdl_closed_loop_initial_targets,
                closed_loop_replay_targets: $vhdl_closed_loop_replay_targets,
                closed_loop_parseability_shadow_enabled: $vhdl_closed_loop_parseability_shadow_enabled,
                closed_loop_parseability_shadow_acceptance_rate_percent: $vhdl_closed_loop_parseability_shadow_acceptance_rate_percent
            },
            systemverilog: {
                grammar_name: $sv_grammar_name,
                state_dir: $sv_state_dir,
                summary_txt: $sv_summary_txt,
                summary_csv: $sv_summary_csv,
                contract_file: $sv_contract_file,
                stimuli_mode: $sv_default_mode,
                sample_count: $sv_sample_count,
                profile_count: $sv_profile_count,
                run_profiles: $sv_run_profiles,
                seed_base: $sv_seed_base,
                closed_loop_profiles_passed: $sv_closed_loop_profiles_passed,
                closed_loop_initial_targets_total: $sv_closed_loop_initial_targets_total,
                closed_loop_replay_targets_total: $sv_closed_loop_replay_targets_total,
                closed_loop_parseability_shadow_enabled: $sv_closed_loop_parseability_shadow_enabled,
                closed_loop_parseability_shadow_effective: $sv_closed_loop_parseability_shadow_effective
            }
        }
    }' >"${SUMMARY_JSON}"

echo >>"${SUMMARY_TXT}"
echo "✅ stimuli cross-family platform gate passed." | tee -a "${SUMMARY_TXT}"
echo "summary_json: ${SUMMARY_JSON}" | tee -a "${SUMMARY_TXT}"
