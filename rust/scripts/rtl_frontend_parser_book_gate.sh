#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ROOT_DIR="$(cd "${RUST_DIR}/.." && pwd)"
REPORT_DIR="${PGEN_RTL_FRONTEND_PARSER_BOOK_GATE_REPORT_DIR:-${RUST_DIR}/target/rtl_frontend_parser_book_gate}"
SUMMARY_TXT="${REPORT_DIR}/summary.txt"
BUILD_LOG="${REPORT_DIR}/mdbook_build.log"
BOOK_ROOT="${ROOT_DIR}/docs/rtl_frontend_parser_book"

mkdir -p "${REPORT_DIR}"

fail() { echo "error: $*" >&2; exit 1; }
require_tool() { command -v "$1" >/dev/null 2>&1 || fail "required tool not found on PATH: $1"; }

: >"${SUMMARY_TXT}"
echo "PGEN rtl_frontend Parser Book Gate" >>"${SUMMARY_TXT}"
echo "report_dir: ${REPORT_DIR}" >>"${SUMMARY_TXT}"
echo "book_root: ${BOOK_ROOT}" >>"${SUMMARY_TXT}"
echo >>"${SUMMARY_TXT}"

require_tool mdbook

for repo_file in \
  "${BOOK_ROOT}/book.toml" \
  "${BOOK_ROOT}/src/SUMMARY.md" \
  "${BOOK_ROOT}/src/welcome.md" \
  "${BOOK_ROOT}/src/quickstart.md" \
  "${BOOK_ROOT}/src/build-recipe.md" \
  "${BOOK_ROOT}/src/public-api.md" \
  "${BOOK_ROOT}/src/ast-envelope.md" \
  "${BOOK_ROOT}/src/json-carrier.md" \
  "${BOOK_ROOT}/src/walking-the-ast.md" \
  "${BOOK_ROOT}/src/rules-top-level.md" \
  "${BOOK_ROOT}/src/examples-empty-module.md" \
  "${BOOK_ROOT}/src/schema-versioning.md" \
  "${BOOK_ROOT}/src/glossary.md" \
  "${BOOK_ROOT}/src/changelog-index.md"; do
  [[ -f "${repo_file}" ]] || fail "required rtl_frontend parser book surface missing: ${repo_file}"
done

echo "==> mdbook_build" | tee -a "${SUMMARY_TXT}"
if (cd "${ROOT_DIR}" && mdbook build docs/rtl_frontend_parser_book) >"${BUILD_LOG}" 2>&1; then
  echo "pass: mdbook_build (${BUILD_LOG})" | tee -a "${SUMMARY_TXT}"
else
  echo "fail: mdbook_build (${BUILD_LOG})" | tee -a "${SUMMARY_TXT}" >&2
  tail -n 120 "${BUILD_LOG}" >&2 || true
  exit 1
fi

HTML_ROOT="${ROOT_DIR}/docs/rtl_frontend_parser_book-html"
echo "==> tracked_html_check" | tee -a "${SUMMARY_TXT}"
for landing in "${HTML_ROOT}/index.html" "${HTML_ROOT}/welcome.html" "${HTML_ROOT}/ast-envelope.html"; do
  if [[ ! -f "${landing}" ]]; then
    echo "fail: tracked_html_check (missing ${landing})" | tee -a "${SUMMARY_TXT}" >&2
    exit 1
  fi
done
echo "pass: tracked_html_check (HTML output present at ${HTML_ROOT})" | tee -a "${SUMMARY_TXT}"

echo >>"${SUMMARY_TXT}"
echo "✅ rtl_frontend parser book gate passed." | tee -a "${SUMMARY_TXT}"
