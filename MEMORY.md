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
- Durable facts / standing directives / decisions: `docs/decisions/INDEX.md` (137 records — **the authoritative list**). Live status: `LIVE_ACHIEVEMENT_STATUS.md`; changelog: `CHANGES.md`.
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
- **latest_commit**: `PGEN-LANG-CAPABILITY-AUDIT-0027` (leaf `.10.11`) — ⭐ **DIRECTOR RULING: only the Perl EBNF FRONTEND was unwanted; the Perl INTERPRETER stays and is to be DECLARED.** `require_tool perl` added to `sota_exit_gate.sh` + `hdl_frontend_readiness_gate.sh`; 5 routed → `CI-PARITY-GATE-ROT.10`. ⛔ Two of my numbers were wrong (16/47, not 17/~48; the hole is pre-existing, not `.10.6`'s).
- **active_work_unit**: `LANG-CAPABILITY-AUDIT.10`. ✅ `.10.1`–`.10.5`, `.10.7` and `.10.6` part 1 all landed; ✅ `.10.10` (`fx/` deleted). ✅ `.10.11` (Perl interpreter DECLARED, director-ruled). ⛔ OPEN: `.10.9` (live RED), `.10.12` (DOCPATH gap), `.10.8` (two dead harnesses). ⛔ **`.10.7` CARRIES STANDING DIRECTOR APPROVAL — DO NOT RE-ASK** → [[feedback_routine_decisions_are_not_escalations]]: escalating chartered work offloads judgement; the bar is SOTA/signoff/professional, not permission.
- **next_action**: ⛔⛔ **`.10.9` — HIGHEST PRIORITY, a LIVE RED in the flagship aggregate.** `.10.6` left 4 gate scripts reading Perl telemetry the producer no longer emits; `regex_parser_family_contract_gate.sh` **cannot pass** (`:217` asserts an absent key; `:222-228` demands `raw_ast_status=parity|perl_under_reports` but the producer emits **`exported`**) and — the dangerous one — `:230`'s floor `(( rust_rule_count < perl_rule_count ))` reads `null` as **0** and **passes vacuously**. `sota_exit_gate` invokes it. ⛔ Do NOT just add `exported` to the allowlist — that converts a broken gate into a quiet one. Then **`.10.6` part 2** (the raw-AST differential, which is what would give the contract a real second arm), then `.10.8`, then `.3c`. ⭐ **`fx/` remains a SEPARATE one-line director call** (tracked, **267 files**, referenced by nothing in `rust/`/`scripts/`/`.github/`).
- **in_flight_uncommitted**: none.
- ⚠️ **two sweep traps banked in `.10.3`** (read it before any before→after sweep): the embedded `-o` path, and `focus_*` silently changing `ast_pipeline`'s feature set at the same path → also `CI-PARITY-GATE-ROT.19`.
- **blockers**: none. ⏳ One escalated **director call** stands open: a priced `schedule:` lane for the 4h39m `sota_exit_gate`, the only thing that would make the bar's re-proof unconditional rather than per-push. It spends Actions minutes, so it is not the engineer's to take.
- **routed, waiting behind product** (do NOT pull forward): `CI-PARITY-GATE-ROT.10`/`.11` remainder/`.16`/`.17`/`.18`, `OPS-MEMSAFE.4`, `DOCTRINE-GAP-OWNERSHIP.3`, `DONE-BAR.5d` gate/`.5f`.3/`.5g`, `README-POLICY.3`/`.6` (`.6` = `pipefail`+`grep -q` fails OPEN in 2 enforcers — latent, priced; ✅ the FIXED form now exists in bedrock, so `.6` is an ADOPTION not a design).
- **push**: 118 commits ahead of `origin/main` (as of this commit). ⛔ Cadence is **300 unpushed OR an explicit exceptional order** — never ask or suggest below 300 → [[feedback_push_pacing]].
- **live doctrine anchor** (kept here deliberately — `REGEX-ORACLE-ANCHOR-SYNC` checks every occurrence in this file against the contract): regex oracle real tuple **`2189/1879/262/48`**.
- **flow health**: `sota_exit_gate` went green end-to-end for the first time at `CI-PARITY-GATE-ROT.7` (32/32 stages, 4h39m, one machine, one run). Automatic CI tier = the 15 doctrines on every push + 3 cheap gate targets; the other 120 `make` gate targets remain operator-invoked.
