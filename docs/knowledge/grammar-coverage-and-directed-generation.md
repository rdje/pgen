---
id: grammar-coverage-and-directed-generation
title: Don't reinvent stimuli coverage / targeted generation — k-path, Tribble, FDLOOP, Boltzmann
answers:
  - "what is the literature for grammar-based test/stimuli generation coverage"
  - "how to generate an input that reaches a specific grammar production (directed)"
  - "what coverage metric for grammar-based generation (k-path)"
  - "how to generate deep witnesses without timeout (.7.4.6)"
  - "is there a published technique for the literal-0 / derivation-directed generation idea"
tags: [stimuli, coverage, generation, literature, dont-reinvent]
date: 2026-06-03
status: current
evidence: "Purdom 1972 sentence generator; Havrikov & Zeller, Systematically Covering Input Structure (k-paths), ASE 2019 + tool Tribble; Kirschner & Soremekun, Directed Grammar-Based Test Generation (FDLOOP), arXiv 2508.01472 (2025); Boltzmann samplers (Duchon et al. 2004; USAIN BOLTZ); EMI (PLDI 2014)"
reverify: see docs/tasks/SV-EXH-PROOF.md (.7.4.x) + docs/tasks/STIMULI-SIGNOFF.md
---

For Pillar D (stimuli EBNF coverage → literal-0), the relevant published work — so we
**adopt rather than reinvent**:

- **Coverage metric:** **k-path coverage** (Havrikov & Zeller, *Systematically Covering
  Input Structure*, ASE 2019) + its tool **Tribble** (github.com/havrikov/tribble). PGEN's
  `replay_target_count` is essentially k=1/k=2 path coverage; the residual is k-path debt.
- **Shortest-derivation:** **Purdom 1972** — already used (`.7.4.2` min-terminal-length
  table, `.7.4.4` Purdom ordering). See [[sv-witness-purdom-ordering]].
- **The `.7.4.6` "reach a specific target" lever is a NAMED technique:** **Directed
  Grammar-Based Test Generation** (Kirschner & Soremekun, **arXiv 2508.01472, 2025**, FDLOOP)
  — goal-specific inputs via probabilistic-grammar learning + evolutionary feedback; beat 5
  baselines on 86% of settings. Our "derivation-directed construction" should read FDLOOP
  first and likely adopt/adapt it.
- **Uniform deep generation:** **Boltzmann samplers** (Duchon-Flajolet-Louchard-Schaeffer
  2004; USAIN BOLTZ) — uniform random generation of tree structures in linear time, incl.
  multi-dimensional variants targeting per-symbol frequency. Complements Purdom for the deep
  tail.
- **Reduction/minimization:** EMI-style pruning; bonsai fuzzing (iterative deepening).

Owned by `STIMULI-SIGNOFF` + `SV-EXH-PROOF.7.4` (D is the engine that proves Pillars A/C —
[[parser-signoff-four-pillars]]). `.7.4.6` = derivation-directed construction.
