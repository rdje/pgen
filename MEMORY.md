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
- **EVERY issue raised must be FIXED** — routing decides WHEN, never WHETHER; a design is a schedule, not the deliverable → [[feedback_every_finding_must_be_fixed_not_logged]]; worked NOW only when it BLOCKS → [[feedback_flow_findings_are_routed_not_worked]].
- **An instrument with no ground truth is a confident guess** — pin both a positive and a negative control inside it and REFUSE on a miss → [[feedback_instrument_needs_ground_truth]].
- **Project data stays on the repo's volume** → [[project_data_locality_same_volume]]; reclaim artifacts routinely, proving safety each time → [[feedback_delete_reclaimable_artifacts_regularly]]; the portable spine is a separate repo → [[project_bedrock_spine_repo]].

## Current state (OVERWRITE this block each update — do not append)
> ⛔ **DERIVED, never stored** (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3): a field a command answers EXACTLY is not written here — it is looked up. `git log -1 --oneline` = last commit · `git rev-list --count origin/main..HEAD` = unpushed (cadence **300** or an explicit order → [[feedback_push_pacing]]) · `docs/TASK_TREE.md` = every tree, its frontier, and the routed queue. A stored copy of these drifts silently; the push counter did, and was wrong BY CONSTRUCTION (writing it takes a commit, which increments it).
- **active_work_unit**: ⭐ **the corpus program — real parser work** (director, 2026-08-08) in the **EXISTING** `CORPUS-GRAD-ALL` tree (⛔ do NOT open another — `.1` done, rosters frozen), per [[project_all_parsers_fully_pass_stimuli_and_external_corpora]] (**read it first**). `.2.0` DONE: `furthest_position` now on **all 12** detail parse paths (was 2), so corpus stuck-point clustering works for EVERY family. ⛔ PARKED: `LIVE-DOC-CONTAINMENT.4`, `LIVE-MEANS-LIVE`, `SV-EXH-PROOF.7.4.6.18` — governance.
- **next_action**: `CORPUS-GRAD-ALL.2` — VHDL grammar GROWTH, burning down `docs/tasks/artifacts/corpus_grad_all/vhdl_fail_clusters.md` (**the ranked worklist — RE-READ it, it is regenerated per leaf**). `.2.1`+`.2.2` DONE → corpus **29.8%** (4082/13720); `wait` LRM §10.2 closed (class 387→0). Next class: `for ID :` **338** (grew from 110 — the burn-down pushes files deeper), then `after` 265, `file of` 219, `attribute` 174, `access` 149. ⛔ The gap is ABSENT LANGUAGE, not bugs — `grammars/vhdl.ebnf` is a declared SEED subset (221 rules). ⛔⛔ A RISING PASS COUNT IS NOT CORRECTNESS: `.2.2`'s buggy draft scored 4086 vs the correct 4082 — always check the negative axis (VESTS `non_compliant/`) and `grep -oE '^[a-z_]+ :=' grammars/vhdl.ebnf | sort | uniq -d` (empty = no silent rule-name merge). Then the stale SV `403`. ⛔ no family is at 100 % on axis 2 today.
- **in_flight_uncommitted**: none.
- **blockers**: none. ⏳ 3 director calls open (⛔ all OBJECTIVE/SCOPE — never escalate a technical one → [[feedback_answer_your_own_technical_questions]]): `LESSON-RETRIEVAL.2` scope; a priced `schedule:` lane for `sota_exit_gate`; the ANVIL README-policy feedback.
- ⚠️ **standing tripwires** (read the leaf before acting): `TASK-ACCEPTANCE` leaf-scoping VACUOUS for `- ID:` list trees, **65/66** → `GENERATED-LINT-CORRECTNESS.10`; `ci_workflow_local_gate` CANNOT COMPLETE → `CI-PARITY-GATE-ROT.20`; **`--lib` is RED on HEAD and no gate reads it** (`1002/1`) → `CI-PARITY-GATE-ROT.21`; sweep traps → `LANG-CAPABILITY-AUDIT.10.3`, `CI-PARITY-GATE-ROT.19`/`.4b`·2.
- **live doctrine anchor** — a DELIBERATE class-(b) pinned duplicate, legal because `REGEX-ORACLE-ANCHOR-SYNC` fails on drift: regex oracle real tuple **`2189/1879/262/48`**.
- **flow health**: `sota_exit_gate` not re-proven end-to-end since `CI-PARITY-GATE-ROT.7`; automatic CI tier = the **17** doctrines per push + 3 cheap gate targets, the other ~120 `make` targets operator-invoked.
