---
id: a-conservative-criterion-and-a-measurement-are-different-objects
title: A criterion built to never MISS a hazard cannot tell you a hazard exists — reading its refusal as a measurement sends whole slices designing around something that was never there
answers:
  - "my analysis says every candidate is unsafe — does that mean the construct is unfixable"
  - "how do I tell whether a static analysis verdict is a hazard or just its own conservatism"
  - "a report says 0/28 safe — can I conclude the fix is impossible"
  - "when may I promote a refined criterion into the verdict a pass acts on"
  - "should a report print the conservative verdict or the measured one"
  - "how do I price a fix against a report that only says no"
  - "my analysis says a fix is feasible — does that mean it closes the defect"
  - "why did a starvation analysis count holders that cannot starve"
tags: [analysis, static-analysis, instruments, first-sets, left-recursion, ast-pipeline, evidence]
date: 2026-08-13
status: current
evidence: rust/src/ast_pipeline/indirect_lr_plan.rs (`collect_starvation_sites` fires on any non-empty residual and consults no FIRST set; `assess_guard` adds the measurement beside it, without changing `is_starvation_safe`); `./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-indirect-lr-plan` printing `starvation-safe candidates: 0/28` next to `guard-feasible candidates: 16/28` and `residual_nullable=29`; ENGINE-UNIVERSAL-SERVICES.17 slice 2
reverify: "bash docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh   # 9/9; C1 pins 0/28 conservative vs 16/28 measured, C7 pins 157/157 over-approximated"
---

**A conservative criterion is designed so that a `safe` verdict is never wrong. That says nothing
about what an `unsafe` verdict means** — it collapses "I measured a hazard" and "I did not look"
into one word, and the difference is usually where the fix lives.

PGEN's indirect-LR survey decided starvation structurally: any rule holding the candidate at a left
corner with a **non-empty residual** is a hazard. Correct, cheap, and it cannot miss. It also
consults no FIRST set, so it cannot see that a residual which can match **empty** belongs to a
holder that cannot starve at all. Measured on the shipped grammar: **29 of 126** SystemVerilog
starvation sites are exactly that shape.

For two slices `starvation-safe candidates: 0/28` was quoted as *"UNPLANNABLE by chain absorption at
ANY base rule"*. The honest reading was always *"the criterion I have refuses all 28"*. When the
FIRST machinery that had shipped since `RGX-0078.5.c.2` was finally called, the same population
split **16/28** in favour of a fix — including both of SystemVerilog's remaining knots, at their
dominators, with one guard variant each.

## The tell

A verdict is doing conservatism, not measurement, when **every input gets the same answer**. `0/28`
and `28/28` are both worth a second look; a criterion that discriminates produces a spread. Ask what
the criterion actually reads: if it never consults the property the hazard is defined in terms of
(here, whether two things can begin on the same byte), its `unsafe` is a default, not a finding.

## What to do about it — print both, promote neither by accident

Add the measurement **beside** the conservative verdict, and leave the conservative one load-bearing:

```text
starvation-safe candidates: 0/28                 ← what the pass ACTS on, unchanged
guard-feasible candidates: 16/28                 ← what it would cost to fix
guard-verdict census: guard_incomplete=68 guardable=29 residual_nullable=29
```

⛔ **Promoting the refined criterion into the verdict is a behaviour change, not a reporting one.**
Here it would change which knots the eliminator absorbs — a real parser change, owed a two-sided
repro ratchet ([[a-grammar-rewrite-must-verify-its-own-postcondition-because-a-static-criterion-only-answers-what-you-encoded]]).
A slice that refines a criterion and applies it in the same breath has spent its evidence budget on
the wrong half.

## The corollary that costs the most

**A refined criterion can be sound and still never fire.** The disjointness case above is dead on
SystemVerilog — `trivia := (line_comment | block_comment)*` is nullable and leads every token, so
`/` is in every FIRST set and no two are ever disjoint. That is not a bug and not contamination; it
is the grammar's layout model, and finding it out required root-causing a surprising number rather
than classifying it ([[a-check-whose-inputs-all-pass-has-not-been-tested]]). Pin such a bucket at
**zero in both directions** so the day it becomes live is noticed rather than assumed.

## ⛔ And say which of the two a positive verdict is

Splitting the criterion in two is only half the job: the refined verdict then has to say whether it
is a **proof** or a **possibility**, or readers will spend the difference. Here `guard-feasible
16/28` means *a guard is expressible and provably sound at 16 candidates* — not *16 knots close*.
The byte test it would emit is an over-approximation at **157 of 157** sites, so the guard can
silently decline to fire. Sound and closed are different claims, and the report now prints `~` on
every approximated set rather than leaving the distinction in a README:

```text
guard: FEASIBLE  suffix_first={'/}~  variants=1 {'/}~  max_hops=1
```

The general rule: **a refined criterion needs its own confidence marker in the output**, because the
moment it is quoted as a headline someone will read the optimistic half. Measuring exactness turned
the "cheap option" of an earlier design slice into the one that is not provably sufficient — a
conclusion no amount of re-reading the same verdict would have produced.

See also [[a-transparent-rule-inherits-the-greed-of-the-rule-it-forwards-to]] — the same survey's
other criterion, and the case where widening a conservative check was the *fix*.
