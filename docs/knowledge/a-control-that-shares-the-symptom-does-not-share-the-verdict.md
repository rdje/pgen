---
id: a-control-that-shares-the-symptom-does-not-share-the-verdict
title: A control that reproduces the symptom exonerates nothing unless its input is legal in the same way — shape-matching is not control-matching
answers:
  - "a second input reproduces the same failure, so my suspect is innocent — is that sound"
  - "how do I pick a control that can actually exonerate a defect"
  - "two identical-looking rejections, is one of them a bug"
  - "my control reproduced the symptom without the suspect, what does that prove"
  - "I retracted a claim and the retraction was also wrong, what did I skip"
  - "is correcting against my own interest evidence that the correction is right"
tags: [controls, claim-verification, evidence, grammars, conformance, retraction]
date: 2026-08-17
status: current
evidence: |
  SV-CORPUS-GRAD.13c.2f slices 1 and 2, session #244. Slice 1 measured
  `specparam PATHPULSE$ = (1, 2);` REJECT and dismissed it as unrelated to the defect under
  investigation, because `specparam CAP = (1, 2);` — carrying no PATHPULSE — rejects too. Read
  against IEEE 1800-2023 A.7.5 the two verdicts are opposite:
  `specparam_assignment ::= specparam_identifier = constant_mintypmax_expression | pulse_control_specparam`,
  `pulse_control_specparam ::= PATHPULSE$ = ( reject_limit_value [ , error_limit_value ] ) | …`,
  `limit_value ::= constant_mintypmax_expression`. So `CAP = (1, 2)` rejecting is CORRECT and
  `PATHPULSE$ = (1, 2)` rejecting is UNDER-ACCEPTANCE. The slice-1 write-up published "only ONE of
  the sites rejects"; the true count is THREE of five, and the error was found only by a director
  challenge asking whether the claims still held.
reverify: |
  # both reject; only the second is a defect. The discriminator is the LRM, not the parser.
  ./rust/target/release/parseability_probe --parse systemverilog <plain>.sv  --profile sv_2017  # REJECT, correct
  ./rust/target/release/parseability_probe --parse systemverilog <pathpulse>.sv --profile sv_2017  # REJECT, defect
  grep -n -A3 'pulse_control_specparam ::=' docs/systemverilog/2023/txt/section-Annex_A-normative-formal-syntax.txt
related:
  - an-ab-whose-arms-are-secretly-identical-does-not-fail-it-passes
  - an-instrument-firing-is-not-the-defect-reproducing
  - a-conservation-control-cannot-catch-a-misassignment
  - a-negative-control-can-disable-the-assertion-it-is-testing
  - a-hypothesis-list-is-a-snapshot-of-what-you-knew-that-day
---

**A control is supposed to answer "would this happen anyway, without my suspect?"** So you build an
input that omits the suspect, observe the same symptom, and conclude the suspect is innocent. That
reasoning is valid only when the control input is **equivalent to the subject on the axis the verdict
depends on**. Reproducing the *symptom* is not enough — the control must deserve the *same verdict*.

## The shape

```text
subject :  specparam PATHPULSE$ = (1, 2);   ->  REJECT
control :  specparam CAP        = (1, 2);   ->  REJECT     "same symptom without the suspect"
conclusion: the comma is a general problem; my suspect is innocent.          ← WRONG
```

The two inputs share their syntactic shape — an identifier, `=`, a parenthesised comma — and differ
on the only axis that decides the verdict:

| input | production the standard gives it | is the rejection correct? |
|---|---|---|
| `CAP = (1, 2)` | `specparam_identifier = constant_mintypmax_expression` — and `(1, 2)` is **not** one | ✅ yes |
| `PATHPULSE$ = (1, 2)` | `pulse_control_specparam ::= PATHPULSE$ = ( reject_limit_value [ , error_limit_value ] )` | ⛔ **no — a defect** |

So the control was **expected to reject**. An outcome the control was always going to produce carries
no information about the subject, which is the same defect as
[[an-ab-whose-arms-are-secretly-identical-does-not-fail-it-passes]] arriving from the control side
rather than the treatment side.

## The question that fixes it

Before letting a control exonerate anything, ask: **is my control input SUPPOSED to succeed?**

- If the control is *expected to fail*, its failure is not evidence — it is a re-statement of the
  spec, and it cannot distinguish "the suspect is innocent" from "both are broken".
- A control that exonerates must be **legal in the same way as the subject** — same production, same
  clause, same dialect — so that only the suspect differs.
- ⛔ The discriminator lives in the **specification, not in the tool's output**. Two identical
  rejections from the parser look identical *because the parser is the thing under test*. Asking the
  parser harder cannot separate them; asking the standard can.

## Why this survives a retraction

The slice that made this mistake had *already* caught itself overclaiming and had published a
retraction. Correcting against your own interest feels like rigour and is easy to mistake for
verification — **the direction of a correction is not evidence of its accuracy.** A retraction is a
claim, and it earns the same three legs (`docs/CLAIM_VERIFICATION.md` §3) as the claim it replaces.
Here the retraction was re-derived (leg 1) but never falsified (leg 2), and the falsifying oracle —
the normative grammar — was already open in the same session.

## The general form

> A control shares a **cause** with the subject, not merely an **appearance**. If you cannot say
> which production, clause or rule the control exercises, you have built a look-alike, not a control.
