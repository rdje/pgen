---
id: absence-in-the-corpus-is-not-a-property-of-the-rule
title: "\"Nothing exploits this today\" prices a corpus, not a rule — measure the REACH before routing a soundness hole"
answers:
  - "a check has a hole but no leaf currently exploits it — is that a reason to route it instead of fixing it"
  - "how do I price a NARROWING of a gate rule, versus a widening"
  - "why did a corpus census miss a real defect in the same rule it was measuring"
  - "what evidence closes a soundness question about a gate — a census or a probe"
  - "the acceptance gate blocked my leaf even though all three boxes are ticked and backed — why"
tags: [doctrine, enforcement, acceptance-checklist, gates, measurement, false-negative, false-positive]
date: 2026-08-01
status: current
evidence: "measured — GENERATED-LINT-CORRECTNESS.7 priced ROOT_KW's bare `\\bwhy\\b` over-match at 4 headers, none signature-carrying, and routed it as latent. `.9` then fixed it and found a SECOND defect of the opposite polarity in the same alternative: an unticked `**FIX**` box saying \"why\" tripped `unchecked()` and BLOCKED a leaf whose ROOT CAUSE / ADDRESSED / NO REGRESSION boxes were all ticked and backed. Probes 10/10 post-fix; before->after replay 8/2 with the 2 being exactly RED-W1 (exit=0, fails-open) and RED-W2 (exit=1, fails-closed). Census 413 -> 409 ticked, BACKED 106 -> 106."
reverify: "bash docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_root_kw_probes.sh; git show HEAD~1:scripts/check_diagnosis_evidence.sh > rust/target/head_check.sh && PGEN_DIAG_CHECK_OVERRIDE=rust/target/head_check.sh bash docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_root_kw_probes.sh"
---

## The claim

A corpus census answers *"is anything exercising this right now?"*. It does **not** answer *"can
anything exercise this?"* — and for a soundness question about a **rule**, only the second question
matters. Treating the first answer as the second is how a known hole gets routed instead of fixed.

## The measured case

`scripts/check_diagnosis_evidence.sh` recognizes the ROOT CAUSE acceptance box by a keyword regex.
It used to carry a third alternative matching the bare word `why`:

```bash
ROOT_KW='root cause|why ?\+ ?where|\bwhy\b'      # before
ROOT_KW='root cause|why ?[-+/&] ?where'          # after (GENERATED-LINT-CORRECTNESS.9)
```

`GENERATED-LINT-CORRECTNESS.7` found the over-match, censused it — **4 ticked headers matched via
the bare word, all `**FIX**` / `**LOCKSTEP**` / `**REPRODUCE**` boxes, none carrying a diagnosis
signature** — and concluded that none could satisfy the gate *today*. Accurate, and it routed the
finding rather than fixing it.

Both halves of that reading were wrong about the rule:

1. **Fails OPEN.** A `**FIX**` box routinely quotes a command. So the distance between "no leaf
   exploits it" and "a leaf with **no ROOT CAUSE box at all** passes box 1" was one ordinary
   sentence. Probe RED-W1 constructs exactly that leaf; under the pre-fix enforcer it **exits 0**.
2. **Fails CLOSED, and the census could not have seen it.** `unchecked()` blocks on an *unticked*
   box matching the keyword wherever it sits. So `- [ ] **FIX** — not yet chosen. Why the engine
   tier is likely: …` **blocked** a leaf whose ROOT CAUSE, ADDRESSED and NO REGRESSION boxes were
   all ticked and backed. The census had measured that surface at **0 → 0** — true, and a statement
   that the shape is *absent*, not that it is unreachable.

## How to apply it

- **A census prices a change; a probe closes a soundness question.** Run both. The census tells you
  what the fix costs; only a constructed RED arm tells you what the rule admits.
- **Price a NARROWING against "does it break anything", not "is there corpus pressure."** Corpus
  pressure is the bar a *widening* must clear (`.4` refused its own chartered family at 2/304, `.7`
  refused a sixth at 0/307). A narrowing that closes a hole is justified by costing nothing:
  4 boxes dropped, **0 backed**, 0 files losing their last backed box, unticked surface 0 → 0.
- **When you route a hole as "latent", state its REACH, not its current population.** "None carries
  a signature today" is a population. "One `**FIX**` box quoting one command away" is a reach, and a
  reach that short is a fix, not a routing.
- **Check both polarities of any keyword a gate matches.** The same alternative was simultaneously
  admitting the wrong box and rejecting the right leaf. A rule used in two places fails in two ways.

## See also

- `docs/decisions/project_acceptance_box_must_be_written_by_the_change.md` — the sibling lesson one
  level up: box-scoping without leaf-scoping was *vacuous*, i.e. a correct proof about a component
  is not a proof about the composition.
- `docs/decisions/feedback_instrument_needs_ground_truth.md` — an instrument with no ground truth is
  a confident guess; the census that produced the "population 0" number now refuses on a control miss.
- `TOOLBOX.md` § *The task-acceptance checklist* — the author-facing rule.
