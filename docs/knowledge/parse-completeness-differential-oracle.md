---
id: parse-completeness-differential-oracle
title: Don't reinvent the no-reject oracle — differential testing + ready-made SV corpus
answers:
  - "how do we prove the parser never rejects valid input"
  - "what reference parsers / corpus to use for SystemVerilog differential testing"
  - "is there a ready-made SV compliance test suite"
  - "what is the literature for parser completeness testing"
tags: [parser, completeness, differential, literature, dont-reinvent]
date: 2026-06-03
status: current
evidence: "McKeeman 1998 Differential Testing; Csmith (Yang et al. PLDI 2011); EMI (Le/Afshari/Su PLDI 2014); Lammel Grammar Testing FASE 2001; Grammar Mutation TOSEM 2025; slang sv-lang.com; Verible; chipsalliance sv-tests"
reverify: see docs/tasks/PARSE-COMPLETENESS.md literature grounding
---

For Pillar A (no false **reject**), the literature says **adopt, don't invent**:

- **Differential testing** is the proven oracle: feed identical input to multiple
  implementations, diff accept/reject. McKeeman (*Differential Testing for Software*, 1998);
  **Csmith** (PLDI 2011); **EMI** (PLDI 2014, 147 GCC/LLVM bugs). Any input ≥2 references
  **accept** but PGEN **rejects** = a (1.a) defect.
- **Ready-made SV oracle + corpus** — slang (`sv-lang.com`, most-compliant open frontend),
  Verible, Verilator/UHDM as references; the **chipsalliance `sv-tests`** tool-independent
  compliance suite as a completeness corpus. Plus real designs (Chipyard, CVA6, ibex, UVM,
  opencores).
- **Grammar-vs-spec coverage** — Lämmel *Grammar Testing* (FASE 2001): EBNF rule inventory
  vs IEEE 1800 Annex A; gaps = reject risk. **Grammar Mutation for Testing Input Parsers**
  (TOSEM 2025) quantifies completeness maturity.

PGEN already emits a differential surface (`diff_taxonomy_rust_failed_reference_passed`,
`sv_external_corpus_triage_gate.sh`) — unowned + small-corpus. The work is to *own + scale*
it, invariant `reference_passed ∧ rust_failed == 0`. Owned by [[parser-signoff-four-pillars]]
→ `PARSE-COMPLETENESS`.
