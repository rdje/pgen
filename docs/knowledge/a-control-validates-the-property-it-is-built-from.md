---
id: a-control-validates-the-property-it-is-built-from
title: A control validates the property it is built from — a NEW property needs a new argument, never the old control run again
answers:
  - "my instrument has a control and it passes — is the new conclusion covered by it"
  - "I added a question to an existing instrument, do I need a new control"
  - "the control is green but the answer feels wrong — what can a control not see"
  - "is a passing control evidence that the ordering/succession logic is right"
  - "how do I tell whether my control is blind to the thing I just started measuring"
  - "I am comparing two counters — are they even on the same clock"
  - "why did my cross-check pass while the derived verdict was nonsense"
  - "when is it safe to reuse an instrument's existing control for a new claim"
tags: [instruments, controls, evidence, verification, measurement, allocators, ordering]
date: 2026-08-29
status: current
evidence: "ENGINE-UNIVERSAL-SERVICES.67(c)(iii) (PGEN-ENGINE-UNIVERSAL-SERVICES-0131). The allocation LIFETIME tape in rust/src/bin/parse_alloc_census_probe.rs stamped allocations with ALLOC_CALLS and releases with FREE_CALLS — two counters that count different events — so every succession verdict (alloc(A) < alloc(B) < free(A)) compared incomparable numbers. Its control, `live at instant I`, reconstructs the size-class table at the snapshot instant and passed with 148 of 148 (input, class) pairs matching byte-for-byte, BOTH before and after the defect was fixed: that comparison puts alloc ordinals against I's alloc ordinal and free ordinals against I's free ordinal, staying inside one clock each time. Fixed with a single EVENT_SEQ that every allocation AND every release advances."
reverify: "python3 docs/tasks/artifacts/eus67c/rehash_spike_report.py docs/tasks/artifacts/eus67c/rehash_tape.tsv docs/tasks/artifacts/eus67c/rehash_meta.tsv docs/tasks/artifacts/eus67c/rehash_classes.tsv | head -3   # '148 (input, class) pairs ... 0 mismatched'. This re-derives the control and shows its SCOPE is residency-at-an-instant; it does NOT replay the two-clock form, which was never committed (see the honest limit below)."
---

**A control is not a blessing on an instrument. It is evidence about ONE property, and it stays
silent about every other property that instrument is later asked to report.**

The failure is easy to miss because the control keeps passing. You extend an instrument to answer a
new question, the existing control still runs, still goes green, and the green reads as coverage.
It is not: the control was designed around the old property, and its comparison may be structurally
incapable of noticing that the new one is wrong.

## The measured instance

An allocation census probe kept two counters — `ALLOC_CALLS`, incremented on every allocation, and
`FREE_CALLS`, incremented on every release. A lifetime tape was added to answer a question the
existing size-class snapshot could not: **succession** — *was table A still live when table B was
allocated, i.e. did B replace A?* That is decided by `alloc(A) < alloc(B) < free(A)`.

Those three numbers came from two different counters. An allocation stamped `alloc_ord=988` beside
a release stamped `free_ord=195` is not out of order; the two are simply not comparable, and every
succession verdict was meaningless.

The tape's control could not have caught it. The control asks *"which allocations were live at
instant I?"* and answers with

```text
alloc_ord(X) <= alloc_ord(I)   AND   free_ord(X) > free_ord(I)
```

Each comparison stays **inside one clock**. So the control reconstructs the size-class table
correctly — 148 of 148 (input, class) pairs byte-for-byte — whether or not the two clocks are
comparable with each other. It is a control over *residency at an instant*, and succession is a
control over *ordering between events*. Different property, no coverage.

The fix was a single `EVENT_SEQ` that every allocation **and** every release advances, so that a
comparison between an allocation and a release is a comparison between two readings of one counter.

⚠️ **Honest limit, added by the same session's self-audit.** The two-clock form was never committed,
so *"the control would still have passed"* is **reasoning about the code, not a replay of it**: the
committed analyser's `live_at` already takes a single cut. What IS checkable from the tree is the
part the rule rests on — that `ALLOC_CALLS` and `FREE_CALLS` are separate statics, and that the
control's comparison is residency-at-an-instant rather than ordering-between-events. Treat the
counterfactual as an argument you can follow, not as a measurement you can re-run.

## The rule

Before quoting an existing control as evidence for a new claim, ask: **what would have to be wrong
for this control to notice?** Write that sentence down. If it does not mention the property the new
claim rests on, the control is silent about it and the claim is unverified.

Two practical corollaries:

- **Two counters are two clocks.** Any comparison that crosses them needs an argument, not an
  assumption. Prefer one monotone sequence that every event advances over per-kind ordinals when
  ordering matters at all.
- **A green control on a widened instrument is the most persuasive kind of nothing.** It was earned
  by the old property and spends none of that credit on the new one.

Related: [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] — a census claim needs its own
census; this is the same shape one level down, where a *control* claim needs its own control.
