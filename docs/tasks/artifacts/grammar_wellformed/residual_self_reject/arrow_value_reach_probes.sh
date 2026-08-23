#!/usr/bin/env bash
# GRAMMAR-WELLFORMED.H.16.6c — the THREE residual rows the containment arm does NOT widen.
#
# The arm ledger clears five of the eight as the greedy path/URL terminal. These three are
# the genuine arrow residual. This script establishes, by entry-rule probe rather than by
# reading, the exact rule that governs an arrow in the map VALUE position:
#
#   an arrow `X => Y` is an `annotation_value` iff
#       X is an identifier_literal (route 1: lambda_expression, Y then unrestricted)
#    OR X and Y are both logical_or_expr (route 2: implication_expr)
#
# and each of the three residual rows falls outside BOTH routes.
# Read-only. Prints one ACCEPT/reject line per probe; rc 1 if any expectation is violated.
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
BIN="$ROOT/rust/target/debug/ast_pipeline"
G="$ROOT/grammars/semantic_annotation.ebnf"
TMP="$ROOT/rust/target/h1666c/arrow_probe.txt"
[ -x "$BIN" ] || { echo "arrow_value_reach_probes: REFUSED — no ast_pipeline at $BIN"; exit 2; }
[ -f "$G" ]   || { echo "arrow_value_reach_probes: REFUSED — no grammar at $G"; exit 2; }
mkdir -p "$(dirname "$TMP")"
fail=0
# probe <expected accept|reject> <entry-rule> <input>
probe() {
  local want="$1" rule="$2" in="$3" got
  printf '%s' "$in" > "$TMP"
  if "$BIN" "$G" --interpret-parse "$TMP" --interpret-entry-rule "$rule" >/dev/null 2>&1; then
    got=accept; else got=reject; fi
  if [ "$got" = "$want" ]; then printf '  %-6s [%-18s] %s\n' "$got" "$rule" "$in"
  else printf '  %-6s [%-18s] %s   ⛔ EXPECTED %s\n' "$got" "$rule" "$in" "$want"; fail=1; fi
}

echo "### route 1 — lambda_expression: the LHS must be a lambda_parameter (an IDENTIFIER)"
probe accept annotation_value 'a => b'
probe accept annotation_value 'a => ""'          # RHS unrestricted once route 1 is taken
probe reject annotation_value '%RXCy => b'       # symbol_reference is not a lambda_parameter
probe reject annotation_value "'F+|=' => b"      # a char literal is not a lambda_parameter

echo "### route 2 — implication_expr: BOTH sides must be logical_or_expr"
probe accept logical_or_expr  'a'
probe accept logical_or_expr  '0x4F'
probe accept logical_or_expr  '72e10'
probe accept logical_or_expr  '$ 2.H'
probe reject logical_or_expr  '%RXCy'            # no reach to symbol_reference
probe reject logical_or_expr  '"x"'              # no reach to string_literal
probe reject logical_or_expr  "'F+|='"
probe accept annotation_value '0x4F => b'        # both sides logical_or_expr -> route 2

echo "### the three residual rows — outside BOTH routes"
probe reject annotation_value '%RXCy   =>    72e10'   # LHS: not an identifier, not logical_or_expr
probe reject annotation_value '0x4F=>   ""'           # LHS not an identifier; RHS not logical_or_expr
probe reject annotation_value "'F+|='=>  \$ 2.H"      # LHS: not an identifier, not logical_or_expr

echo "### and the CHAIN itself is innocent — the control that clears the leaf's framing"
probe accept annotation_value '{ a => b => c }'       # a chained arrow in the VALUE position parses
probe accept annotation_value '{ a => b }'
probe reject annotation_value '{ W => %RXCy => 72e10 }'

[ "$fail" = 0 ] && echo "ARROW-VALUE-REACH: all expectations held" || echo "ARROW-VALUE-REACH: EXPECTATIONS VIOLATED"
exit "$fail"
