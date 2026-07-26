#!/usr/bin/env bash
# LANG-CAPABILITY-AUDIT.3b — measure the `.3` matrix's ❓ UNMEASURED rows.
#
#   bash docs/tasks/artifacts/lang_capability_audit/run_capability_probes.sh
#
# WHAT IT DOES
#   Drives two probe banks through the PARSE-HARNESS **scratch slot** (TOOLBOX 1.3
#   — authoritative BY CONSTRUCTION: the real register → codegen → drive pipeline),
#   one bank per grammar-level layout policy. Each row is parsed in ISOLATION via
#   `--entry-rule`, so the banks' top-level alternation can never launder one row's
#   verdict into another's.
#
# COST / SIDE EFFECTS (honest, not silent)
#   Overwrites `grammars/scratch/scratch.ebnf` (the blessed throwaway slot) and
#   rebuilds the DEBUG `parseability_probe` ONCE PER BANK (~1-2 min each). The
#   scratch fixture is RESTORED (`git checkout`) on exit, including on interrupt.
#   The DEBUG probe is used deliberately: parse verdicts are build-mode-independent
#   and it avoids the release fat-LTO relink.
#
# READING THE OUTPUT
#   Every line is `ACCEPT`/`REJECT` + the case id + the input. `[want]` marks the
#   verdict the capability requires; a line whose verdict differs from `[want]` is
#   a MEASURED GAP, not a formatting accident.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../../.." && pwd)"
cd "$REPO" || exit 1
PROBE="$REPO/rust/target/debug/parseability_probe"
SLOT="$REPO/grammars/scratch/scratch.ebnf"
TMP="$(mktemp -d)"
trap 'git checkout -- "$SLOT" 2>/dev/null; rm -rf "$TMP"' EXIT

build_bank() {
  local bank="$1"
  echo "### building bank: $(basename "$bank")"
  cp "$bank" "$SLOT"
  make -C rust SHELL=/bin/bash focus_scratch >/dev/null 2>&1 || { echo "  focus_scratch FAILED"; return 1; }
  scripts/run_with_memory_guard.sh --budget-mb 12288 --timeout-s 3600 -- \
    bash -c "cd rust && cargo build --features generated_parsers --bin parseability_probe" \
    >"$TMP/build.log" 2>&1 || { echo "  build FAILED (see $TMP/build.log)"; return 1; }
  echo "    ok"
  echo
}

# run <case-id> <input(%b-escaped)> <entry-rule> <want:ACCEPT|REJECT>
run() {
  printf '%b' "$2" > "$TMP/in.txt"
  local rc verdict
  "$PROBE" --parse scratch "$TMP/in.txt" --entry-rule "$3" >/dev/null 2>&1
  rc=$?
  if [ $rc -eq 0 ]; then verdict=ACCEPT; else verdict=REJECT; fi
  local mark="   "
  [ "$verdict" != "$4" ] && mark=" ⛔"
  printf '  %-6s [want %-6s]%s %-22s %s\n' "$verdict" "$4" "$mark" "$1" "\"$2\""
}

echo "feature surface: $("$REPO/rust/target/debug/ast_pipeline" --report-feature-surface 2>&1 | head -1)"
echo

build_bank "$HERE/probes/probe_default_layout.ebnf" || exit 1

echo "=== ROW 0 (CONTROL) — the DEFAULT branch policy, measured not assumed ==="
echo "    ('a | a b' on \"ab\": first-match-commit strands the b; longest-match keeps the longer)"
run r0-default-ab  'ab' r00_policy  ACCEPT
run r0-ordered-ab  'ab' r00_ordered REJECT
echo

echo "=== ROW 8 — CONTEXTUAL / SOFT KEYWORDS (identifier token deliberately UN-taxed) ==="
run r8-match-stmt  'match x:'  r08_stmt ACCEPT
run r8-call        'match(x)'  r08_stmt ACCEPT
run r8-assign      'match = 1' r08_stmt ACCEPT
run r8-plain       'foo = 1'   r08_stmt ACCEPT
echo

echo "=== ROW 9 — COVER GRAMMARS / DELAYED DISAMBIGUATION ==="
run r9-paren       '(a,b)'             r09_expr ACCEPT
run r9-arrow       '(a,b)=>c'          r09_expr ACCEPT
run r9-paren-deep  '(a=(((x))),b)'     r09_expr ACCEPT
run r9-arrow-deep  '(a=(((x))),b)=>c'  r09_expr ACCEPT
echo

