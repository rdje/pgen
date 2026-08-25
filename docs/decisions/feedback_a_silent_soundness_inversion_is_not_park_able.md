---
name: feedback_a_silent_soundness_inversion_is_not_park_able
description: "DIRECTOR ORDER (2026-08-25, PGEN-SV-CORPUS-GRAD-0304): *\"Regarding (2) do not just log or route, but also fix because the issue raised there is critical to me.\"* I had routed `ENGINE-UNIVERSAL-SERVICES.46` — a `@profiles` gate makes every `!X` on the gated rule succeed VACUOUSLY, so the STRICT profile becomes MORE PERMISSIVE — as parked-behind-the-SV-lane, on three defensible-sounding grounds: the live population is 3, measured over-acceptances today are 0, and the remedy involves a semantics choice I labelled a language-design call. ⛔ ALL THREE ARE ARGUMENTS ABOUT COST AND CERTAINTY, AND NONE IS AN ARGUMENT ABOUT THE FAILURE MODE. The failure mode is that a STRICTNESS mechanism can WIDEN the language, silently (--lint-grammar clean), in the ACCEPTING direction, with nothing in the repository able to see it — and PGEN's north star makes over-acceptance a defect. ⇒ a finding whose shape is *the safety mechanism itself is unsound* is not park-able on a low population count: the population is a measure of TODAY'S exposure, not of the defect's severity, and it is one grammar edit away from changing — the very next unit, SV-CORPUS-GRAD.13e.10(c4), is the edit that trips it. ⭐ ROUTING DECIDES WHEN, NEVER WHETHER (feedback_every_finding_must_be_fixed_not_logged), and the WHEN for a silent soundness inversion is NOW."
id: feedback_a_silent_soundness_inversion_is_not_park_able
title: A silent soundness inversion is not park-able — ask the failure-mode question before the cost question
reverify: "grep -n 'DIRECTOR ORDER' docs/tasks/ENGINE-UNIVERSAL-SERVICES.md   # `.46` carries the order verbatim and is marked TOP PRIORITY ahead of the SV lane. Then: bash docs/tasks/artifacts/sv_corpus_grad/profile_gate_sizing/probe.sh (6/6) shows the inversion itself — arm [2], !X rejects under loose and ACCEPTS under strict."
answers:
  - "I found a defect with a tiny live population — is that a reason to park it"
  - "when may I route a finding instead of fixing it now"
  - "the remedy involves a design choice I am unsure about — does that justify parking"
  - "what makes a finding un-park-able regardless of cost"
  - "does the SV lane lock cover an engine defect I found while doing SV work"
metadata:
  type: feedback
date: 2026-08-25
status: current
---

# A silent soundness inversion is not park-able

**Director order, 2026-08-25** (`PGEN-SV-CORPUS-GRAD-0304`), verbatim:

> *"Regarding (2) do not just log or route, but also fix because the issue raised there is critical
> to me."*

## What I did, and why it looked reasonable

I found that a `@profiles` gate makes every negative lookahead on the gated rule succeed
**vacuously**, so the **strict** profile becomes **more permissive** — and I routed it as
`ENGINE-UNIVERSAL-SERVICES.46`, *parked behind the SV lane*, on three grounds:

1. the live population in the SV grammar is **3** sites;
2. measured over-acceptances from it today: **0**;
3. the remedy turns on whether a gated rule should be *absent* or *unmatchable-but-present* — which
   I labelled a language-design call, i.e. not mine.

⛔ **All three are arguments about cost and certainty. None is an argument about the failure mode.**

## The failure mode is the thing that decides

- A **strictness** mechanism can **widen** the language. The gate's entire purpose is inverted.
- It is **silent**: `--lint-grammar` is completely clean on it.
- It fails in the **accepting** direction — the class PGEN's north star calls a defect, and the one a
  positive-only corpus can never see, because the symptom is an *extra accept*.
- Nothing in the repository can detect it.

⇒ **a population count measures TODAY'S exposure, not the defect's severity.** Here it was one
grammar edit away from changing — and the very next scheduled unit, `SV-CORPUS-GRAD.13e.10`(c4), is
precisely the edit that trips it.

## The rule

**Routing decides WHEN, never WHETHER** ([[feedback_every_finding_must_be_fixed_not_logged]]) — and
for a finding whose shape is *the safety mechanism itself is unsound*, the WHEN is **now**. Ask the
failure-mode question before the cost question:

> If this fires, does the thing that is supposed to protect us do the opposite, without anything
> noticing?

A **yes** overrides a small population, a zero current-impact measurement, and an unresolved design
choice. ⚠️ It does not override *measuring first*: the fix still starts by reproducing the defect on
the shipped engine rather than on the interpreter I happened to measure it with.

## Two corollaries I got wrong here

- ⛔ **"It involves a design decision" is not a reason to park** — it is a reason to *make the
  decision and defend it* ([[feedback_answer_your_own_technical_questions]]). I had already
  escalated one such call and been told it was mine.
- ⛔ **The SV lane lock did not license parking this.** The lock's own named exception is *a defect
  that BLOCKS the SV release*, and this one blocks the next SV commit. I used a
  focus-protection rule as a deferral excuse for work inside the lane — the identical mistake
  recorded in [[feedback_every_finding_is_owned_and_scheduled_never_just_logged]].
