# EXTERNAL-CORPUS: every parser proven by BOTH the stimuli generator AND officially-recognized external corpora

## Metadata

- Tree ID: `EXTERNAL-CORPUS`
- Status: `active`, BUT the **full-JSON-parser sub-tree (`.2F` umbrella + `.2a`/`.2b`/`.2c.1`/`.2d`) is ON HOLD / DEFERRED** per director (2026-06-08). json was reverted to the **simplified** grammar (`PGEN-EXTERNAL-CORPUS-0006`), which keeps json **cert-coverage CLEAN** (`fully_certified`, `sample_parse_failures=0`); the RFC-8259 upgrade (`.2a`, prototyped at y_ 95/95 in commit `050b4cb2`) is **owned-for-later**. The current locked program is **all existing parsers → `Done`** (incl. cert-coverage wired + UNKNOWN=0) — full-JSON resumes only when the director un-holds it. The `.1`/`.2` corpus characterization stays as the json external-corpus surface. [history: `.1` SCOPING + `.2` JSON corpus characterized json as a simplified subset (y_ 81/95, n_ 158/188, 3 crashes); `.2a`/`.2c` prototyped/root-caused the upgrade before the hold.]
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
- `.2` — **DONE (`PGEN-EXTERNAL-CORPUS-0002`, 2026-06-08): JSON external corpus acquired + the simplified
  `json.ebnf` characterized against it.** Vendored **JSONTestSuite** (MIT, pinned commit `1ef36fa0`, 318
  `test_parsing` files) as an immutable snapshot under `json_corpus_bundle/third_party/upstream/` (+ LICENSE
  + PROVENANCE), with a reproducible runner `json_corpus_bundle/scripts/run_json_corpus.sh` and the dated
  root-caused report `json_corpus_bundle/results/characterization.md`. **MEASURED (the data answer to the
  director's Q1 "does json.ebnf match the official JSON standard?" → NO):** `y_` (MUST accept) **81/95**;
  `n_` (MUST reject) **158/188** (28 wrongly accepted + 2 crash); `i_` 13 accept/21 reject/1 crash; **3
  deep-nesting files ABORT (stack overflow)**. Every divergence root-causes to the SIMPLIFIED grammar (no
  number **exponent**; no string **escapes** — `"[^"]*"` is both too narrow and too wide; **leading zeros**
  allowed; loose **trailing/whitespace**) — grammar-scope limits, NOT engine bugs — plus a real
  **robustness** defect (no recursion/stack guard; cf. RGX-0085). This is exactly the gap an external corpus
  exposes and the internal generator (by construction) cannot. NO grammar/parser change this slice (pure
  characterization, per characterize-don't-game). Spawns two evidence-gated follow-ups: (a) upgrade
  `json.ebnf` toward RFC 8259; (b) a json-parser recursion/stack guard. Recognized corpora used/available:
  - **JSONTestSuite** (Nicolas Seriot, *"Parsing JSON is a Minefield"*, github.com/nst/JSONTestSuite,
    MIT) — the de-facto comprehensive corpus: `test_parsing/` files prefixed `y_` (MUST accept), `n_`
    (MUST reject), `i_` (implementation-defined). ~300+ files.
  - **json.org JSON_checker** (Crockford) — the classic `pass1-3.json` / `fail1-33.json` set.
  - Oracle = the corpus's OWN `y_/n_/i_` (and `pass/fail`) labels (an external, fix-independent oracle).
  - Expectation (honest, pre-measurement): `grammars/json.ebnf` is a SIMPLIFIED grammar → expect many
    `y_*` accepts to fail (escapes, `\uXXXX`, exponents, deep nesting, whitespace forms) and some `n_*`
    to be mis-accepted. The slice MEASURES this; it does not pre-judge it.
- **`.2F` — UMBRELLA GOAL: `json.ebnf` shall FULLY match the official JSON standard** (RFC 8259 /
  ECMA-404), as a **proof that PGEN can handle a complete standard "with no sweat"** (director 2026-06-08,
  [[project_json_full_standard_proof]]). JSON is the deliberate proof vehicle — small, complete, and
  adversarially corpus'd — so driving its recognized corpora fully green is objective evidence the platform
  implements a full standard end-to-end. Acceptance = the corpora fully green: **`y_` 95/95 (DONE, `.2a`)**,
  **`n_` 188/188** (needs `.2c` trailing-content + `.2b` deep-nesting crashes-→-rejects), **no crashes**,
  `i_` outcomes documented; *(stretch)* cert-coverage raw-witness `sample_parse_failures` → 0. Rolls up
  `.2a` (done) + `.2c` + `.2b` + `.2d`, then a closure re-run. Only when green: describe json as *fully*
  standard-conformant in the live tracker / book.
- **`.2d` — collect ADDITIONAL recognized JSON corpora** (director 2026-06-08: "we will need to look for and
  collect [more] recognized JSON test corpus"). Broaden the external oracle beyond JSONTestSuite so the
  full-standard proof rests on multiple independent recognized sources. Candidates: **json.org JSON_checker**
  (Crockford `pass1-3.json` / `fail1-33.json`), **nativejson-benchmark** conformance set (Milo Yip; itself
  aggregates JSON_checker + JSONTestSuite), the **RFC 8259 / ECMA-404** worked examples, and any other
  recognized parser conformance suites (each vendored as an immutable snapshot with provenance + license
  under `json_corpus_bundle/third_party/upstream/`, wired into the runner with its own label oracle). Verify
  license compatibility before vendoring; prefer permissive (MIT/BSD/public-domain) sources.
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
| — | `EXTERNAL-CORPUS.2` | `done` (`PGEN-EXTERNAL-CORPUS-0002`) | JSONTestSuite vendored + the simplified `json.ebnf` characterized (y_ 81/95, n_ 158/188, 3 crashes); the data answer to Q1 (json.ebnf does NOT match the standard). `json_corpus_bundle/`. |
| — | `EXTERNAL-CORPUS.2a` (json.ebnf → RFC 8259) | `prototyped, then REVERTED + DEFERRED` (`-0003` landed, `-0006` reverted) | Prototyped the RFC-8259 terminal upgrade (PROVED the path: y_ 81→95/95, n_ 158→181/188) — but **REVERTED to the simplified grammar and DEFERRED per director (2026-06-08)**: the full JSON parser is owned-for-later (the director tunes the roadmap deliberately), and the simplified grammar keeps json **cert-coverage CLEAN** (`fully_certified`, `sample_parse_failures=0`), whereas the upgrade introduced 31 raw-witness fails. The prototype proof (the exact terminal forms + the 95/95 result) is preserved in git (`PGEN-EXTERNAL-CORPUS-0003`, commit `050b4cb2`) for when this is un-held. |
| ★ | `EXTERNAL-CORPUS.2F` (umbrella: FULL JSON-standard conformance) | **`ON HOLD` / deferred for later** (director 2026-06-08) | **The proof that PGEN handles a complete standard "with no sweat"** ([[project_json_full_standard_proof]]) — OWNED but DEFERRED; the director will say when. Rolls up `.2a` (prototyped/reverted) + `.2c.1` + `.2b` + `.2d`; acceptance = recognized JSON corpora fully green (y_ 95/95, n_ 188/188, no crashes). NOT worked until un-held. |
| — | `EXTERNAL-CORPUS.2c` (root-cause + design) | `done` (`PGEN-EXTERNAL-CORPUS-0005`) | **ROOT-CAUSED (tools-first, overturning the "strict end-of-input" framing): the 5 `n_` are accepted because the json parser SKIPS COMMENTS as LAYOUT — `parse_full` DOES enforce end-of-input (`position == input.len()`), but `consume_layout_for_terminal` (`ast_based_generator.rs:5234-5293`) unconditionally skips `#` / `//` / `/* */` comments, which JSON forbids.** 4 cases = trailing comment skipped at the `<EOF>` layout (`allow_trailing_layout`, `:1090`); 1 inline `/*comment*/` = skipped at the per-terminal layout (`allow_layout_skip_for_terminals`, `:3931`). The codegen ALREADY disables both for `regex` — but by GRAMMAR NAME (`grammar_name != "regex"`), which the parser-agnostic doctrine ([[feedback_features_parser_agnostic_enable_all_parsers]]) forbids extending. DESIGN: a parser-agnostic CAPABILITY ("grammar self-manages layout / has no comment syntax") true for BOTH regex and json (both bake whitespace into terminals), replacing the regex name-gate; json then behaves like regex (no comment layout) → the 5 `n_` reject. Implementation = `.2c.1` (careful: cross-grammar byte-identical for SV/VHDL/regex; the capability declaration mechanism). |
| ⏸ | `EXTERNAL-CORPUS.2c.1` (capability-based comment-layout fix) | **`ON HOLD`** (under `.2F`) | [design ready, deferred with the full-JSON umbrella] Implement the `.2c` design: a per-grammar "self-manages layout / no-comment-layout" CAPABILITY flag, set for regex + json, replacing the `grammar_name == "regex"` name-gates at `:1090`/`:3931`/`:3932`. |
| ⏸ | `EXTERNAL-CORPUS.2b` (json recursion/stack guard) | **`ON HOLD`** (under `.2F`) | The 3 deep-nesting aborts — deferred with the full-JSON umbrella. |
| ⏸ | `EXTERNAL-CORPUS.2d` (collect more recognized JSON corpora) | **`ON HOLD`** (under `.2F`) | Broaden the external oracle — deferred with the full-JSON umbrella. |
| 2 | `EXTERNAL-CORPUS.3+` | `pending` (not blocking the all-parsers-Done goal) | Generalize the external-corpus surface to VHDL / SV / rtl_* / annotation grammars. |

## Decisions

- `2026-06-08`: Created from the director directive (three messages). The frame: confidence requires TWO
  independent oracles per parser — the internal generator AND an external recognized corpus — generalizing
  the existing `regex` + `regex_corpus_bundle/` precedent. JSON is the first concrete instance. See
  [[project_external_corpus_doctrine]].
