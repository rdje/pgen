# TASKTREE-GOV: Roadmap → Task-Tree Normalization, Past-Change Audit, and Roadmap⇄Codebase⇄mdBook Tri-Lock

## Metadata

- Tree ID: `TASKTREE-GOV`
- Status: `active`
- Roadmap lane: `governance / continuity / doctrine enforcement`
- Created: `2026-05-31`
- Last updated: `2026-05-31`
- Owner: repo-local workflow

## Goal

Make the task-tree doctrine fully realized across the whole project, per the
user directive (2026-05-31, non-negotiable):

1. **Full-roadmap coverage** — every roadmap activity/phase is owned by a
   task-tree (not just the in-flight lane).
2. **Past-change audit** — code changes made before the task-tree system
   existed are audited (thorough/accurate/meticulous) and the outcome is
   annotated back into the associated task-trees.
3. **Tri-lock, zero drift** — the roadmap, the codebase, and the mdBook are
   aligned and locked together at all times; mdBook reflects exactly what the
   codebase does. Drift is a tracked correctness defect.

This tree OWNS the normalization effort itself, so even the doctrine-compliance
work is task-tree-tracked and survives session loss/crash.

## Non-Goals

- This tree does NOT itself make parser/codegen/grammar code changes. It is
  pure task-tree + documentation authoring. Any code change surfaced by an
  audit is routed to (or creates) a leaf in the OWNING family tree, never made
  under this governance tree.
- It does not retrofit already-`done` per-leaf campaigns that are adequately
  recorded in `CHANGES.md` + git + `calibration_history` — it AUDITS them for
  coverage and annotates gaps, not re-does them.
- It does not change the doctrine; it implements it.

## Acceptance Criteria

