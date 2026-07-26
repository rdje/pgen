---
name: project-capability-growth-is-zero-cost-and-neutral
description: "DIRECTOR INVARIANT (2026-07-26, session #208): every decision shall move PGEN toward the universal-parser horizon WITHOUT degrading (a) parser-neutrality/agnosticism or (b) the parser's ability to run as fast as it theoretically can. These are not three goals to trade off — they are one goal plus two non-negotiable constraints, and they form the ACCEPTANCE TEST for any new primitive. The mechanism that satisfies all three is COMPILE-AWAY: a declarative capability is resolved at codegen time wherever its semantics permit; a grammar that does not use a primitive pays literally nothing (byte-identical), and one that does pays at generation time, not per parse step."
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-07-26
---

**Director, verbatim (2026-07-26, session #208):**

> *"So, every decisions made shall be towards reaching that goal without affecting
> the capability of the PGEN parser neutral, agnostic to be able to run as fast as
> it theoretically can."*

## The invariant

One goal, two non-negotiable constraints, applied to **every** decision:

| | statement | prior record |
|---|---|---|
| **GOAL** | move toward parsing any precisely-described human-designed language with the right EBNF | [[project_horizon_universal_parser]] |
| **CONSTRAINT 1** | never degrade parser-**neutrality / agnosticism** — no per-language special case, no hook, no privileged grammar | [[feedback_ast_pipeline_parser_agnostic]], [[feedback_no_parser_hooks_full_neutrality]] |
| **CONSTRAINT 2** | never degrade the parser's ability to run **as fast as it theoretically can** | [[feedback_correctness_before_speed]] (⭐ speed is co-equal, monitored "like milk on fire" for every generated parser) |

These are **not** three things to balance. Capability growth that costs neutrality or
peak speed is **rejected**, not traded — the same way accuracy is an immovable floor
rather than a dial.

## The mechanism that satisfies all three: COMPILE AWAY

The reason this is achievable rather than aspirational is PGEN's architecture: a
declarative `.ebnf` compiled by a parser-agnostic pipeline into a generated parser.
That gives a **codegen-time seam** where capability can be resolved before the parse
loop ever runs.

> **The test for any new primitive:**
> 1. **Non-users pay ZERO.** A grammar that does not use the primitive produces a
>    **byte-identical** parser. Not "negligible" — *identical*. This is the
>    `@quantified_separator` inertness precedent (an empty policy map leaves the
>    path inert), and it is now a stated acceptance criterion on `LEX-ADJACENCY.2`.
> 2. **Users pay at CODEGEN time, not per parse step**, wherever the primitive's
>    semantics permit. Resolve statically; specialize; emit the decision. Do not
>    add a runtime flag consulted in a hot loop.
> 3. **Runtime cost only where semantically unavoidable**, and then confined to the
>    grammars that ask for it. Some capabilities genuinely need parse-time state —
>    the semantic store's context-sensitivity (`has_fact`, scopes, `phase:final`)
>    cannot be compiled away, because the constraint depends on the input. That is
>    legitimate; what is not legitimate is levying it on grammars that never use it.

**Worked example, this session.** `LEX-ADJACENCY.1` had two ways to enforce a
no-layout boundary at parse time: a runtime depth counter mirroring the generator, or
static specialization of the annotated rule's closure. The runtime counter was
**rejected on this invariant** — it would tax all **1,798** terminal sites in the SV
parser for a primitive used by two rules. Static specialization was chosen and its
affordability *measured* (`time_literal`'s closure is 14 of 1,475 rules, <1%), so a
bare parse pays nothing. That is constraint 2 deciding a design, not decorating it.

**Architectural precedent already in the tree.** PGEN has shipped exactly this shape
before: the **observability twin** (`RGX-0078.5.i.7 D2-A`). A bare parse runs the
fused `cascade_*` graph; any diagnostic consumer (trace, coverage, counters, memo
stats) automatically routes to the protocol graph instead. Full observability
capability, **zero cost on the fast path**. New capability work should reach for this
pattern rather than inventing a runtime toggle.

## ⇒ Consequence for the in-flight lexical-adjacency design

This invariant promotes a question flagged as "worth measuring" in
[`LEX-ADJACENCY`](../tasks/LEX-ADJACENCY.md) into a **load-bearing design test**:
does the per-seam directive
([[project_ebnf_steers_the_engine_at_full_granularity]]) need the parse-time
**action** machinery that `INLINE-ACTIONS` would complete, or is it purely
compile-time (steering codegen + the stimuli generator)?

- If **compile-time only** → it satisfies the invariant by construction, decouples
  from `INLINE-ACTIONS`, and costs a bare parse nothing.
- If it needs **parse-time evaluation at every sequence position** → that is a
  constraint-2 problem and the design must be reworked before implementation, not
  after.

Answering that is now the first measurement of the re-adjudication, not a footnote.

## How to apply

- **Every primitive proposal states its cost model up front**: what non-users pay
  (must be zero/byte-identical), what users pay, and at which stage.
- **Prefer the codegen-time realization.** A runtime flag in a hot path is the smell;
  ask what could be decided statically instead.
- **Neutrality has no exceptions**: no `if grammar_name == …`, no hard-coded rule
  NAME governing behaviour ([[project_ebnf_is_single_source_of_truth]]), no hooks.
  A capability that only one language can use is not a capability, it is a special
  case wearing one.
- **Measure, do not assert, the speed claim.** "This is cheap" is not a cost model;
  a closure size, an entry count, or a benchmark is.
