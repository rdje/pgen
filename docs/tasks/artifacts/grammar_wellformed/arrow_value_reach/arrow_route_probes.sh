#!/usr/bin/env bash
# GRAMMAR-WELLFORMED.H.16.6e — an arrow in a VALUE has THREE routes, not two.
#
# `H.16.6c` handed this leaf the framing "route 1 = lambda, route 2 = implication". There is a
# THIRD: `function_type := "(" (type_reference,…)? ")" "=>" type_reference`. It is distinguishable
# because it is the only route whose RHS must itself be a TYPE — so `(Foo<Bar>) => Baz` parses and
# `(Foo<Bar>) => 1` does not, and no lambda/implication reading can produce that pair.
#
# Read-only: every verdict comes from `--interpret-parse`, which reads the `.ebnf` directly and
# never links a generated parser. rc 1 if any expectation is violated.
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && while [ ! -d .git ] && [ "$PWD" != / ]; do cd ..; done; pwd)"
BIN="$ROOT/rust/target/debug/ast_pipeline"
G="${1:-$ROOT/grammars/semantic_annotation.ebnf}"
TMP="$ROOT/rust/target/h1666e/route_probe.txt"
[ -x "$BIN" ] || { echo "arrow_route_probes: REFUSED — no ast_pipeline at $BIN"; exit 2; }
[ -f "$G" ]   || { echo "arrow_route_probes: REFUSED — no grammar at $G"; exit 2; }
mkdir -p "$(dirname "$TMP")"
fail=0
probe() { local want="$1" in="$2" note="${3:-}" got
  printf '%s' "$in" > "$TMP"
  if "$BIN" "$G" --interpret-parse "$TMP" >/dev/null 2>&1; then got=accept; else got=reject; fi
  if [ "$got" = "$want" ]; then printf '  %-6s  %-46s %s\n' "$got" "$in" "$note"
  else printf '  %-6s  %-46s %s   ⛔ EXPECTED %s\n' "$got" "$in" "$note" "$want"; fail=1; fi; }

echo "ARROW-ROUTE-PROBES  grammar=$(basename "$G")"
echo "### route 1 — lambda_expression (LHS is a lambda_parameter: identifier or destructuring)"
probe accept '@x: a => b'                       'identifier parameter'
probe accept '@x: [a] => b'                     'destructuring parameter, bracket form'
probe accept '@x: {a} => b'                     'destructuring parameter, brace form'
probe accept '@x: (a, b) => c'                  'parenthesised parameter list'
probe accept '@x: a => "s"'                     'RHS unrestricted once route 1 is taken'
echo "### route 2 — implication_expr (BOTH sides logical_or_expr)"
probe accept '@x: 0x4F => b'                    'numeric LHS, identifier RHS'
probe accept '@x: (1) => 2'                     'parenthesised arithmetic LHS'
probe reject '@x: 0x4F => "s"'                  'string RHS is not a logical_or_expr'
echo "### route 3 — function_type (the RHS must itself be a type_reference)"
probe accept '@x: (Foo<Bar>) => Baz'            'ONLY function_type derives this'
probe reject '@x: (Foo<Bar>) => 1'              'the discriminator: a non-type RHS kills route 3'
probe accept '@x: (Foo[]) => Baz'               'array type in the parameter list'
probe accept '@x: { k => (Foo<Bar>) => Baz }'   'route 3 is reachable in a VALUE, not only at top level'
echo "### outside every route — correctly rejected, because no arrow CONSTRUCT accepts that operand"
probe reject '@x: %S => 1'                      'a symbol is not a parameter, an operand or a type list'
probe reject "@x: 's' => 1"                     'a char/quoted literal is none of the three either'
probe reject '@x: ./p => 1'                     'nor is a path'
echo "### …and each is still a perfectly legal map KEY, which is the asymmetry this leaf ruled on"
probe accept '@x: { %S => 1 }'
probe accept "@x: { 's' => 1 }"
probe accept '@x: { ./p => 1 }'
[ "$fail" = 0 ] && echo "ARROW-ROUTE-PROBES: all expectations held" || echo "ARROW-ROUTE-PROBES: EXPECTATIONS VIOLATED"
exit "$fail"
