---
id: stimuli-generator-capability-gaps
title: Stimuli generator — what it has vs the literature signoff bar (6 gaps)
answers:
  - "does the stimuli generator have all the necessary features"
  - "what is missing from the stimuli generator vs the literature"
  - "what are the stimuli generator capability gaps"
  - "is PGEN's generator behind or ahead of academic grammar fuzzers"
  - "what should a signoff-grade EBNF stimuli generator do that ours doesn't"
tags: [stimuli, generation, capability-gap, literature, signoff]
date: 2026-06-03
status: current
evidence: "grep stimuli_generator.rs (reach_plan, StimuliCoverageTarget, min_terminal/purdom, {mutation,constraint,negative,recovery}_mode, shrink); literature sweep 2026-06-03; docs/tasks/STIMULI-SIGNOFF.md (.1 audit)"
reverify: see docs/tasks/STIMULI-SIGNOFF.md leaves .2-.7
---

Capability-gap audit (`STIMULI-SIGNOFF.1`, from the 2026-06-03 literature sweep + the
code-verified feature surface).

**HAS (code-verified)** — and AHEAD of typical academic fuzzers on two axes:
- Purdom shortest-derivation; rule+branch coverage targets + gap report + reach_classification;
  directed reach plans (SEARCH-based); constraint/negative/recovery profiles; delimiter-aware
  shrinking.
- ⭐ **Closed-loop round-trip self-consistency** (`parser_rejections == 0`) — most generators
  never reparse their output.
- ⭐ **Semantic-store-aware (data-dependent) generation** → context-VALID inputs
  (declare-before-use), not merely syntactically valid.

**SIX GAPS vs the signoff bar** (→ leaves `STIMULI-SIGNOFF.2`–`.7`):
1. **k-path coverage (k>2)** — Havrikov & Zeller, ASE 2019 (Tribble). Ours is rule+branch
   (≈k=1/2). The principled "covered the input STRUCTURE" metric; **defines** exhaustive
   coverage. **TOP gap** (`.2`).
2. **Learned/probabilistic directed generation** — FDLOOP, arXiv 2508.01472 (2025). Ours is
   deterministic reach-plan SEARCH (times out on deep targets). **Co-owned with
   `SV-EXH-PROOF.7.4.6`** — the literal-0 reach (`.4`).
3. **Uniform random generation** — Boltzmann samplers (Duchon 2004). Ours is weighted, not
   uniform-by-size; no distribution guarantee. Secondary (`.5`).
4. **Code-coverage feedback** — coverage-guided grammar fuzzing. Ours is grammar-coverage
   ONLY; never closes the loop on the PARSER's code coverage (Havrikov-Zeller: input k-path →
   code coverage). Deepest "did we exercise the parser" signal. Pairs with #1 (`.3`).
5. **Full grammar-tree-aware shrinking** — drop-optional / collapse-alternation / prune-subtree.
   Ours is delimiter-aware only. Secondary (`.6`).
6. **Grammar-mutation maturity metric** — TOSEM 2025. We generate mutations but don't use
   them as a maturity METRIC. Secondary (`.7`).

**Signoff-critical few:** #1 (k-path, the metric that defines "done") + #4 (code-coverage
feedback) are a pair (Havrikov-Zeller); #2 (directed/FDLOOP) is the literal-0 reach.
#3/#5/#6 are quality/secondary. See [[parser-signoff-four-pillars]] (this is Pillar D);
[[grammar-coverage-and-directed-generation]] for the D-side literature.
