<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_no_codebase_change_without_tool_backed_facts.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback-no-codebase-change-without-tool-backed-facts
description: NEVER change the codebase on a hypothesis; every code change must be backed by a fact gathered from the toolbox FIRST — guessing caused a real regression
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 5737d722-67a3-4fc9-8c42-f79e2ff1db07
---

STANDING DISCIPLINE (user 2026-06-01, emphatic, repeated 4×, after I caused a real regression): **Do NOT make wild guesses and then change the codebase based on them. Every claim AND every code change must be backed by a FACT that was gathered using the toolbox FIRST.** Wild-guess-driven changes are dangerous — they create regressions.

**The cautionary exemplar (SV-EXH-PROOF.7.2.10/.7.2.11):** I diagnosed the 888-residual as "quantifier gap + head-of-line blocking" from reading code (plausible), wrote two fixes, unit-tested them (each passed its own narrow test), committed, and only THEN measured the real corpus effect — it was a REGRESSION (replay_target_count 888 → 1717, resolved halved). Then I "analyzed" the regression with a narrative ("diversity collapse, deeper samples time out") that was **partly provably false** — I compared `.7.2.12`'s timeouts (1837) against `.7.2.6`'s 784 instead of `.7.2.8`'s 2721, so my "timeouts went up" claim was backwards (they went DOWN). I asserted mechanism without measuring it.

**What I did wrong (the anti-patterns to never repeat):**
1. Changed code on a code-reading hypothesis without first PROVING the hypothesis with a measurement/tool.
2. Unit-tested the mechanism ("does it do X") but never measured the GLOBAL effect ("does X actually help the metric") before committing — unit-correct ≠ globally better.
3. Changed TWO things at once (.7.2.10 + .7.2.11) and measured once → can't attribute cause (no ablation).
4. Never verified the measurement is signal vs noise (is the gate seed deterministic?) before drawing conclusions.
5. Built a regression "analysis" on a misread number instead of pulling the actual figures side-by-side from the artifacts.

**The required loop instead (tools-first, reinforces [[feedback_tools_first_no_guessing]], [[feedback_why_and_where_before_solution]], [[feedback_always_signoff_decisions]]):**
- Before ANY code change: gather the fact that proves the diagnosis using the toolbox (gate metrics, replay_gap.json target diffs, reach_plan_activations + per-outcome tallies, trace, capped replay, ablation). If the toolbox can't show it, BUILD/extend the tool first.
- Change ONE thing at a time; measure each against the real metric before the next.
- Confirm determinism (fixed seed) so deltas are signal. (FACT established 2026-06-01: the SV aggregate-contract closed-loop replay seed IS deterministic — `closed_loop_replay_seed = seed_base(12001 from systemverilog_core_v0_contract.json) + profile_idx*1_000_000 + 700_000`; the `$RANDOM` at sv_stimuli_quality_gate.sh:675 is only a temp filename. So 888 vs 1717 is real signal.)
- Every regression analysis must cite the actual artifact numbers pulled side-by-side, not remembered/narrated ones.

**How to apply:** when tempted to edit grammar/Rust/generated to "fix" something, STOP and ask "what tool output proves this is the cause, and have I run it?" If no → run the tool / build it first. A clean reusable lever ("ablation", "diff the replay_gap target sets", "capped replay") beats a 50-min full-gate guess-and-check.
