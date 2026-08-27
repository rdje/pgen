---
name: read-the-line-the-instrument-printed-before-explaining-the-one-you-computed
description: "DISCIPLINE (2026-08-27, ENGINE-UNIVERSAL-SERVICES.44(b) step 2 / PGEN-ENGINE-UNIVERSAL-SERVICES-0102 retracted by -0103) — when a rich instrument prints several families of output and you use one of them, read the OTHERS against your conclusion before explaining anything from source. MEASURED: a fusibility-census run printed `INLINE-EXPOSURE-DECIDED: decided_entries=4425993 (23.4% of total) ... memo_hits_on_decided=0`; a separate arm of the same run computed a class of rules with zero memo hits; I explained that class from a source-code hypothesis (a failure-store suppression gate), published it, and a later measurement showed the gate NEVER FIRES on the input — while the census's own line had already given both the mechanism (inlining elides the memo) and the exact population, 4 425 993, digit-for-digit. The instrument was not missing and the run was not repeated: the line was in the committed log. ⇒ any printed number matching your finding to more than two significant figures is ABOUT your finding, and a line whose name contains the mechanism is not noise to scroll past."
id: read-the-line-the-instrument-printed-before-explaining-the-one-you-computed
title: "Read the line the instrument printed before explaining the one you computed"
date: 2026-08-27
evidence: docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/logs/attempt_elision_census.txt (arm C prints INLINE-EXPOSURE-DECIDED, arm D computes the zero-hit class — both in the SAME log, committed in PGEN-ENGINE-UNIVERSAL-SERVICES-0102); the retraction and its census are docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .44 (b) STEP 2 self-retraction (PGEN-ENGINE-UNIVERSAL-SERVICES-0103); the eliding emission is rust/src/ast_pipeline/ast_based_generator.rs:7302 (inlined_frame_call, RGX-0078.5.i.4 P1b)
reverify: "bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_attempt_elision.sh 2>&1 | grep -E 'INLINE-EXPOSURE-DECIDED|rules the memo NEVER serves'   # the decided_entries figure and the zero-hit class must be read TOGETHER; if the two lines ever stop agreeing on the inlined share, the inline plan or the counter attribution has moved"
answers:
  - "my counter shows a class of rules doing something strange, where do I look first"
  - "why did I explain a measurement from source code and get it wrong"
  - "what should I do with the parts of a census output I did not ask for"
  - "why do PGEN rules report zero memo hits"
  - "does an inlined rule use the packrat memo"
  - "how do I avoid re-deriving something my own log already says"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

A rich instrument prints several families of line. You ran it for one of them. **The others are not
noise — they are the parts you did not know to ask for**, and they were written by someone who
already understood the machine you are currently guessing about.

So, before explaining a computed number from source code:

1. **Scan every printed line for your number.** Anything matching to more than two significant
   figures is about your finding, not a coincidence.
2. **Read the names.** A line whose name contains a mechanism (`INLINE-…`, `CASCADE-…`) is a claim
   about that mechanism's role in the very run you are interpreting.
3. **Only then** go to the source — and go there to *confirm a mechanism fires*, not to find one
   that could explain the shape.

## What it cost

`ENGINE-UNIVERSAL-SERVICES.44`(b) step 2 ran a four-arm census. Arm C printed, among many lines:

```
INLINE-EXPOSURE-DECIDED: … decided_entries=4425993 (23.4% of total) … memo_hits_on_decided=0
```

Arm D, separately, computed *"438 rules receive zero memo hits, carrying 33.4 % of all calls"*. I
explained arm D from the source: a failure-store suppression gate that refuses to cache a failure
whose body depended on a recursion block outside itself. It has a file:line. Its condition would
produce exactly that signature. **It never executes on this input** — the floor it tests is lowered
only by two arms that record zero events on this parse, so the condition is false throughout.

The census had already said it: **`memo_hits_on_decided=0`** is the mechanism (the P1b emission
elides the packrat memo at inlined frames, and every `record_memo_hit` site lives inside
`memoized_call`), and **4 425 993** is the population — digit-exact with the inlined share of the
zero-hit class. It was in the log I committed, one arm above the number I was explaining.

⇒ 23.4 of the 33.4 points were **an optimization working as designed**, published as a defect.

## The generalisation, and why it is not just "read your output"

The failure is specific and repeatable: **I used the instrument as a source of one number instead of
as a description of the run.** The line I needed had a different name, appeared in a different arm,
and answered a question I had not asked yet. Scrolling past it was not carelessness about a number —
it was treating a rich report as a narrow probe.

Two cheap habits close it:

- **Grep your own log for your own figures** before publishing an explanation of them.
- **When a report's arm is named after a mechanism, assume it has an opinion about your finding** and
  check what that opinion is.

## What it does NOT mean

It does not mean the source read was wrong to do — confirming a mechanism in source is required. It
means source reading tells you what a mechanism *would* do, and only a measurement tells you whether
it *ran*. The gate cited was real, correct, and irrelevant.

Related: [[a-blocker-firing-on-the-whole-population-is-one-fact-not-n]] is the same instrument being
misread in the other direction — a verdict quoted without the histogram that explains it. And
[[a-healthy-cache-is-not-evidence-the-cache-is-the-bottleneck]] is the finding this census was
serving, whose own corollary had to be corrected twice.
