---
name: project-certcoverage-measures-structural-not-validator
description: cert-coverage `sample_parse_failures` (--report-certificate-coverage) is computed on the witness path `parse_and_cover_<grammar>`, which runs ONLY the generated parser's structural parse and OMITS out-of-band validators (e.g. regex `validate_regex_compile_contract`). So it measures the generator↔GRAMMAR structural duality, NOT consumer-path acceptance. The regex cert-coverage failures are STRUCTURAL (generator word-boundary over-insertion, LEXICAL), not the validator — correcting the PGEN-EBNF-SOT-0001 attribution. Tool-backed 2026-06-07 (PGEN-EBNF-SOT-0003).
metadata:
  node_type: memory
  type: project
  created: 2026-06-07
  owning_tree: EBNF-SOURCE-OF-TRUTH
  corrects: PGEN-EBNF-SOT-0001
---

**The finding (tool-backed, `PGEN-EBNF-SOT-0003`, EBNF-SOURCE-OF-TRUTH.2.1).**

PGEN exposes two parse paths with different acceptance sets:

- **cert-coverage / witness path** — `parse_and_cover_<grammar>` (`parser_registry.rs:327` for regex).
  Runs ONLY `parse_full_*()` (the structural grammar); for regex it **intentionally omits**
  `validate_regex_compile_contract` (explicit comment `parser_registry.rs:324`). The
  `--report-certificate-coverage` gate's `sample_parse_failures` is computed here.
- **consumer / detail path** — `parse_with_<grammar>_detail` (`:301`) and `parseability_probe --parse`.
  For regex this runs `parse_full_regex()` **then** `validate_regex_compile_contract` (`:305`).

**Consequence:** `cert-coverage sample_parse_failures` measures the generator↔GRAMMAR **structural**
duality — it is **validator-free**. A construct the validator rejects but the grammar structurally accepts
(regex `\u{…}`, unrecognized `(*verb)`) parses in cert-coverage → scored as a witness, never a failure.

**Why this matters (the correction).** `PGEN-EBNF-SOT-0001` attributed the regex cert-coverage
`sample_parse_failures` (the `GRAMMAR-WELLFORMED.H.1` "6", and 39 at count 200) to the out-of-band
validator (`\u`/`(*verb)`). That is wrong: the cert-coverage path never runs the validator. Measured
breakdown (count 200, seed 0, deterministic): 39 structural + 8 validator-only (via `parseability_probe`);
the 39 structural == cert-coverage's 39. `--no-word-boundary-spacing` collapses structural 39→2, so ~37/39
are the generator's `apply_word_boundary_spacing` over-inserting a trailing `" "` before a closing `)`
(a LEXICAL faithfulness defect → `LEXICAL-ANNOTATIONS.5`). The EBNF-SOT validator defect is real but lives
on the **consumer** path only (→ `EBNF-SOURCE-OF-TRUTH.3`, re-scoped).

**The lesson (binding).** A "semantic" reject message from `parseability_probe` proves the CONSUMER path
runs a validator — it does NOT explain a cert-coverage failure (a different, validator-free path). Before
attributing a metric to a cause, confirm WHICH parse path the metric computes on. This is the same
mis-attribution mode (regex parse-path conflation) that the BE-ALERT discipline
([[feedback_be_alert_root_cause_fishy_immediately]]) targets. Composes with
[[project_ebnf_is_single_source_of_truth]] (the rule, still valid) and
[[project_lexical_annotations_fourth_pillar]] (the fix owner). KM:
`cert-coverage-measures-structural-not-validator`.
