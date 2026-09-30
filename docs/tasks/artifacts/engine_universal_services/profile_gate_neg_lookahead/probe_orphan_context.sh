#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.47 (b) — the orphan arm's context control, asserted.
#
# Runs `--lint-grammar` on `default_profile_orphan_control.ebnf` and requires the CORRECTED
# reading: one profile orphan (`a` under `pcre2`), a universe that contains the `@default_profile`,
# and a non-zero exit — the ERROR-class arm blocking as designed. Before `.47` the same command read
# `profile_orphans=0 (error; profiles=[])` at rc 0 (recorded in the grammar's header), which is the
# RED direction this control pins: an instrument that reads an empty universe and says "clean".
#
# Also re-reads the `.46`(b) lookahead control, whose universe the same context now feeds.
#
# Usage (from the repository root):  bash <this script>        # exit 0 iff every assertion holds
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../../../../.." && pwd)"
cd "$REPO_ROOT" || exit 2
AST_PIPELINE="${PGEN_AST_PIPELINE:-rust/target/debug/ast_pipeline}"
[[ -x "$AST_PIPELINE" ]] || { echo "missing $AST_PIPELINE — build it: (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)" >&2; exit 2; }
fail=0
check() {  # $1 label, $2 grammar, $3 expected headline fragment, $4 expected rc
  local out rc
  out="$("$AST_PIPELINE" "$2" --lint-grammar 2>&1)"; rc=$?
  local head; head="$(printf '%s\n' "$out" | grep -oE 'profile_orphans=[0-9]+ \([^)]*\)' | head -1)"
  if [[ "$head" == "$3" && "$rc" == "$4" ]]; then
    echo "  ✓ $1: $head rc=$rc"
  else
    echo "  ✗ $1: read '$head' rc=$rc — expected '$3' rc=$4" >&2; fail=1
  fi
  [[ "$1" == orphan* ]] && printf '%s\n' "$out" | grep -E 'profile orphan|ProfileOrphan|orphan' | head -3 | sed 's/^/      /'
}
check "orphan control (.47)"    "$HERE/default_profile_orphan_control.ebnf"   'profile_orphans=1 (error; profiles=["pcre2", "relaxed"])' 1
check "lookahead control (.46)" "$HERE/default_profile_universe_control.ebnf" 'profile_orphans=0 (error; profiles=["pcre2", "relaxed"])' 0
if [[ $fail -eq 0 ]]; then echo "ORPHAN-CONTEXT-CONTROL: OK (2/2 as declared)"; else echo "ORPHAN-CONTEXT-CONTROL: FAIL" >&2; fi
exit $fail
