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

**Canonical gate confirmation** (`make sv_stimuli_quality_gate`, `PGEN-SV-EXH-PROOF-0144`):
the witness pass runs in-gate and cut the closed-loop residual **2770 → 753 (−73%)**
(2017: 1736→2291/2590; 2023: 775→2238/2691); `closed_loop_replay_targets_total=753`, of
which **~99% is `target_timeout` (746)** — ~zero genuine errors. The residual tail's root
cause is the gate's tiny per-witness budget **`closed_loop_target_generation_timeout_ms=5`**
(5 ms) — witnesses for deeply-factored rules can't finish in 5 ms even Purdom-ordered.

**`.7.4.5` (PGEN-SV-EXH-PROOF-0145):** gave the witness pass its OWN budget decoupled from
that 5 ms primary — `witness_generation_timeout() = max(primary, WITNESS_TIMEOUT_FLOOR_MS=200)`
(env `PGEN_WITNESS_TIMEOUT_FLOOR_MS` overrides). Canonical gate **753 → 273 (−64%)**, still
PASS, ~25.8 min. Budget is a real but *diminishing* lever (150-sample curve: 200 ms→77,
500→72, 1000→79, 2000→89 resolved of 150, wall-clock 42/82/146 s), so 200 ms is the
conservative routine default; literal-0 pushes use the env.

**`.7.4.6` RE-SCOPED by profiling (2026-06-04 — the "derivation-construction" assumption was
WRONG):** a macOS `sample` profile of slow witness generation shows the cost is NOT
search/backtracking (so no construction engine is needed) — it is **redundant recomputation**:
`node_is_nullable` (~26K self-time; recomputed recursively per `generate_sequence` call, no
cache, recurses through referenced rules) + regex compile (~6K; terminal patterns recompiled
per call). Surgical CACHING landed (monotone, no regen): **`.7.4.6.1` regex compile cache (DONE)** +
**`.7.4.6.2` nullability cache (DONE — sound cache-only-cycle-free, ~24%)**.
**⚠️ BUT caching is INSUFFICIENT for literal-0 (honest correction, 2026-06-04):** combined
~25-40% speedup, but witnesses are still ~9 s each — ~45× the gate's 200 ms budget — so
caching does NOT bridge seconds→budget and does NOT meaningfully reduce the residual (273).
The witnesses are fundamentally slow from the **search** itself (~75% of self-time is the
generate_node/rule/sequence recursion + backtracking, which per-step caching can't remove).
So the **original derivation-CONSTRUCTION idea IS the real lever** (`.7.4.6.3`): build the
minimal derivation tree directly (Purdom shortest-terminating, backtrack-free, O(tree-size),
no timeout). Lesson refined: profile pinned the *per-step* costs (caching, real but partial);
measuring against the GLOBAL goal (literal-0) showed the *step-count/search* is the dominant
barrier → construction. Caching is kept (free speedup for all generation + gate runtime).

**`.7.4.6.3` derivation-directed CONSTRUCTION (DONE, PGEN-SV-EXH-PROOF-0148):** a
`construct_mode` flag — a SUB-mode of `witness_mode`, so it is NEVER active in the diverse pass
(monotone). When set, `generate_or`'s witness arm commits to the SINGLE shortest-terminating
alternative (`ordered.truncate(1)` after the Purdom min-length sort — no backtrack) and
`generate_quantified` emits the minimum legal repeat count. `generate_target_witnesses` tries
construction FIRST (bounded, O(tree-size), no search) and falls back to the `.7.4.5` search on
failure. It reuses ALL existing validity machinery (spacing/terminals/round-trip guards) — only
the production CHOICE is forced. **Witness-pass proof** (150-sample, 7000 ms, seed 712001):
resolved **146/150** (was 124 search / 81 original; +22), **target_timeout=1** (was 23–55 —
near-eliminated), 31 s — construction makes deeply-factored witnesses BOUNDED + fast, the
seconds→budget bridge caching could not give. This is the confirmation that **construction, not
caching, is the literal-0 lever.** lib 588/588; clippy 0; monotone (witness-pass-only, search
fallback). **Canonical gate (`PGEN-SV-EXH-PROOF-0149`, exit 0, ~22 min): closed-loop residual
273 → 97 (−64%)** — full arc ~2660 → 888 (.7.2) → 753 (Purdom) → 273 (budget) → **97
(construction)**; realistic corpus 730/730, parse_full_failures 0. **NOT literal-0:** the 97 is
dominated by ~66 `target_timeout` (replay 2017+2023 = 33+33) — targets where the single
shortest-branch commit DEAD-ENDS and falls back to the .7.4.5 search, which times out at the
200 ms floor. The literal-0 follow-up (`.7.4.6.4`) is bounded LIMITED-BACKTRACK construction:
on a dead-end, try the next-shortest sibling before full search. SV main parser stays
**Mostly Done** (the `focused_replay_target_debt_zero` criterion is unmet at 97 — honest).
