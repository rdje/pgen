---
id: presence-is-not-a-pass-a-conformance-surface-records-its-own-verdict
title: Presence is not a pass — a conformance surface records its own verdict as a bit the artifact carries, every reader evaluates that same declared assertion, and the exit code is a different bit (regression)
answers:
  - "why did the done-bar audit say leg 3 MET when the family-status gate said UNMET"
  - "what is the difference between a conformance gate's exit code and its pass assertion"
  - "how should a leg-3 surface be declared and evaluated"
  - "can a gate that only checks for no regression satisfy 'the corpus, PASSING'"
  - "where does the family-status battery get the leg-3 artifact from"
tags: [done-bar, conformance, gates, evidence, audit, sv]
date: 2026-09-03
status: current
evidence: "PGEN-DONE-BAR-0022: scripts/audit_done_bar.sh set met_any on ANY artifact of a non-triage, external-backed, invoked corpus gate; the first honest artifact (full_pass=false with 268 known defects and 6 174 no-verdict rows) would have read MET there and UNMET in rust/scripts/lib/parser_family_status_bar.sh, which evaluates the register's pass_query. Fixed with leg3_pass_from_artifact + controls C7; the SV surface publishes both bits on every run."
reverify: "bash docs/tasks/artifacts/done_bar/sv_external_corpus_conformance/probe.sh   # arms 5-8: the real artifact reads full_pass=false, the evaluator says UNMET, a triage surface is refused, the audit's C7 controls hold"
---

**A file in `rust/target/` proves a gate ran. What the gate concluded is written inside the file.**

## The rules

1. **Two bits, two questions.** Exit code = "did the parser regress against the recorded answer-key agreement?"
   (`match` → not match, explained → unexplained). `full_pass` = "does the external corpus PASS?" (every row
   adjudicated, zero unexplained). A green exit is not a pass; a pass is asserted, not implied.
2. **Declare the assertion once, evaluate it everywhere.** The register's `leg3_surface.pass_query` is the one
   sentence; the family-status evaluator and the done-bar audit both run it over the same `summary.json`. Two
   readers with two rules disagreed the first time one was exercised.
3. **Produce the surface where you evaluate it.** The battery runs the gate before `family_done_bar_leg3`; a
   standalone-default hand-off consumes evidence of unknown vintage.
4. **Do not let the instrument narrow the bar.** "No regression" is a ratchet; "PASSING" is the bar. The surface
   publishes the distance so the bar stays the director's sentence.

## A second instance (2026-10-19, `SV-CORPUS-GRAD.13c.2v.21`) — a proof umbrella's `passed` is its computation's bit

`rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh` runs six stages and exits 0 when they ran; the proof
it exists to establish is a line INSIDE its summary (`zero_plausible_grammar_level_gap_proof_surface`). Release `1.0.9`
(2026-10-18) stranded nine guard rules and nine edition-gated alternatives, the zero-plausible-gap contract (v3: an
EMPTY surface) turned false, and the release ceremony's step log read `✅ … passed`, rc 0. The reader that binds the
surface to the family's tier, `sv_parser_family_status_gate`, joined the ceremony a day later (`-0120`); the push
mirror's aggregate was the first to run it and said `tracker='Provisional (corpus pending)' computed='Mostly Done'`.

5. **A status umbrella's exit code says its stages ran; list the gate that binds the surface to a verdict.** The
   umbrella's banner now carries the surface's own bit, and the contract explains the surface class by class against
   the gap report and the grammar's `@profiles` instead of asserting it empty — rule 1 again, one level down.

## Related
[[a-commit-that-quotes-a-bank-must-be-gated-on-the-banks-exit-code]] — the same discipline one level up.
[[a-red-arm-that-returns-rc-1-from-a-crash-is-not-a-red-arm]] — an artifact that exists is not an artifact that passed.
