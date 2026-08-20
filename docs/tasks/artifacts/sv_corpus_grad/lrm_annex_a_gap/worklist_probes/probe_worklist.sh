#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2v slice 1 — adjudicate the census's `sv_source_syntax` worklist.
#
# One witness per clause-only production the census classes as SystemVerilog SOURCE syntax, run on
# both LRM editions.  ⚠️ HONEST BOUND: a PASS here says the production is REACHABLE, never that
# every form it admits parses.  The row that FAILED when this was first run is `scope_randomize`
# (IEEE 1800 Syntax 18-11), fixed in PGEN-SV-CORPUS-GRAD-0244 as ledger SV-0065.
#
# Usage: bash docs/tasks/artifacts/sv_corpus_grad/lrm_annex_a_gap/worklist_probes/probe_worklist.sh
# Exit 0 = every witness parses on every edition.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../../.." && pwd)"
cd "$ROOT" || exit 2
HERE="docs/tasks/artifacts/sv_corpus_grad/lrm_annex_a_gap/worklist_probes"
PIPELINE="rust/target/debug/ast_pipeline"
GRAMMAR="grammars/systemverilog.ebnf"

[ -f "$HERE/w5_scope_randomize.sv" ] || { echo "⛔ ROOT resolved to $ROOT, which holds no probes"; exit 2; }
[ -x "$PIPELINE" ] || { echo "⛔ $PIPELINE is not built — a probe that cannot run must SAY SO"; exit 2; }

fail=0
for f in "$HERE"/w*.sv; do
  for prof in sv_2017 sv_2023; do
    v=$("$PIPELINE" "$GRAMMAR" --interpret-parse "$f" --grammar-profile "$prof" 2>&1 \
          | grep -o 'accepted=[a-z]*' | cut -d= -f2)
    printf '%-38s %-8s %s\n' "$(basename "$f")" "$prof" "$v"
    [ "$v" = "true" ] || fail=1
  done
done
echo "LRM-ANNEX-A-GAP-WORKLIST: $([ $fail -eq 0 ] && echo 'all witnesses parse' || echo 'FAILURES above')"
exit $fail
