---
id: a-derived-block-publishes-the-first-reason-it-recognises-so-the-assertion-must-be-derivable-from-the-tracked-tree
title: A derived block publishes the first reason it recognises — when a leg's real verdict rests on an untracked artifact, declare the assertion's derivation from the tracked tree, or the book publishes the weaker line beside it for as long as nobody looks
answers:
  - "why did the SystemVerilog book say leg 3 rests on a triage gate when a conformance gate existed"
  - "how does the done-bar audit judge leg 3 in the tracked-only reading the books publish"
  - "what is tracked_assertion_command in the done-bar register's leg3_surface"
  - "a reason table picks the first matching line — what happens when an audit prints two gates for one leg"
  - "why does the audit read a gate's artifact under its invoker's work directory"
  - "the family-status block for SystemVerilog leg 3 — where does its sentence come from"
tags: [done-bar, family-status, derived-blocks, tracked-only, leg-3, conformance-gate, audit, book-currency]
date: 2026-10-08
status: current
evidence: "DONE-BAR.3c (`PGEN-DONE-BAR-0028`). The SystemVerilog book's status block read *\"the only gate reading the external corpus is a TRIAGE gate\"* from 2026-09-03 — the day `sv_external_corpus_conformance_gate` landed (DONE-BAR.3b) — to 2026-10-08. The block is derived TRACKED-ONLY (the audit with its artifact root pointed at an empty directory), so the conformance gate's line read *no artifact — UNPROVEN*, and `report_family_status.sh`'s reason table, scanned in its own order, recognised the triage sibling's ⛔ line first. FAMILY-STATUS-CURRENCY held the page equal to that derivation on every commit: the sentence was current and wrong. The assertion the book should have published — `full_pass`: every row adjudicated, zero unexplained divergences — is a function of the tracked adjudication manifest alone (`corpus_conformance_compare.py --assert-tracked` → `rows=16336 adjudicated=7485 no_verdict=6245 unexplained=0 full_pass=false`). The register now declares that derivation (`tracked_assertion_command`), the audit evaluates the same `pass_query` over it when no artifact exists (controls C8), a triage gate beside a conformance gate prints an informational line rather than a ⛔ (C9), and the audit also reads an invoked gate's artifact under its invokers' work directories (C10) — leg 1 had been reading a July copy under the SOTA aggregate while the formal umbrella held a seven-hour-old one."
reverify: "bash scripts/report_family_status.sh systemverilog --markdown | grep '3 — external' (expect the CONFORMANCE gate's own assertion, not the TRIAGE sentence); bash scripts/report_family_status.sh vhdl --markdown | grep '3 — external' (expect the TRIAGE sentence — VHDL has no conformance gate); python3 stimuli/sv/corpus_conformance_compare.py --assert-tracked stimuli/sv/characterization/adjudication_manifest.tsv | jq .full_pass; bash scripts/audit_done_bar.sh | grep 'controls reproduced' (expect ≥ 24)"
---

**A tracked-only derivation can only say what the tracked tree can say.** The book's status blocks
are derived with no artifact in sight, on purpose: a block that read *green* on the machine that
ran the gate and *stale* everywhere else would be a sentence about the author's disk. So a leg whose
verdict lives in an untracked artifact can publish only its fallback — and the fallback is whatever
line the reason table recognises first. For SystemVerilog that was the triage sibling's sentence,
which stopped being the reason the day the conformance gate landed.

- **Declare the derivation, not a better fallback.** When the assertion is a function of tracked
  inputs — here `full_pass` over the adjudication manifest — the register names the command that
  derives it (`leg3_surface.tracked_assertion_command`), and the audit evaluates the SAME
  `pass_query` over its output. The book then states the assertion's own verdict with its counts;
  the artifact, when present, is read by the identical query. One question, two readers.
- **A sibling gate's line is information once a stronger gate exists.** The reason table reads ⛔
  lines; a TRIAGE gate beside a CONFORMANCE gate prints ℹ️. Order the table by strength of claim
  too, and pin the discrimination in the producer's self-check with both orders of the lines.
- **An artifact lands where its invoker runs it.** The audit looked in the gate's own state dir and
  under the SOTA aggregate; every other invoker's `work/<gate>/` was invisible, so a fresh proof
  under the formal umbrella lost to a July copy. The candidates are derived from the caller graph,
  nested as the invokers nest, and pinned on a literal chain.
- **Currency is not correctness.** `FAMILY-STATUS-CURRENCY` held the page equal to its derivation for
  35 days; the derivation was the defect. A currency doctrine proves the sentence is what the
  instrument says — the instrument's calibration controls are what prove the sentence is true.
