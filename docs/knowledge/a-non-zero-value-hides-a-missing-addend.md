---
id: a-non-zero-value-hides-a-missing-addend
title: A non-zero value hides a missing addend — the row where the count goes to ZERO is the only one where an incomplete sum becomes a visible contradiction
answers:
  - "my counter looks plausible everywhere; how would I know a term is missing"
  - "two of my instruments disagree on one row and agree everywhere else — what does that mean"
  - "where should I look first when auditing a published count"
  - "I added a new code path; what else needs changing"
  - "how did an under-reported number survive across many releases"
  - "is a matching total good corroboration"
tags: [measurement, evidence, instruments, reporting, audit]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.27, 2026-08-16 session #241. A grammar linter published `left_recursion_eliminated=N`, taken from the elimination pass's own outcome. A second pass (indirect elimination) had been added earlier along with its own outcome fields, and the lint's call site was never extended to read them - so the headline counted only the direct pass. Across eleven grammars the omission was invisible on ten of them - SystemVerilog published 2 where the true figure is 5, and "the pass did something" reads as correct. On the eleventh, `ebnf`, the direct count was ZERO while the generated parser demonstrably carried six `_lr_*` rule names emitted by the eliminator, so the published value read as "nothing was eliminated" and two instruments openly contradicted each other. That contradiction is what opened the investigation. The names the count had been hiding were `casting_type` and `property_expr` - the two rules the entire surrounding campaign was about.
reverify: "ast_pipeline grammars/ebnf.ebnf --lint-grammar | head -1   # left_recursion_eliminated=1 (info — 0 direct + 1 indirect); before the fix this read 0 while generated/ebnf.rs declared 6 _lr_* names"
---

**An incomplete sum is almost always indistinguishable from a complete one.** If a counter reports
`2` when the truth is `5`, every consumer sees a plausible number doing a plausible thing. Nothing
in `2` says *"a term is missing"*.

The exception is the row where the reported value reaches **zero**. There, an incomplete sum stops
being *inaccurate* and starts being *false* — and false is loud, because some other artifact
disagrees with it outright.

```text
grammar          published   direct   indirect   truth
systemverilog        2          2         3        5     ← plausible. Nobody asks.
return_annotation    1          1         0        1     ← correct by luck of having no indirect term
ebnf                 0          0         1        1     ← "nothing was eliminated", while the
                                                            generated parser carries six rules only
                                                            the eliminator emits. CONTRADICTION.
```

Ten rows were wrong-or-right in ways nobody could see. One row was **impossible**, and it is the
only reason anyone looked.

## What to do with this

- **When auditing a published count, sort by the value and start at zero.** The zero rows are where
  a missing term is falsifiable against some other artifact. The large rows are where it hides.
- **When you add a code path that contributes to an existing count, grep every READER of the field
  you extended** — not just the surface you happen to be writing. Here the new pass, its outcome
  fields, and its own survey report all landed together and were correct; a *different* reporting
  surface kept reading the old field alone, and the gap opened at that moment.
- ⛔ **A matching total is not corroboration when the totals sum over different populations.** Both
  instruments here reported *"144 rules"* for the same grammar — the one number a reader would
  sanity-check — because a 139-rule grammar with one elimination and a 144-rule grammar with none
  land on the same figure. The agreement was arithmetic coincidence and it actively reassured.
- **Two instruments disagreeing is a gift, and the wrong first question is *"which is stale?"***
  Both were current here; they were measuring **different quantities**. Date them only after you
  have established they are the same quantity at all.

## The severity is not the number

The count being wrong mattered less than what it silenced. The reporting line that *names* the
eliminated rules was gated on the direct list being non-empty, so on a purely-indirect grammar it
**did not print at all** — and on the large grammar it named two of five. The two it omitted were
the exact rules the surrounding campaign had spent multiple task leaves on. ⇒ ask not only *"is this
count right"* but *"what does this count gate, and what is not being printed because of it?"*

Related: [[the-row-that-does-not-fit-the-pattern-is-the-next-investigation]],
[[a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness]].
