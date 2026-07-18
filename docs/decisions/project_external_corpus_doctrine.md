---
name: project-external-corpus-doctrine
description: STANDING DIRECTIVE (director 2026-06-08) — every PGEN parser shall be exercised by TWO independent confidence sources — (1) the internal stimuli generator (manufactured stimuli) AND (2) officially-recognized EXTERNAL test corpora (or, where none exists, curated external test files/fragments) — in sufficient number and quality to build confidence the parser is accurate, robust, and handles any type of input. Internally manufactured + externally derived, both required. Regex already does this (PCRE2 regex_corpus_bundle/ + the generator); generalize to ALL parsers. JSON is the first concrete instance. Owned by the EXTERNAL-CORPUS tree.
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-08
  owning_tree: EXTERNAL-CORPUS
---

**THE DIRECTIVE (binding, director 2026-06-08).** Every PGEN parser shall be exercised by **two
independent sources of confidence**, both required:

1. **Internally manufactured** — the EBNF-driven **stimuli generator** (the existing
   `--generate-stimuli` / closed-loop / certificate-coverage machinery). This proves the parser against
   what the grammar *can* produce.
2. **Externally derived** — an **officially-recognized external test corpus** for that language (or,
   where no official corpus exists, **curated external test files / fragments**), in **sufficient number
   and quality** to build confidence the parser is **accurate, robust, and can handle any type of input**.

Neither source alone is enough. The generator can only manufacture what the grammar already describes
(so it cannot, by construction, surface a gap between the grammar and the *real* language); an external
corpus, authored independently of our grammar, is exactly the oracle that exposes that gap. Conversely a
fixed external corpus is finite and cannot exhaustively cover the grammar's branch/structure space the
way the generator's coverage-driven witnessing does. The two are **complementary, not redundant** — one
proves *grammar-internal* well-formedness/coverage (see `GRAMMAR-WELLFORMED`), the other proves
*real-world conformance/robustness*.

**Why this is the director's exact wording.** Three messages (2026-06-08): "We need officially and
substantive recognized JSON test corpus that will exercise the JSON parser through and through" →
"outside those created by the stimuli generator, of course" → "a general request: all parsers should use
the stimuli generator for testing but in addition we should also use officially recognized test corpus,
if not, external test files, fragments in sufficient number and quality to build confidence that the
corresponding parser is accurate and robust and can handle any type of input, internally manufactured
(stimuli generator) and derived from outside (external test corpus)."

**The precedent (already in the repo).** `regex` already satisfies this: the **PCRE2** corpus bundle
(`regex_corpus_bundle/` — immutable upstream snapshots under `third_party/upstream/`, normalized
`corpus/`, an executable `oracle/`, `manifests/`, `scripts/`, and the maintained gates
`regex_corpus_bundle_contract_gate` / `regex_pcre2_textsafe_corpus_gate` /
`regex_pcre2_compile_oracle_gate`) is the **external** source, and the stimuli generator is the
**internal** source. This directive **generalizes that pattern to every parser family**.

**How to apply.**
- For each parser family, stand up an external-corpus surface modelled on `regex_corpus_bundle/`: vendor an
  **immutable upstream snapshot** (with provenance — upstream URL, commit SHA, license) separate from any
  normalized/oracle outputs, a **manifest**, a **runner/gate**, and honest **characterization** of results.
- Prefer an **officially recognized** corpus when one exists (e.g. **JSONTestSuite** "Parsing JSON is a
  Minefield", MIT, + the classic **json.org JSON_checker** for JSON; the IEEE/Accellera LRM corpora and
  real-world designs for SV/VHDL; PCRE2 `testdata` for regex). Where none exists, curate a sufficient set
  of external fragments and SAY it is curated (do not pass curated fragments off as an official corpus).
- ⚠️ **Characterize, do not game.** Some PGEN grammars are deliberately *simplified subsets* of their
  language (e.g. `grammars/json.ebnf` has no string escapes / unicode escapes / number exponents). An
  external conformance corpus WILL surface divergences that are **grammar-scope limits, not parser bugs**.
  MEASURE the gap tools-first ([[feedback_no_codebase_change_without_tool_backed_facts]]), report it
  honestly ([[feedback_always_signoff_decisions]]), and let that evidence decide whether the grammar
  should be upgraded toward full conformance — never derive expecteds from the parser
  ([[feedback_corpus_expected_from_spec_not_fix]]; the corpus's own `y_/n_/i_` / `pass/fail` labels are the
  independent oracle). Parser-AGNOSTIC ([[feedback_ast_pipeline_parser_agnostic]]).
- A parser family's closure bar is extended: confidence now requires BOTH the generator proof AND the
  external-corpus characterization at sufficient number/quality.

Composes with — does not replace — `GRAMMAR-WELLFORMED` (the grammar-internal duality),
[[project_stimuli_generator_signoff_vision]] (the generator half), and the existing per-family integration
contracts. Tracked by the `EXTERNAL-CORPUS` task tree; JSON (`EXTERNAL-CORPUS.2`) is the first concrete
slice.

---

**⭐ EXTENDED (director 2026-07-18, session #150, verbatim):** *"any PGEN parser shall have its
extern test corpus for functionality/accuracy and speed. Also by using official external test
corpus the stimuli generator can learn new things are generate better real-world sample, so it
is overall a good thing to confront PGEN parsers to external test corpus."*

1. **Per-parser external corpus now covers SPEED too** — every parser family's officially-
   recognized external corpus serves BOTH the accuracy proof (accept/reject vs the authority)
   AND the speed measurement (per-sample parse-time distribution, reported as geomean AND max —
   the RGX `-0129`/`-0130` reporting directive generalized).
2. **Corpus-informed stimuli generation (new direction, director idea):** external corpora are
   a LEARNING source for the stimuli generator — mine real-world sample structure to steer
   generation toward realistic shapes (better coverage of what users actually write, not just
   what the grammar admits). To be scoped as a capability-gap item under the stimuli-signoff
   umbrella when prioritized; parser-agnostic by doctrine.
3. Delegation reaffirmed: engineering decisions engineer-owned; the director throws ideas to
   adopt or discard.
