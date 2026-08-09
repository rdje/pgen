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
- Durable facts / standing directives / decisions: `docs/decisions/INDEX.md` (**authoritative**; its record count is DERIVED — read it, never store it). Live status: `docs/book/src/roadmap-and-live-status.md`; changelog: `CHANGES.md`.
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
- **active_work_unit**: ⭐⭐ **MAKE THE SV PARSER RELEASE-READY — ALL AXES. ⛔ LANE LOCK: do NOT leave SV until it is RELEASED to Nexsim** (director 2026-08-08 ×3) → [[project_nexsim_sv_signoff_delivery_focus]] (**read its 2026-08-08 section first**). A finding in another family is ROUTED to a parked leaf, never worked; the only exception is a defect that BLOCKS the SV release. ⛔ PARKED, not abandoned: `CORPUS-GRAD-ALL.2` VHDL (31.6 %, worklist queued), `LIVE-DOC-CONTAINMENT.4`, `LIVE-MEANS-LIVE`, `SV-EXH-PROOF.7.4.6.18`.
- **next_action**: cut the next axis-2 family from the LIVE worklist (**296 rows / 182 sigs**) on a *token-level tell* — ⛔ **NOT the bucketer** (`.3.19`: 6 of 8 clause-33 rows filed under `interface/modport (ch25)`). ⛔ The frontier and the routed queue are DERIVED — read `docs/TASK_TREE.md`. Coverage half = `.7c` (**92.3 % / 104 gaps**), never `.7a`; reports SELF-DATE — **re-hash before quoting**. Axis 1 is literal 0 over a FROZEN universe. ⛔⛔ [[a-rising-pass-rate-is-not-evidence-of-correctness]]; no family is at 100 % on axis 2.
- **in_flight_uncommitted**: none.
- **blockers**: none. ⏳ 3 director calls open (⛔ all OBJECTIVE/SCOPE → [[feedback_answer_your_own_technical_questions]]): `LESSON-RETRIEVAL.2` scope; a priced `schedule:` lane for `sota_exit_gate`; the ANVIL README-policy feedback.
- ⚠️ **standing tripwires** (read the leaf before acting): ⛔⛔ **a `#` comment at COLUMN 0 in a rule body silently DELETES the following alternatives — INDENT IT**; every census is blind → `EBNF-FRONTEND-SILENT-TRUNCATION` (+2 instrument traps in `SV-CORPUS-GRAD.3.19`/`.3.21`); a corpus `timeout` is a fact about the INSTRUMENT, not the parser → `SV-CORPUS-GRAD.11a`; `TASK-ACCEPTANCE` leaf-scoping VACUOUS for `- ID:` list trees, **65/66** → `GENERATED-LINT-CORRECTNESS.10`; `ci_workflow_local_gate` CANNOT COMPLETE → `CI-PARITY-GATE-ROT.20`; **`--lib` is RED on HEAD and no gate reads it** (`1002/1`) → `CI-PARITY-GATE-ROT.21`; ⛔ **a rule-census move ⇒ re-baseline the cert contract(s) `--dump-rule-profiles` names, SAME commit** → `CI-PARITY-GATE-ROT.22`; ⛔ **`clippy_on_rust_change` prints ✅ and SKIPS on a grammar-only change** (gitignored `generated/`) — use `PGEN_CLIPPY_FORCE=1` → `.23`; sweep traps → `LANG-CAPABILITY-AUDIT.10.3`, `CI-PARITY-GATE-ROT.19`/`.4b`·2.
- **live doctrine anchor** — a DELIBERATE class-(b) pinned duplicate, legal because `REGEX-ORACLE-ANCHOR-SYNC` fails on drift: regex oracle real tuple **`2189/1879/262/48`**.
- **flow health**: `sota_exit_gate` not re-proven end-to-end since `CI-PARITY-GATE-ROT.7`; automatic CI tier = the **17** doctrines per push + 3 cheap gate targets, the other ~120 `make` targets operator-invoked.
