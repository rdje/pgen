---
id: an-x-is-checked-by-nothing-claim-is-a-census-claim
title: An "X is checked by nothing" claim quantifies over every implementation in the tree, so publishing it without a grep is asserting a census you never ran
answers:
  - "I am about to write 'nothing in this repo checks X' — what do I owe first"
  - "is it safe to say no instrument crosses these two things"
  - "I read two implementations and neither does X — can I publish that nothing does"
  - "a number in my own output does not add up but my conclusion looks right"
  - "my new instrument disagrees with an existing one — which of us is wrong"
  - "how do I falsify a gap-finding before routing it to a task leaf"
  - "why did the same lesson fail to prevent the same mistake one commit later"
  - "I named two implementations and concluded the registry has none — is that a census"
  - "does this apply to a finding I write for a human, or only to a task leaf"
  - "I am about to escalate a design question — is it actually answerable from the tree"
tags: [evidence, census, claims, retraction, falsification, controls, claim-verification, sv-corpus-grad]
date: 2026-08-25
status: current
evidence: "SV-CORPUS-GRAD.13e.8, RETRACTED by PGEN-SV-CORPUS-GRAD-0294 under director challenge. The leaf claimed 'BOTH instruments that could notice are bottom-up ... nothing crosses them' about profile-filtered reachability. FALSE: gather_verified_profile_proof_covered_rules (rust/src/ast_pipeline/grammar_wellformedness.rs:3017, wired at rust/src/main.rs:3409, shipped as VERILOG-2005-PROFILE.6.7) crosses exactly those directions over the WHOLE active rule set and issues each profile-entry-unreachable rule a RE-VERIFIED ProfileEntryUnreachable certificate. Measured: all 323 rules the model called unreachable are in the cert pass proof category — 323 of 323, an exact set match. This was the SECOND instance in three days by the same author on the same subject: PGEN-SV-CORPUS-GRAD-0288 published 'verilog_2005 faithfulness is checked by NOTHING' (false — 85 curated conformance cases) and was retracted by -0289, whose recorded lesson was this exact sentence. The refuting evidence was already in the author's own output and explained away: the cert pass reported 1144 certified rules while the model said 827 were reachable."
reverify: "grep -rn 'fn .*profile.*proof\\|fn .*proof.*profile' rust/src/ --include=*.rs   # one line names gather_verified_profile_proof_covered_rules, the function whose existence refuted the claim. Then the set test that settles it: intersect the cert pass PROOF-COVERED list with the model unreachable set -> 323 of 323."
---

# An "X is checked by nothing" claim is a CENSUS claim

**Question it answers:** I am about to write "nothing in this repo checks X" / "no instrument
crosses these two things" — what do I owe before publishing that?

**Answer:** a census. The claim quantifies over every implementation in the tree, so reading two or
three of them and not finding it is *absence of evidence*, not evidence of absence. Run the grep
that would name the function, or do not make the claim.

## Measured, twice, three days apart, by the same author on the same subject

1. `PGEN-SV-CORPUS-GRAD-0288` published *"`verilog_2005` faithfulness is checked by NOTHING"*.
   **False** — `verilog_2005_conformance_contract_v0.json` carries 85 curated per-profile cases.
   Retracted by `-0289`, whose recorded lesson was precisely this sentence.
2. `PGEN-SV-CORPUS-GRAD-0290` then opened `SV-CORPUS-GRAD.13e.8` with *"BOTH instruments that could
   notice are bottom-up … **nothing crosses them**"*, about profile-filtered reachability.
   **False** — `gather_verified_profile_proof_covered_rules`
   (`rust/src/ast_pipeline/grammar_wellformedness.rs:3017`, wired at `rust/src/main.rs:3409`,
   shipped as `VERILOG-2005-PROFILE.6.7`) crosses exactly those two directions over the WHOLE active
   rule set and issues each profile-entry-unreachable rule a **re-verified** certificate.
   Retracted by `-0294`, **one commit after the lesson that would have prevented it.**

⇒ The lesson did not fail because it was unwritten. It failed because it was written as a *fact
about one claim* instead of as a *check to run before a class of claim*.

## The check, and it is one line

```bash
# before writing "nothing does X", ask the tree for the thing you say does not exist
grep -rn "fn .*profile.*proof\|fn .*proof.*profile" rust/src/ --include=*.rs
```

Two functions had been read. A third existed. Reading is sampling; grep is the census.

## The tell that should have stopped it earlier

The refuting evidence was already in my own output and was explained away rather than closed: the
certificate pass reported **1 144 certified** rules under `verilog_2005` while my model said only
**827** were reachable. I attributed the gap to "witness re-routing" without checking. Intersecting
the two sets took one command and settled it — **323 of 323** of my "unreachable" rules were in the
pass's `proof` category, an exact set match.

⭐ **A number in your own evidence that you cannot account for is not a loose end — it is the
finding, arriving early.**

## See also

- `docs/CLAIM_VERIFICATION.md` — the three legs (re-derive · falsify · make durable). This class
  fails **leg 2**: the falsifier is cheap, obvious, and skipped because the claim felt safe.
- [[a-heading-census-is-only-as-good-as-the-heading-grammar]] — the same shape one level down: a
  census whose *pattern* is wrong reports a confident, wrong population.

## The same failure twice more, in findings written FOR A HUMAN rather than for a leaf

2026-08-30, same author, same week, and this is the fourth and fifth instances:

1. *"The stale-number class has no enforcement"* — published from naming **two** doctrines out of
   **thirty-one**. The census takes one line and returns **six** doctrines enforcing derived-number
   co-publication, two of them in that exact wording. The real finding was an order of magnitude
   smaller and far more actionable: *this number is not registered with an existing, proven pattern.*
2. *"The six co-publication doctrines all govern a number with ONE derivation over a closed surface
   set, so registering an incomplete one would gate on a moving quantity — your call."* Also
   unchecked, and refuted by the registry's own text: `BASELINE-IDENTITY` says verbatim that *"a
   baseline nobody can re-derive yet can still be watched for input drift and still name the leaf
   that owes the re-derivation."* The pattern was **built** for the case I claimed it could not
   cover, so the question was never the human's to answer.

⛔ **WHAT THESE TWO ADD TO THE RECORD.**

- **The rule is not scoped to task leaves.** Both of these were "Findings for your review" written for
  a director. A finding addressed to a human gets read as settled and acted on, so it carries the same
  census obligation as a leaf — arguably more, because no gate reads it.
- **Naming N implementations and concluding about the population is the whole failure, and N=2 feels
  like enough.** It is not sampling; it is an existence proof pointed the wrong way. Two examples can
  only ever establish that at least two behave that way.
- ⭐⭐ **AN ESCALATION IS A CLAIM TOO.** *"This is genuinely your call"* asserts that the tree does not
  already decide it. That assertion needs the same grep as any other — here the registry decided it,
  and escalating it burned the director's attention on a question the repository had already answered.
  → [[feedback_answer_your_own_technical_questions]]
- ⚠️ **The error direction is flattering, which is why it survives self-review.** A related miscount in
  the same exchange reported *"two of my last three findings needed correcting"* when the callout in
  question contained exactly **two** findings and **both** were corrected — 2 of 2 restated as 2 of 3.
  Nobody checks a number that makes their own record look better.
