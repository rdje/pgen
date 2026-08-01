---
name: project-ebnf-is-single-source-of-truth
description: "THE EBNF (+ its @predicate/@generate/@semantic annotations) is the SINGLE SOURCE OF TRUTH for what a PGEN parser accepts. Any acceptance constraint in a hand-written out-of-band post-parse validator is INVISIBLE to the stimuli generator, so the generator WILL emit structurally-valid strings the parser rejects — breaking the generator⟷parser duality. Such out-of-band gates are DEFECTS. Trigger: regex `\\u`/`(*verb)` cert-coverage failures root-caused to regex_compile_validation.rs (2026-06-07). ⭐ GENERALIZED (director 2026-07-26 session #208): EVERY user-controllable feature — dialect tolerance or any other — is declared IN the EBNF, never as a runtime flag, engine table, or hard-coded rule-NAME `matches!` arm; a hard-coded rule name governing behaviour is this doctrine's breach in its commonest disguise."
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-07
  owning_tree: EBNF-SOURCE-OF-TRUTH
id: project-ebnf-is-single-source-of-truth
title: The EBNF is the SINGLE SOURCE OF TRUTH — an out-of-band validator or a hard-coded rule name is a defect
date: 2026-06-07
answers:
  - "where do I add a new acceptance constraint for a parser"
  - "can I add a post-parse validator to reject something the grammar accepts"
  - "why does the stimuli generator emit strings the parser rejects"
  - "can I add a runtime flag or engine table for a dialect option"
  - "is a hard-coded rule name in engine logic allowed"
reverify: bash scripts/check_ebnf_source_of_truth.sh 2>&1 | tail -3
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

## ⭐ GENERALIZED (director 2026-07-26, session #208) — EVERY user-controllable feature is EBNF-declared

Director, verbatim: *"even dialect tolerance or any other feature shall be user
controllable via the EBNF, remember the sole source of truth."*

The rule above was written about **acceptance validators**. It generalizes: **any
knob a grammar author or downstream user is meant to control is declared IN the
EBNF** — not as a runtime parser flag, not as an engine-side table, not as a
hard-coded rule-NAME `matches!` arm, not as a per-grammar `if grammar_name == …`
branch. The EBNF is the whole spec of a parser's behaviour, not merely of its
structure.

This is not aspirational — it is the pattern the repo has already executed
repeatedly, each time retiring an engine-side gate in favour of a declaration:

| directive | replaced |
|---|---|
| `@whitespace_sensitive` (WS-DIRECTIVE.2) | a grammar-NAME layout gate |
| `@default_profile` (DEFAULT-PROFILE.2) | `regex`→`pcre2` name literals in the engine |
| `@profile_alias` (PROFILE-ALIAS.2) | the engine's alias tables (`parser_registry.rs`, `main.rs`) |
| `@quantified_separator` (STIMULI-SIGNOFF.12) | the svpp grammar-name + container-rule name gate |
| `@lexical_token` (LEX-ADJACENCY.2, in flight) | the hard-coded `string_content_double`/`string_content_single` no-layout allowlist |

**Consequences to apply:**

- **A dialect-tolerance switch, if ever built, MUST be EBNF-native** and orthogonal
  to `@profiles` — see [[feedback_sv_strict_lrm_compliance_default]]. A runtime
  "lax mode" flag is forbidden by this doctrine and would fail the pre-commit gate.
- **A hard-coded rule NAME in the engine is the smell.** `matches!(rule_name, "x" | "y")`
  governing behaviour means one grammar was privileged and no other can opt in —
  it is this doctrine's breach in its most common disguise, because it looks like
  a local special case rather than a policy decision. `LEX-ADJACENCY.1` found one
  that had shipped unnoticed and led a later session to conclude a whole capability
  "does not exist" (see [[project_no_layout_primitive_is_undeclarable]]).
- **When adding a capability, ship the declaration with it.** A capability the
  engine has but no grammar can request is, for every practical purpose, absent —
  and it will be re-derived from scratch by someone who could not find it.

**How to apply.** When a generated sample fails to re-parse: do NOT classify-and-route it as "structural,
follow-up". Root-cause it — is the failure structural (PEG ordering, e.g. the SV `use_clause` G.4.8 case)
or out-of-band-semantic (a validator the EBNF/generator don't share, this case)? If out-of-band: encode
the constraint as an EBNF semantic annotation (so generation honors it), OR relax/remove the validator.
Owned by the `EBNF-SOURCE-OF-TRUTH` task tree; composes with [[feedback_certifying_linter_trustworthiness]]
(the duality this protects) and [[feedback_ast_pipeline_parser_agnostic]]. See KM `ebnf-single-source-of-truth`.
