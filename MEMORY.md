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
- **latest_commit**: `PGEN-SV-EXH-PROOF-0162` (leaf `.7.4.6.12`, docs) — class C's WHY+WHERE pinned; ⛔ that leaf's OWN Goal hypothesis is **DISPROVEN**. Prior: `-0161` (`.7.4.6.11`) — ⭐ **CLASS B ELIMINATED 43 → 0**, residual **127 → 83** at an UNCHANGED universe (5461).
- **active_work_unit**: **`SV-EXH-PROOF.7`** — ✅ **DIRECTOR-CHOSEN LANE, 2026-07-31** (*"-> SV-EXH-PROOF.7"*), after pausing the live-docs axis (*"we are spending way too much time on these live-docs size, we need to pause and switch back to feature related work"*) → [[feedback_prefer_feature_work_over_governance_lanes]]. ⛔ `LIVE-MEANS-LIVE` stays `active` with `.6` `.7` `.8` open but **PARKED, not abandoned** — do NOT pull them forward.
- **next_action**: **`.7.4.6.10` (the RATCHET)** — 44 targets were just won and NOTHING banks them; the 84→127 drift is exactly what an unratcheted residual does. Then `.7.4.6.9` (class A = 79, the big one, ⛔ open with a Protocol D trace). ⏳ `.7.4.6.12` (class C, 4) is **BLOCKED ON A DIRECTOR CALL** — its fix is a witness-ENTRY-policy change, not wiring; cost must be sized. SV stimuli gate ~29 min.
- ⭐ **residual = 83 and DETERMINISTIC** (was 127). Classes now: **A** `property_expr` cone **79** (ONE defect, `.7.4.6.9`) · **B** **0 ✅** · **C** store-gated **4** (`.7.4.6.12`). ⛔ Still **NO RATCHET** on the residual (`.7.4.6.10`) — nothing banks this win.
- ⭐ **Enumeration is the defect; refusal is the fix** → [[feedback_enumerating_instrument_must_refuse]]. *(The `LIVE-MEANS-LIVE` lane's own findings — `.4b`'s generate-FACTS-not-CLAIMS rule, `.6`'s 64 450-byte `docs/TASK_TREE.md` cell — are demoted to that tree; the lane is PARKED, so they are history, not `now`.)*
- ⛔ **`ci_workflow_local_gate` CANNOT COMPLETE** (pre-existing; full diagnosis in `CI-PARITY-GATE-ROT.20`).
- **in_flight_uncommitted**: none.
- ⚠️ **sweep traps**: `LANG-CAPABILITY-AUDIT.10.3` + `CI-PARITY-GATE-ROT.19` (generated-artifact sweeps); `.4b` item 2 (markdown sweeps).
- **blockers**: none. ⏳ Three **director calls** open: **`.7.4.6.12`'s witness-ENTRY-policy change** (raise the entry for store-gated targets — re-prices the witness budget, 4 targets); a priced `schedule:` lane for the 4h39m `sota_exit_gate`; the ANVIL README-policy feedback.
- **routed, waiting behind product** (do NOT pull forward): `LANG-CAPABILITY-AUDIT.10.12`/`.10.8`/`.10.13`/`.10.14`/`.3c`, `CI-PARITY-GATE-ROT.10`/`.11` remainder/`.16`/`.17`/`.18`, `OPS-MEMSAFE.4`, `DOCTRINE-GAP-OWNERSHIP.3`, `DONE-BAR.5d` gate/`.5f`.3/`.5g`, `README-POLICY.3`/`.6`.
- **push**: 140 commits ahead of `origin/main` (as of this commit). ⛔ Cadence is **300 unpushed OR an explicit exceptional order** — never ask or suggest below 300 → [[feedback_push_pacing]].
- **live doctrine anchor** (kept here deliberately — `REGEX-ORACLE-ANCHOR-SYNC` checks every occurrence in this file against the contract): regex oracle real tuple **`2189/1879/262/48`**.
- **flow health**: `sota_exit_gate` last green at `CI-PARITY-GATE-ROT.7` (32/32, 4h39m); UNBLOCKED since `.10.9` but not re-proven end-to-end. Automatic CI tier = the **16** doctrines per push + 3 cheap gate targets; the other 120 `make` targets are operator-invoked.
