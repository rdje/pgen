# JSON: commit to the FULL official standard, zero restriction (director 2026-07-22)

**Directive (director, 2026-07-22, verbatim):** "The JSON parser is not the official JSON
standard or reference. So that's good this limited version is fully certified but for me it
is like a toy parser. We should commit to parse the full fledged JSON official standard
with zero restriction."

**Standing meaning:**
- The committed target is the official JSON standard — **RFC 8259 / ECMA-404** — with
  ZERO restriction: the full string escape set (`\"` `\\` `\/` `\b` `\f` `\n` `\r` `\t`
  `\uXXXX` incl. surrogate pairs), the full number grammar (optional minus, no leading
  zeros, fraction, exponent), the exact RFC whitespace set (space/tab/LF/CR only), and
  the RFC's rejection surface (raw control chars in strings, leading zeros, trailing
  commas, etc.) — BOTH directions of fidelity (accepts-valid AND rejects-invalid).
- A "fully certified" verdict on a restricted grammar is honest about coverage but not
  about the LANGUAGE; certification only means every grammar rule is proven/witnessed.
  The grammar itself must now equal the standard.
- Owner: task tree `JSON-RFC8259` (`docs/tasks/JSON-RFC8259.md`). Conformance expecteds
  are derived from the SPEC independently of the implementation
  ([[feedback_corpus_expected_from_spec_not_fix]]); EBNF stays the sole source of truth
  ([[project_ebnf_is_single_source_of_truth]]); this is also the first worked exemplar of
  the horizon goal ([[project_horizon_universal_parser]] — parse ANY precisely-described
  language, and JSON is the most precisely described of all).

**Why (director):** external credibility — a toy JSON parser undermines the claim the
platform can host real languages; JSON is the reference case every developer checks first.
