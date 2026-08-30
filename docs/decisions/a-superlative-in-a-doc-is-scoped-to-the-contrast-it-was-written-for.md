---
name: a-superlative-in-a-doc-is-scoped-to-the-contrast-it-was-written-for
description: "DISCIPLINE (2026-08-30, ENGINE-UNIVERSAL-SERVICES.78 / PGEN-ENGINE-UNIVERSAL-SERVICES-0140) — a doc's superlative (`the ONLY X`, `the one that matters`, `always`) is scoped to the CONTRAST it was written against, and reusing it as a PREDICATE in different code silently narrows that predicate. MEASURED: `StackProbeRefusal::BudgetExhausted` is documented as *the ONLY refusal that is a fact about the STACK rather than about this process's knowledge* — true, and written to separate it from three causes that report missing KNOWLEDGE. A new ceiling ladder needed the predicate *did the stack run out*, and the doc reads like the answer. It is not: `NoFrameAffordable` is also a fact about the stack (headroom left, but under one frame's worth), and on `verilog_2005` it is the cause that ACTUALLY FIRES — depth 8 668, `used` 251 643 616 B, 14 624 B UNDER a 251 658 240 B budget. A stop condition taking only `BudgetExhausted` would have read that ladder's own crossing as a non-event and climbed until a 2 048 s hang detector fired. ⇒ when a doc sentence is about to become a predicate, re-derive the predicate from the DEFINITIONS of every member, and check the superlative's contrast class — `the ONLY A that is B` excludes non-B, never other A."
id: a-superlative-in-a-doc-is-scoped-to-the-contrast-it-was-written-for
title: "A superlative in a doc is scoped to the contrast it was written for — never lift one into a predicate"
date: 2026-08-30
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .78 (the DONE sub-section and its acceptance checklist); docs/tasks/artifacts/engine_universal_services/eus78_ceiling_locks/README.md (both ladders, rung by rung, with the two refusal causes side by side); rust/src/dedicated_parse_stack.rs (the StackProbeRefusal enum and the doc sentence in question)"
reverify: "grep -c 'NoFrameAffordable' rust/src/embedding_api.rs   # must be >= 1 — the ladder's stop condition must still admit BOTH stack-grounds causes; a 0 here is this record's defect back"
answers:
  - "can I use a doc's the-only-X sentence as a membership test"
  - "why did my ladder miss its own crossing"
  - "which StackProbeRefusal causes mean the stack really ran out"
  - "how do I turn prose into a predicate safely"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-30
---

## The rule

When a sentence from a doc comment is about to become a **predicate in code**, do not lift it.
Re-derive the predicate from the **definitions of every member of the domain**, then check what
contrast the doc's superlative was drawn against. `the ONLY A that is B` excludes things that are
not **B**. It says nothing about other things that are **A**.

## Why — the measurement

`ENGINE-UNIVERSAL-SERVICES.69`(d) added `StackProbeRefusal`, five causes for one `bool`. Its doc
marks one of them:

> ⭐ The **ONLY** refusal that is a fact about the STACK rather than about this process's knowledge:
> the spendable budget is used up.

That sentence is **true**, and it is doing real work: three of the five causes (`ExtentUnknown`,
`DirectionUnknown`, `FrameCostUnmeasurable`) report what the process could not *learn*, and the
star exists to stop a reader treating those as capacity limits.

`.78` then needed a different predicate — *did this parse run out of stack?* — for a ladder that
climbs until the ceiling really refuses. The doc reads exactly like the answer to that question. It
is not the answer, because the contrast class is wrong: the fifth cause, `NoFrameAffordable`
(headroom remains, but less than one frame's worth), is **also** a fact about the stack. It was
simply not what the star was drawn against.

Measured on the run that shipped the ladder:

| profile | crosses at | frames | cause | `used` vs `budget` |
|---|---|---|---|---|
| `sv_2017` | 1 024 levels | 9 934 | `BudgetExhausted` | 251 677 984 vs 251 658 240 — **19 744 B OVER** |
| `verilog_2005` | 2 048 levels | 8 668 | `NoFrameAffordable` | 251 643 616 vs 251 658 240 — **14 624 B UNDER** |

A stop condition admitting only the starred cause would have read `verilog_2005`'s own crossing as
a non-event and kept doubling — 4 096, 8 192, … — until the 2 048 s hang detector fired. The failure
would have surfaced as *"the ceiling is unreachable in this family"*, pointing at the engine, for a
defect entirely in the reader.

## What actually saved it

Not care, and worth being honest about: the arm was written to admit **both** causes on the
defensive reading that a five-member enum should be handled by definition rather than by emphasis.
There was **no instance of `NoFrameAffordable` in hand** when it was written — it was a hedge, and
the measurement is what turned it into a fact. ⇒ the hedge is the cheap part; **running the thing
that would expose the wrong reading is what makes it knowledge.**

## The generalisation

Both stack-grounds causes satisfy one inequality — `used + bytes_per_frame > budget` — because they
differ only in whether the last whole frame still fitted. That is what a predicate re-derived from
definitions looks like: it covers the domain and needs no branch on the label. The anchoring lock
checks that inequality rather than matching a cause, which is why it cannot rot when a sixth cause
is added.

## Related

[[an-x-is-checked-by-nothing-claim-is-a-census-claim]] — the same shape one level up: a claim about
a POPULATION assembled from what you thought of rather than from what exists.
[[contacting-a-limit-is-not-the-same-as-being-stopped-by-it]] — the `.69`(d) finding this ladder's
other half is built on, and the reason the stop condition cannot read the verdict alone.
