# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> **Layer A per `MEMORY_ARCHITECTURE.md` §6: where we are NOW, and nothing else.**
> ⛔ OVERWRITE the "Current state" block each update — never append. History is git (layer D);
> per-unit work is `docs/tasks/` (B); durable facts and standing directives are
> `docs/decisions/` (C). If an entry belongs to a session rather than to *now*, it belongs in B/C.
> ⛔ Bounded by a **line cap AND a byte cap** (`scripts/check_memory_architecture.sh`, doctrine
> `MEMORY-ARCH`). ⭐ **A cap is never raised to land content** — demote the content instead. The
> line-only cap held 60 lines and 138,403 bytes at once; both caps exist because of that
> (`README-POLICY.2`).

## How to resume
- Read `MEMORY_ARCHITECTURE.md` (memory/continuity), `README.md` (project), and `TOOLBOX.md` (debug toolbox — USE FIRST).
- Work is tracked in task-trees under `docs/tasks/`; index `docs/TASK_TREE.md`; commit per `COMMIT.md`.
- Durable facts / standing directives / decisions: `docs/decisions/INDEX.md` (137 records — **the authoritative list**). Live status: `docs/book/src/roadmap-and-live-status.md`; changelog: `CHANGES.md`.
- ⛔ A code change MUST pass the **acceptance checklist** (ROOT CAUSE + ADDRESSED + NO REGRESSION, evidence-backed) in its task leaf — enforced by `scripts/check_doctrines.sh` via `.githooks/pre-commit` (run `git config core.hooksPath .githooks` once per clone).

## North star (pointers only — the layer-C record is authoritative, this is not a summary)
- **The goal**: parse any precisely-described language with the right EBNF; the bar is *eagerness*, not sufficiency — expressive awkwardness is a defect class → [[project_horizon_universal_parser]].
- **The two non-negotiable constraints on reaching it**: parser-neutrality and theoretical peak speed. Capability that costs either is REJECTED, not traded; the mechanism is COMPILE AWAY (non-users pay ZERO) → [[project_capability_growth_is_zero_cost_and_neutral]].
- **`Done` is first-tier only** — three legs, all measured, never a snapshot; `Provisional (ceiling)` / `Provisional (corpus pending)` is the shipping tier → [[feedback_done_bar_is_first_tier_only]].
- **Accuracy is the immovable floor; speed is maximized on top** and monitored for every generated parser → [[feedback_correctness_before_speed]]; regex closure bar → [[project_rgx_0078_sub_1us_closure_target]].
- **SV is 100% LRM-compliant by default** — over-acceptance is a defect, tolerance is additive and future → [[feedback_sv_strict_lrm_compliance_default]].
- **The EBNF is the sole source of truth** and steers the engine at the author's granularity → [[project_ebnf_is_single_source_of_truth]], [[project_ebnf_steers_the_engine_at_full_granularity]].
- **Prove no existing surface covers it before designing one**, and RE-MEASURE engine behaviour rather than quoting a doc → [[feedback_read_prior_art_before_designing]] (mechanized: `DESIGN-PRIOR-ART`).
- **Flow findings are ROUTED to a leaf by default; worked only when they block** (a gate cannot run / a verdict cannot be trusted / a published claim is false) → [[feedback_flow_findings_are_routed_not_worked]].
- **An instrument with no ground truth is a confident guess** — pin both a positive and a negative control inside it and REFUSE on a miss → [[feedback_instrument_needs_ground_truth]].
- **Project data stays on the repo's volume** → [[project_data_locality_same_volume]]; reclaim artifacts routinely, proving safety each time → [[feedback_delete_reclaimable_artifacts_regularly]]; the portable spine is a separate repo → [[project_bedrock_spine_repo]].

