#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.89 — the existence predicates are STORE-WIDE, reproduced outside SystemVerilog.
#
# Two six-rule grammars, run at the interpreter (TOOLBOX 1.5b, no codegen, no compile):
#   scope_leak.ebnf   `{x}` opens a `region` scope, emits fact (nm, T) INTO it, closes it; then `T` is
#                     gated by has_fact(nm, T).   `{x}T`  ⇒ ACCEPT   ← the fact in the CLOSED scope is found
#   scope_ctrl.ebnf   the same gate with no region anywhere.   `T`  ⇒ REJECT   ← the RED control: the gate
#                     can refuse, so the ACCEPT above is the query seeing the closed scope, not a dead gate.
# Both verdicts are asserted. The .89 fix landed as an OPT-IN (`@fact_kind: { …, visibility: lexical }`), so
# arm A1 keeps its store-wide ACCEPT by design — the default never moved, because a grammar (regex) depends
# on it — and two arms were added on the third grammar:
#   scope_lexical.ebnf  the same six rules with the kind declared `visibility: lexical`.
#                       `{x}T` ⇒ REJECT  ← the closed region's fact is no longer found from outside it
#                       `{xT}` ⇒ ACCEPT  ← inside the open region the same gate passes (the RED control for A3)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
BIN=./rust/target/debug/ast_pipeline
[ -x "$BIN" ] || { echo "eus89 probe: $BIN missing — build it: cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline" >&2; exit 2; }
D=docs/tasks/artifacts/engine_universal_services/eus89_store_wide_has_fact
S=rust/target/eus89; mkdir -p "$S"   # on-volume scratch (CLAUDE.md §13)
printf '{x}T' > "$S/leak_in.txt"; printf 'T' > "$S/ctrl_in.txt"
fail=0
arm() { # $1 label  $2 grammar  $3 input  $4 expected accepted=…
  local line; line="$("$BIN" "$2" --interpret-parse "$3" 2>/dev/null | grep -E '^INTERPRET-PARSE:' || true)"
  if printf '%s' "$line" | grep -q "$4"; then echo "  PASS  $1: $line"; else echo "  FAIL  $1: expected $4, got: ${line:-<no verdict line>}"; fail=1; fi
}
arm "A1 fact emitted into a scope that then CLOSED still satisfies has_fact from outside it" "$D/scope_leak.ebnf" "$S/leak_in.txt" "accepted=true"
arm "A2 RED control: the same gate with no fact anywhere REJECTS" "$D/scope_ctrl.ebnf" "$S/ctrl_in.txt" "accepted=false"
printf '{xT}' > "$S/lex_in_open.txt"
arm "A3 visibility: lexical — the closed region's fact is NOT found from outside it (REJECT)" "$D/scope_lexical.ebnf" "$S/leak_in.txt" "accepted=false"
arm "A4 visibility: lexical — inside the open region the same gate passes (ACCEPT)" "$D/scope_lexical.ebnf" "$S/lex_in_open.txt" "accepted=true"
echo "--- the scope events, for the record (trace, high) ---"
PGEN_TRACE_VERBOSITY=high "$BIN" "$D/scope_leak.ebnf" --interpret-parse "$S/leak_in.txt" 2>&1 | sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g' | grep -E '🔓 @open_scope|📥 @emit_fact|🔒 @close_scope|🔍 has_fact\(' | sed 's/^.*\] //' | cut -c1-110
if [ "$fail" = 0 ]; then echo "eus89 probe: 4/4 as declared (fail=0)"; else echo "eus89 probe: FAILED"; fi
exit $fail
