---
id: a-deterministic-counter-cannot-see-a-per-entry-cost-rise
title: A counter counts EVENTS, so a cost that moved PER event is invisible to it — but that blind spot is a PROPERTY of the metric, and the moment you put a NUMBER on it you have made a claim that needs measuring like any other
answers:
  - "should my performance ratchet key on wall clock or on a deterministic counter"
  - "my counter-based gate is green but the thing got measurably slower — how"
  - "how do I build a perf ratchet that survives a different machine or a hosted runner"
  - "a wall-clock baseline gave me the wrong regression number — what do I replace it with"
  - "is a deterministic oracle always better than a noisy direct measurement"
  - "how sensitive is my proxy metric to the regression I care about, actually"
  - "what do I write down when the only machine-independent metric is a weak proxy"
tags: [instruments, performance, ratchets, gates, evidence, proxies, measurement]
date: 2026-08-15
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.20 slice 1, corrected by .21, thesis RE-DERIVED by .26 (2026-08-16). PGEN specified a parse-cost ratchet on exact rule-ENTRY counters and published a numeric bound on their insensitivity — *"at least ~8.9x less sensitive than wall clock"* — derived as **+24.3 % wall clock / 2.741 % family share**. ⛔ BOTH terms were wrong. The numerator is REFUTED (.20 slice 5): not reproducible from the raw data of the runs that produced it, 7 estimator x era combinations giving ARM2/ARM1 in [0.9909, 1.0433], mechanism a fixed-arm-order artifact. The denominator was the wrong QUANTITY independently of that: sensitivity is how much the counter MOVED, not how large the rule family is — and the tracked A/B in the same leaf measures the move at **812 963 769 -> 899 064 022 entries = +10.59 %**, 3.49x LARGER than the family's own 24 644 435 entries. The counters SAW that change plainly. ⇒ the bound is retired, not re-computed (0.41x on the point estimate, 1.82x on the most adversarial pairing, and no admissible wall-clock figure survives). What stands is the STRUCTURAL limit, which needs no number.
reverify: "bash scripts/check_parse_cost_ratchet.sh   # OK — its every-run tier re-hashes the four inputs the 2.761 % share is a function of against docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/family_share.json, re-checks that artifact's raw counts against the share it declares, AND holds this card's `Live LR-family share` anchor equal to it (ENGINE-UNIVERSAL-SERVICES.21 (f), .26); re-derive with `make -C rust SHELL=/bin/bash sv_parse_cost_family_share` (~70 s)"
---

⭐ The CONVERSE blind spot of the same metric is banked separately: it also cannot tell a cache HIT from work, so a rise can be pure cache traffic ([[a-counter-that-cannot-tell-a-cache-hit-from-work-prices-them-alike]]).

**A counter counts EVENTS. A slowdown can live entirely in the COST OF AN EVENT.** When it does, a
counter-based ratchet is exact, deterministic, reproducible on any machine — and blind to that
change. ⭐ **That much is a PROPERTY of the metric: it follows from what the metric counts, needs no
measurement, and cannot go stale.** Everything after it in this card is about what happened when the
blind spot was given a *number*.

PGEN needed a standing guard against parse-cost regressions, after a left-recursion change shipped
with every gate in the repository green. Wall clock was rejected as the primary metric for a good
reason, already paid for: the first figure recorded for that change was `~11 %`, and it was wrong
because it compared against a baseline measured under *"materially faster machine conditions"*. A
machine-dependent number had biased the estimate, in the flattering direction. The corrected figure
was **+24.3 %**.

So the ratchet was specified on a deterministic substrate: exact per-rule ENTRY counters, verified
deterministic across repeated runs and byte-identical between the debug and release builds. And —
correctly — the design refused to ship the proxy without measuring how tightly it was coupled to the
regression it was chosen to watch. That coupling was published as a bound:

```text
wall-clock delta for the change                 +24.3 %     <- REFUTED
the changed rule family's share of all entries    2.741 %
=> "the counter is at least ~8.9x less sensitive"           <- RETIRED
```

