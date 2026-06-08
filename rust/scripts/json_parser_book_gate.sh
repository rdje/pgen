#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ROOT_DIR="$(cd "${RUST_DIR}/.." && pwd)"
REPORT_DIR="${PGEN_JSON_PARSER_BOOK_GATE_REPORT_DIR:-${RUST_DIR}/target/json_parser_book_gate}"
SUMMARY_TXT="${REPORT_DIR}/summary.txt"
BUILD_LOG="${REPORT_DIR}/mdbook_build.log"
BOOK_ROOT="${ROOT_DIR}/docs/json_parser_book"

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
echo "PGEN JSON Parser Book Gate" >>"${SUMMARY_TXT}"
echo "report_dir: ${REPORT_DIR}" >>"${SUMMARY_TXT}"
echo "book_root: ${BOOK_ROOT}" >>"${SUMMARY_TXT}"
echo >>"${SUMMARY_TXT}"

require_tool mdbook

for repo_file in \
  "${BOOK_ROOT}/book.toml" \
  "${BOOK_ROOT}/src/SUMMARY.md" \
  "${BOOK_ROOT}/src/welcome.md" \
  "${BOOK_ROOT}/src/build-recipe.md" \
  "${BOOK_ROOT}/src/grammar-and-scope.md" \
  "${BOOK_ROOT}/src/ast-envelope.md" \
  "${BOOK_ROOT}/src/external-corpus-characterization.md" \
  "${BOOK_ROOT}/src/glossary.md"; do
  [[ -f "${repo_file}" ]] || fail "required json parser book surface missing: ${repo_file}"
done

echo "==> mdbook_build" | tee -a "${SUMMARY_TXT}"
if (
  cd "${ROOT_DIR}"
  mdbook build docs/json_parser_book
) >"${BUILD_LOG}" 2>&1; then
  echo "pass: mdbook_build (${BUILD_LOG})" | tee -a "${SUMMARY_TXT}"
else
  echo "fail: mdbook_build (${BUILD_LOG})" | tee -a "${SUMMARY_TXT}" >&2
  tail -n 120 "${BUILD_LOG}" >&2 || true
  exit 1
fi

echo "json parser book gate: pass" | tee -a "${SUMMARY_TXT}"
