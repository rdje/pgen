---
id: interleaving-cancels-drift-for-two-arms-and-hides-it-for-three
title: Interleaving cancels host drift for TWO arms and quietly biases THREE — with a fixed order each arm is pinned to a slot, so the bias accumulates every round instead of cancelling
answers:
  - "how do I order the arms of an A/B/C benchmark so machine drift cancels"
  - "my interleaved benchmark still disagrees with itself between passes"
  - "is running A B C A B C enough to remove a warm-up or thermal trend"
  - "why does my third arm always look slowest (or fastest)"
  - "how many rounds does a counterbalanced benchmark need"
  - "I refused a noisy measurement — what else was resting on it"
  - "how do I tell a noisy host from a shrunken effect"
tags: [measurement, benchmarking, performance, evidence, instruments, experiment-design]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.20 slice 5. A three-arm parser A/B ran rounds in the FIXED order `arm2, arm1, arm3`, believing interleaving cancelled drift. It does not for three arms — and the leaf's founding figure, a **+24.3 %** parse-time regression that had driven five slices, three sessions and a director-facing ruling, did not survive re-analysis: 7 estimator x era combinations put ARM2/ARM1 in **[0.9909, 1.0433]**, including the PRE-change raw data of the very run that reported it. Both contaminating passes had run the arms sequentially with ARM 1 LAST.
reverify: "python3 docs/tasks/artifacts/engine_universal_services/guard_ab_cross_era/analyze_cross_era.py   # re-derives all 7 combinations from the tracked 147 KB matrix, asserts provenance against slice 4's published medians, and REFUSES if any estimator reproduces the 1.243 anchor; `--self-test` drives both refusals RED (3/3)"
---

**Interleaving is not counterbalancing, and the difference only shows up at three arms.**

The standard argument for interleaving is sound: run `A B A B` instead of `AAA BBB`, and a monotonic
machine trend — thermal state after a long compile, page cache, a background indexer — lands on both
arms instead of on whichever went first. It cancels because within each pair the slot order reverses.

Extend that to three arms in a fixed order and the argument silently stops holding. With rounds of
`A B C` repeated, and a within-round drift of Δ per slot, arm A takes `+0Δ`, arm B `+1Δ` and arm C
`+2Δ` — **in every round**. Adding rounds does not average this away; it is a constant per-arm offset
that accumulates with the round count. A fixed three-arm order is a slower version of the sequential
run it was meant to replace, and it looks rigorous while being biased.

The fix is a **Latin square**: rotate the order so each arm occupies each slot exactly once.

```
round 1   arm2  arm1  arm3
round 2   arm1  arm3  arm2
round 3   arm3  arm2  arm1
```

Over one complete square a slot-linear drift cancels **exactly**, for every arm, instead of
approximately. This makes the round count part of the design and not a budget decision: it must be a
multiple of the number of arms, or the square is incomplete and the cancellation claim is void. An
instrument that cannot complete its square should refuse rather than report.

PGEN paid the full price for learning this. A three-arm parser comparison reported that a
left-recursion fix cost **+24.3 %** parse time. That number was carried for three sessions: into the
task leaf's heading, into its routing evidence, into a performance ratchet built specifically to
watch it, into a knowledge card's central bound, and into a ruling that forbade a parser family from
reaching `Done` while it stood. Re-derived from the raw per-file data under four estimators, in both
the pre-change and post-change eras, the ratio came out between **0.99 and 1.04** every single time.
The mechanism was the ordering: both passes that produced a large number ran the arms sequentially,
and in both, the cheap-looking arm ran **last** on a host that drifts within a session — the same run
that reported the regression also recorded one arm moving **+21 %** between passes on an identical
binary.

Three practices follow, and the third is the one that actually failed.

**Counterbalance, and prove it from the artifacts.** Record each run's arm, round and slot so the
design can be audited afterwards rather than trusted from the script's own description of itself.

**Make admissibility a computation.** State up front what noise floor would make the effect
unreadable — a per-arm spread under a quarter of the effect is a serviceable bar — and have the
instrument withhold the number when the bar is not met, rather than printing it with a caveat. Carry
a second FIXED anchor (the effect you originally set out to measure) alongside the measured one, or
the bar collapses along with the effect and cannot distinguish *"the host is noisy"* from *"the
effect shrank"*.

**When you refuse a measurement, audit what was already resting on it.** This is the real failure.
The earlier slice DID notice its timing pass was noise-limited, and DID refuse to publish the
absorption-vs-guard split it had gone looking for — correctly, and for exactly the right reason. But
the refusal was scoped to the split. The same pass had also measured the TOTAL effect at **1.002**,
and that reading was left on the floor while the leaf went on carrying **+24.3 %** in its heading.
The tidy number that disagreed with the premise was refused; the untidy number that agreed with it
was retained. A refusal that only reaches the conclusion you were currently chasing is not
skepticism — it is skepticism aimed in one direction, and it will preserve exactly the belief that
put you there.

⛔ **Refuting a wall-clock figure does not refute the cost.** In this case the deterministic tiers
survived untouched and still measure real prices — the same change cost **+10.6 %** rule entries and
**+9.3 %** parser bytes. What collapsed was one axis, wall clock, measured one way. Say which axis
you refuted, and leave the others standing.

Related: [[a-bank-pinned-to-the-shipped-behaviour-is-pinned-to-a-moving-target]] (a baseline measured
under other conditions), [[a-deterministic-counter-cannot-see-a-per-entry-cost-rise]] (whose numeric
bound was derived from the figure this card refutes),
[[a-conservation-control-cannot-catch-a-misassignment]] and
[[deriving-a-control-from-the-producer-is-not-the-same-as-observing-it]] (controls that cannot fail
the way you need them to).