- Every roadmap lane in `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
  and the enablement/closure roadmaps (linter, compiler/elaborator, semantic
  steering, annotation closure, PNR) maps to a task-tree (active, proposed, or
  explicitly `completed` with provenance) listed in `docs/TASK_TREE.md`.
- A documented past-change audit pass exists for pre-task-tree code changes,
  with findings annotated into the relevant trees (or a dated "audited, no
  gap" note).
- A tri-lock contract is defined + a repeatable check (gate or documented
  procedure) flags roadmap/codebase/mdBook drift.
- `docs/TASK_TREE.md` Active Task Trees table is the accurate single index.
- Each completed leaf committed through `COMMIT.md`.

## Task Tree

- ID: `TASKTREE-GOV`
  Status: `active`
  Goal: `Realize the task-tree doctrine across the whole project: full-roadmap tree coverage + past-change audit + roadmap/codebase/mdBook tri-lock, zero drift.`
  Children: `TASKTREE-GOV.1` (inventory), `TASKTREE-GOV.2` (roadmap→tree coverage), `TASKTREE-GOV.3` (past-change audit), `TASKTREE-GOV.4` (tri-lock contract + drift check)

- ID: `TASKTREE-GOV.1`
  Status: `pending`
  Goal: `INVENTORY (pure docs): produce the authoritative current-state map — (a) every existing docs/tasks/*.md tree + its status; (b) every roadmap lane across docs/reference/*ROADMAP*.md + LIVE_ACHIEVEMENT_STATUS.md parser-family + capability rows; (c) the gap matrix = roadmap lanes with NO owning tree. Confirm/correct the 2026-05-31 observation that 17 tree files exist (SV-EXH-PROOF, POST-SV-AUDIT, INLINE-ALT-FIX, SEMREF-SHAPED, RGX-008x ×6, RTL-{FE,CE}-{MDBOOK,CONTRACT-BODY}, SVPP-{MDBOOK,CONTRACT-BODY}, VHDL-{MDBOOK,CONTRACT-BODY}) but roadmap-level phases (linter enablement, compiler/elaborator enablement, semantic-steering, PNR parser family, annotation 100% closure, stimuli-generator-signoff per [[project_stimuli_generator_signoff_vision]]) are not yet tree-ized.`
  Acceptance: `A gap matrix (roadmap lane -> owning tree | NONE) committed into this tree file's Decisions/Inventory section; docs/TASK_TREE.md Active Task Trees table reconciled to reality (every existing docs/tasks/*.md represented with correct status).`
  Verification: `pending`
  Commit: `pending`

- ID: `TASKTREE-GOV.2`
  Status: `pending`
  Goal: `ROADMAP -> TREE COVERAGE (pure docs): for each gap-matrix lane with NONE owner, create a task-tree skeleton (docs/tasks/<LANE>.md from TEMPLATE.md) with goal/non-goals/acceptance + an initial frontier, and register it in docs/TASK_TREE.md. One sub-leaf per lane so each is independently trackable + commitable.`
  Acceptance: `Every roadmap lane has an owning tree file + a TASK_TREE.md entry; no roadmap activity is untracked.`
  Verification: `pending`
  Commit: `pending`

- ID: `TASKTREE-GOV.3`
  Status: `pending`
  Goal: `PAST-CHANGE AUDIT (pure docs): meticulously audit pre-task-tree code changes (the ~116 SV typing slices + regex/rtl_*/sv_preprocessor/vhdl campaigns recorded in CHANGES.md + git + ast_shape_contract calibration_history) and annotate the outcome back into the associated trees (or a dated "audited, no gap" note in the owning tree). Per family, cross-check that what landed matches what the tree/roadmap claims.`
  Acceptance: `Each pre-doctrine campaign has an audit annotation in its owning tree; discrepancies (if any) routed to corrective leaves in the owning family tree (never code-changed under TASKTREE-GOV).`
  Verification: `pending`
  Commit: `pending`

- ID: `TASKTREE-GOV.4`
  Status: `pending`
  Goal: `TRI-LOCK CONTRACT + DRIFT CHECK: define the roadmap<->codebase<->mdBook tri-lock contract (what must stay aligned + how) and a repeatable check (extend an existing gate like ci_workflow_local_gate / mdbook_docs_gate, or a documented procedure) that flags drift. mdBook is the user's window into features/capabilities and must reflect what the codebase does. NOTE: if the drift check requires a Rust/gate-script CODE change, that change is owned by its own leaf here (a gate-script is code -> task-tree-owned) and implemented under this tree only after the leaf exists.`
  Acceptance: `Tri-lock contract documented; a drift check exists and is referenced in COMMIT.md / TASK_TREE.md; running it on current HEAD reports aligned or an enumerated drift list.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `TASKTREE-GOV.1` | `pending` (frontier) | Inventory must precede everything — can't cover gaps or audit without the authoritative current-state map + gap matrix. Pure docs; no dependency on the in-flight SV-EXH-PROOF.6 verification. |
| 2 | `TASKTREE-GOV.2` | `pending` | Create owning trees for each uncovered roadmap lane. Needs `.1`'s gap matrix. |
| 3 | `TASKTREE-GOV.3` | `pending` | Past-change audit annotations. Can run in parallel with `.2` but sequenced after inventory. |
| 4 | `TASKTREE-GOV.4` | `pending` | Tri-lock contract + drift check. Last, since it depends on the tree set being complete. |

## Decisions

- `2026-05-31`: Created at user direction (doctrine re-affirmed + expanded: full-roadmap coverage, past-change audit, tri-lock). Sequencing chosen by user = "Governance tree first, now" — SV-EXH-PROOF pauses after its in-flight `.6` verification reports; this tree leads. Rationale: maximize doctrine compliance immediately.
- `2026-05-31`: This tree is pure task-tree/doc authoring. The doctrine's own escape hatch (pure non-code single-slice doc work is task-tree-exempt) does NOT apply to standing up trees — but standing up trees IS the tracking mechanism, so this governance tree is self-owning and its leaves are committed normally.

## Open Questions

- Whether completed/pruned trees (e.g. RGX-* whose `docs/tasks/*.md` still exist, and any referenced-but-pruned ones) should be marked `completed` in-place or moved to a `docs/tasks/completed/` archive — resolve in `.1`. Does not block the frontier.

## Blockers

- None. (Independent of the in-flight `SV-EXH-PROOF.6` family-status verification.)

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `TASKTREE-GOV.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `TASKTREE-GOV.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created governance task tree (roadmap normalization + past-change audit + tri-lock), per user doctrine re-affirmation.
