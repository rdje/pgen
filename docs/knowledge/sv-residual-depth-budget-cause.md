---
id: sv-residual-depth-budget-cause
title: Why the SV stimuli residual doesn't reach literal zero — depth-budget exhaustion
answers:
  - "why doesn't the SystemVerilog stimuli residual reach zero"
  - "why does replay_target_count plateau (e.g. at 888)"
  - "what is the root cause of the uncovered SV coverage branches"
  - "what does the Stimuli generation depth exceeded message mean"
  - "why can't deeply-nested SV rules be generated from the top entry"
  - "why does the witness pass die on depth when a shallower derivation exists"
  - "what is Purdom min-terminal-length ordering and why can it pick a too-deep derivation"
tags: [stimuli, systemverilog, coverage, root-cause]
date: 2026-08-01
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (depth gate :9491, rule body descends at :9740, rule_reference descends at :11347, witness-pass flat `original * 2` budget :5477, Purdom ordering + construct_mode's `truncate(1)` :10054-10089, min_terminal_length_of_node :7639, min_full_derivation_depth_of_node :7714, the per-target budget that consumes it :4973-4985); docs/tasks/SV-EXH-PROOF.md leaves .7.4.3a (PGEN-SV-EXH-PROOF-0140) and .7.4.6.9 (WHY_WHERE_CLASS_A_DEPTH_2026-08-01)
reverify: "grep -n 'depth > self.config.max_depth\\|saturating_mul(2)\\|ordered.truncate(1)' rust/src/ast_pipeline/stimuli_generator.rs; python3 docs/tasks/artifacts/sv_exh_proof/class_a_residual_depth_population.py"
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

## 2026-08-01 — the mechanism is now EXACT, and it is not "the target is too deep"

`SV-EXH-PROOF.7.4.6.9` measured the whole surviving residual (79 branches, both LRM profiles) against
the engine's own recursion accounting. **Every one is a BUDGET failure, and not one is structural:**

| metric | what it is | value on the 79 |
|---|---|---:|
| witness budget | the gate's `--max-depth 20`, doubled by `generate_target_witnesses` (:5477) | **40** |
| `purdom_depth` | the derivation `construct_mode` actually commits to | **48 – 54** |
| `min_depth` | the shallowest derivation that exists | **18 – 36** |

⭐ **The budget sits BETWEEN them.** The witness pass orders each `Or` by minimum **terminal length**
(Purdom SHORT, :10054-10079) and `construct_mode` then `truncate(1)`s the attempt order (:10087) — it
commits to that one alternative with no fallback inside the committed path. Minimum terminal *length*
is not minimum *depth*: the shortest-in-tokens derivation of an SV `property_expr` descends the
property-operator ladder and then the expression precedence cascade, 8–14 levels past a budget that a
shallower derivation would have fitted inside. The forced branch dies `depth exceeded max_depth=40`,
`generate_or`'s sibling fallback rescues the enclosing rule, and the target is silently left
uncredited — see [[branch-failure-reasons-are-the-witness-why]] for why every pass-level counter
still reads 0.

⛔ **The 2026-06-03 fix direction above stands and is now quantified.** A global `max_depth` raise is
still the wrong lever (it reshapes the diverse pass and inflates every sample); the right one is a
**per-target** budget — and it already exists in this file. `min_full_derivation_depth_of_node`
(:7714, `RTL-FE-CLOSURE.5.2`) computes exactly the missing number, and the cert-coverage
plannable-rule witness pass consumes it at :4973-4985 as `reach_prefix + min_subtree[target]`. The
STIMULI closed-loop witness pass never got it and still multiplies by a flat 2. Two witness passes,
one capability — the same asymmetry class `.7.4.6.11` closed one level down.

Re-derive the numbers at any time, with ground-truth controls, from artifacts an ordinary gate run
already leaves on disk:

```bash
python3 docs/tasks/artifacts/sv_exh_proof/class_a_residual_depth_population.py
```
