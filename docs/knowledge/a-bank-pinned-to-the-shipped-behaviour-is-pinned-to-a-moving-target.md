---
id: a-bank-pinned-to-the-shipped-behaviour-is-pinned-to-a-moving-target
title: A bank that measures "what the shipped path leaves undone" goes RED the day the shipped path does it — and every one of those reds reads as good news
answers:
  - "my probe bank went red after a behaviour change and the new numbers look better — is that a regression"
  - "a census dropped from 129/129 to 0/0 — did the problem go away"
  - "how do I write a bank whose subject survives the fix it is measuring"
  - "should a bank name the policy it measures, or just run the default command"
  - "why did a would_absorb=2 become would_absorb=0 after we shipped the absorption"
  - "how do I keep a before/after baseline once the after becomes the default"
  - "is a zero-check still a check after the thing it counted became non-zero on purpose"
tags: [instruments, probe-banks, evidence, regression, baselines, ast-pipeline, gates]
date: 2026-08-14
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.17 slice 9 (the indirect-LR admission flip). `docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh` went C1 `0/28 → 0/0`, C7 `129/129 → 0/0`, C8a `32 → 0`; `guard_dry_run/probe.sh` went D1 `2/0 → 0/0`, D13 `6 → 0`. Both were CORRECT extractions of an EMPTY population — the knots they census had been absorbed. Fixed by passing `--indirect-lr-admit-starvation-safe-only` at each report site, after which all 36 declared values are byte-identical to the pre-flip run.
reverify: "bash docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh && bash docs/tasks/artifacts/engine_universal_services/guard_dry_run/probe.sh   # 18/18 and 18/18; every value unchanged across the flip because each names its admission"
---

**A bank whose subject is *"the work the shipped path has not done"* stops having a subject the day
the shipped path does it.** The extraction still works, the rows still parse, the totals still
print — and every number collapses toward zero, which is the shape of success.

`ENGINE-UNIVERSAL-SERVICES.17` flipped one criterion in PGEN's indirect-left-recursion eliminator:
it began absorbing knots that a call-site guard makes safe, instead of declining them. Two probe
banks measured that population — how many candidates are starved, how many guard byte tests are
over-approximated, how many sites need a trailing guard. Both went RED on the first post-flip run:

```text
C1   starvation-safe / guard-feasible     0/28 / 16/28   ->   0/0 / 0/0
C7   over-approximated guard byte tests   129/129        ->   0/0
C8a  sites that need the TRAILING guard   32             ->   0
D1   would_absorb / would_refuse          2/0            ->   0/0
D13  sites where the chain BRANCHES       6              ->   0
```

Read cold, that table says *no starved candidates, no approximated guards, nothing left to absorb* —
a page of good news. It is a page of empty sets. The candidates were not fixed one by one; they
stopped being candidates, because the pass ate the knots they lived on.

## Why this is not the scraper class

The related failure — [[a-report-scraper-must-anchor-on-structure-not-on-a-substring]] — is about an
extraction reading the wrong thing. Here **every extraction was correct**. The `grep` matched the
right rows, the `sed` cut the right fields, the totals derived honestly from the cases that ran.
What moved was the **subject**: the phrase "the shipped grammar" silently means "the grammar as the
current policy leaves it", and a policy is a thing that changes.

⇒ the two are different retrieval keys. *"Did my grep match the right rows?"* and *"is the
population these rows are about still the one I meant?"* are asked at different times and answered
by different evidence.

## The tell

**A bank is pinned to a moving target when its command line names no policy.** If the report
invocation is the bare default — `--report-indirect-lr-plan`, `--lint-grammar`, `status` — then the
bank measures *whatever the current default does*, and its rows are only meaningful while that
default holds. That is invisible for as long as nobody changes the default, which is exactly how
long the bank looks correct.

Ask of each row: *if the defect this measures were FIXED tomorrow, would this row go green, go red,
or become vacuous?* Vacuous is the dangerous answer, and it is dangerous because it prints as green
once the expectation is "helpfully" updated to match.

## What to do about it — name the admission, not the default

Pass the policy explicitly at every report site, with the reason written at the variable:

```bash
# the census rows: the population is "knots the eliminator has NOT absorbed", so ask for the
# admission under which they are unabsorbed. Bare defaults would measure an empty set.
NARROW="--indirect-lr-admit-starvation-safe-only"
SV_REPORT="$("$PIPELINE" grammars/systemverilog.ebnf --report-indirect-lr-plan "$NARROW")"

# the shipped rows: no lever, deliberately — these are about what `make` builds.
SV_SHIPPED="$("$PIPELINE" grammars/systemverilog.ebnf --report-indirect-lr-plan)"
```

After that edit **all 36 declared values across both banks were byte-identical to the pre-flip
run** — and that invariance became the slice's strongest evidence, because the dry run had been
*predicting* `would_absorb=2` at those two rules since three slices earlier. A bank that keeps its
subject can testify about the change; a bank that loses it can only be re-baselined.

⭐ **The pattern is worth reaching for before the flip, not after.** The same file already contained
one row doing it right — a case that had always passed `--no-eliminate-indirect-left-recursion` to
see a knot *before* the pass ate it, with that reason beside it. One author saw the hazard for one
row and nobody generalised it to the file.

## ⛔ And a zero-check does not survive the flip either

The companion failure is a gate asserting `counter == 0` as its safety property, when the whole
point of the change is to make that counter non-zero on purpose. `indirect_guard_chains=0 on every
shipped grammar` was a real protection — *"non-zero means the admission changed"* — and after the
flip it would have pinned the **absence of the fix**.

Replacing it with `== 3` is not enough. The honest replacement is an **exact set**:

```text
casting_type_lr_guard0[loop],casting_type_lr_guard1[loop],property_expr_lr_guard0[loop+trailing]
```

which fails on a silent revert (the set empties), on an accidental widening (a fourth chain), and on
a re-plan that moves a guard position — three distinct parser-behaviour changes a count cannot
separate. ⛔ A count of `0` also cannot distinguish *"the criterion holds"* from *"the pass stopped
running"*; a set can ([[a-fused-counter-is-not-evidence-about-any-of-its-parts]]).

See also [[a-conservative-criterion-and-a-measurement-are-different-objects]] — the criterion this
flip promoted, and the slice that deliberately did not promote it.
