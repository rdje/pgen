#!/usr/bin/env bash
# GRAMMAR-WELLFORMED.H.16.4a — the layout-owning-terminal probe.
#
# Three arms, all re-runnable from a clean tree in well under a minute:
#
#   A. CENSUS — the closed population of regex terminals that OWN their leading layout
#      (whitespace-only AND cannot match empty), over every grammar the frontend can load.
#      ⛔ HIR-authoritative: `ast_pipeline --report-layout-owning-terminals` calls codegen's
#      OWN predicate (`AstBasedGenerator::regex_pattern_layout_facts`), never a regex over
#      grammar text. The inherited census in `H.16.4a` was a text sweep and was wrong in two
#      directions — it invented a second `systemverilog_lrm_profiled_*` row and missed
#      `systemverilog_preprocessor`'s `newline := /\r?\n/`, which no `\s` search can see.
#
#   B. REPRODUCER — `ebnf`'s `whitespace := /(\s+)/` on an input of four spaces. Before the
#      fix the alternation ran ZERO iterations (`elements: []`, span 0..0) because
#      `consume_layout_for_regex` ate the bytes first; after it, the rule commits.
#
#   C. CONTROL — the arm that REFUTED the declarative tier. All four settings of the
#      grammar-wide `@whitespace_sensitive:` directive stop `grammars/json.ebnf` parsing
#      through the meta-parser; the per-terminal guard must not. This is the arm that
#      tells a real fix from the one H.16.4 measured and rejected.
#
# Usage:  bash docs/tasks/artifacts/grammar_wellformed/layout_owning_terminals/probe.sh
# Needs:  rust/target/debug/ast_pipeline built with --features "generated_parsers ebnf_dual_run"
#         (the script refuses an under-featured binary rather than printing empty rows).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"
PIPELINE=rust/target/debug/ast_pipeline

bash scripts/require_ast_pipeline_features.sh "$PIPELINE" generated_parsers ebnf_dual_run || exit 2

echo "== ARM A — LAYOUT-OWNING-TERMINALS census (closed population) =="
owning_total=0
loadable=0
unloadable=0
for g in $(find grammars -name '*.ebnf' | sort); do
  out="$("$PIPELINE" "$g" --report-layout-owning-terminals 2>&1)"
  line="$(printf '%s\n' "$out" | grep -E '^LAYOUT-OWNING-TERMINALS:')"
  if [ -z "$line" ]; then
    unloadable=$((unloadable + 1))
    printf '  [not-loadable] %s — %s\n' "$g" \
      "$(printf '%s\n' "$out" | grep -m1 -oE '^Error: .{0,80}')"
    continue
  fi
  loadable=$((loadable + 1))
  printf '%s\n' "$out" | grep -E '^  \[layout-owning\]|^LAYOUT-OWNING-TERMINALS:'
  owning_total=$((owning_total + $(printf '%s\n' "$line" | grep -oE 'layout_owning=[0-9]+' | cut -d= -f2)))
done
echo "H164A-CENSUS: loadable_grammars=$loadable not_loadable=$unloadable layout_owning_total=$owning_total"

echo
echo "== ARM B — reproducer: ebnf 'whitespace' on four spaces =="
printf '    ' > rust/target/h164a_ws4.txt
"$PIPELINE" grammars/ebnf.ebnf --interpret-parse rust/target/h164a_ws4.txt \
    --interpret-parse-ast-json rust/target/h164a_ws4.json 2>&1 | grep -E '^INTERPRET-PARSE:' | head -1
ws_committed=$(python3 -c "
import json,sys
try:
    d=json.load(open('rust/target/h164a_ws4.json'))
    els=d['content']['Json']['elements']
    print(sum(1 for e in els if e.get('type')=='whitespace'))
except Exception:
    print(0)
")
echo "H164A-REPRODUCER: whitespace_elements=$ws_committed (pre-fix: 0, span 0..0)"

echo
echo "== ARM C — control: the meta-parser still reads a real shipped grammar =="
json_verdict="$("$PIPELINE" grammars/ebnf.ebnf --interpret-parse grammars/json.ebnf 2>&1 \
    | grep -oE 'accepted=[a-z]+ furthest_position=[0-9]+')"
echo "H164A-CONTROL: grammars/json.ebnf -> ${json_verdict:-NO-VERDICT}"

echo
if [ "$ws_committed" -ge 1 ] && printf '%s' "$json_verdict" | grep -q 'accepted=true'; then
  echo "H164A-PROBE: PASS (reproducer commits AND the json.ebnf control still parses)"
  exit 0
fi
echo "H164A-PROBE: FAIL (reproducer=$ws_committed control='${json_verdict:-NO-VERDICT}')"
exit 1
