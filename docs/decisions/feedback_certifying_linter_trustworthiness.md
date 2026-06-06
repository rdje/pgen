---
name: feedback-certifying-linter-trustworthiness
description: The grammar linter is the fulcrum of sign-off and must be one we NEVER doubt — achieved by making it a CERTIFYING algorithm (every verdict ships a checkable certificate; a tiny independent checker validates it; we trust the checker, not the linter). Sound-not-complete (exact reachability is undecidable for data-dependent PEG); the cost is an honest UNKNOWN, never a guess; for the shipped grammar UNKNOWN is drained to 0. Goal = verified, not trusted.
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-06-06
---

**Director directive (2026-06-06, extended brainstorm).** "We can't afford to doubt
the grammar linter." The linter is the FULCRUM of the whole sign-off model — it is the
PROVER of well-formedness, the ADJUDICATOR of every generator reach-failure (see
[[feedback_unreachable_target_attribution_rule]]), and the THEOREM-MAKER that turns
literal-0 coverage from a chased number into a proven property. A tool with that much
riding on it must be trustworthy 100%.

**The honest theoretical bound.** Exact reachability of an arbitrary EBNF fragment is
UNDECIDABLE for PGEN's grammar class (stateful, data-dependent PEG with `@predicate`s):
structural rule-reachability is decidable (Hopcroft–Ullman reduced grammar), but PEG
ordered-choice arm-selection is a language-difference question (undecidable in general)
and predicate-satisfiability is program-state reachability (Rice's theorem). So NO tool
can be COMPLETE (catch every dead fragment with a guaranteed definite answer).

**The achievable, right property = SOUNDNESS (one-sided), not completeness.** When the
linter commits to a verdict it is correct; it never declares a live fragment dead nor a
dead fragment live. The price is an honest third answer, **`UNKNOWN`**, for what it can't
settle — NOT a wrong answer, the linter refusing to GUESS. (A judge that never convicts
the innocent, even if it sometimes returns "not proven", is exactly right; a complete
judge that sometimes convicts the innocent would be worse than useless.) This is WHY every
PGEN check is a SOUND DECIDABLE SUBSET and why unsound heuristics (general FIRST-domination,
parse-order predicate reachability) are deliberately EXCLUDED.

**The mechanism — a CERTIFYING ALGORITHM (Mehlhorn; McConnell et al.).** Every verdict
ships a checkable CERTIFICATE that a deliberately tiny, auditable independent CHECKER
validates — so you trust the small checker, not the linter's complex internals; doubt is
replaced by VERIFICATION.
- **reachable → a WITNESS**: a concrete derivation + an input string that exercises the
  fragment; replay it through the REAL parser. The stimuli GENERATOR is the witness
  producer — the linter⟷generator DUALITY made operational.
- **unreachable → a PROOF**: the exact decidable argument (which sound rule fired + the
  chain); the checker re-validates it.
- **`UNKNOWN` → no certificate** (by design — honest).
- BINDING DISCIPLINE: **no definite verdict without a certificate.** It is BECAUSE the
  linter refuses to speak without a proof that you never doubt it when it does.

**Undecidability lives ENTIRELY in `UNKNOWN` — and is drained on the grammar we ship.**
The theorem is about ALL grammars, not the ONE we sign off. For the actual grammar we drive
`UNKNOWN` to ZERO: every fragment is witnessed-reachable or proven-unreachable; the residue
is adjudicated ONCE per the attribution rule (→ a witness, or a proof, never silently
accepted). When `UNKNOWN`=0 with all certificates checking, the shipped grammar is FULLY
CERTIFIED — verified, not trusted. Goal: **"100% SOUND, with `UNKNOWN` driven to 0 and never
hidden."**

**Work item: `GRAMMAR-WELLFORMED.G` (the certifying linter)** — G.1 certificate model +
certify the decidable `dead` checks (unreachability PROOFs); G.2 the independent checker
(replays witnesses through the real parser + re-validates proofs); G.3 generator-as-witness
producer; G.4 certificate-coverage gate (`UNKNOWN`=0 on SV, hard). Folds in A2.1 (each
grammar fix moves a fragment from dead/`UNKNOWN` to witnessed-reachable).

**Scope: ALL grammars, current AND future (director directive 2026-06-06).** Full certification is
NOT SV-only — every PGEN grammar (SystemVerilog, VHDL, regex, RTL, the annotation/EBNF grammars, and
any future one) must reach the same bar: well-formed + well-defined (static checks pass) AND
`UNKNOWN`=0 with all certificates checking. The machinery already generalizes — the linter
(`--lint-grammar`) and Phase G are PARSER-AGNOSTIC by construction (they key only on the grammar AST +
annotations, zero grammar-specific identifiers; cf. [[feedback_ast_pipeline_parser_agnostic]]), so the
SAME certification applies to each grammar unchanged. Empirical head start: the F1 all-grammars sweep
(`PGEN-GRAMMAR-WELLFORMED-0008`) showed the hand-authored non-SV grammars (json, regex, vhdl, rtl_*,
return/semantic/builtin annotation grammars, sv_preprocessor) ALREADY at 0 always-matches / 0
unbound-fact / 0 unreachable / 0 orphan — they are already statically clean; SV is the outlier (its
LRM-PDF extraction artifacts). So per-grammar certification = (a) static checks (mostly already green
off-SV) + (b) the G.4 certificate-coverage gate (`UNKNOWN`=0 with witnesses) run PER grammar. `G.4`
becomes a per-grammar gate; a future `GRAMMAR-WELLFORMED` phase rolls each non-SV grammar to full
certification (most are a short hop).

Book: `docs/book/src/grammar-wellformedness.md` — "Trusting the linter: certificates, not
faith". Tree: `docs/tasks/GRAMMAR-WELLFORMED.md` Phase G. Reinforces
[[feedback_always_signoff_decisions.md]] (verify, don't over-claim) and
[[feedback_corpus_expected_from_spec_not_fix]] (never game the metric).
