---
id: parse-fidelity-oracles
title: Don't reinvent the no-mis-parse oracle — invertible syntax, metamorphic/EMI, round-trip
answers:
  - "how do we detect mis-parse (accepted but wrong AST)"
  - "what is the test oracle for parser fidelity / correct AST"
  - "what is invertible syntax / bidirectional parsing pretty-printing"
  - "how to test parser correctness without a reference AST"
tags: [parser, fidelity, mis-parse, oracle, literature, dont-reinvent]
date: 2026-06-03
status: current
evidence: "Rendel & Ostermann, Invertible Syntax Descriptions, Haskell 2010 (invertible-syntax/partial-isomorphisms); Metamorphic Testing (Chen/Cheung/Yiu 1998); EMI (Le/Afshari/Su PLDI 2014); PGEN round_trip_tests.rs + ast_shape_contract manifests"
reverify: see docs/tasks/PARSE-FIDELITY.md literature grounding
---

For Pillar C (no **mis-parse** = accepted-but-wrong-AST, which has no error signal), the
literature provides oracles — adopt them:

- **Invertible syntax descriptions** (Rendel & Ostermann, Haskell 2010): derive parser AND
  printer from ONE description, so round-trip is correct **by construction** — a whole class
  of mis-parses becomes impossible. PGEN's return-annotations already lean bidirectional;
  this is the principled end-state (and the home of the A5 `_meta` round-trip).
- **Reference-free oracles**: **metamorphic testing** (Chen/Cheung/Yiu 1998) + **EMI**
  (PLDI 2014) — a semantically-preserving source transform must yield an equivalent AST.
  Catches mis-parses a reference parser can't (cross-tool ASTs differ).
- **Round-trip / unparse equivalence** (generate → unparse → reparse → identical AST) —
  PGEN HAS `test_runner/round_trip_tests.rs` + per-grammar round-trip gates + AST-shape
  contracts; the A5 `_meta` carrier ([[ast-two-surface-construction]]) unlocks per-node
  `parse(node._meta.source_text) ≡ node`.
- **Differential on a normalized projection**: compare *semantic facts* vs slang's
  elaboration (is-X-a-type/net/param), since full cross-tool AST diff is impractical.

KEY: (1.c) is exhaustively proven exactly when Pillar D (stimuli) reaches literal-0 and
every witness round-trips. Owned by `PARSE-FIDELITY`; see [[parser-signoff-four-pillars]].
