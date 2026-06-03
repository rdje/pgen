---
name: project-parser-signoff-pillars
description: The four parser sign-off pillars — (1.a) reject → PARSE-COMPLETENESS, (1.b) hang → PARSE-TERMINATION, (1.c) mis-parse → PARSE-FIDELITY, (2) coverage → STIMULI-SIGNOFF/SV-EXH-PROOF.7.4. Parser-agnostic, literature-grounded. Director-commissioned 2026-06-03.
metadata:
  node_type: memory
  type: project
---

**Director decision (2026-06-03):** decompose "remaining parser problems" into **four
sign-off pillars**, each with its own task tree, parser-AGNOSTIC (SV-first), and each
grounded in the literature FIRST (citation + worked mapping) so we adopt existing solutions
rather than reinvent. Triggered by a clear-picture request that surfaced a constantly-
conflated distinction.

**Two surfaces, four pillars:**
1. **Parser correctness on input** — three modes:
   - (1.a) **reject** (errors on valid input) → tree `PARSE-COMPLETENESS`
   - (1.b) **hang** (non-termination / super-linear) → tree `PARSE-TERMINATION`
   - (1.c) **mis-parse** (accepts but wrong AST, silent) → tree `PARSE-FIDELITY`
2. **Stimuli generator EBNF coverage** → trees `STIMULI-SIGNOFF` + `SV-EXH-PROOF.7.4`
   (the "753"/literal-0 work; already owned).

**Tool-verified state (2026-06-03):** Surface 1 has **no known failing inputs** (SV
external corpus 14/14; generated round-trip 16/16; realistic corpus 730/730). The residual
("753" → 273 after `.7.4.5`) is **Surface 2**, a generator coverage gap, NOT parser-input
failures — these are routinely conflated. See [[parser-signoff-four-pillars]].

**The deep link:** Surface 2 is the *engine that proves Surface 1 exhaustively* — literal-0
stimuli + round-trip = exhaustive (1.c); the differential corpus = (1.a); linear-time +
watchdog = (1.b).

**Structuring rule (how findings are owned):** task trees own **work** (adoption as leaves);
the **Knowledge Map owns the literature findings** as fact cards (so nobody re-searches);
**warnings become acceptance criteria** of the owning tree (not separate trees). KM cards:
[[parse-completeness-differential-oracle]], [[stateful-packrat-not-linear]],
[[parse-fidelity-oracles]], [[grammar-coverage-and-directed-generation]].

**Literature grounding (sweep 2026-06-03, [[feedback_research_grounded_sota_no_trial_and_revert]]):**
- A (reject): differential testing (McKeeman 1998; Csmith PLDI 2011; EMI PLDI 2014) + ready
  oracle/corpus (slang, Verible, Verilator, chipsalliance `sv-tests`); grammar-vs-spec
  coverage (Lämmel FASE 2001); grammar-mutation maturity (TOSEM 2025).
- B (hang): Ford packrat ICFP 2002 + PEG well-formedness POPL 2004; **⚠️ Chida & Kawakoya
  CC 2020 — stateful packrat may be EXPONENTIAL (PGEN's semantic store = state)** + its
  conditional-memoization fix; Warth PEPM 2008.
- C (mis-parse): invertible syntax (Rendel & Ostermann, Haskell 2010); metamorphic testing
  (Chen et al. 1998) + EMI; round-trip + A5 `_meta`.
- D (coverage): Purdom 1972; k-path + Tribble (Havrikov & Zeller ASE 2019); **Directed
  Grammar-Based Test Generation / FDLOOP (arXiv 2508.01472, 2025)** for `.7.4.6`; Boltzmann
  samplers.

Composes with [[project_vision_and_discipline]], [[feedback_ast_pipeline_parser_agnostic]],
[[project_knowledge_map_retrieval_layer]].
