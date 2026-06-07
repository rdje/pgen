---
id: ebnf-single-source-of-truth
title: The EBNF is the single source of truth for the accepted language — out-of-band post-parse validators break the generator⟷parser duality
answers:
  - "how can the stimuli generator produce strings the parser rejects"
  - "why does a grammar-driven generator emit parser-rejected output"
  - "what makes generation and parsing inconsistent in PGEN"
  - "what is an out-of-band acceptance validator and why is it a defect"
  - "why do regex \\u{...} and (*verb) generate but fail to re-parse"
  - "where does regex_compile_validation.rs fit and why is it invisible to the generator"
  - "how do I debug what the stimuli generator is deriving"
tags: [ebnf, generator, parser, duality, certifying-linter, regex, validation, source-of-truth, defect-class]
date: 2026-06-07
status: current
evidence: "tool-backed 2026-06-07 (PGEN-EBNF-SOT-0001). parseability_probe: `\\u{b7a2}` -> 'unsupported regex escape \\u'; `(*xjDD)` -> 'unrecognized PCRE2 verb or start option'; `\\x{b7a2}` / `(?|a)` / `(?P>n)` / `(?(1)a)` PARSE. EBNF structurally accepts the failing forms: grammars/regex.ebnf `unicode_escape = \"u{\" hex_digits \"}\"`, `directive_verb = \"(*\" directive_body \")\"`. Rejection source: rust/src/regex_compile_validation.rs (validate_regex_compile_contract). grep: stimuli_generator.rs references it 0x; regex.ebnf encodes it as @predicate/@generate 0x."
reverify: "`./rust/target/debug/parseability_probe --parse regex <(printf '\\\\u{b7a2}')`; `grep -n 'unsupported regex escape' rust/src/regex_compile_validation.rs`; `grep -c 'regex_compile_validation' rust/src/ast_pipeline/stimuli_generator.rs` (expect 0); `grep -nE 'unicode_escape|directive_verb' grammars/regex.ebnf`"
---

## THE RULE (binding)

**The EBNF — together with its `@predicate` / `@generate` / `@semantic` annotations — is the SINGLE
SOURCE OF TRUTH for what a PGEN parser accepts.** The stimuli generator derives samples from the EBNF
(structure + semantic annotations) and **nothing else**. So **any acceptance constraint that lives
OUTSIDE the EBNF — in a hand-written post-parse validation layer — is invisible to the generator, and the
generator WILL emit structurally-valid strings the parser rejects**, silently breaking the
generator⟷parser duality (and the certifying-linter's "generation constructively corroborates the
grammar" guarantee). Such an out-of-band acceptance gate is a **DEFECT**: encode the constraint IN the
EBNF (a semantic annotation, shared by generation and parsing), or remove/relax it.

## Why a grammar-driven generator can emit parser-rejected output (the mechanism)

It can only happen when the parser's accepted language is **narrower** than the EBNF describes. PGEN's
generator generates *by construction* from the EBNF, so by construction it produces EBNF-valid strings.
If the parser ALSO runs a separate validator the EBNF doesn't encode, then
`accepted = EBNF ∩ validator`, the generator targets only `EBNF`, and `EBNF \ validator` is exactly the
generated-but-rejected set. It is **not** the generator violating the grammar — it is the grammar not
being the whole spec.

## The canonical instance (regex, 2026-06-07) — and a correction on the METRIC

The real EBNF-SOT defect (CONSUMER parse path): the EBNF structurally accepts
`\u{…}` (`unicode_escape = "u{" hex_digits "}"`) and `(*<any-name>)`
(`directive_verb = "(*" directive_body ")"`), but `rust/src/regex_compile_validation.rs`
(`validate_regex_compile_contract`, referenced by neither the generator nor the EBNF) rejects `\u`
("unsupported regex escape") and unrecognized `(*verb)` names. So `parse_with_regex_detail` (the path
consumers use) rejects generator-valid `\u{…}` / `(*xjDD)`. (`\x{b7a2}` / `(?|a)` / `(?P>n)` / `(?(1)a)`
parse — only `\u` and unrecognized verbs fail.) This is a genuine duality break and is owned by
`EBNF-SOURCE-OF-TRUTH.3`.

**⚠️ Metric correction (`PGEN-EBNF-SOT-0003`, EBNF-SOURCE-OF-TRUTH.2.1):** the `.1` claim that regex
cert-coverage `sample_parse_failures` (the `GRAMMAR-WELLFORMED.H.1` "6") was caused by this validator was
**wrong**. cert-coverage's witness path `parse_and_cover_regex` (`parser_registry.rs:327`) **deliberately
omits the validator** (comment `:324`), so `\u`/`(*verb)` PARSE there and are scored as witnesses, never
failures. The cert-coverage `sample_parse_failures` are STRUCTURAL — caused by the generator's
`apply_word_boundary_spacing` over-inserting a trailing `" "` before a closing `)` (LEXICAL), not the
validator. See `cert-coverage-measures-structural-not-validator`. Lesson: a "semantic message via
`parseability_probe`" only proves the CONSUMER path runs a validator — it does NOT prove the cert-coverage
metric (a different, validator-free path) is caused by it. Always check WHICH parse path the metric uses.

## How to act on a generated sample that fails to re-parse

1. Do NOT classify-and-route it as "structural, follow-up". Root-cause the **mechanism**.
2. Get the exact error: `parseability_probe --parse <grammar> <file>`. "did not consume input" ⇒
   structural / PEG-ordering (e.g. SV `use_clause`, fixed by reordering, G.4.8). A semantic message
   ("unsupported …", "unrecognized …") ⇒ an out-of-band validator — this defect class.
3. See what the **generator** derived: run the generator with `--trace high` (or `--trace debug`).
4. Resolve: encode the constraint as an EBNF semantic annotation (so generation honors it), OR
   relax/remove the validator.

Owned by the `EBNF-SOURCE-OF-TRUTH` task tree (decision: `project_ebnf_is_single_source_of_truth`).
Protects the duality in `grammar-linter-trustworthiness`. Related: `grammar-coverage-and-directed-generation`,
`ast-pipeline-cli-reference`.
