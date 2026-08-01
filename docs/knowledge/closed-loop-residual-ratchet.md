---
id: closed-loop-residual-ratchet
title: The SV closed-loop residual is TWO-SIDED ratcheted — improving coverage FAILS the gate until you lower the pin
answers:
  - "the SV stimuli gate failed saying the residual IMPROVED — is that a bug?"
  - "why does sv_stimuli_quality_gate fail after I improved coverage"
  - "where do I lower the closed-loop residual ceiling"
  - "what is closed_loop.replay_target_ceilings"
  - "why did the residual ratchet skip / say status skip"
  - "why does the SV gate refuse with exit 2 after I edited the contract"
  - "how do I see which coverage targets are still residual without running the gate"
tags: [stimuli, coverage, gates, systemverilog, ratchet]
date: 2026-08-01
status: current
evidence: rust/scripts/sv_stimuli_quality_gate.sh (replay_target_ceiling_verdict + replay_target_ceiling_self_check; the ratchet fires where the per-profile count is computed, and adjudicates after the profile loop); rust/test_data/grammar_quality/systemverilog_core_v0_contract.json (closed_loop.replay_target_ceilings); docs/tasks/SV-EXH-PROOF.md leaf .7.4.6.10; docs/tasks/artifacts/sv_exh_proof/run_replay_target_ratchet_probes.sh
reverify: jq '.closed_loop.replay_target_ceilings' rust/test_data/grammar_quality/systemverilog_core_v0_contract.json; grep -n 'replay_target_ceiling_verdict()\|replay_target_ceiling_self_check()\|residual RATCHET' rust/scripts/sv_stimuli_quality_gate.sh | head
---

`closed_loop_replay_targets_total` — the coverage targets the replay stage still cannot witness — is
**pinned per LRM profile** in the contract and enforced **two-sided**:

| residual vs. pin | outcome |
|---|---|
| above the pin | **FAIL** — regression |
| **below** the pin | **FAIL** — *"lower the ceiling"* |
| equal to the pin | pass (the only quiet outcome) |
| profile not pinned | **FAIL** — a blind spot is not a pass |

⭐ **A COVERAGE WIN FAILS THE GATE. THAT IS THE DESIGN, NOT A REGRESSION.** When a leaf closes
coverage, the gate stops and tells you to lower the pin in
`closed_loop.replay_target_ceilings.profiles` (and `measured_configuration` if the run configuration
moved). A ceiling that can only be met and never tightened lets a hard-won improvement evaporate on
the next change with nobody told — the same silence the ratchet exists to end.

⛔ **A ceiling is LOWERED as a coverage leaf lands. It is NEVER RAISED to land a change.**

**Why it existed to be built.** The residual was *echoed* into `summary.txt` and never compared, so
the gate passed at any value; it drifted **84 → 127 over ~7 weeks with every gate green**. A number
that is reported but never compared is not a gate.

### The two skip/refuse paths are different on purpose

- **`status: skip`** — an *environment* override moved the run off the pinned configuration
  (`PGEN_SV_STIMULI_QUALITY_COUNT`, `..._SEED_BASE`, `..._LRM_PROFILES`, …). The pinned residual
  genuinely does not apply. This is what keeps `sv_parse_full_ratio_promotion_gate` and
  `sv_declared_shadow_promotion_gate` working — they drive this gate at their own counts and seeds and
  record its exit code as *data*.
- **`exit 2`, in about a second, before any generation** — the *contract's own* closed-loop
  configuration no longer matches `measured_configuration`. Skipping there would let a one-line
  contract edit disarm the ratchet, which is the original failure one level up. Re-measure and re-pin.

The **grammar** is deliberately outside that configuration: a grammar change moving the residual is
exactly what the ratchet is for.

### The residual manifest — read the targets without running the gate

Each run writes `<state_dir>/closed_loop_replay_targets.json`: per profile, the count, the pinned
ceiling, the verdict, and the full target **list** (`id`, `reason`, `rule_name`, `node_path`,
`branch_index`, `depends_on`). Saving it before a change makes the next delta a list diff instead of
an inference — and it is the cheap way to partition a residual by
[[coverage-gap-reason-codes-are-generator-verdicts]] without a ~31-minute run.

### It proves itself on every run

Eight pinned controls — the positive control and all three negatives — drive the real comparison at
gate start and **refuse (exit 2) on a miss**, before any expensive work. End-to-end ground truth,
driving the real gate with only the pinned number planted, is re-runnable:

```bash
scripts/run_with_memory_guard.sh --budget-mb 16384 --timeout-s 7200 -- \
  bash docs/tasks/artifacts/sv_exh_proof/run_replay_target_ratchet_probes.sh
```

An instrument with no ground truth is a confident guess ([[feedback_instrument_needs_ground_truth]]),
and this one guards a ~31-minute measurement.
