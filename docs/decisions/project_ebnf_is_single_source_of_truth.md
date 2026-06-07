---
name: project-ebnf-is-single-source-of-truth
description: THE EBNF (+ its @predicate/@generate/@semantic annotations) is the SINGLE SOURCE OF TRUTH for what a PGEN parser accepts. Any acceptance constraint in a hand-written out-of-band post-parse validator is INVISIBLE to the stimuli generator, so the generator WILL emit structurally-valid strings the parser rejects — breaking the generator⟷parser duality. Such out-of-band gates are DEFECTS. Trigger: regex `\u`/`(*verb)` cert-coverage failures root-caused to regex_compile_validation.rs (2026-06-07).
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-07
  owning_tree: EBNF-SOURCE-OF-TRUTH
---

**THE RULE (binding).** The EBNF — together with its `@predicate` / `@generate` / `@semantic`
annotations — **is the single source of truth for what a PGEN parser accepts.** The stimuli generator
derives samples from the EBNF (structure + semantic annotations) and **nothing else**. Therefore **any
acceptance constraint that lives OUTSIDE the EBNF — in a hand-written post-parse validation layer — is
invisible to the generator, so the generator WILL emit structurally-valid samples the parser rejects**,
silently breaking the generator⟷parser duality and the certifying-linter's "generation constructively
corroborates the grammar" guarantee. An out-of-band acceptance gate the generator cannot see is a
**DEFECT**: encode the constraint IN the EBNF (a semantic annotation — the one shared source of truth for
both generation and parsing), or remove/relax it.

**Why a grammar-driven generator can emit parser-rejected output (the mechanism).** It can only happen
when the parser's accepted language is NARROWER than the EBNF describes. PGEN's generator generates by
construction from the EBNF, so by construction it produces EBNF-valid strings. If the *parser* additionally
runs a separate validator the EBNF doesn't encode, then `accepted = EBNF ∩ validator`, the generator
targets only `EBNF`, and `EBNF \ validator` is exactly the set of generated-but-rejected strings. It is
NOT the generator violating the grammar — it is the grammar not being the whole spec.

**The triggering case (2026-06-07, tool-backed).** regex cert-coverage (`GRAMMAR-WELLFORMED.H.1`) reported
6 `sample_parse_failures`. `parseability_probe`: `\u{b7a2}` → "unsupported regex escape \u"; `(*xjDD)` →
"unrecognized PCRE2 verb or start option". But `regex.ebnf` STRUCTURALLY accepts both
(`unicode_escape = "u{" hex_digits "}"`; `directive_verb = "(*" directive_body ")"`). The rejection comes
from `rust/src/regex_compile_validation.rs` — a hand-written post-parse PCRE2-compile check that is
referenced by neither the stimuli generator nor the EBNF. So the generator faithfully emits `\u{…}` /
`(*anyname)` (EBNF-valid) and the validator rejects them. `\x{b7a2}` / `(?|a)` / `(?P>n)` / `(?(1)a)`
parse — only `\u` and unrecognized verbs are out-of-band-rejected.

**How to apply.** When a generated sample fails to re-parse: do NOT classify-and-route it as "structural,
follow-up". Root-cause it — is the failure structural (PEG ordering, e.g. the SV `use_clause` G.4.8 case)
or out-of-band-semantic (a validator the EBNF/generator don't share, this case)? If out-of-band: encode
the constraint as an EBNF semantic annotation (so generation honors it), OR relax/remove the validator.
Owned by the `EBNF-SOURCE-OF-TRUTH` task tree; composes with [[feedback_certifying_linter_trustworthiness]]
(the duality this protects) and [[feedback_ast_pipeline_parser_agnostic]]. See KM `ebnf-single-source-of-truth`.
