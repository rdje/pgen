# PARSE-COMPLETENESS — the parser never rejects valid input (parser-agnostic; SV-first)

> Task tree. **Metadata** — Status: `proposed` (literature-grounded; awaiting first leaf);
> Created: 2026-06-03; Roadmap lane: parser sign-off pillar **A** (of 4). Owns failure mode
> **(1.a) reject** — the parser returns a parse error on input that is actually valid.
>
> Director-commissioned 2026-06-03 ("1 tree for each parser failure mode; solve elegantly,
> efficiently, zero regression; research the literature first so we don't reinvent").
> Parser-AGNOSTIC by directive (the mechanisms harden every PGEN family) per
> [[feedback_ast_pipeline_parser_agnostic]] + [[project_vision_and_discipline]]. SV is the
> first target. Disciplines: [[feedback_research_grounded_sota_no_trial_and_revert]],
> [[feedback_tools_first_no_guessing]], [[feedback_no_workarounds_fix_hierarchy]].

## The principle (binding)
A PGEN parser shall **accept every input its target language accepts**. A reject of valid
input is a **completeness** defect (grammar too strict / a missing production / an
over-tight gate). "Solve for good" = a standing, mechanical oracle that surfaces any such
reject before it ships, on a corpus broad enough to be trustworthy.

## What this is NOT
Not about the *stimuli generator's* coverage residual (that is Pillar D / `STIMULI-SIGNOFF`
+ `SV-EXH-PROOF.7.4`). Not about hang (Pillar B `PARSE-TERMINATION`) or mis-parse (Pillar C
`PARSE-FIDELITY`). See [[parser-signoff-four-pillars]].

## Literature grounding (sweep 2026-06-03 — citations + worked mapping)
- ♻️ **Differential testing is the proven oracle** — McKeeman, *Differential Testing for
  Software* (1998); **Csmith** (Yang, Chen, Eide, Regehr, PLDI 2011); **EMI** (Le, Afshari,
  Su, PLDI 2014, 147 GCC/LLVM bugs). Feed identical input to multiple implementations; any
  input ≥2 references **accept** but PGEN **rejects** is a (1.a) bug. See
  [[parse-completeness-differential-oracle]].
- ♻️ **Ready-made SV oracle + corpus — DO NOT build from scratch:** **slang**
  (sv-lang.com, most-compliant open frontend), **Verible**, **Verilator/UHDM** as reference
  parsers; the **chipsalliance `sv-tests`** tool-independent compliance suite as a ready
  completeness corpus.
- ✅ **Grammar-vs-spec coverage** — Lämmel, *Grammar Testing* (FASE 2001); rule-coverage
  reduction (Empirical SE 2008): track PGEN's EBNF rule inventory vs the language spec
  (IEEE 1800 Annex A for SV); unimplemented productions = reject risk.
- ✅ **Maturity metric** — *Grammar Mutation for Testing Input Parsers* (TOSEM 2025): grammar
  mutation quantifies how complete a parser implementation is.
- **Worked mapping:** PGEN ALREADY emits a differential surface — the SV gate reports
  `diff_taxonomy_rust_failed_reference_passed: 0`, and `sv_external_corpus_triage_gate.sh`
  runs real corpora. The infra exists but is **unowned + small-corpus**. Elegant,
  zero-regression path: it's a *test harness* — own it, scale the references + corpus, make
  `reference_passed ∧ rust_failed == 0` a sign-off invariant. No engine change unless a real
  reject is found (then a normal targeted grammar fix slice).

## Leaves
### `.1` — differential-oracle harness design (pure docs) — PENDING (next)
Own the existing `diff_taxonomy` infra; pin the reference set (slang primary; Verible;
Verilator/UHDM) + verdict semantics (accept/reject only — AST-diff is Pillar C) + the
`reference_passed ∧ rust_failed == 0` invariant. Parser-agnostic: each family names its own
reference oracle(s).

### `.2` — ingest the `sv-tests` compliance corpus + run differential — PENDING
Wire chipsalliance `sv-tests` as a completeness corpus; run rust-vs-references; triage every
`reference_passed ∧ rust_failed` as a (1.a) defect (own grammar-fix sub-leaves).

### `.3` — scale the real-world differential corpus — PENDING
Chipyard, CVA6, ibex, BlackParrot, the full UVM library, opencores. Each new reject → a
targeted, task-tree-owned grammar fix (no broad sweeps).

### `.4` — grammar-vs-spec coverage ledger — PENDING
EBNF rule inventory vs IEEE 1800 Annex A (SV); gaps ranked by reject risk. Generalizes to
each family's spec.

### `.5` — grammar-mutation maturity metric (TOSEM 2025) — PROPOSED (later)
Quantify completeness maturity; optional, after `.2`-`.4`.

## Frontier
`.1` (harness design). Then `.2` (sv-tests differential) is the first measurable
completeness proof.
