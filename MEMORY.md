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
- Durable facts / standing directives / decisions: `docs/decisions/INDEX.md` (142 records — **the authoritative list**). Live status: `docs/book/src/roadmap-and-live-status.md`; changelog: `CHANGES.md`.
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
- **latest_commit**: `PGEN-SV-EXH-PROOF-0177` (`.7.4.6.13`; `(i)` landed at `-0176`, `.7.4.6.14` DONE at `-0175`) — ⭐ the per-branch runaway backstop `4096` is IN: attempts `6.5x`/`7.2x` down, `branch_retry_max 726 836→4 096`, `sv_2023` target-drive `568→1673`, stage `458→267 s`/`569→437 s`, **residual still `0`/`0`**, landed build **byte-identical** to the priced arm. Unblocked by `-0175`'s **BRANCH half** of the raised witness entry (the **prelude** + **branch-scoped gate discovery**; `store_entry_raises 1→15`/`1→14`) → [[witness-entry-policy-store-gated-targets]], [[depth-slack-retry-needs-a-runaway-backstop]].
- **active_work_unit**: `SV-EXH-PROOF` (director-chosen lane → [[feedback_prefer_feature_work_over_governance_lanes]]). ⛔ `LIVE-MEANS-LIVE` stays `active` (`.6` `.7` `.8` open) but **PARKED, not abandoned** — do NOT pull forward.
- **next_action**: ⛔⛔ **`SV-EXH-PROOF.7.4.6.13` defect (ii) PREDICTABILITY — OPEN, and its obvious fix is MEASURED-AND-REFUTED (`-0177`).** Slack-from-configured-depth collapses the ladder as designed (`106→8` levels) and blows the residual `0→17`/`0→10`: the cumulative escalation is **LOAD-BEARING** for ~10 rules + ~7 branches per profile. A "nesting cap" is the SAME knob (`ceiling = configured + 4N`), so no cap is shippable. ⇒ next candidate is a different shape: an **EXPLICIT per-target depth grant**, as `witness_target_depth_budget` (`.7.4.6.9`) already computes for the witness pass. ⛔ Acceptance gate is LEAF-SCOPED.
- ⭐ **residual = 0, DETERMINISTIC, RATCHETED at `2017: 0`/`2023: 0`** (**A ✅ · B ✅ · C ✅**) — two-sided, so moving it EITHER way fails the gate.
- ⛔ **`ci_workflow_local_gate` CANNOT COMPLETE** (pre-existing; full diagnosis in `CI-PARITY-GATE-ROT.20`).
- **in_flight_uncommitted**: none.
- ⚠️ **sweep traps**: `LANG-CAPABILITY-AUDIT.10.3` + `CI-PARITY-GATE-ROT.19` (generated-artifact sweeps); `.4b` item 2 (markdown sweeps).
- **blockers**: none. ⏳ Three **director calls** open (⛔ all OBJECTIVE/SCOPE — never escalate a technical one → [[feedback_answer_your_own_technical_questions]]): `LESSON-RETRIEVAL.2` back-fill scope; a priced `schedule:` lane for `sota_exit_gate`; the ANVIL README-policy feedback.
- **routed, waiting behind product** (do NOT pull forward): `GENERATED-LINT-CORRECTNESS.8` (`NOREGRESS_SIG` repeats group 2's wrong-vocabulary defect, **120/416 (29%)** unbacked; price the REACH, not only the population), `LANG-CAPABILITY-AUDIT.10.12`/`.10.8`/`.10.13`/`.10.14`/`.3c`, `CI-PARITY-GATE-ROT.10`/`.11` remainder/`.16`/`.17`/`.18`, `OPS-MEMSAFE.4`, `DOCTRINE-GAP-OWNERSHIP.3`, `DONE-BAR.5d` gate/`.5f`.3/`.5g`, `README-POLICY.3`/`.6`, `BOOK-PARAGRAPH-SHAPE` (**`.2` runs BEFORE `.1`**).
- **push**: 174 commits ahead of `origin/main`. ⛔ Cadence is **300 unpushed OR an explicit exceptional order** — never ask or suggest below 300 → [[feedback_push_pacing]].
- **live doctrine anchor** (kept here deliberately — `REGEX-ORACLE-ANCHOR-SYNC` checks every occurrence in this file against the contract): regex oracle real tuple **`2189/1879/262/48`**.
- **flow health**: `sota_exit_gate` last green at `CI-PARITY-GATE-ROT.7` (32/32, 4h39m); UNBLOCKED since `.10.9` but not re-proven end-to-end. Automatic CI tier = the **17** doctrines per push + 3 cheap gate targets; the other 120 `make` targets are operator-invoked.
