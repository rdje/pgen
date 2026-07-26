---
name: feedback_read_prior_art_before_designing
description: "STANDING DIRECTIVE (director 2026-07-26, session #208, after LEX-ADJACENCY.1 reinvented a surface the director and engineer had already designed together in June): ALWAYS read and understand the existing CODEBASE, BOOK and TASK-TREES before designing anything. A design that proposes a NEW surface/primitive must first prove no existing one covers it. Four failures in one session all shared one root: designing or asserting from a PROSE SUMMARY instead of the SOURCE. Mechanically enforced by the DESIGN-PRIOR-ART doctrine check — a design leaf proposing a new surface must carry a PRIOR ART section citing what it searched."
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-07-26
---

**Director, verbatim (2026-07-26, session #208):**

> *"`.1`'s `@lexical_token` was reinventing an already-designed surface — that is why
> you should always make you read and understood the existing codebase, book and
> task-trees."*

## What happened

`LEX-ADJACENCY.1` designed a brand-new rule-level `@lexical_token` semantic
annotation for lexical adjacency. PGEN already had:

- a **lexical annotation** notation (`[> … ]` / `[>! … ]`) that expresses the exact
  constraint — `[>! /\s/]` *is* "must not be followed by white space";
- a **per-seam inline form** of it, **designed with the director on 2026-06-06** and
  recorded in [[project_lexical_annotations_fourth_pillar]] (*"an inline lexical
  annotation binds the **preceding** item"*), listed in the live book under
  *"still to come"*.

The design leaf proposed a new surface without ever consulting the decision record,
the book chapter, or the meta-grammar that together already specified it.

## The root cause — one pattern, four failures in a single session

Every one was **designing or asserting from a prose summary instead of the source**:

| # | claim | source used | reality |
|---|---|---|---|
| 1 | "mainstream simulators accept `10 ns`" | general knowledge | measured FALSE (0 spaced vs 273 tight / 16,336 files); it had *already* been retracted in-tree, then leaked into `MEMORY.md` and was repeated back to the director as their own position |
| 2 | "per-element annotations are not generator-visible" ⇒ reject the per-seam surface | quoted the book | true of the *current runtime*, silently upgraded into a permanent property of the design space |
| 3 | "branch-start `EmitFact` hits a no-op arm" | quoted `INLINE-ACTIONS`' `.1`-era table | superseded by `INLINE-ACTIONS.2`, which wired it |
| 4 | invented `@lexical_token` | — | reinvented a surface designed 2026-06-06 with the director |

⚠️ Note #2 and #3 in particular: **a doc that describes engine behaviour goes stale
the moment a leaf lands.** A tree's own historical sections are *evidence of what was
once true*, not a description of HEAD.

## The rule

> **Before proposing any new surface, primitive, directive or notation: prove no
> existing one covers it.** The four places to check, in order of authority:
>
> 1. **`grammars/ebnf.ebnf`** — the meta-grammar is the authority on what a grammar
>    author can already *write*. (The runtime is only the authority on what currently
>    *happens*, and the two differ — see [[project_ebnf_steers_the_engine_at_full_granularity]].)
> 2. **`docs/decisions/`** — has this already been decided or designed? Search by
>    concept, not just by name.
> 3. **`docs/tasks/`** — does a tree already own it? Is there a `deferred` leaf?
> 4. **`docs/book/`** — the pillar chapter often documents a designed-but-unbuilt form.
>
> And when citing engine behaviour: **re-measure it.** Do not quote a doc's
> description of what the engine does.

## Enforcement — mechanical, not a promise

Per [[feedback_no_workarounds_fix_hierarchy]] and `DOCTRINE_ENFORCEMENT.md` (*"a
doctrine that is not mechanically checked is not enforced — it is a suggestion"*),
this is gated, not trusted:

- **Doctrine `DESIGN-PRIOR-ART`** (`scripts/check_design_prior_art.sh`, registered in
  `scripts/check_doctrines.sh`): a task leaf that proposes a NEW surface/primitive
  must carry a **`PRIOR ART`** section citing the four sources above and stating what
  it found. A design leaf without it is blocked at pre-commit.
- The check is **evidence-archetype** (`DOCTRINE_ENFORCEMENT.md` §3) — it verifies the
  search was recorded and cites real, resolvable paths; it cannot verify the search
  was *thorough*, which is an honest limit stated rather than hidden.

## How to apply

- A design leaf's FIRST section is `PRIOR ART`, written before the proposal — not a
  justification added afterwards.
- Finding prior art is a **success**, not a setback: the `LEX-ADJACENCY` design
  collapsed from "invent a primitive" to "implement a designed one + make it
  parser-consumed" the moment the prior art surfaced.
- Related: [[project_no_layout_primitive_is_undeclarable]] (a capability the engine
  has but nobody can find is practically absent — the same disease, seen from the
  engine side), [[feedback_systematically_use_debug_toolbox]] (measure, never guess),
  [[feedback_always_signoff_decisions]].
