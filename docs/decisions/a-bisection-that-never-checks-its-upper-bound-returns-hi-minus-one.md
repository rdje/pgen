---
name: a-bisection-that-never-checks-its-upper-bound-returns-hi-minus-one
description: "DISCIPLINE (2026-08-28, ENGINE-UNIVERSAL-SERVICES.44(d) third ruling / PGEN-ENGINE-UNIVERSAL-SERVICES-0108) — a search for a boundary reports whatever its own window and rungs allow, and it will do so confidently. MEASURED, twice in one session on the same quantity: a hand-written LADDER put VHDL's parse-accept cliff at depth 400 and SystemVerilog's at 318/320, because those were the deepest rungs each ladder happened to contain; the true cliffs are 681 and 322. Then the BISECTION written to correct that reported VHDL at 499 — because its window was (200, 500) and depth 500 ACCEPTS, so a bisection whose upper bound was never verified to reject converges to hi-1 and prints it as the answer. Caught only because a second bisection with a wider window disagreed. ⇒ a boundary search must ASSERT its bracket (lo accepts, hi rejects) before it searches, and a ladder result must be labelled a lower bound, never the boundary."
id: a-bisection-that-never-checks-its-upper-bound-returns-hi-minus-one
title: "A bisection that never checks its upper bound returns hi−1 — assert the bracket before searching"
date: 2026-08-28
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .44 (d) third ruling (the retracted cliff depths and the VHDL verdict-vs-depth ladder that settled it); producer docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_layout_aware_dispatch.sh arm E"
reverify: "bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_layout_aware_dispatch.sh   # arm E must report vhdl's cliff at 681 and systemverilog's at 322 from a window whose upper end genuinely rejects; widen any window whose top still accepts and the answer must not move"
answers:
  - "my bisection found a boundary, can I trust it"
  - "how do I find the deepest input a parser accepts"
  - "a ladder and a bisection disagree, which is right"
  - "what preconditions does binary search need that I keep skipping"
  - "why did my cliff depth change when I widened the search window"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-28
---

## The rule

Before bisecting for a boundary, **assert the bracket**: the low end must have the property, the high
end must not. A bisection with an unverified upper bound cannot fail — it converges to `hi − 1` and
returns it with the same confidence as a correct answer.

And a **ladder is not a boundary search**. It reports the deepest rung it happened to contain, which
is a *lower bound* on the boundary and must be labelled as one.

## Why — the measurement

The same quantity — the deepest nesting depth a generated parser accepts — was measured wrong twice
in one session, by two different methods, in the same direction.

**First, by ladder.** Two published slices took the accept anchor from the deepest passing rung of a
hand-written depth list:

| family | ladder's top rung | published cliff | **true cliff (bisected)** |
|---|---:|---:|---:|
| systemverilog | 400 | 318, then 320 | **322** |
| vhdl | 400 | 310, then 400 | **681** |

VHDL's ladder stopped at 400. The parser accepts to 681. Every ratio built on that anchor was wrong,
and one of them inverted a verdict.

**Then, by bisection — which was the fix, and was also wrong.** The correcting run searched the
window `(200, 500)` and reported *"VHDL deepest ACCEPTED depth = 499 (first rejected = 500)"*. Depth
500 accepts. The loop had never tested `hi`; it only ever tested midpoints, so it narrowed to
`(499, 500)` and printed `499`. The output even *stated* the false half — "first rejected = 500" —
which is the tell: the code asserted something it had never evaluated.

It was caught only because a second bisection over `(50, 1000)` returned **681**, and two bisections
of a monotone predicate must agree. A direct verdict sweep then confirmed 681 accepts and 682 rejects.

## The shape to reuse

```python
ok_lo, _ = probe(lo);  assert ok_lo,      "bracket: lo must have the property"
ok_hi, _ = probe(hi);  assert not ok_hi,  "bracket: hi must NOT — widen the window"
while lo + 1 < hi: ...
```

- **Assert both ends.** Two extra probes; they are the cheapest part of the search.
- **Never print a fact you did not evaluate.** `first rejected = hi` is a measurement, not a
  consequence of the loop.
- **Widen and re-run as the check.** A bisection of a monotone predicate is window-independent; if the
  answer moves when the window grows, either the bracket was wrong or the predicate is not monotone,
  and both are findings.
- ⚠️ The failure is silent and in the confident direction: the returned depth really does have the
  property, so every downstream sanity check passes. Only the *deepest* claim is false.
