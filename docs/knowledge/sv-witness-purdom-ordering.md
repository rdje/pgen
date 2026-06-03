---
id: sv-witness-purdom-ordering
title: The SV residual tail is slow witness generation — Purdom ordering (witness-mode) fixes it
answers:
  - "why do SV witnesses time out instead of erroring"
  - "what is witness_mode / the Purdom witness ordering"
  - "how were the slow witness timeouts reduced"
  - "what does PGEN_WITNESS_NO_PURDOM do"
  - "how to make slow deeply-factored SV witnesses converge"
tags: [stimuli, systemverilog, coverage, witness, purdom]
date: 2026-06-03
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (generate_or witness arm + generate_target_witnesses witness_mode/witness_min_terminal_lengths); docs/tasks/SV-EXH-PROOF.md leaf .7.4.4 (PGEN-SV-EXH-PROOF-0143)
reverify: PGEN_WITNESS_NO_PURDOM=1 vs unset, rerun the witness pass (--target-report-input <sample> --target-max-attempts 0 --target-generation-timeout-ms 7000 --seed 712001) and compare resolved / target_timeout
---

After `.7.4.4.1` proved the witness-pass residual tail is **slow generation (timeouts),
not context-gating and not errors** (`other=0`), `.7.4.4` made the slow witnesses converge
with **Purdom 1972 shortest-derivation ordering**, scoped to the witness pass:

- A generator flag `witness_mode` is true ONLY inside `generate_target_witnesses`. When set,
  the shared `generate_or` core orders the surviving alternatives by **ascending
  min-terminal-length** (the `.7.4.2` `compute_min_terminal_lengths` table), so the
  shortest-terminating branch is tried first. It is a pure attempt-ORDER change — other
  branches stay as fallbacks, so correctness is unaffected. Reach-plan branch forcing still
  wins on-path (branch targets unaffected).
- **Monotone by construction:** the diverse background pass never sets `witness_mode`, so its
  output is byte-identical → `replay_target_count` can only shrink. See
  [[stimuli-residual-coverage-model]] and [[sv-residual-depth-budget-cause]].
- **Same-binary A/B** (seed 712001, 150-target real-SV sample, 7 s cap): Purdom OFF =
  resolved 101/150, target_timeout 39; Purdom ON = resolved **124/150**, target_timeout
  **23** (+23 resolved, −16 timeouts; `other=0` both). `PGEN_WITNESS_NO_PURDOM=1` disables
  the ordering for this A/B.

Caveat: the ordering is deterministic, but the resolved *count* under a wall-clock cap has
small timing-induced wobble at the boundary; the +23 delta dwarfs it.
