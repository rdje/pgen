---
id: a-deterministic-counter-cannot-see-a-per-entry-cost-rise
title: An event counter can be exact, deterministic and machine-independent — and still ~9x blind to the regression it was chosen to watch, because the cost moved PER event and not in the COUNT
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
evidence: ENGINE-UNIVERSAL-SERVICES.20 slice 1, corrected by .21. The guarded left-recursion admission cost **+24.3 %** SV parse time (`.17` slice 9, clean one-binary A/B). A full 16 336-file rule-entry census measured the admission's own rule family at **24 644 435 / 899 064 022 entries = 2.741 %** of all entries (published as 0.681 % until `.21` corrected a classifier that counted 97 of 127 LR rule names), and the flip's entry DELTA is strictly smaller (the rules it replaced were themselves entered). ⇒ entries moved ~2.7 % while wall clock moved +24.3 %: the counter is at least ~8.9x less sensitive to that regression. Both metrics shipped, with the bound published on every run.
reverify: "bash scripts/check_parse_cost_ratchet.sh   # OK; then read docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/cost.md — the 2.741 % bound and the ~8.9x statement are printed by the instrument from a NAMED constant, not hand-written into prose"
---

**A counter counts EVENTS. A slowdown can live entirely in the COST OF AN EVENT.** When it does, a
counter-based ratchet is exact, deterministic, reproducible on any machine — and nearly blind.

PGEN needed a standing guard against parse-cost regressions, after a change shipped at **+24.3 %**
parse time with every gate in the repository green. Wall clock was rejected as the primary metric
for a good reason, already paid for: the first figure recorded for that same regression was
`~11 %`, and it was wrong because it compared against a baseline measured under *"materially faster
machine conditions"*. A machine-dependent number had biased the estimate, and in the flattering
direction.

So the ratchet was specified on a deterministic substrate: exact per-rule ENTRY counters, verified
deterministic across repeated runs and byte-identical between the debug and release builds. The
reasoning was that entries are *"the mechanism the cost moves through"*.

Then the census was run.

```text
corpus rule entries                      899,064,022
the changed rule family's entries         24,644,435   = 2.741 %
wall-clock delta for the same change            +24.3 %
```

The family the change created accounts for **2.741 %** of all entries, and the change's *delta* is
smaller still, because the rules it replaced were themselves being entered before. The counter moves
by a couple of percent where the clock moves **24 %** — roughly a **9×** sensitivity gap. The metric
is not wrong; it is answering a different question. The parse did not do proportionally MORE rule
entries — each entry got MORE EXPENSIVE.

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

## What to do about it — measure the coupling, publish the bound, keep both metrics

The wrong responses are both tempting: discard the counter (and lose the only metric that survives a
machine change), or ship it quietly (and let a `+24 %` regression pass a green gate again).

1. **Measure the coupling instead of assuming it — with a classifier derived from the PRODUCER.**
   The `2.741 %` above cost one full-corpus census and is the single most decision-relevant number
   in the whole instrument. Nobody had it before it was measured, and the design rationale had
   already been written as though it were high. ⛔ Then it was measured *wrong*, by four times,
   because the classifier came from the rationale's own prose (`X_lr_base ( X_lr_suffix )*`) instead
   of from the code that emits the names — which emits **eight** shapes across two files. Derive the
   membership test from the emission sites and give it one control per shape.
2. **Publish the bound where the number is read**, in the same words every time — the artifact, the
   gate header, the toolbox entry, the doctrine mirror. A proxy whose weakness is documented once,
   in a task file, is a proxy that will be over-read within two sessions.
3. **Keep the noisy metric as a declared advisory** on a band wide enough that it never cries wolf,
   and say which one binds. Two metrics covering two different failure modes beats one number that
   quietly means less than it appears to.
4. **Look for a sharper counter before settling.** Here, the raw entry counter had a strictly better
   sibling available at *identical* cost: `raw − committed` = FAILED SPECULATION, the probing work a
   structural guard actually spends through. Measured, **98.3 %** of sampled entries are rolled back,
   and the LR family commits **514 of 12 440 690**. Same determinism, same machine-independence,
   far closer to the mechanism. It was found by measuring the alternative rather than by taking the
   first deterministic thing to hand.

```python
# the ratchet binds on THREE exact counters, not one — and none of them claims to price the
# fused execution graph the +24.3 % was measured on. The report says so on every run.
BINDING = ("entries", "committed", "memo_hits")
```

## ⭐ And the bound is what keeps the gate honest, not what weakens it

Stating *"at least ~8.9x less sensitive to this regression"* does not make the ratchet worth less. It
makes it **usable**: it catches structural growth exactly — a probe arm fired it on a rise of 349
entries in 416 841 264, `+0.00 %`, because an exact comparison has no band for a regression to hide
in — and it tells the next reader precisely which question to take somewhere else. The alternative
is a gate that is trusted for a job it cannot do, which is the failure the whole surface exists to
end ([[a-check-whose-inputs-all-pass-has-not-been-tested]]).

⇒ **a proxy with a measured bound is an instrument; a proxy with an assumed bound is a claim.**
