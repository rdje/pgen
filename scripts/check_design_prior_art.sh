#!/usr/bin/env bash
# DESIGN-PRIOR-ART — a task leaf that proposes a NEW annotation/directive surface
# must record a prior-art search first.
#
# Provenance: director directive 2026-07-26 (session #208), after LEX-ADJACENCY.1
# proposed `@lexical_token` for a constraint the repo could ALREADY express via the
# lexical-annotation notation whose per-seam inline form had been designed with the
# director on 2026-06-06. See docs/decisions/feedback_read_prior_art_before_designing.md.
#
# THE RULE
#   If a staged `docs/tasks/*.md` introduces a backticked `@name` token that exists
#   in NEITHER the semantic-directive registry NOR any tracked grammar, that file
#   must contain a `PRIOR ART` section. Otherwise: block, and name the token.
#
# ARCHETYPE: evidence (DOCTRINE_ENFORCEMENT.md §3).
#   HONEST LIMIT — this verifies the search was RECORDED, not that it was THOROUGH.
#   Stated rather than hidden; it is the bound every evidence check carries.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on
# stderr; deterministic; read-only; staged-scope-aware; path-agnostic; fast.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

REGISTRY="rust/src/ast_pipeline/semantic_directive_registry.rs"

# Staged task-tree files only. No staged set (e.g. a manual run) => nothing to judge.
staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null \
          | grep -E '^docs/tasks/.*\.md$' || true)"
[ -n "$staged" ] || exit 0

# A directive name already known to the project: present in the registry or in any
# tracked grammar. Built once; empty (missing registry) degrades to "grammars only".
known_names="$(
  { [ -r "$REGISTRY" ] && grep -oE 'name:[[:space:]]*"[a-z_][a-z0-9_]*"' "$REGISTRY" \
      | sed -E 's/.*"([a-z_][a-z0-9_]*)".*/\1/'
    grep -rhoE '@[a-z_][a-z0-9_]*' grammars/ 2>/dev/null | sed 's/^@//'
  } | sort -u
)"

fail=0
for file in $staged; do
  [ -r "$file" ] || continue

  # Candidate proposals: backticked `@name` on ADDED lines of this file's staged diff.
  candidates="$(git diff --cached -U0 -- "$file" 2>/dev/null \
                | grep '^+' | grep -v '^+++' \
                | grep -oE '`@[a-z_][a-z0-9_]*' \
                | sed 's/^`@//' | sort -u || true)"
  [ -n "$candidates" ] || continue

  novel=""
  for name in $candidates; do
    printf '%s\n' "$known_names" | grep -qx "$name" || novel="$novel $name"
  done
  [ -n "$novel" ] || continue

  # A recorded prior-art search discharges it.
  if grep -qE '^[^[:alnum:]]*PRIOR ART|## .*[Pp]rior [Aa]rt' "$file"; then
    continue
  fi

  fail=1
  {
    echo "DESIGN-PRIOR-ART: $file proposes a NEW annotation surface with no recorded prior-art search."
    echo "  novel directive name(s):$novel"
    echo "  (not found in $REGISTRY, nor in any grammars/*.ebnf)"
    echo
    echo "  Add a 'PRIOR ART' section to $file stating what you searched and found:"
    echo "    1. grammars/ebnf.ebnf   — is it already EXPRESSIBLE? (the meta-grammar is the"
    echo "                              authority on what an author can write; the runtime is"
    echo "                              only the authority on what currently happens)"
    echo "    2. docs/decisions/      — has it already been decided or DESIGNED?"
    echo "    3. docs/tasks/          — does a tree already own it (incl. a 'deferred' leaf)?"
    echo "    4. docs/book/           — does the pillar chapter document a designed-but-unbuilt form?"
    echo
    echo "  Why: LEX-ADJACENCY.1 proposed '@lexical_token' for a constraint already"
    echo "  expressible as '[>! /\\s/]', whose per-seam inline form had been designed"
    echo "  on 2026-06-06. Finding prior art is a SUCCESS — it collapsed that design"
    echo "  from 'invent a primitive' to 'implement a designed one'."
    echo "  See docs/decisions/feedback_read_prior_art_before_designing.md"
  } >&2
done

exit "$fail"
