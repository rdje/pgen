# EXTERNAL-CORPUS: every parser proven by BOTH the stimuli generator AND officially-recognized external corpora

## Metadata

- Tree ID: `EXTERNAL-CORPUS`
- Status: `active` (director-commissioned 2026-06-08; `.1` SCOPING DONE `PGEN-EXTERNAL-CORPUS-0001`; frontier `.2` JSON corpus acquisition + characterization)
- Family / slice-id prefix: `PGEN-EXTERNAL-CORPUS-<NNNN>`
- Roadmap lane: cross-cutting parser sign-off — confidence requires TWO independent oracles
  (internal generator + external corpus) for every parser family
- Created: `2026-06-08`
- Owner: repo-local workflow
- Parser-AGNOSTIC ([[feedback_ast_pipeline_parser_agnostic]]). Disciplines:
  [[feedback_no_codebase_change_without_tool_backed_facts]] (measure the corpus result, don't guess),
  [[feedback_corpus_expected_from_spec_not_fix]] (the corpus's own labels are the oracle — never derive
  expecteds from the parser), [[feedback_always_signoff_decisions]] (report the gap honestly; don't game).
- Decision record: [[project_external_corpus_doctrine]]

## The frame (the directive — binding)

Director directive, 2026-06-08 (three messages): "all parsers should use the stimuli generator for
testing but in addition we should also use officially recognized test corpus, if not, external test
files, fragments in sufficient number and quality to build confidence that the corresponding parser is
accurate and robust and can handle any type of input, internally manufactured (stimuli generator) and
derived from outside (external test corpus)."

So **every PGEN parser shall be exercised by two independent confidence sources**, both required:

1. **Internally manufactured** — the EBNF-driven stimuli generator (`--generate-stimuli`, the closed-loop
   gates, the `--report-certificate-coverage` Phase-H surface). Proves the parser against what the grammar
   can produce.
2. **Externally derived** — an officially-recognized external corpus (or curated external fragments where
   none exists), in sufficient number and quality to build confidence the parser is accurate, robust, and
   handles any type of input. Authored independently of our grammar, it is the oracle that exposes the gap
   between the grammar and the *real* language — which the generator, by construction, cannot.

The two are **complementary, not redundant** (full rationale in [[project_external_corpus_doctrine]]).

### The precedent (already in the repo) — `regex`

`regex` already satisfies the directive: the **PCRE2** corpus bundle `regex_corpus_bundle/` is the external
source and the stimuli generator is the internal source. Its shape is the template for every other family:

- `third_party/upstream/` — immutable upstream snapshot (provenance: URL, commit SHA, license)
- `corpus/` — normalized corpus separate from the immutable snapshot
- `oracle/` — the executable reference (e.g. `pcre2test`) that adjudicates accept/reject
- `manifests/`, `schemas/`, `scripts/`, `docs/`, `.cache/downloads/`
- maintained gates: `regex_corpus_bundle_contract_gate`, `regex_pcre2_textsafe_corpus_gate`,
  `regex_pcre2_compile_oracle_gate`

This tree generalizes that pattern to every parser family.

### ⚠️ Characterize, do not game (binding)

Some PGEN grammars are deliberately **simplified subsets** of their language. `grammars/json.ebnf` is the
clearest example — its `string` is `"[^"]*"` (no escapes, no `\uXXXX`), its `number` has no exponent, and
it predates RFC 8259 corner cases. An external **conformance** corpus WILL therefore surface many
divergences that are **grammar-scope limits, not parser bugs**. The deliverable of a corpus slice is an
HONEST **characterization** (measure the accept/reject vs the corpus's own `y_/n_/i_` or `pass/fail`
labels, classify each divergence as grammar-scope vs real defect), NOT a green "conformance pass" forced
by gaming. That evidence then DECIDES whether the grammar should be upgraded toward full conformance — a
separate, tools-backed decision, never a silent reclassification.

## Ordered build list

- `.1` — **SCOPING (this slice, DOCS):** capture the directive (decision record + this tree +
  registration + memory). The per-family plan + the recognized-corpus inventory below. NO code/test-data.
- `.2` — **JSON external corpus (the director's first concrete ask).** Acquire an officially-recognized
  JSON corpus, vendor an immutable snapshot (provenance + license), run the json parser over it, and
  produce an honest characterization. Recognized corpora:
  - **JSONTestSuite** (Nicolas Seriot, *"Parsing JSON is a Minefield"*, github.com/nst/JSONTestSuite,
    MIT) — the de-facto comprehensive corpus: `test_parsing/` files prefixed `y_` (MUST accept), `n_`
    (MUST reject), `i_` (implementation-defined). ~300+ files.
  - **json.org JSON_checker** (Crockford) — the classic `pass1-3.json` / `fail1-33.json` set.
  - Oracle = the corpus's OWN `y_/n_/i_` (and `pass/fail`) labels (an external, fix-independent oracle).
  - Expectation (honest, pre-measurement): `grammars/json.ebnf` is a SIMPLIFIED grammar → expect many
    `y_*` accepts to fail (escapes, `\uXXXX`, exponents, deep nesting, whitespace forms) and some `n_*`
    to be mis-accepted. The slice MEASURES this; it does not pre-judge it.
- `.3+` — **roll the pattern to the other families** where an external corpus/fragment set adds confidence
  beyond the generator: VHDL, SystemVerilog (+ preprocessor), rtl_* (real-world RTL fragments / LRM
  examples), and the annotation grammars (curated fragment sets). regex is already done (the precedent).
- `.N` — **closure-bar extension:** a parser family's confidence claim requires BOTH the generator proof
  AND the external-corpus characterization at sufficient number/quality (reflected in
  `LIVE_ACHIEVEMENT_STATUS.md`).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `EXTERNAL-CORPUS.1` | `done` (`PGEN-EXTERNAL-CORPUS-0001`) | SCOPING — captured the directive durably (decision record [[project_external_corpus_doctrine]] + this tree + TASK_TREE/INDEX registration) so the corpus work is task-tree-owned before any test-data lands. |
| 1 | `EXTERNAL-CORPUS.2` | `pending` | The director's first concrete ask: a recognized JSON corpus + an honest characterization of the simplified `json.ebnf` against it. |
| 2 | `EXTERNAL-CORPUS.3+` | `pending` | Generalize to VHDL / SV / rtl_* / annotation grammars. |

## Decisions

- `2026-06-08`: Created from the director directive (three messages). The frame: confidence requires TWO
  independent oracles per parser — the internal generator AND an external recognized corpus — generalizing
  the existing `regex` + `regex_corpus_bundle/` precedent. JSON is the first concrete instance. See
  [[project_external_corpus_doctrine]].
