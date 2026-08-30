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

⭐ **Generalized into a portable standard at `docs/CLAIM_VERIFICATION.md`** (director-requested,
2026-08-15) — the 5th portable architecture, project- and harness-agnostic, so other projects and
other agents can adopt the same procedure. This record is the PGEN-local *why*; that file is the
adoptable *what*. Keep them in step: the three legs are defined there.

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

## ⭐⭐⭐ THE BAR IS THE OUTCOME, NOT THE RITUAL — director, 2026-08-30 (the THIRD statement)

Verbatim, on being handed a findings callout: *"I guess you double, triple checked the accuracy of
your finding. … ensure that's sota and signoff."* Then, when the reply began a re-audit:
*"I am not sure about the 3-way thing, but you should stand by them, that's all I am saying."* And
finally, the rule itself:

> *"What I am expecting is that whenever you publish the claim, I am assuming you made all the
> necessary check, double-check, triple-check or your 3-way check to be able to tell right away
> 'Yes, I stand by them'. without being force to start a re-verification or re-audit every time I
> ask you 'are you sure about your findings' or 'do your finding hold' or 'do you stand by your
> findings'."*

⛔ **This RE-AIMS the record.** The three legs above are a means; they are not the deliverable. The
deliverable is that **at publication time the verification is already complete**, so *"do you stand
by them?"* is answerable in one word and never opens an investigation. A re-audit triggered by that
question is itself the evidence that the claim was published before it was earned — the cost is paid
by the director, in trust, not by the author.

⛔⛔ **AND THE CHALLENGE THAT PROMPTED THIS WAS RIGHT, WHICH IS WHY IT IS RECORDED HERE.** Of the
five findings in that callout, re-auditing found:

⛔⛔ **AND THE DIRECTOR SET THE TEST FOR WHAT COUNTS AS *WRONG*, WHICH IS NOT "the number moved"**:

> *"the prose in your finding shall roughly stay true no matter how many times you verify … A
> specific number maybe a little bit off as long as it capture the trajectory of what it tries to
> measure. Ideally of course the number shall be accurate, but … if you re-verify N times, we may
> end up with N different number value, but … those number supposed to measure one particular
> aspect shall be close enough."*

⇒ score a re-audit on **(a) does the PROSE still hold** and **(b) is the number CLOSE on the aspect
it measures** — and, separately, **(c) was any NAMED INSTANCE false**, because a named example is
not an estimate, it is a specific assertion about a specific row.

| the published finding | prose | number | verdict |
|---|---|---|---|
| *"the two prose status arms DISAGREE on 4 of 113 trees"* | HOLDS | 3.5 % → **2.7 %**, close | ⛔ **the FAILURE**: the NAMED instance `VERILOG-2005-PROFILE` was false, and manufactured by MY OWN READER — it took ANY open word in a narrative cell, and that row says `complete` in both arms while a LEAF deep in its narration is `blocked` |
| *"a required section is missing from 40 of 115 tree files"* | HOLDS | 34.8 % → **31.1 %**, a 3.6-point move | ⚠️ the RATE barely moved; the absolute count halved only because I quoted the wrong POPULATION (closed trees need no frontier). Directionally right, wrongly denominated |
| *"nine other books may carry the same thing"* | ⛔ does NOT hold | measured **1 of 9** | speculation published as a class claim; the instance is real (`rtl_frontend` publishes a live-tracker row of `Done` the register demoted), the class is not |
| *"`PARSER-BOOK-CURRENCY` cannot see it"* | HOLDS | — | now stronger: of 32 enforcers only **4** read a per-parser book at all, none its status prose |
| *"leg 1 rests on an untracked artifact"* | HOLDS | — | holds |

⇒ **the prose held in four of five; ONE named instance was false and ONE class claim was
speculation.** That is the honest scorecard, and it is a different — smaller and more precise —
statement than *"two of five were wrong"*, which was this record's own first draft and failed the
very standard it exists to state. **A self-criticism is a claim too.**

**The operational rules, then.**

1. **Name nothing you have not opened.** An aggregate may be an estimate; a NAMED row is an
   assertion, and it is the part a reader checks first. Hand-check every instance you name.
2. **Denominate the population a reader would recognise** — open vs closed, required vs optional.
   The rate is usually the finding; the absolute count inherits whatever population you chose.
3. **Never publish "may" as if it were "does".** Measure the class or say plainly that you have
   measured one instance and not the class.
4. Publish what you can defend under challenge **without touching a keyboard**, or publish it with
   the gap named.

### My counter-argument, recorded because the director asked for it (2026-08-30)

I agree with the standard and it improved my own scoring — my first draft of the table above said
*"two of five were wrong"*, which over-punished a finding whose RATE moved 3.6 points and
under-punished the one that was actually broken. Three refinements, though, and the first is a
correction to the *mechanism*, not the rule:

1. ⛔ **The variance here is DEFINITIONAL, not probabilistic.** The director's premise was *"you work
   in a probabilistic way … if you re-verify N times, we may end up with N different number
   values."* That is true of the prose I write and FALSE of these numbers: they come from
   deterministic commands, and `git grep` returns the same count every time. When 40 became 19,
   nothing sampled differently — I changed the POPULATION and the heading spelling, so the two
   numbers answer **different questions**. This matters operationally: noise is tolerated, but a
   definitional shift means one of the two numbers is not the question the reader cares about. ⇒ my
   acceptance test stays **same question, re-derivable, deterministic**, not *close enough* — if I
   graded myself on *close enough* I would stop noticing when a number silently changed meaning.
2. ⭐ **The tolerance must be TIERED, because two classes of number live here.** A *judgement* number
   supports a qualitative call and ±4 points changes no decision — trajectory-grade is right. A
   *contract* number IS the claim: a corpus pass count behind a release statement, the doctrine
   count two meta-checks hold equal, an AST-dump schema version in a shipped contract. For that
   class a small error is the whole defect — the founding example in this very session was a book
   publishing schema `10` and `13` against a contract's `26`. Trajectory-grade must never be
   extended to it; those numbers are exact and gate-held or they are not published.
3. ⛔ **A NAMED INSTANCE has no tolerance band at all**, and that is the exception that must be said
   out loud, because *"the number can be a bit off"* could otherwise be read as covering *"one of
   the four rows I named was wrong."* It does not. An aggregate may be an estimate; a named row is
   an assertion, and it is the first thing a reader checks.
