#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.89 — the existence predicates are STORE-WIDE, reproduced outside SystemVerilog.
#
# Two six-rule grammars, run at the interpreter (TOOLBOX 1.5b, no codegen, no compile):
#   scope_leak.ebnf   `{x}` opens a `region` scope, emits fact (nm, T) INTO it, closes it; then `T` is
#                     gated by has_fact(nm, T).   `{x}T`  ⇒ ACCEPT   ← the fact in the CLOSED scope is found
#   scope_ctrl.ebnf   the same gate with no region anywhere.   `T`  ⇒ REJECT   ← the RED control: the gate
#                     can refuse, so the ACCEPT above is the query seeing the closed scope, not a dead gate.
# Both verdicts are asserted; a chain-visible has_fact (the routed .89 fix) would flip arm 1 to REJECT and
# this probe would then say so — re-run it when .89 lands and flip the expectation deliberately.
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
echo "--- the scope events, for the record (trace, high) ---"
PGEN_TRACE_VERBOSITY=high "$BIN" "$D/scope_leak.ebnf" --interpret-parse "$S/leak_in.txt" 2>&1 | sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g' | grep -E '🔓 @open_scope|📥 @emit_fact|🔒 @close_scope|🔍 has_fact\(' | sed 's/^.*\] //' | cut -c1-110
if [ "$fail" = 0 ]; then echo "eus89 probe: 2/2 as declared (fail=0)"; else echo "eus89 probe: FAILED"; fi
exit $fail
