---
id: grammar-linter-trustworthiness
title: Trusting the grammar linter — sound-not-complete, certifying (witness/proof), attribution rule
answers:
  - "can we trust the grammar linter 100%"
  - "is the grammar linter's reachability answer always correct"
  - "is the grammar linter trustworthy"
  - "is exact grammar reachability decidable"
  - "why is the grammar linter sound but not complete"
  - "what does UNKNOWN mean in the grammar linter"
  - "what happens when the stimuli generator cannot reach a grammar fragment"
  - "how does PGEN decide if an uncovered branch is a generator bug or a grammar bug"
  - "what is the attribution rule"
  - "what is a certifying linter or certificate"
  - "what is a witness vs a proof in the grammar linter"
  - "how is a grammar fully certified"
  - "why is literal-0 coverage a theorem not a target"
tags: [grammar, linter, well-formedness, certifying, trustworthiness, duality, attribution, signoff]
date: 2026-06-06
status: current
evidence: "docs/decisions/feedback_certifying_linter_trustworthiness.md + feedback_unreachable_target_attribution_rule.md; docs/tasks/GRAMMAR-WELLFORMED.md Phase G/H; rust/src/ast_pipeline/grammar_wellformedness.rs (verify_unreachability_certificate, UnreachabilityCertificate); book docs/book/src/grammar-wellformedness.md"
reverify: "grep -n 'verify_unreachability_certificate\\|UnreachabilityCertificate\\|EarlierArmAlwaysSucceeds' rust/src/ast_pipeline/grammar_wellformedness.rs"
---

The grammar linter is the FULCRUM of sign-off: it PROVES well-formedness, ADJUDICATES every
generator reach-failure, and turns literal-0 coverage from a chased number into a proven property.
So it must be one we never doubt — and trust is EARNED, not asserted.

**Sound, not complete (the honest bound).** Exact reachability of an arbitrary EBNF fragment is
UNDECIDABLE for PGEN's grammar class (stateful, data-dependent PEG): structural rule-reachability is
decidable (Hopcroft–Ullman), but PEG ordered-choice arm-selection is a language-difference question
and predicate-satisfiability is program-state reachability (Rice). No tool can be COMPLETE. The right
property is SOUNDNESS: when the linter commits to a verdict it is correct; the cost is an honest
`UNKNOWN` (it refuses to guess). Hence every check is a SOUND DECIDABLE SUBSET; unsound heuristics
(general FIRST-domination, parse-order predicate reachability) are deliberately excluded.

**Certifying algorithm (verified, not trusted).** Every verdict ships a checkable CERTIFICATE that a
tiny independent CHECKER validates — you trust the small checker, not the linter's internals:
- reachable → a WITNESS (an input string; the stimuli generator produces it; replay through the real
  parser = the linter⟷generator DUALITY).
- unreachable → a PROOF (the decidable argument; re-derived by `verify_unreachability_certificate`,
  which re-navigates to the cited node and re-checks directly).
- `UNKNOWN` → no certificate (honest). Binding discipline: NO definite verdict without a certificate.

**The attribution rule.** When the generator fails to reach a target, the cause is EXACTLY ONE of
(1) generator deficiency or (2) ill-formed EBNF — attributed via the linter (the ADJUDICATOR), NEVER
silently accepted. Order of suspicion = GRAMMAR-FIRST (unreachability is first a well-formedness
signal, not generator weakness). ⇒ literal-0 is a THEOREM: coverage is complete exactly when, for
every target, the two proofs agree. The undecidability lives entirely in `UNKNOWN`; the theorem is
about ALL grammars, so for the grammar we SHIP we drain `UNKNOWN` to 0 (every fragment
witnessed-reachable or proven-unreachable). When `UNKNOWN`=0 with all certificates checking, the
grammar is FULLY CERTIFIED — and this applies to EVERY PGEN grammar (SV, VHDL, regex, …, current and
future), since the linter + certifier are parser-agnostic.

First worked example: the SV boolean-abbrev `?`-per-arm bug (a dropped-`[ ]`-delimiter extraction
artifact) — generator couldn't reach 7 branches → linter PROVED them unreachable → grammar fixed.
