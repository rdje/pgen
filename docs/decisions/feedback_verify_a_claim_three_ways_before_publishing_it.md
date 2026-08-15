---
name: feedback-verify-a-claim-three-ways-before-publishing-it
description: STANDING DIRECTIVE (director, 2026-08-15, session #235 — the SECOND time, which is the point) — "before making claims for me to review, please double-check, triple-check that they hold, because we need sota, signoff claims, decisions." Triple-checking is THREE DIFFERENT PASSES, not three repetitions of the same one, because a repeated pass repeats its own blind spot: (1) RE-DERIVE by command from the source; (2) FALSIFY against an INDEPENDENT oracle and prove the control can fail; (3) DURABILITY — is the producer tracked and is the claim watched. All three legs are derived from measured failures in the session that prompted the directive, and leg 3 is the one that was skipped.
metadata:
  node_type: memory
  type: feedback
---

**The directive, verbatim (director, 2026-08-15):** *"From now on, before making claims for me to
review, please double-check, triple-check that they hold, because we need sota, signoff claims,
decisions."*

⛔⛔ **It is the SECOND statement of this rule, and that fact is the most important thing in this
record.** The first was *"You should always double-check all your claims and make sota, signoff and
production-grade decisions"* (quoted in `ENGINE-UNIVERSAL-SERVICES.20` slice 3). That directive was
followed — a self-audit ran and found a real defect — and **the very next two slices still published
claims that did not hold.** So the failure is not willingness. Restating the rule a third time will
not fix it either; what follows is therefore a *procedure with named legs*, so "did I check?" becomes
answerable instead of felt.

## Why one more careful pass does not work

Every claim corrected in session #235 had already been checked once, carefully, by its author:

| the claim | it was checked by… | why that check could not fail |
|---|---|---|
| LR self-time `18.12 %` | a ground-truth control, green | the control was a **conservation** identity (`sum(self) == root`) and the bug was a **misassignment** — which conserves the total |
| the LR family is `0.681 %` of entries | a ten-case control suite that **refuses** on a miss | all ten cases were drawn from the same **prose** as the classifier, so they could only agree with it |
| *"two independent instruments agree, 75.1 % vs 76 %"* | cross-checking two measurements | the **quantities** were independent; the **classifier was shared**, and the classifier was the defect |
| the corrected `2.741 %` | re-derived from a fresh 16 335-file census | correct — and then written into a hand-carried constant guarded by a **comment** |

⇒ **a second pass of the same kind re-runs the same blind spot.** Triple-checking has to mean three
*different* questions.

## The three legs

**1 — RE-DERIVE. Does it reproduce, by command, from the source?**
Not "do I remember measuring it" and not "is it written down consistently". Run the command, paste
the output. Applies to every number, including one taken from this repository's own prose — measured
in `.20` slice 4, `TOOLBOX.md` 5.1 published a left-recursion status that had been stale in two of
three numbers since the flip landed, and nothing noticed because nobody re-ran the lint.

**2 — FALSIFY. What would make this false, and is there an oracle that is not mine?**
- Name the competing hypothesis and find evidence that **separates** them. Symbol counts could not
  separate linker-folding from inlining; a `bl` target address could
  ([[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]]).
- Prefer an oracle the claim's author did not build. `/usr/bin/sample`'s own per-symbol table caught
  an 8× error that the author's own control had passed.
- **Make the control fail on purpose.** A control never observed RED is not known to work
  ([[a-check-whose-inputs-all-pass-has-not-been-tested]]). The 21-case predicate suite was only
  trustworthy once it was run against the OLD predicate and missed 8 of 21.
- Derive membership tests, classifiers and shape lists from the **producer** (the code that emits),
  never from the description of the producer.

**3 — DURABILITY. Can the reader re-run it, and does anything fail when it goes stale?**
⭐ **This is the leg that was skipped, and it is the one that makes a claim signoff-grade rather than
merely true-today.** Two questions:
- **Is the producer tracked?** A measured number whose instrument sits in a gitignored scratch
  directory is a *"trust me"* with extra steps. Measured: four audit instruments and eight profile
  reports, `git ls-files` → **0**, leaving published intervals unreproducible.
- **Is the claim watched?** A number nothing re-derives will go stale silently. Replacing a wrong
  unwatched number with a right unwatched number is not a fix — `0.681 %` became `2.741 %` and
  remained referenced only by the file that defines it, guarded by a *comment*, while the census
  that would re-derive it costs a measured **71 s** and is invoked by nothing.
  `DOCTRINE_ENFORCEMENT.md` §1: a rule nothing checks is a suggestion. The same is true of a number.

## What to hand the director

State the claim, then the leg that earns it. When a leg is missing, **say which one** rather than
publishing the claim unqualified — *"re-derived and falsified; NOT durable, the instrument is
untracked"* is a signoff-grade sentence. **A claim with a named gap is usable; a claim with a hidden
gap is the defect.** And an interval beats a point estimate for anything stochastic: `.20` slice 2's
single-run *"agreeing within 0.6 points"* was luck, since the within-file spread alone is 3.1 points.

⛔ Finally, the asymmetry that catches auditors: **when a re-derivation disagrees with a published
number, the re-derivation is the newer instrument and carries the heavier burden of proof.** It has
been run once; the thing it contradicts has at least been read. Session #235 drafted a finding
accusing a prior slice of a defect, and the defect was in the auditor's own parser
([[a-conservation-control-cannot-catch-a-misassignment]]).

⇒ **re-derive · falsify · make durable — three different questions, in that order.**