⛔⛔ **Both terms were wrong, and they were wrong in different ways — which is the whole lesson.**

- **The numerator was never measurable.** `+24.3 %` is not reproducible from the raw data of the
  runs that produced it: seven estimator × era combinations put the ratio in **[0.9909, 1.0433]**,
  and the mechanism was a fixed-arm-order artifact on a drifting host. It was the *second* wall-clock
  figure for this change to be refuted. Rejecting wall clock as the ratchet's metric while keeping it
  as the ratchet's *numerator* left the whole bound resting on the thing the design had just
  disqualified.
- **The denominator was the wrong quantity, independently.** *"How sensitive is my counter to this
  change?"* is answered by **how much the counter MOVED** — not by how large the changed component
  is. The bound reached for the family's SHARE (2.741 %) and argued the delta must be smaller still,
  *"because the rules it replaced were themselves entered"*. That was an inference; the leaf's own
  tracked A/B measured it: **812 963 769 → 899 064 022 entries = +10.59 %**, which is **3.49×
  LARGER** than the entire family's entry count. The counters did not miss this change. They saw it
  plainly, far outside any band a ratchet could hide.

⇒ **The factor is retired, not re-computed.** Under the point estimate it would be 0.41× (the
counter moving ~2.4× *more* than the clock) and under the most adversarial pairing available only
1.82×; and no admissible wall-clock figure survives to rebuild it from at all. Replacing one
unearned ratio with another would repeat the mistake in a smaller font.

**Live LR-family share `2.761`** (corpus-entry share %). ⭐ Since `ENGINE-UNIVERSAL-SERVICES.21`
acceptance (f) that number is **GATED, not quoted**: it is derived by a full-corpus census into
`docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/family_share.json`, and
`PARSE-COST-RATCHET`'s every-run tier re-hashes the four inputs it is a function of, re-checks the
artifact's raw counts against the share they imply, and holds this paragraph equal to the
derivation. The share itself is a correctly measured quantity and survives all of the above
untouched — what failed was the ratio built on top of it.

⛔⛔ **This card first published `0.681 %` and `~35×`, and both were wrong by a factor of four —
which is itself the lesson's sharpest instance.** The classifier that measured the coupling matched
only `_lr_base`/`_lr_suffix`: **97 of the 127** LR rule names the parser declares, with no
`_lr_seed` and — in a family the code called *guarded* — **not one `_lr_guard` rule**, leaving
**75.1 %** of the family's entries uncounted (`ENGINE-UNIVERSAL-SERVICES.21`). ⭐ Note what that
means and what it does not: the **binding counters were unaffected** (entries, committed and
memo-hits are byte-identical across the correction), so no cost claim moved. What was wrong was the
number describing **how blind the gate is** — the gate had been UNDER-claiming its own sensitivity.
⇒ *measuring your proxy's coupling is step one; measuring it with a classifier derived from the
PRODUCER rather than from your own prose is step two, and step two is the one that was skipped.*
⭐ And `.26` added step three, the sharpest of the three: **the corrected `2.741 %` was right, and
the bound built on it was still wrong** — because a correct denominator in the wrong ROLE, under a
numerator nobody could re-derive, still yields a fabricated number. Fixing the input you noticed is
not the same as re-deriving the claim.

## The tell

**Ask what unit your metric counts, then ask whether the regression changed the number of units or
the price of one.** They are independent, and only one of them is visible to a counter.

| the change... | a counter sees it | wall clock sees it |
|---|---|---|
| adds backtracking / a new loop / a memo miss | ✅ exactly | ✅ noisily |
| makes each existing step traverse more code | ❌ **invisible** | ✅ noisily |
| perturbs i-cache / branch prediction | ❌ **invisible** | ✅ noisily |

The trap is that the counter's *good* properties — exact, deterministic, machine-independent — are
all properties of its **reliability**, and none of them is a property of its **sensitivity**. A
metric can be perfectly trustworthy about a quantity that barely moved. Reliability reads like
rigour, which is why this is worth catching before shipping rather than after.

