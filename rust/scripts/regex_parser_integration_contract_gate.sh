#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPORT_DIR="${PGEN_REGEX_PARSER_INTEGRATION_CONTRACT_REPORT_DIR:-${RUST_DIR}/target/regex_parser_integration_contract_gate}"
SUMMARY_TXT="${REPORT_DIR}/summary.txt"

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
    return 1
}

: >"${SUMMARY_TXT}"
echo "PGEN Regex Parser Integration Contract Gate" >>"${SUMMARY_TXT}"
echo "report_dir: ${REPORT_DIR}" >>"${SUMMARY_TXT}"
echo >>"${SUMMARY_TXT}"

run_stage "bootstrap_regex_parser_integration_contract_tests" \
    cargo test --lib regex_parser_integration_contract_

run_stage "generated_regex_parser_integration_contract_tests" \
    cargo test --features generated_parsers --lib regex_parser_integration_contract_

echo >>"${SUMMARY_TXT}"
echo "✅ Regex parser integration contract gate passed." | tee -a "${SUMMARY_TXT}"
