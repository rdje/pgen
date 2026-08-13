---
name: feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence
description: "DISCIPLINE (2026-08-13, ENGINE-UNIVERSAL-SERVICES.17 slice 4) — an instrument's ground truth constrains its VERDICT, never the MECHANISM you read out of it. Before citing a probe as evidence for an explanation, state the competing explanation and ask whether this case would have come out DIFFERENTLY if yours were false. If not, it discriminates nothing and may not be cited for a mechanism — however green, however falsifiable the bank. Measured cost: `.17` slice 1's Q2 (`( \"a\" | \"ab\" ) \"c\"` on `abc` ⇒ ACCEPT) was read as *the choice gives back*, but under the longest_match default the longest alternative is ALSO the one that works, so the verdict is identical either way. It became a decision record and framed three slices; the separating shape (a two-byte follower, so only the SHORTER alternative can finish) REJECTS. ⛔ Every existing probe-bank discipline — declared expectations, both directions, a RED-proof, self-check — was satisfied and NONE could catch it, because the assertion being checked was TRUE."
id: feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence
title: "A control that passes under both hypotheses is not weak evidence — it is none"
date: 2026-08-13
evidence: docs/tasks/artifacts/engine_universal_services/guard_effectiveness/ (cases C1/C2/C3, 37 rows, both oracles); docs/tasks/artifacts/engine_universal_services/quantifier_policy/README.md (the corrected Q2 row); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .17 slice 4 RESULT 4
reverify: "bash docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh --interp-only | grep -q '37/37 as declared' && echo DISCRIMINATING-CONTROLS-HOLD"
answers:
  - "is my probe bank enough to claim a mechanism"
  - "why did a green self-checking gate still let a wrong claim through"
  - "how do I tell a real control from one that proves nothing"
  - "what makes a one-difference pair actually a one-difference pair"
  - "can I cite a passing test as evidence for why something happens"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-13
---

## The rule

Before a measurement is cited as evidence **for an explanation**, write down the competing
explanation and answer one question:

> **Would this case have come out differently if my explanation were false?**

If the answer is no, the case discriminates nothing. It may still be a perfectly good regression
pin — it just cannot carry a *mechanism*. Say so where it is recorded, or construct the input where
the two explanations disagree.

## Why the usual safeguards do not catch this

`.17` slice 1's bank did everything this repository asks of an instrument
([[feedback_instrument_needs_ground_truth]], [[feedback_enumerating_instrument_must_refuse]]): every
case declared its verdict up front, the bank carried must-accept **and** must-reject cases, a
deliberately flipped expectation exited non-zero naming the case, and it reported 7/7.

None of that can help, and the reason is structural: **every one of those safeguards checks whether
the verdict is right.** The verdict *was* right. What was wrong was the sentence written next to it.

| | Q2: `( "a" \| "ab" ) "c"` on `abc` |
|---|---|
| predicted by *"the choice gives back"* | ACCEPT |
| predicted by *"the choice commits to the longest"* | ACCEPT |
| measured | ACCEPT |

The bank's own next row (`longest_match` is the default) is what makes the second column true, so
the refutation was already sitting inside the same file.

## What the fix looks like — it is small

Make the element after the choice **two** bytes instead of one, so only the SHORTER alternative can
finish:

```text
ch := "a" | "ab"
scratch := ch "bc"        on "abc"  ⇒  REJECT      ← the two hypotheses now disagree
```

One character of grammar, and the answer flips. Twelve minutes of work sat on the other side of a
premise that three slices and a decision record were built on.

## The cost when it is missed

The false law framed `.17`'s blocker as *"an asymmetry between two of PGEN's own combinators"*, which
quietly guaranteed that a repair scoped to one combinator would suffice. It does not: the choice
starves its holder too, and a design guarding only the quantifier would have shipped a new regression
on `initial k = int'(1);` — a statement measured to parse at HEAD.

⭐ **The measurement that refuted the law and the measurement that found the second defect are the
same measurement.** That is not a coincidence and it is the real reason this matters: a wrong story
about a mechanism hides exactly the cases the story says cannot exist.

## Practical checks

- **Name the alternative explanation in the probe file itself.** If you cannot state one, you are
  not yet testing a mechanism.
- **Do not run a case that cannot discriminate** — leave it out rather than run it and over-read it.
  (`.17` slice 4 applied this to its own bank: two of its rungs are measured on two inputs each,
  because their other inputs are absorbed by a second alternative and would pass either way.)
- **A one-difference pair needs the difference to be the variable under test.** Q1/Q2 differed in
  the combinator *and* in whether the winning alternative was the one that worked. That is two
  differences, and only one was named.
- **When correcting, keep the verdict and change the reading.** The expectation was never wrong;
  editing it would destroy a working regression pin and hide the real error.

Sibling of [[a-check-whose-inputs-all-pass-has-not-been-tested]] — that one is about inputs that
never exercise a branch, this one about an output that never separates two explanations. Both are
ways a green gate means less than it looks like it means, and neither is visible from inside the
gate. See also [[project_pgen_gives_back_at_neither_combinator]] (the corrected law) and
[[feedback_read_prior_art_before_designing]] (*re-measure an engine claim, never quote it* — which
slice 1 obeyed, and which is not sufficient on its own).
