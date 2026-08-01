---
name: feedback-answer-your-own-technical-questions
description: "STANDING DIRECTIVE (director, 2026-08-01): the director steers OBJECTIVES — north star, features, priorities, scope. Deep technical questions are NOT theirs to answer and must not be escalated: the engineer has the roadmap, the codebase and the toolbox, so a technical question is a task, not a blocker. Corollary, stated by the director in the same breath: any claim you make, you must be able to challenge and PROVE OR DISPROVE yourself — an untested claim recorded as fact is the defect."
metadata:
  node_type: memory
  type: feedback
id: feedback-answer-your-own-technical-questions
title: The director steers OBJECTIVES — technical questions AND execution sequencing are the engineer's to answer
date: 2026-08-01
answers:
  - "should I ask the director which of two approved lanes to do first"
  - "is choosing the order of work a director call"
  - "when is escalating to the director appropriate"
  - "what counts as an objective/scope question versus a task"
  - "I have a deep technical question — do I ask or answer it myself"
  - "must I be able to disprove my own claims"
reverify: sed -n '/SEQUENCING IS EXECUTION/,/^$/p' docs/decisions/feedback_answer_your_own_technical_questions.md
---

**⭐ SEQUENCING IS EXECUTION, NOT OBJECTIVE-SETTING (director, 2026-08-01, sharpening this same
record the day it was written).** Having approved two lanes, the engineer asked which to run first.
The director's answer:

> *"Regardless of the order the sweep and `SV-EXH-PROOF.7.4.6.9` or vice-versa I am fine with it.
> Honestly, there was no reason to ask me this question. You could have sequenced them your way."*

⇒ Once scope is set, **ORDERING the approved work is the engineer's call** — it is not a scope
question merely because it concerns two lanes. Escalate only when the answer would CHANGE WHAT gets
built (objective, priority, scope, cost the director is paying), never when it only changes WHEN.
A menu offered where a decision was owed is the [[feedback_pinpoint_real_blocker_not_menu]] failure
wearing a scheduling costume.

⛔ **Corollary on vocabulary:** the same message asked *"what's the sweep about"* — the escalation
used an internal shorthand the director had never been given. If a question is worth asking at all,
it must be self-contained; if it needs a glossary, it was probably not a director question.


**STANDING DIRECTIVE (director, 2026-08-01):** *"That is a very technical question that as a
director I can't answer, in all honesty. … you are the elite, expert coder, programmer with a deep
knowledge of PGEN roadmap, objectives and codebase, meaning you have all you need to precisely
answer your own questions. Also whenever you make a claim on something you should be able to
challenge yourself and be able to prove or disprove your claim. … As a director I can provide
guidance on objectives, north star, new features, I mean everything but deep and technical
questions."*

## The division of labour

| The director owns | The engineer owns |
|---|---|
| objectives, north star, scope | **every technical question, without exception** |
| which lane to work (`-> SV-EXH-PROOF.7`) | how to work it, what to build, which mechanism |
| whether a capability is wanted at all | whether a fix is sound, sized, monotone, worth it |
| priced trade-offs stated in *their* terms (time, money, risk to a shipped claim) | trade-offs stated in engine terms — those are not a director question wearing a costume |

**Why:** escalating a technical question to someone without the codebase in their head does not
transfer the decision — it *stalls* it, and it hands back a question dressed as a choice. The
director cannot adjudicate "witness-entry policy vs prelude wiring"; asking them to costs a round
trip and returns nothing. Worse, it disguises unfinished engineering as governance: the honest
description of "should we do X or Y?" was almost always **"I have not measured X or Y yet."**

## The trigger that produced this record

`SV-EXH-PROOF.7.4.6.12` escalated: *"is the class-C fix worth it now, or does it park behind class
A?"* — framed as a director call because the fix was *"a slice-sized design that re-prices the
witness budget."* Both halves were wrong, and **both were mine to settle**:

- The sizing was **asserted, never tested.** One command disproved it: SV certificate-coverage
  already reports `UNKNOWN=0 fully_certified=true`, i.e. a *sibling witness pass already witnesses
  the exact rule* by entering high and steering down. Nothing to design; the cost is already paid by
  a pass that runs today. A positive/negative parser control then confirmed it is a real witness and
  not one of the 17 unreachability proofs.
- The priority question was **arithmetic, not judgement**: 4 targets vs 79, with 44 freshly-won
  targets protected by no ratchet. That ordering needs no director.

⭐ The sharpest part: that same leaf had *just* caught its own Goal hypothesis by testing it — and
then recorded a fresh untested claim one field below. **The discipline is not a step you perform
once per leaf; it applies to every claim you write down, including the ones inside a correction.**

## How to apply

1. **Never escalate a question you can measure.** Before writing "director call", ask: *is there a
   command that settles this?* If yes, that is not a director call — it is the next thing to run.
   Escalate only genuine objective/scope/priority-of-outcome questions.
2. **Every recorded claim must name how it was checked.** A sentence stating cost, soundness,
   reachability or blast radius is a *hypothesis* until a tool says otherwise. Prefer a sibling
   surface that already exercises the mechanism — the cheapest disproof is usually "something in
   this repo already does this."
3. **Prefer disproof.** Try to refute your own claim first; an unrefuted claim that survived a real
   attempt is worth far more than one that was never attacked. Positive **and** negative controls
   where a gate is involved → [[feedback_instrument_needs_ground_truth]].
4. **State the decision and the reason, then act.** "Parked behind class A: 4 targets vs 79, and the
   44 just won are unratcheted" is a decision. "Your call?" on the same facts is an abdication.
5. **When you do surface something**, surface *findings and decisions taken* — things that change
   the director's picture — not questions whose answers live in the source tree.

Companions: [[feedback_why_and_where_before_solution]] (know WHY+WHERE before designing),
[[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_instrument_needs_ground_truth]],
[[feedback_read_prior_art_before_designing]] (the sibling surface that already does it),
[[feedback_prefer_feature_work_over_governance_lanes]] (the director DOES own lane choice).