echo "=== ROW 10 PRE-REQUISITE — does '*' give back a successful iteration? ==="
run r10-bt-ab      'ab'    r10_bt ACCEPT
run r10-bt-aaab    'aaab'  r10_bt ACCEPT
echo

echo "=== ROW 10 — STATIC delimiter (the negative-lookahead PEG idiom) ==="
run r10-static-ok    'q/abc/' r10_static ACCEPT
run r10-static-empty 'q//'    r10_static ACCEPT
echo

echo "=== ROW 10 — OPEN-SET delimiter, naive value_compare over an unguarded body ==="
run r10-same-slash  'q/abc/' r10_same ACCEPT
run r10-same-bang   'q!abc!' r10_same ACCEPT
run r10-same-mixed  'q/abc!' r10_same REJECT
echo

echo "=== ROW 10 — MIRRORED delimiters, enumerated (CLOSED set only) ==="
run r10-mirror-brace 'q{abc}' r10_mirror ACCEPT
run r10-mirror-brack 'q[abc]' r10_mirror ACCEPT
echo

echo "=== ROW 10 — OPEN-SET delimiter via the semantic store (DYNAMIC guard) ==="
run r10-dyn-slash  'q/abc/'  r10_dyn ACCEPT
run r10-dyn-bang   'q!abc!'  r10_dyn ACCEPT
run r10-dyn-hash   'q#abc#'  r10_dyn ACCEPT
run r10-dyn-inner  'q/ab!c/' r10_dyn ACCEPT
run r10-dyn-empty  'q//'     r10_dyn ACCEPT
run r10-dyn-mixed  'q/abc!'  r10_dyn REJECT
echo

echo "=== ROW 10 — the BOUND: the fact store is parse-GLOBAL and monotone ==="
run r10-two-ok           'q/abc/q!de!'  r10_dyn_pair  ACCEPT
run r10-two-inner-slash  'q/abc/q!d/e!' r10_dyn_pair  ACCEPT
run r10-two-wrong-close  'q/abc/q!de/'  r10_dyn_pair  REJECT
echo "  -- same three under @open_scope/@close_scope (the repair attempt) --"
run r10-sdyn-single      'q/abc/'       r10_sdyn      ACCEPT
run r10-sdyn-two         'q/abc/q!de!'  r10_sdyn_pair ACCEPT
run r10-sdyn-inner-slash 'q/abc/q!d/e!' r10_sdyn_pair ACCEPT
run r10-sdyn-wrong-close 'q/abc/q!de/'  r10_sdyn_pair REJECT
echo

echo "=== ROW 13(a) — UNICODE IDENTIFIER CLASSES ==="
run r13-ascii       'abc'   r13_uident ACCEPT
run r13-latin1      'café'  r13_uident ACCEPT
run r13-cjk         '变量'   r13_uident ACCEPT
run r13-ligature    'ﬁle'   r13_uident ACCEPT
run r13-digit-start '1abc'  r13_uident REJECT
echo

build_bank "$HERE/probes/probe_ws_sensitive.ebnf" || exit 1

echo "=== ROW 11 — HERE-DOCUMENTS (terminator named at parse time) ==="
run r11-basic     '<<EOF\nline1\nline2\nEOF\n'     r11_heredoc ACCEPT
run r11-empty     '<<EOF\nEOF\n'                    r11_heredoc ACCEPT
run r11-lookalike '<<END\nline1\nEOF\nline2\nEND\n' r11_heredoc ACCEPT
run r11-unterm    '<<EOF\nline1\n'                  r11_heredoc REJECT
run r11-wrongtag  '<<EOF\nline1\nEND\n'             r11_heredoc REJECT
echo
echo "=== ROW 11 — the SAME parse-global-store bound, on line structure ==="
run r11-two-ok    '<<A\nx\nA\n<<B\ny\nB\n'    r11_pair ACCEPT
run r11-two-cross '<<A\nx\nA\n<<B\nA\ny\nB\n' r11_pair ACCEPT
run r11-two-wrong '<<A\nx\nA\n<<B\ny\nA\n'    r11_pair REJECT
echo
echo "(scratch slot restored on exit)"
