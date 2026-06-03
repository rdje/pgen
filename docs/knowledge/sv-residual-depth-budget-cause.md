---
id: sv-residual-depth-budget-cause
title: Why the SV stimuli residual doesn't reach literal zero — depth-budget exhaustion
answers:
  - "why doesn't the SystemVerilog stimuli residual reach zero"
  - "why does replay_target_count plateau (e.g. at 888)"
  - "what is the root cause of the uncovered SV coverage branches"
  - "what does the Stimuli generation depth exceeded message mean"
  - "why can't deeply-nested SV rules be generated from the top entry"
tags: [stimuli, systemverilog, coverage, root-cause]
date: 2026-06-03
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (depth gate ~:4377, near-limit guard ~:4509/:5413, depth-slack retry ~:4869); docs/tasks/SV-EXH-PROOF.md leaf .7.4.3a (PGEN-SV-EXH-PROOF-0140)
reverify: grep -n "max_depth\|depth exceeded\|DEPTH_EXCEEDED" rust/src/ast_pipeline/stimuli_generator.rs
---

The SV closed-loop generator's residual (`replay_target_count`, best observed **888**
before the witness pass) does not fall to literal **0** primarily because of
**depth-budget exhaustion**, not because of weak weighting.

- The generator recurses with a default `max_depth` of **24** rule-calls. The SV grammar
  is deeply *factored*, so reaching **and** completing a deep rule from the top entry can
  need more budget than that — e.g. `ansi_port_declaration` sits at depth ≥10 below its
  entry. The depth floor fires *before* the weighted choice, so uncovered branches the
  generator already boosts (×24/×2) are never even entered.
- It is therefore a **reachability** problem (a structural gate fires first), not a
  weighting one. See [[project_sv7_never_selected_rootcause]].

**Fix direction (validated, `.7.4.3`):** per-target *minimal witnesses* generated with
fresh/adequate depth (root the generation at/near the target rule, force its branch via a
reach plan, restore depth/visit slack) — **not** a global `max_depth` raise (that blows up
`property_expr` and reshapes the diverse pass). The witness pass is monotone-additive and
resolved real-SV 72/150 sampled targets with **0** depth/visit failures (depth fix
validated).

**Caveat learned 2026-06-03:** of 24 residual rules generated standalone at `--max-depth
48`, 16 succeed and 8 fail with **no diagnostic emitted at all** (the per-target timeout
does not bound the plain `--entry-rule` path). Those 8 are the `.7.4.4` context-gating tail
— a *separate* cause from depth. Do not conflate the two.
