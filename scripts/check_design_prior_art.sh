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

# Staged task LEAF files only. No staged set (e.g. a manual run) => nothing to judge.
#
# ⛔ `docs/tasks/artifacts/**` is EXCLUDED (DESIGN-PRIOR-ART.2). That subtree holds
# machine-GENERATED evidence — cluster reports, sweeps, probe dumps — not task leaves,
# and a design proposal by definition lives in a leaf. Scanning it produced a measured
# false positive: `rejects_valid_clusters.md` quotes corpus source lines verbatim, and
# the verible fixture line `@x[y];` reads as a proposed directive `@x`. That FP is also
# unfixable in the intended way — the only remedy the check offers is adding a
# `PRIOR ART` section to the file, which the next regeneration would delete — so the
# gate would have failed permanently on a regenerated artifact. Coverage is unchanged:
# leaves are exactly where a proposal can be made.
staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null \
          | grep -E '^docs/tasks/.*\.md$' \
          | grep -vE '^docs/tasks/artifacts/' || true)"
[ -n "$staged" ] || exit 0

# A directive name already known to the project: present in the registry, USED as `@name`
# in a tracked grammar, or DECLARED as an annotation/directive name by a tracked grammar.
# Built once; empty (missing registry) degrades to the grammar sources only.
#
# ⛔ THE THIRD SOURCE IS DESIGN-PRIOR-ART.3, AND IT CLOSES A MEASURED FALSE POSITIVE.
# A grammar declares its annotation names as QUOTED ALTERNATIVES, never as `@name`:
# `grammars/semantic_annotation.ebnf:52` carries `"throws" | "catches" | "handles" | …`
# among 122 such names, and `grammars/ebnf.ebnf:684` declares `optimization_directive` the
# same way. None of them is ever SPELLED `@name` in a grammar, so the `@`-usage sweep is
# blind to every one — and a task leaf quoting `@handles` (a real generated stimulus, in
# `GRAMMAR-WELLFORMED.H.16.6e`) was reported as proposing a NEW annotation surface.
# The only remedy the check offers is a PRIOR ART section, which would have been a
# recorded search for a name the project has always had: a FALSE finding written down.
#
# ⚠️ SCOPED TIGHTLY, and the scoping is what makes it safe to widen `known_names` at all —
# widening this set widens the doctrine's blind spot, so each restriction is deliberate:
#   · only inside a rule whose NAME mentions `annotation` or `directive`;
#   · the text after `->` is dropped — a return annotation is a PAYLOAD (`{type: "…"}`),
#     not a name declaration (this is what excludes `annotation_key`, `return_annotation`,
#     `compiler_directive`, `scoped`);
#   · a trailing `#` comment is dropped — prose is not a declaration (excludes `file1`…);
#   · a grammar-level directive line (`@profiles: ["sv_2017"]`) belongs to the NEXT rule
#     and is skipped entirely (excludes `sv_2017`, `sv_2023`).
# Measured on the tracked grammars: the loose form harvests 139 names including 12 that are
# not names at all; this form harvests 127 — the 122 predefined annotation names plus the 5
# lowercase `optimization_directive` names — and nothing else.
# ⚠️ HONEST FLOOR: the literal pattern is lowercase-only, so camelCase declarations
# (`"pushMode"`, `"popMode"`) are NOT harvested. That leaves the check strict where it was
# strict, which is the safe direction for an evidence gate.
known_names="$(
  { [ -r "$REGISTRY" ] && grep -oE 'name:[[:space:]]*"[a-z_][a-z0-9_]*"' "$REGISTRY" \
      | sed -E 's/.*"([a-z_][a-z0-9_]*)".*/\1/'
    grep -rhoE '@[a-z_][a-z0-9_]*' grammars/ 2>/dev/null | sed 's/^@//'
    awk '
      /^[a-z_][a-z0-9_]*[ \t]*:=/ { cur=$1; sub(/[ \t]*:=.*/, "", cur) }
      /^[ \t]*@/                  { next }
      cur ~ /annotation|directive/ {
        line=$0
        sub(/->.*/, "", line)
        sub(/#.*/,  "", line)
        while (match(line, /"[a-z_][a-z0-9_]*"/)) {
          print substr(line, RSTART+1, RLENGTH-2)
          line=substr(line, RSTART+RLENGTH)
        }
      }
    ' grammars/*.ebnf 2>/dev/null
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
