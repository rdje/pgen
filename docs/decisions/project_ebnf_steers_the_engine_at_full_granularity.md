---
name: project-ebnf-steers-the-engine-at-full-granularity
description: "DIRECTOR PRINCIPLE (2026-07-26, session #208): the EBNF is the sole source of truth for steering PGEN's parser-neutral, parser-agnostic engine — AT THE GRANULARITY THE GRAMMAR AUTHOR NEEDS, not merely the granularity the engine currently finds convenient. Concretely: PGEN's meta-grammar already admits INLINE semantic annotations between rule references inside a branch's sequence, so a steering directive should be expressible per-SEAM (which pair of adjacent elements), not only per-rule. When the runtime does not yet honour a placement the meta-grammar admits, that is UNFINISHED WIRING to complete — never a reason to design the coarser surface."
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-07-26
---

**Director, verbatim (2026-07-26, session #208), on the lexical-adjacency design:**

> *"For example, this lexical adjacency thing, right now you plan to control it using
> a semantic annotation. This shall allow the user really fine grain control on which
> pair of rules shall be spaced allowed or not. PGEN support inline semantic
> annotations, that is between rules' references in a sequence of a branch. You get
> this idea. EBNF shall really be the sole source of truth which can steer PGEN
> parser neutral, agnostic engine."*

## The principle

[[project_ebnf_is_single_source_of_truth]] says *where* steering lives (in the EBNF,
never in a runtime flag or engine table). This record adds *at what granularity*:

> **A steering directive should be expressible at the granularity the language
> actually needs — per grammar, per rule, per branch, AND per seam (between two
> adjacent elements of a sequence) — because that is the granularity at which real
> language standards state their constraints.**

A rule-level directive can only say "this property holds everywhere inside this
rule". Many real constraints are narrower: in `a b c`, an LRM may forbid layout
across `a b` while permitting it across `b c`. Only an **inline** placement sits at
the seam.

## Why this is not a feature request — the surface already exists

`grammars/ebnf.ebnf:117-130` already admits it, and says so in its own comment:

```ebnf
sequence         := sequence_element+
sequence_element := ( inline_semantic_annotation | quantified_element | primary_element )
# - branch-start inline annotations can mean branch-local steering
# - later inline annotations in a sequence can mean true mid-sequence actions
inline_semantic_annotation := semantic_annotation
```

What is incomplete is the **runtime wiring**, measured by [`INLINE-ACTIONS`](../tasks/INLINE-ACTIONS.md):
branch-start inline directives compile into `branch_directives_by_rule` but only
`Predicate` is acted on; mid-sequence inline directives are extracted into
`branch_mid_sequence_semantic_annotations` and then **never compiled**, so they are
silently dropped. The registry that records *which position in which branch* already
exists — it simply is not consumed.

## ⛔ The anti-pattern this exists to stop

**Do not let "the engine does not honour that placement today" harden into "the
design space does not contain that placement."** That inversion happened twice:

- `INLINE-ACTIONS` (2026-06-09) — the director had to point out that the syntax was
  already there, against an engineer claim that no branch-level precedent existed.
  The tree records: *"The director is right about the syntax and identified a real
  runtime gap."*
- `LEX-ADJACENCY.1` (2026-07-26) — the sequence-level surface was rejected by
  **quoting** the book's *"per-element ones are not [generator-visible]"* rather than
  measuring it, turning a statement about the current runtime into a permanent
  property of the design. The director reopened it the same session.

Both times the cheap check was the same: **read `ebnf.ebnf` and probe the placement**,
rather than trusting a prose summary of what the engine supports. Compare the sibling
lesson in [[project_no_layout_primitive_is_undeclarable]] — a capability the engine
has but no grammar can request is, practically, absent; here, a *placement the
grammar can express but the engine drops* is the same failure seen from the other
side.

## How to apply

- **When designing any new steering directive, price the inline/per-seam placement
  explicitly** and record a measured reason if you do not adopt it. "Not visible
  today" is a wiring statement, not a design verdict.
- **Prefer the finest placement the constraint genuinely needs.** A coarser
  rule-level form may still ship as a convenience shorthand, but it should not be
  the only surface where the constraint is really per-seam.
- **Check `grammars/ebnf.ebnf` first.** The meta-grammar is the authority on what a
  grammar author can already write; the runtime is only the authority on what
  currently happens.
- Related: [[project_ebnf_is_single_source_of_truth]] (where steering lives),
  [[feedback_ast_pipeline_parser_agnostic]] (steering must stay parser-agnostic),
  [[project_horizon_universal_parser]] (every primitive duality-complete).
