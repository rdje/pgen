---
id: cert-coverage-measures-structural-not-validator
title: Certificate-coverage sample_parse_failures measures the generator↔GRAMMAR structural duality (validator-free), not consumer-path acceptance
answers:
  - "what does cert-coverage sample_parse_failures measure"
  - "what does --report-certificate-coverage sample_parse_failures count"
  - "does cert-coverage run the regex compile validator"
  - "why do regex cert-coverage failures differ from parseability_probe failures"
  - "what causes the regex cert-coverage sample_parse_failures"
  - "which parse path does parse_and_cover use vs parse_with_regex_detail"
  - "why does word-boundary spacing break regex re-parse"
tags: [cert-coverage, certifying-linter, grammar-wellformed, regex, parse-path, word-boundary-spacing, lexical, generator, duality]
date: 2026-06-07
status: current
evidence: "tool-backed 2026-06-07 (PGEN-EBNF-SOT-0003). Source: parser_registry.rs parse_and_cover_regex (:327) calls only parse_full_regex(), validator intentionally omitted (comment :324); parse_with_regex_detail (:301) calls parse_full_regex() THEN validate_regex_compile_contract (:305). Measured: ast_pipeline regex.ebnf --report-certificate-coverage --entry-rule regex --count 200 --seed 0 = sample_parse_failures=39 (deterministic, 2 runs). Bucket of same 200 via parseability_probe: 39 structural ('did not consume full input') + 8 validator-only (1 \\u + 7 other); the 39 structural == cert-coverage's 39. --no-word-boundary-spacing regen collapses structural 39->2. clean vs spaced: (*ACCEPT) parses / (*ACCEPT ) fails; (?P>Nae) parses / (?P>Nae ) fails; (?(j)) parses / (?(j )) fails."
reverify: "grep -n 'intentionally NOT applied' rust/src/parser_registry.rs; ./rust/target/debug/ast_pipeline grammars/regex.ebnf --report-certificate-coverage --entry-rule regex --count 200 --seed 0 | grep CERTIFICATE-COVERAGE (expect sample_parse_failures=39); regenerate with --no-word-boundary-spacing and re-bucket (structural collapses to ~2)"
---

## The distinction (two parse paths, two acceptance sets)

PGEN has two parse paths with DIFFERENT acceptance, and they measure different things:

- **Witness / cert-coverage path** — `parse_and_cover_<grammar>` (e.g. `parse_and_cover_regex`,
  `parser_registry.rs:327`). Runs ONLY the generated parser's `parse_full_*()` — the STRUCTURAL grammar.
  For regex it **deliberately omits** `validate_regex_compile_contract` (explicit comment, `:324`). The
  `--report-certificate-coverage` gate's `sample_parse_failures` is computed on THIS path.
- **Consumer / detail path** — `parse_with_<grammar>_detail` (e.g. `parse_with_regex_detail`,
  `parser_registry.rs:301`), and `parseability_probe --parse`. For regex this runs `parse_full_regex()`
  **then** the out-of-band `validate_regex_compile_contract`. This is what downstream consumers see.

**Therefore `cert-coverage sample_parse_failures` measures the generator↔GRAMMAR (structural) duality
only — it is validator-free.** A construct the validator rejects but the grammar accepts (regex `\u{…}`,
`(*verb)`) PARSES in cert-coverage → scored as a witness, never a failure. Conversely a sample that fails
cert-coverage failed the STRUCTURAL parse.

## The regex instance (count 200, seed 0)

`sample_parse_failures=39`, deterministic. All 39 are STRUCTURAL: the stimuli generator's
`apply_word_boundary_spacing` (`ast_pipeline/stimuli_generator.rs:~7016`) inserts a trailing `" "`
separator after an identifier-class token to prevent fusion with a following token — but over-inserts when
the next char is a closing delimiter `)` that cannot fuse. The spurious space breaks `(*VERB )` /
`(?P>NAME )` / `(?(COND ))` on re-parse. `--no-word-boundary-spacing` collapses 39→2 (so ~37/39 are this).
Sub-rules that genuinely allow trailing whitespace (`(*MARK:x )`, `a{1 ,1 }`) are unaffected. Owned by
`LEXICAL-ANNOTATIONS.5`.

## Lesson (avoid the mis-attribution)

A "semantic" reject message from `parseability_probe` (`unsupported …`, `unrecognized …`) proves the
**consumer** path runs a validator — it does NOT explain a **cert-coverage** failure, which uses a
different, validator-free path. Before attributing a metric to a cause, confirm WHICH parse path the metric
computes on. Related: `ebnf-single-source-of-truth`, `grammar-linter-trustworthiness`,
`project_lexical_annotations_fourth_pillar`.