⛔ This is a different failure from [[a-bank-pinned-to-the-shipped-behaviour-is-pinned-to-a-moving-target]].
There, the extraction was correct and the *population* moved. Here the extraction is correct, the
population is correct, and the **quantity is only weakly coupled to the thing being guarded**.

## What to do about it — state the limit structurally, and price any NUMBER you put on it

The wrong responses are tempting in both directions: discard the counter (and lose the only metric
that survives a machine change), ship it quietly (and let the next regression pass a green gate), or
— the one this project actually took — **quantify the blind spot with whatever two numbers are to
hand and publish the ratio as though it were measured.**

1. **Separate the STRUCTURAL limit from the QUANTIFIED one, and say which you are stating.** *"A
   counter cannot see a cost that moved per event"* is a property: it needs no measurement and never
   expires. *"…and it is ~9× less sensitive here"* is a claim about two measured quantities, and it
   is only as good as the worse of them. Publishing them in one sentence let the second inherit the
   first's air of inevitability. The first belongs in the gate header forever; the second has to be
   earned every time it is stated.
2. **Check the NUMERATOR is a quantity you are willing to defend.** This bound's numerator was wall
   clock — the metric the very same design had just disqualified as unreliable. If a number is too
   noisy to bind a gate, it is too noisy to define that gate's sensitivity.
3. **Check the DENOMINATOR is the right QUANTITY, not merely a right number.** Sensitivity is how
   much *your metric moved*, not how big the changed component is. Those differ by 3.49× here, and
   in the direction that made the gate look blinder than it was. ⛔ The substitution was made by an
   *inference* — "the delta must be smaller than the share" — that a measurement in the same task
   leaf refuted, and nobody went back. **When a later measurement lands, re-read the inferences it
   was standing in for.**
4. **Measure the coupling with a classifier derived from the PRODUCER.** The `2.741 %` cost one
   full-corpus census and is still the most decision-relevant number in the instrument. ⛔ It was
   first measured *wrong*, by four times, because the classifier came from the design's own prose
   (`X_lr_base ( X_lr_suffix )*`) instead of from the code that emits the names — which emits
   **eight** shapes across two files. Derive the membership test from the emission sites, one
   control per shape.
5. **Publish it where it is read, and GATE it there** — the artifact, the gate header, the toolbox
   entry, the doctrine mirror. A number hand-copied into four documents goes stale in four documents
   at once; this one did, twice.
6. **Look for a sharper counter before settling.** Here the raw entry counter had a strictly better
   sibling at *identical* cost: `raw − committed` = FAILED SPECULATION, the probing work a structural
   guard actually spends through. Measured, **98.3 %** of sampled entries are rolled back, and the LR
   family commits **514 of 12 440 690**. Same determinism, same machine-independence, far closer to
   the mechanism — found by measuring the alternative rather than taking the first deterministic
   thing to hand.

```python
# the ratchet binds on THREE exact counters, not one — and none of them claims to price the
# fused execution graph a production parse runs. The report says so on every run.
BINDING = ("entries", "committed", "memo_hits")
```

## ⭐ The structural limit is what keeps the gate honest. The number was what made it look weak.

Removing the ratio does not make the ratchet worth less — and it turns out the ratio had been
*understating* it. On the one change this gate was built for, the binding counters moved **+10.59 %**
while wall clock produced no admissible figure at all. The counters were the half that could answer.

What the ratchet is genuinely good at is unchanged and worth stating without a factor attached: it
catches structural growth exactly — a probe arm fired it on a rise of **349 entries in 416 841 264**,
`+0.00 %`, because an exact comparison has no band for a regression to hide in — and its declared
blind spot tells the next reader which question to take to a sampler instead. The alternative is a
gate trusted for a job it cannot do, which is the failure the whole surface exists to end
([[a-check-whose-inputs-all-pass-has-not-been-tested]]).

⇒ **state a blind spot as a property and it stays true; state it as a ratio and you have taken on a
second measurement to keep alive — one whose failure direction is to make your own gate look worse
than it is.**

⇒ **a proxy with a measured bound is an instrument; a proxy with an assumed bound is a claim.**
