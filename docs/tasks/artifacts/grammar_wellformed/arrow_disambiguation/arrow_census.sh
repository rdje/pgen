#!/usr/bin/env bash
# GRAMMAR-WELLFORMED.H.16.6a — ARROW-CENSUS
#
# WHAT: for every rule of a grammar, ask "starting at THIS rule, is `K => V` accepted?" — i.e.
#       which rules can CONSUME a `=>` arrow at their own level. `--lint-grammar` is blind to
#       this class (`H.16.6`: `ordered_choice_shadowing=0`, exit 0) because the two readings do
#       not shadow each other at ONE choice point: they share a TOKEN across two rules at
#       different depths.
# HOW:  `ast_pipeline <g>.ebnf --interpret-parse <probe> --interpret-entry-rule <rule>`
#       (TOOLBOX 1.5b), one invocation per (rule, probe). Read-only — no codegen, no
#       regeneration, no registry edit. Every row is backed by an `INTERPRET-PARSE:` verdict.
#
# ⛔ SOUNDNESS: this is a PROBE-BASIS census, so it is a FLOOR, never a closed set — a rule that
#       consumes the arrow only on a shape outside the basis reads clean. Measured: the first run
#       used a 7-probe all-lowercase basis and MISSED `function_type` entirely; `(Foo) => Bar`
#       (a capitalised `type_reference`) is what surfaced it. Extend the basis; never trust silence.
#
# usage: arrow_census.sh <grammar.ebnf> [rules.txt]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
G="$1"
RULES="${2:-}"
TMP="$ROOT/rust/target/arrow_census.$$"
mkdir -p "$TMP"
trap 'rm -rf "$TMP"' EXIT

# ⛔ Refuse rather than print an EMPTY census. A missing or under-featured binary makes every
# probe REJECT, and "no rows" reads exactly like "no rule consumes the arrow" — a silent failure
# in the PASSING direction. This is the `require_ast_pipeline_features.sh` trap (TOOLBOX 1.4).
BIN="$ROOT/rust/target/debug/ast_pipeline"
[[ -x "$BIN" ]] || { echo "ARROW-CENSUS: REFUSED — no ast_pipeline at $BIN" >&2; exit 2; }
"$ROOT/scripts/require_ast_pipeline_features.sh" "$BIN" ebnf_dual_run >/dev/null || exit 2

if [[ -z "$RULES" ]]; then
  RULES="$TMP/rules.txt"
  grep -oE '^[a-z_][a-z0-9_]*[[:space:]]*:?=' "$G" | sed 's/[[:space:]]*:\{0,1\}=//' | sort -u > "$RULES"
fi
[[ -s "$RULES" ]] || { echo "ARROW-CENSUS: REFUSED — empty rule list for $G" >&2; exit 2; }

PROBES=( '1 => 2' 'a => b' '[a] => b' '(a) => b' '"a" => "b"' '(a, b) => c' 'true => false' '(Foo) => Bar' 'Foo => Bar' )
for i in "${!PROBES[@]}"; do printf '%s' "${PROBES[$i]}" > "$TMP/p$i.txt"; done

echo "ARROW-CENSUS: grammar='$G' rules=$(wc -l < "$RULES" | tr -d ' ') probes=${#PROBES[@]}"
for i in "${!PROBES[@]}"; do printf 'ARROW-CENSUS: probe[%d]=%s\n' "$i" "${PROBES[$i]}"; done

rows=0
while read -r r; do
  [[ -z "$r" ]] && continue
  hits=""
  for i in "${!PROBES[@]}"; do
    if "$BIN" "$G" --interpret-parse "$TMP/p$i.txt" --interpret-entry-rule "$r" 2>/dev/null \
         | grep -q 'accepted=true'; then
      hits="$hits[$i]"
    fi
  done
  if [[ -n "$hits" ]]; then
    printf 'ARROW-CENSUS: rule=%-28s consumes_arrow_on=%s\n' "$r" "$hits"
    rows=$((rows+1))
  fi
done < "$RULES"
echo "ARROW-CENSUS: consuming_rules=$rows"
