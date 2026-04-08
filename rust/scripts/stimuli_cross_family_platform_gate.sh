#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

REPORT_DIR="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REPORT_DIR:-${RUST_DIR}/target/stimuli_cross_family_platform_gate}"
SUMMARY_TXT="${REPORT_DIR}/summary.txt"

REGEX_STATE_DIR="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REGEX_STATE_DIR:-${REPORT_DIR}/regex}"
REGEX_CONTRACT_FILE="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REGEX_CONTRACT_FILE:-${RUST_DIR}/test_data/grammar_quality/regex_family_stimuli_contract.json}"
REGEX_COUNT="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REGEX_COUNT:-4}"
REGEX_TARGET_MAX_ATTEMPTS="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_REGEX_TARGET_MAX_ATTEMPTS:-1000}"

VHDL_STATE_DIR="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_VHDL_STATE_DIR:-${REPORT_DIR}/vhdl}"
VHDL_CONTRACT_FILE="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_VHDL_CONTRACT_FILE:-${RUST_DIR}/test_data/grammar_quality/vhdl_stimuli_cross_family_platform_contract_v0.json}"
VHDL_TARGET_MAX_ATTEMPTS="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_VHDL_TARGET_MAX_ATTEMPTS:-200}"

SV_STATE_DIR="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_SV_STATE_DIR:-${REPORT_DIR}/systemverilog}"
SV_CONTRACT_FILE="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_SV_CONTRACT_FILE:-${RUST_DIR}/test_data/grammar_quality/systemverilog_stimuli_cross_family_platform_contract_v0.json}"
SV_TARGET_MAX_ATTEMPTS="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_SV_TARGET_MAX_ATTEMPTS:-200}"
SV_LRM_PROFILES="${PGEN_STIMULI_CROSS_FAMILY_PLATFORM_SV_LRM_PROFILES:-2017}"

mkdir -p "${REPORT_DIR}"

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
echo "report_dir: ${REPORT_DIR}" >>"${SUMMARY_TXT}"
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
echo >>"${SUMMARY_TXT}"

run_stage "regex_family_stimuli_quality" \
    env \
    PGEN_EBNF_STIMULI_QUALITY_STATE_DIR="${REGEX_STATE_DIR}" \
    PGEN_EBNF_STIMULI_QUALITY_CONTRACT="${REGEX_CONTRACT_FILE}" \
    PGEN_EBNF_STIMULI_QUALITY_COUNT="${REGEX_COUNT}" \
    PGEN_EBNF_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${REGEX_TARGET_MAX_ATTEMPTS}" \
    bash ./scripts/ebnf_stimuli_quality_gate.sh

run_stage "vhdl_stimuli_quality_bounded" \
    env \
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
    PGEN_SV_STIMULI_QUALITY_STATE_DIR="${SV_STATE_DIR}" \
    PGEN_SV_STIMULI_QUALITY_CONTRACT="${SV_CONTRACT_FILE}" \
    PGEN_SV_STIMULI_QUALITY_PARSE_FULL_MODE=0 \
    PGEN_SV_STIMULI_QUALITY_LRM_PROFILES="${SV_LRM_PROFILES}" \
    PGEN_SV_STIMULI_REALISTIC_CORPUS_MODE=0 \
    PGEN_SV_STIMULI_DIFF_MODE=0 \
    PGEN_SV_STIMULI_PERF_BUDGET_MODE=0 \
    PGEN_SV_STIMULI_QUALITY_SEMANTIC_CLOSURE_MODE=0 \
    PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS="${SV_TARGET_MAX_ATTEMPTS}" \
    bash ./scripts/sv_stimuli_quality_gate.sh

echo >>"${SUMMARY_TXT}"
echo "✅ stimuli cross-family platform gate passed." | tee -a "${SUMMARY_TXT}"
