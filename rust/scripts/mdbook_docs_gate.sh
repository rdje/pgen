#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ROOT_DIR="$(cd "${RUST_DIR}/.." && pwd)"
REPORT_DIR="${PGEN_MDBOOK_DOCS_GATE_REPORT_DIR:-${RUST_DIR}/target/mdbook_docs_gate}"
SUMMARY_TXT="${REPORT_DIR}/summary.txt"
BUILD_LOG="${REPORT_DIR}/mdbook_build.log"

mkdir -p "${REPORT_DIR}"

fail() {
  echo "error: $*" >&2
  exit 1
}

require_tool() {
  local tool="$1"
  command -v "$tool" >/dev/null 2>&1 || fail "required tool not found on PATH: $tool"
}

: >"${SUMMARY_TXT}"
echo "PGEN mdBook Docs Gate" >>"${SUMMARY_TXT}"
echo "report_dir: ${REPORT_DIR}" >>"${SUMMARY_TXT}"
echo "book_root: ${ROOT_DIR}/docs/book" >>"${SUMMARY_TXT}"
echo >>"${SUMMARY_TXT}"

require_tool mdbook

for repo_file in \
  "${ROOT_DIR}/docs/book/book.toml" \
  "${ROOT_DIR}/docs/book/src/SUMMARY.md" \
  "${ROOT_DIR}/README.md" \
  "${ROOT_DIR}/PGEN_USER_GUIDE.md"; do
  [[ -f "${repo_file}" ]] || fail "required docs surface missing: ${repo_file}"
done

echo "==> mdbook_build" | tee -a "${SUMMARY_TXT}"
if (
  cd "${ROOT_DIR}"
  mdbook build docs/book
) >"${BUILD_LOG}" 2>&1; then
  echo "pass: mdbook_build (${BUILD_LOG})" | tee -a "${SUMMARY_TXT}"
else
  echo "fail: mdbook_build (${BUILD_LOG})" | tee -a "${SUMMARY_TXT}" >&2
  tail -n 120 "${BUILD_LOG}" >&2 || true
  exit 1
fi

echo >>"${SUMMARY_TXT}"
echo "✅ mdBook docs gate passed." | tee -a "${SUMMARY_TXT}"
