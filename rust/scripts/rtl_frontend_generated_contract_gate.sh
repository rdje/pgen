#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPORT_DIR="${PGEN_RTL_FRONTEND_GENERATED_CONTRACT_REPORT_DIR:-${RUST_DIR}/target/rtl_frontend_generated_contract_gate}"
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
echo "PGEN rtl_frontend Generated Contract Gate" >>"${SUMMARY_TXT}"
echo "report_dir: ${REPORT_DIR}" >>"${SUMMARY_TXT}"
echo >>"${SUMMARY_TXT}"

run_stage "rtl_frontend_generated_contract_probe" \
    cargo run --features generated_parsers --bin rtl_frontend_generated_contract_probe

echo >>"${SUMMARY_TXT}"
echo "✅ rtl_frontend generated contract gate passed." | tee -a "${SUMMARY_TXT}"
