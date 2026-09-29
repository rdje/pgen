#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ROOT_DIR="$(cd "${RUST_DIR}/.." && pwd)"
REPORT_DIR="${PGEN_SYSTEMVERILOG_PARSER_BOOK_GATE_REPORT_DIR:-${RUST_DIR}/target/systemverilog_parser_book_gate}"
SUMMARY_TXT="${REPORT_DIR}/summary.txt"
BUILD_LOG="${REPORT_DIR}/mdbook_build.log"
BOOK_ROOT="${ROOT_DIR}/docs/systemverilog_parser_book"

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
echo "PGEN SystemVerilog Parser Book Gate" >>"${SUMMARY_TXT}"
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
  "${BOOK_ROOT}/src/parse-content-variants.md" \
  "${BOOK_ROOT}/src/json-carrier.md" \
  "${BOOK_ROOT}/src/walking-the-ast.md" \
  "${BOOK_ROOT}/src/rules-top-level.md" \
  "${BOOK_ROOT}/src/examples-minimal-module.md" \
  "${BOOK_ROOT}/src/schema-versioning.md" \
  "${BOOK_ROOT}/src/glossary.md" \
  "${BOOK_ROOT}/src/changelog-index.md"; do
  [[ -f "${repo_file}" ]] || fail "required systemverilog parser book surface missing: ${repo_file}"
done

echo "==> mdbook_build" | tee -a "${SUMMARY_TXT}"
if (
  cd "${ROOT_DIR}"
  mdbook build docs/systemverilog_parser_book
) >"${BUILD_LOG}" 2>&1; then
  echo "pass: mdbook_build (${BUILD_LOG})" | tee -a "${SUMMARY_TXT}"
else
  echo "fail: mdbook_build (${BUILD_LOG})" | tee -a "${SUMMARY_TXT}" >&2
  tail -n 120 "${BUILD_LOG}" >&2 || true
  exit 1
fi

# SV-CORPUS-GRAD.36 (2026-09-29): THE CARRIER TABLE NAMES LIVE RULES. `json-carrier.md`'s `| Rule | … |`
# tables are the director's AST-shape reference; a row's first code span is a rule of
# `grammars/systemverilog.ebnf`, or the row says the rule was retired (`deleted in <release>` /
# `retired in <release>`). Measured 2026-09-29: two rows had drifted with the grammar and nothing read
# them — `method_call_receiver_qualifier` (deleted by SV-CORPUS-GRAD.33 in 1.0.246, still described as
# live) and `strength` (split into `strength0` / `strength1` with suffixed kind labels; the row kept
# SV-Slice-54's pre-split name). The arm scopes to tables whose header's first cell is `Rule`, so the
# directive-kind and version tables of the same page are not read as rule names, and it proves it can
# fail on every run: a planted row naming a rule the grammar does not define must be refused.
carrier_table_rule_names() {  # $1 page  $2 grammar
  python3 - "$1" "$2" <<'PYCHECK'
import re, sys
page, grammar = sys.argv[1], sys.argv[2]
rules = set(re.findall(r'^([A-Za-z_][A-Za-z0-9_]*)\s*:?=', open(grammar, encoding='utf-8').read(), flags=re.M))
in_rule_table = False
rows = live = retired = 0
unknown = []
for lineno, line in enumerate(open(page, encoding='utf-8').read().split('\n'), 1):
    if not line.startswith('|'):
        in_rule_table = False
        continue
    cells = [c.strip() for c in line.strip().strip('|').split('|')]
    if not cells:
        continue
    if re.fullmatch(r':?-+:?', cells[0]):
        continue
    if cells[0] == 'Rule':
        in_rule_table = True
        continue
    if not in_rule_table:
        continue
    first = cells[0]
    if first.startswith('_(none yet)_'):
        continue
    rows += 1
    match = re.search(r'`([A-Za-z_][A-Za-z0-9_*]*)`', first)
    if not match:
        unknown.append((lineno, '<no rule name>', first[:70]))
        continue
    name = match.group(1)
    if re.search(r'\b(retired|deleted) in `?\d', first):
        retired += 1
        continue
    if '*' in name:
        # a token FAMILY row (`kw_dir_*`, 14 tokens): the glob must name at least one rule
        glob = re.compile('^' + re.escape(name).replace('\\*', '[A-Za-z0-9_]*') + '$')
        if any(glob.match(rule) for rule in rules):
            live += 1
        else:
            unknown.append((lineno, name, first[:70]))
        continue
    if name in rules:
        live += 1
    else:
        unknown.append((lineno, name, first[:70]))
print(f"CARRIER-TABLE-RULES: page={page} rows={rows} live={live} retired={retired} unknown={len(unknown)}")
for lineno, name, first in unknown:
    print(f"  unknown rule name at line {lineno}: `{name}` -- {first}")
sys.exit(1 if unknown else 0)
PYCHECK
}
echo "==> carrier_table_rule_names" | tee -a "${SUMMARY_TXT}"
if ! carrier_table_rule_names "${BOOK_ROOT}/src/json-carrier.md" "${ROOT_DIR}/grammars/systemverilog.ebnf" | tee -a "${SUMMARY_TXT}"; then
  echo "fail: carrier_table_rule_names (a carrier-table row names a rule grammars/systemverilog.ebnf does not define and does not say the rule was retired)" | tee -a "${SUMMARY_TXT}" >&2
  exit 1
fi
red_copy="${REPORT_DIR}/json-carrier.red-control.md"
{
  cat "${BOOK_ROOT}/src/json-carrier.md"
  printf '\n| Rule | Annotation | JSON shape produced |\n|---|---|---|\n| `no_such_rule_planted_by_the_gate` | passthrough | RED control |\n'
} >"${red_copy}"
if carrier_table_rule_names "${red_copy}" "${ROOT_DIR}/grammars/systemverilog.ebnf" >/dev/null 2>&1; then
  echo "fail: carrier_table_rule_names RED control -- the arm accepted a planted row naming an undefined rule" | tee -a "${SUMMARY_TXT}" >&2
  exit 1
fi
echo "pass: carrier_table_rule_names (RED control: the planted undefined rule was refused)" | tee -a "${SUMMARY_TXT}"

# The rendered HTML at docs/systemverilog_parser_book-html/ is TRACKED in git
# so the book is browsable directly on GitHub. Verify the canonical landing
# pages are present after build.
HTML_ROOT="${ROOT_DIR}/docs/systemverilog_parser_book-html"
echo "==> tracked_html_check" | tee -a "${SUMMARY_TXT}"
for landing in \
  "${HTML_ROOT}/index.html" \
  "${HTML_ROOT}/welcome.html" \
  "${HTML_ROOT}/ast-envelope.html"; do
  if [[ ! -f "${landing}" ]]; then
    echo "fail: tracked_html_check (missing ${landing})" | tee -a "${SUMMARY_TXT}" >&2
    exit 1
  fi
done
echo "pass: tracked_html_check (HTML output present at ${HTML_ROOT})" | tee -a "${SUMMARY_TXT}"

echo >>"${SUMMARY_TXT}"
echo "✅ systemverilog parser book gate passed." | tee -a "${SUMMARY_TXT}"