## Current state (OVERWRITE this block each update — do not append)
- **latest_commit**: `PGEN-LIVE-MEANS-LIVE-0006` (leaf `LIVE-MEANS-LIVE.1c2`) — ✅ **every referrer that DEPENDS on the tracker is retired**: 4 live `assert_file_contains` → 0, 33 live `.md` → 3 (all merely *narrating* the deletion), 2 content-reading scripts → 0. ⭐ `check_readme_stability.sh`'s overflow now routes to the **schema-bounded register** — that redirect into an uncapped prose file was the ACTUAL root cause of the rot. `.1c1` before it put the 10-family snapshot in the book under a gate.
- **active_work_unit**: `LIVE-MEANS-LIVE` (DIRECT DIRECTOR ORDER — *"Live means live, fullstop"* → *"go for migrate-and-delete"*). ✅ `.0`, `.1a`, `.1b`, `.1c1`, `.1c2`, `.3`. ⛔ OPEN: `.1c3` (the DELETE), `.2` (REOPENED — ANVIL's 2 instruments, not a byte cap), `.4` (10 files self-refute their `Last updated:`).
- **next_action**: **`.1c3`** — delete `LIVE_ACHIEVEMENT_STATUS.md`, **atomically** with the 2 remaining path-only referents (`ci_workflow_local_gate.sh`'s root-md roster is an EXACT-SET check; `check_diagnostics_and_docpaths.sh`'s pathspec), then re-run the 452-ID census + the doctrine enforcer.
- ⛔ **`ci_workflow_local_gate` CANNOT COMPLETE** (pre-existing, routed → `CI-PARITY-GATE-ROT.20`): its `rg` markdown audit descends into the **`anvil` submodule** while the doctrine's own `git grep` enforcer does not — *one doctrine, two enforcers, opposite verdicts*; plus 2 README literals staled by `README-POLICY.1`. Substitute measured: 89 assertion arms, **87 pass, 0 regressions**.
- ⚠️ **Never read a pipeline's exit code from a trailing `echo`** — it reported success while the log said `Error 1`.
- ⭐ **452/452** slice IDs cited by the tracker are reachable WITHOUT it ⇒ the delete is lossless.
- **in_flight_uncommitted**: none.
- ⚠️ **two sweep traps banked in `LANG-CAPABILITY-AUDIT.10.3`** (read before any before→after sweep): the embedded `-o` path, and `focus_*` silently changing `ast_pipeline`'s feature set at the same path → also `CI-PARITY-GATE-ROT.19`.
- **blockers**: none. ⏳ Two escalated **director calls** open: a priced `schedule:` lane for the 4h39m `sota_exit_gate`; and the ANVIL README-policy feedback (measured in `LIVE-MEANS-LIVE`: PGEN clean on 4 of 6, narrower gap on point 2, plus a fifth lesson to send back).
- **routed, waiting behind product** (do NOT pull forward): `LANG-CAPABILITY-AUDIT.10.12`/`.10.8`/`.10.13`/`.10.14`/`.3c`, `CI-PARITY-GATE-ROT.10`/`.11` remainder/`.16`/`.17`/`.18`, `OPS-MEMSAFE.4`, `DOCTRINE-GAP-OWNERSHIP.3`, `DONE-BAR.5d` gate/`.5f`.3/`.5g`, `README-POLICY.3`/`.6`.
- **push**: 121 commits ahead of `origin/main` (as of this commit). ⛔ Cadence is **300 unpushed OR an explicit exceptional order** — never ask or suggest below 300 → [[feedback_push_pacing]].
- **live doctrine anchor** (kept here deliberately — `REGEX-ORACLE-ANCHOR-SYNC` checks every occurrence in this file against the contract): regex oracle real tuple **`2189/1879/262/48`**.
- **flow health**: `sota_exit_gate`'s last end-to-end green was `CI-PARITY-GATE-ROT.7` (32/32 stages, 4h39m); `.10.6` broke a required stage immediately after and `.10.9` repaired it, so the aggregate is UNBLOCKED but has not been re-proven end-to-end in this configuration. Automatic CI tier = the 15 doctrines on every push + 3 cheap gate targets; the other 120 `make` gate targets remain operator-invoked.
