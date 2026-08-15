---
id: a-conservation-control-cannot-catch-a-misassignment
title: A control that checks a TOTAL cannot catch a defect that MOVES value between buckets — the sum is identical either way, so only an INDEPENDENT oracle sees it
answers:
  - "my instrument has a ground-truth control and it passes — is that enough"
  - "what kind of control should I put on a tool that attributes a total across categories"
  - "the totals add up but the per-category numbers look wrong — how do I tell"
  - "how do I validate a profiler / attribution / accounting script before publishing its numbers"
  - "I am auditing a published measurement and my re-derivation disagrees — which one is wrong"
  - "what is the cheapest control that would have caught this"
  - "why did my checksum-style control pass on a broken parser"
tags: [instruments, ground-truth, controls, measurement, profiling, audit, evidence]
date: 2026-08-15
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.20 slice 4. A `/usr/bin/sample` attribution script carried the control `sum(self_time) == worker-thread root count`; it PASSED on all 8 reports while the script was silently ingesting `sample`'s second section (`Total number in stack (recursive counted multiple, when >=5)`, 694 lines) as call-graph rows, re-parenting **14 022** samples onto the last row of section 1. Headline effect — LR self-time read **18.12 %** instead of **2.27 %** (an **8×** error) and inclusive **39.52 %** instead of **23.67 %**. Caught by an EXTERNAL oracle: `sample`'s own `Sort by top of stack` per-symbol table, where **2 of 195** symbols disagreed, one with a self-time of **−14 015**.
reverify: "python3 -c \"import re,sys; sys.exit(0)\"   # the durable artefact is the control itself: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md `.20` slice 4 records controls C1-C5, of which C4 (per-symbol agreement with sample's own table) is the one that fired and C2 (conservation) is the one that did not"
---

**A conservation control constrains the AGGREGATE. A misassignment preserves the aggregate.** So the
two are, by construction, blind to each other — and the conservation control is the one that feels
like proof.

## What happened

An attribution script had to answer *"what share of a parse's CPU is the left-recursion machinery's
own self-time?"* from a `/usr/bin/sample` report. Self-time was derived from the call graph as
`self(node) = count(node) − Σ count(direct children)`, and the script carried what looked like an
airtight ground-truth control:

```python
assert sum(self_counts) == worker_thread_root_count   # C2 — conservation
```

That identity is genuinely true of a correct walk: every non-root node is subtracted from exactly
one parent, so the self-times must sum to the roots. It passed on every report.

The walk was still wrong. A `sample` report has **four** sections and only the first is a call
graph:

```text
Call graph:
Total number in stack (recursive counted multiple, when >=5):   <- NOT a call graph
Sort by top of stack, same collapsed (when >= 5):
Binary Images:
```

The parser stopped at sections 3 and 4 but not at section 2, so 694 lines of a *different* table
were read as call-graph rows and re-parented onto the last node of section 1.

⭐ **And C2 passed anyway — necessarily.** The samples wrongly subtracted from one node's self-time
reappear as the mis-parented rows' own self-time. Value moved between buckets; the total never
changed. A conservation control cannot express the difference.

## Why it mattered

The corrupted numbers were not subtly off. They were off by **8×** on the headline, and they were
off in a direction that produced a *conclusion*: a draft finding declaring that a previously
published measurement was defective. The published measurement was fine. **The audit's own
instrument was the defect** — and it had a green control.

## The tell

**Ask what your control would still permit.** Write the identity down and then ask: *what class of
bug satisfies this?*

| control | catches | permits |
|---|---|---|
| `sum(parts) == total` | dropped rows, double-counted rows, arithmetic slips | ❌ **any redistribution between parts** |
| `count(rows) == expected` | truncation, a missed file | ❌ wrong value in every row |
| a hash of the inputs | stale inputs | ❌ every logic bug downstream of them |
| **per-bucket agreement with an independent source** | ✅ misassignment, redistribution, wrong bucketing | genuinely little |

Internal identities are checks on *arithmetic*. They are not checks on *semantics*. If the tool's
job is to divide a total among categories, then every control that only inspects the total is
checking the half of the job the tool was never at risk of getting wrong.

## What to do instead — two controls, and the cheap one is nearly free

1. ⭐⭐ **Find the independent oracle and bind to it per bucket.** Here the oracle was sitting in the
   same file: `sample` publishes its *own* per-symbol self-time table. Requiring the script's
   per-symbol numbers to equal that table for **every symbol sample lists** turned a vague "seems
   right" into 139–209 exact agreements per report — and the two disagreements named the bug. The
   oracle need not cover everything: `sample`'s table omits any symbol under 5 samples, so it
   validates only the head of the distribution. That is still enough, because a parsing defect does
   not politely confine itself to the tail.
2. ⭐ **Assert the sign, range and type that the quantity has BY DEFINITION.** Self-time cannot be
   negative. One line — `if self < 0: refuse` — would have caught this instantly, for free, with no
   oracle at all. A derived quantity almost always has such a property, and almost nobody asserts it
   because it "cannot happen".

```python
# C3 — free, and it fires before any number is published
if self_counts[i] < 0:
    refuse("negative self-time — rows have been re-parented across a section boundary")
```

⇒ and **refuse rather than publish**: the point of a control is to make the instrument decline to
answer, not to print a warning next to a number somebody will quote.

## ⛔ The audit-specific sting

This surfaced while *re-deriving* someone else's published numbers, which is the situation in which
a wrong instrument is most dangerous: a disagreement is read as *"the published claim is defective"*
rather than *"one of these two is defective, and it might be mine."* When a re-derivation disagrees
with a published measurement, **the re-derivation is a new instrument and carries the heavier burden
of proof** — it has been run once, and the thing it disagrees with has at least been read.

⇒ **validate the auditor before believing the audit** — related:
[[a-deterministic-counter-cannot-see-a-per-entry-cost-rise]] (a metric can be exact and still weakly
coupled) and [[a-check-whose-inputs-all-pass-has-not-been-tested]] (a control never seen to FAIL is
not known to work).
