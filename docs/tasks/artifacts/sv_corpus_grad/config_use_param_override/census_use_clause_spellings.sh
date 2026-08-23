#!/usr/bin/env bash
# How does the REAL world spell a config `use` clause? (`SV-CORPUS-GRAD.3.19`)
#
# The adjudication in this leaf rests on the LRM contradicting itself, so the third,
# independent leg has to be a MEASUREMENT rather than a recollection — the exact trap
# [[feedback_sv_strict_lrm_compliance_default]] records ("a claim about what external tools
# accept is a measurement, not a recollection"). This script is that measurement, and it is
# tracked so the number can be re-derived rather than believed.
#
# It enumerates every `instance …/cell … use …;` config-rule line in the whole vendored
# external corpus and groups them by spelling. Run from anywhere; paths resolve against the
# repository root (directive 12).
#
#   bash docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/census_use_clause_spellings.sh
#
# ⚠️ HONEST BOUND: the match is a single-line lexical one, so a use clause split across lines
# or sitting inside a comment/string would be missed or miscounted. It is corroborating
# evidence for the adjudication, never the adjudication itself — grounds 1 and 2 in the leaf
# are internal to the standard and stand alone.
set -euo pipefail

# ⛔ TERMINATION GUARD (`SV-CORPUS-GRAD.13c.2b`, 2026-08-23). Directive 12 forbids a hard-coded
# depth, so the root is walked up from this script at run time. The walk MUST have its own
# terminator: `cd ..` at `/` SUCCEEDS and is a no-op, so a `cd .. || exit` escape can NEVER fire and
# the loop spins forever outside a checkout (measured: 201+ iterations, `pwd=/`, no error). The
# Python sibling `_repo_root.py` never had this — `Path.parents` is finite and it REFUSES by name.
find_repo_root() {
  local d
  d="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)" || return 1
  while [ ! -f "$d/CLAUDE.md" ] || [ ! -d "$d/grammars" ]; do
    [ "$d" = "/" ] && return 1
    d="$(dirname "$d")"
  done
  printf '%s\n' "$d"
}
root="$(find_repo_root)" || { echo "REFUSE: no repository root (CLAUDE.md + grammars/) above ${BASH_SOURCE[0]}" >&2; exit 2; }
subs="$root/stimuli/sv/subs"
[ -d "$subs" ] || { echo "REFUSE: corpus not vendored at stimuli/sv/subs" >&2; exit 2; }

echo "# every config-rule \`use\` clause in stimuli/sv/subs, grouped by spelling"
grep -rhoE '^[[:space:]]*(instance|cell)\b[^;]*\buse\b[^;]*;' "$subs" \
  --include='*.sv' --include='*.v' --include='*.svh' \
  | sed 's/^[[:space:]]*//' | sort | uniq -c | sort -rn

echo
echo "# split by whether the override list is braced with #( … )"
# ⛔ `|| true` is load-bearing under `set -e`: grep exits 1 on ZERO matches, and zero is
# precisely the answer this census expects for the brace-less arm. Without it the script
# dies exactly when it has found its most interesting result.
count() { grep -rhoE "$1" "$subs" --include='*.sv' --include='*.v' --include='*.svh' \
  | wc -l | tr -d ' ' || true; }
all=$(count '^[[:space:]]*(instance|cell)\b[^;]*\buse\b[^;]*;')
hash=$(count '^[[:space:]]*(instance|cell)\b[^;]*\buse[[:space:]]*#\(')
# The Annex-A brace-less override spelling: a `.name` immediately after `use`.
bare=$(count '^[[:space:]]*(instance|cell)\b[^;]*\buse[[:space:]]*\.[A-Za-z_]')
echo "total use clauses            : $all"
echo "parameter override, '#( … )' : $hash   <- the clause-33 EXAMPLE spelling"
echo "parameter override, brace-less: $bare   <- the Annex-A BNF spelling"
echo "no override (plain lib.cell) : $((all - hash - bare))"
