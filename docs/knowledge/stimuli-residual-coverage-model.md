---
id: stimuli-residual-coverage-model
title: What the stimuli residual / replay_target_count actually is (coverage model)
answers:
  - "what is replay_target_count / the stimuli residual"
  - "what does the stimuli coverage gap report measure"
  - "what are never_hit / never_selected / selected_but_failed reasons"
  - "what is focused_replay_target_debt_zero / literal-0"
  - "how does the generator steer to a specific coverage target"
tags: [stimuli, coverage, model]
date: 2026-06-03
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (StimuliCoverageTarget {Rule,Branch}, generate_gap_report, compute_reach_path, forced_or_branch_for_site, set_reach_plan); docs/tasks/SV-EXH-PROOF.md
reverify: grep -n "StimuliCoverageTarget\|replay_target\|reach_classification" rust/src/ast_pipeline/stimuli_generator.rs
---

The **residual** is the count of coverage **targets** the closed loop has not yet covered.
A target is a `StimuliCoverageTarget`, either a whole **Rule** or a specific **Branch** of
an `Or`. The gap report (`generate_gap_report`) partitions each open target by reason:

- `never_hit` — the rule/branch was never reached during generation.
- `never_selected` — reachable, but the weighted choice never picked it (often a
  reachability gate firing first — see [[sv-residual-depth-budget-cause]]).
- `selected_but_failed` — chosen but generation failed (depth/visit/timeout/other — see
  [[stimuli-generation-error-reasons]]).

Each target also carries a read-only `reach_classification`
(`reachable_by_plan` / `no_reach_path` / `reachable_rule_not_generated`) so the bare count
becomes evidence: per target, *why* it's open and whether it's coverable by steering.

The closed loop drives coverage with **reach plans**: `compute_reach_path` finds a path to
a target, `forced_or_branch_for_site` forces the owning `Or` branch, `set_reach_plan`
installs it for the next attempt. The signoff bar is
**`focused_replay_target_debt_zero`** ("literal-0"): the reachable, bounded-finite residual
driven to 0. The residual is **reachable-only** (profile-pruned branches are reported as
`unreachable_branch_debt`, not actionable work). Determinism matters: only with a fixed
seed are residual deltas signal — see [[feedback_no_codebase_change_without_tool_backed_facts]].
