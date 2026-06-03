---
id: parser-signoff-four-pillars
title: The four parser sign-off pillars (reject / hang / mis-parse + coverage)
answers:
  - "what are the remaining problem types for the SV parser"
  - "how is parser correctness decomposed / the four pillars"
  - "which task tree owns parser reject / hang / mis-parse"
  - "is the parser failing on real inputs or is it the stimuli generator"
  - "what is the difference between parser failures and the stimuli residual"
tags: [parser, signoff, taxonomy, architecture]
date: 2026-06-03
status: current
evidence: docs/tasks/PARSE-COMPLETENESS.md, PARSE-TERMINATION.md, PARSE-FIDELITY.md, STIMULI-SIGNOFF.md, SV-EXH-PROOF.md (.7.4); live SV status = external corpus 14/14, parser clean on tracked input
reverify: grep -l "parser sign-off pillar" docs/tasks/PARSE-*.md
---

Parser issues split into **two surfaces** that are constantly conflated:

1. **Parser correctness on real input** — three modes:
   - **(1.a) reject** — errors on valid input → tree **`PARSE-COMPLETENESS`**
   - **(1.b) hang** — non-termination / super-linear → tree **`PARSE-TERMINATION`**
   - **(1.c) mis-parse** — accepts but wrong AST (silent) → tree **`PARSE-FIDELITY`**
2. **Stimuli generator EBNF coverage** — can't *generate a witness in time* for the deepest
   rules → trees **`STIMULI-SIGNOFF`** + **`SV-EXH-PROOF.7.4`** (this is the "753"/literal-0
   work). See [[stimuli-residual-coverage-model]].

**Current state (tool-verified 2026-06-03):** Surface 1 has **no known failing inputs** —
SV external corpus 14/14, generated round-trip 16/16, realistic corpus 730/730. The "753"/
residual is **Surface 2**, a generator coverage gap, **not** parser-input failures.

**The deep link:** Surface 2 is the *engine that proves Surface 1 exhaustively* — literal-0
stimuli + round-trip = exhaustive (1.c) proof; the differential corpus = (1.a) proof;
linear-time + watchdog = (1.b) proof. Not four separate fights — three guarantees and the
machine that establishes them everywhere. Pillars are parser-AGNOSTIC (SV-first). Decision:
[[project-parser-signoff-pillars]].
