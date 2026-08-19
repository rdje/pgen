#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2k (a) — is a rule MEMOIZED on its executed path, or INLINED at its call sites?
#
# ⛔ WHY THIS IS THE DECIDING INSTRUMENT. `price.py` priced the identifier guard from the two
# candidate hosts' MEASURED memo-hit rates (`identifier` ~90 %, `non_keyword_identifier` 0 %) and
# concluded the guard belongs inside `identifier`. Both numbers were right and the conclusion was
# backwards, because a rule's hit rate is not a property OF THE RULE: the generator inlines a rule
# at its call sites (TOOLBOX 3.6 — 663 of 1481 rules across 2871 sites on this grammar), and an
# INLINED reference gets a full observable frame and NO `memoized_call` at all. So a rule reached
# only through inlined sites reports 0 memo hits however hot it is — and a fix that changes how
# many places call a rule CHANGES ITS INLINING, i.e. changes the very number the price was read
# from. This script reads that decision out of the generated parser instead of inferring it.
#
# READING: `memoized_call(Self::RULE_X` inside `parse_x` means X memoizes itself when entered
# through its own method; `inlined_frame_call(Self::RULE_X` counts the call sites that BYPASS that
# method by inlining X's body — those calls take no memo hit.
#
# USAGE  bash …/inline_census.sh [rule …]      (default: the identifier family)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
[ -d "$ROOT/grammars" ] || { echo "inline_census: not at the repo root (derived $ROOT)" >&2; exit 2; }
cd "$ROOT"
P=generated/systemverilog_parser.rs
[ -f "$P" ] || { echo "inline_census: no generated parser at $P" >&2; exit 2; }

RULES=("$@")
[ ${#RULES[@]} -gt 0 ] || RULES=(identifier non_keyword_identifier simple_identifier escaped_identifier reserved_non_keyword_identifier)

echo "INLINE-CENSUS: parser=$(shasum -a 256 "$P" | awk '{print $1}')"
printf '%-38s %14s %20s   %s\n' rule self-memoized inlined-call-sites verdict
for r in "${RULES[@]}"; do
  R="RULE_$(echo "$r" | tr '[:lower:]' '[:upper:]')"
  m=$(grep -A1 '\.memoized_call($' "$P" | grep -c "Self::$R,")
  i=$(grep -A1 '\.inlined_frame_call($' "$P" | grep -c "Self::$R,")
  if [ "$i" -gt 0 ] && [ "$m" -gt 0 ]; then v="memoizes itself, but $i call site(s) INLINE it"
  elif [ "$i" -gt 0 ]; then v="INLINED at $i call site(s), never memo-served"
  elif [ "$m" -gt 0 ]; then v="MEMOIZED on every entry"
  else v="neither — check the rule name"; fi
  printf '%-38s %14s %20s   %s\n' "$r" "$m" "$i" "$v"
done
