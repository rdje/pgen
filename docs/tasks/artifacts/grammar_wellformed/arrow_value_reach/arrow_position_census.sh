#!/usr/bin/env bash
# GRAMMAR-WELLFORMED.H.16.6e — WHERE MAY AN ARROW OPERAND BE, BY POSITION.
#
# `H.16.6b` gave the map KEY a dedicated `map_key` rule. This census asks the symmetric
# question for the VALUE position: for one operand shape S, which of these parse?
#
#   key      `@x: { S => 1 }`        — S as a map key                 (map_key)
#   plain    `@x: { k => S }`        — S as a plain map value         (annotation_value)
#   vlhs     `@x: { k => S => 1 }`   — S left of an arrow IN a value
#   tlhs     `@x: S => 1`            — S left of a TOP-LEVEL arrow
#   rhs      `@x: { k => a => S }`   — S right of an arrow            (route 1: unrestricted)
#
# Read-only, no codegen: every verdict comes from `--interpret-parse`, which reads the
# `.ebnf` directly and never links a generated parser (immune to the stale-binary trap).
# usage: arrow_position_census.sh [grammar.ebnf]
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && while [ ! -d .git ] && [ "$PWD" != / ]; do cd ..; done; pwd)"
BIN="$ROOT/rust/target/debug/ast_pipeline"
G="${1:-$ROOT/grammars/semantic_annotation.ebnf}"
TMP="$ROOT/rust/target/h1666e/census_in.txt"
[ -x "$BIN" ] || { echo "arrow_position_census: REFUSED — no ast_pipeline at $BIN"; exit 2; }
[ -f "$G" ]   || { echo "arrow_position_census: REFUSED — no grammar at $G"; exit 2; }
mkdir -p "$(dirname "$TMP")"

verdict() { printf '%s' "$1" > "$TMP"
  if "$BIN" "$G" --interpret-parse "$TMP" >/dev/null 2>&1; then printf 'Y'; else printf '.'; fi; }

echo "ARROW-POSITION-CENSUS  grammar=$(basename "$G")"
echo "  Y = parses, . = rejected"
echo
printf '  %-16s %-5s %-5s %-5s %-5s %-5s\n' shape key plain vlhs tlhs rhs
printf '  %-16s %-5s %-5s %-5s %-5s %-5s\n' ---------------- --- ----- ---- ---- ---
n_vlhs_gap=0; n_shapes=0
while IFS='|' read -r name s; do
  [ -z "${name// }" ] && continue
  k=$(verdict "@x: { $s => 1 }")
  p=$(verdict "@x: { k => $s }")
  v=$(verdict "@x: { k => $s => 1 }")
  t=$(verdict "@x: $s => 1")
  r=$(verdict "@x: { k => a => $s }")
  printf '  %-16s %-5s %-5s %-5s %-5s %-5s   %s\n' "$name" "$k" "$p" "$v" "$t" "$r" "$s"
  n_shapes=$((n_shapes+1))
  [ "$k" = Y ] && [ "$v" = . ] && n_vlhs_gap=$((n_vlhs_gap+1))
done <<'ROWS'
string          |"s"
single_quoted   |'s'
integer         |1
hexadecimal     |0x4F
scientific      |72e10
boolean         |true
null            |null
identifier      |nm
array           |[1]
ident_array     |[a]
object          |{a: 1}
ident_object    |{a}
tuple           |(1)
ident_tuple     |(a, b)
set             |#{1}
map             |{a => 1}
symbol_ref      |%S
rule_ref        |$h
path_rel        |./p
url             |https://h/p
generic_type    |Foo<Bar>
function_call   |f(1)
qualified_name  |a.b
ROWS
echo
echo "ARROW-POSITION-CENSUS: shapes=$n_shapes key_ok_but_value_arrow_lhs_rejected=$n_vlhs_gap"
