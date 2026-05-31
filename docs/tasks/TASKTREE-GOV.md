# TASKTREE-GOV: Roadmap → Task-Tree Normalization, Past-Change Audit, and Roadmap⇄Codebase⇄mdBook Tri-Lock

## Metadata

- Tree ID: `TASKTREE-GOV`
- Status: `active`
- Roadmap lane: `governance / continuity / doctrine enforcement`
- Created: `2026-05-31`
- Last updated: `2026-05-31` (`.1` inventory + `.2` roadmap→tree coverage DONE — 9 skeletons; frontier → `.3` past-change audit)
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
  Status: `done`
  Goal: `INVENTORY (pure docs): produce the authoritative current-state map — (a) every existing docs/tasks/*.md tree + its status; (b) every roadmap lane across docs/reference/*ROADMAP*.md + LIVE_ACHIEVEMENT_STATUS.md parser-family + capability rows; (c) the gap matrix = roadmap lanes with NO owning tree.`
  Acceptance: `A gap matrix (roadmap lane -> owning tree | NONE) committed into this tree file's Inventory section; docs/TASK_TREE.md Active Task Trees table reconciled to reality.`
  Verification: `done (2026-05-31) — inventory taken from docs/tasks/*.md status fields, SOTA roadmap phase headers (A–V), the enablement/closure roadmap docs, and LIVE_ACHIEVEMENT_STATUS capability rows. Gap matrix recorded in the Inventory section below. CONFIRMED: 18 tree files exist (17 prior + TASKTREE-GOV); all but SV-EXH-PROOF + TASKTREE-GOV are done/complete. GAPS identified (roadmap lanes with NO owning tree): Phase S PNR parser family (Liberty / SDC / aux readers — Not Started), Phase T Cross-Language Linter, Phase U Compiler/Elaborator Workbench, the stimuli-generator-signoff vision, and (PARTIAL) rtl_frontend/rtl_const_expr parser-CLOSURE (only book/contract trees exist, not a closure tree) + annotation-100%-closure (Phase L, Mostly Done, no explicit tree). These become .2 sub-leaves.`
  Commit: `pending`

- ID: `TASKTREE-GOV.2`
  Status: `done`
  Goal: `ROADMAP -> TREE COVERAGE (pure docs): for each gap-matrix lane with NONE owner, create a task-tree skeleton (docs/tasks/<LANE>.md from TEMPLATE.md) with goal/non-goals/acceptance + an initial frontier, and register it in docs/TASK_TREE.md. One sub-leaf per lane so each is independently trackable + commitable.`
  Acceptance: `Every roadmap lane has an owning tree file + a TASK_TREE.md entry; no roadmap activity is untracked.`
  Verification: `done (2026-05-31) — created 9 thin proposed-status owning-skeleton trees for the .1 gap lanes: PNR-LIBERTY, PNR-SDC, PNR-AUX-READERS (Phase S PNR family); LINTER (Phase T), COMPILER-ELABORATOR (Phase U); ANNOT-CLOSURE (Phase L); STIMULI-SIGNOFF (generator vision); RTL-FE-CLOSURE + RTL-CE-CLOSURE (Phase S rtl parser/elaboration closure, distinct from the existing done book/contract trees). Each has goal/non-goals/acceptance + a .1 SCOPING (pure-docs) frontier leaf; all registered in docs/TASK_TREE.md's new "Proposed / Backlog Task Trees" table (also cleaned a stray duplicate separator row + added the missing "Completed Task Trees" heading). Per user "thin skeletons" choice — they satisfy the doctrine (every roadmap lane owned) without over-investing in not-yet-started work. Every roadmap lane now maps to an owning tree; no roadmap activity is untracked.`
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
| — | `TASKTREE-GOV.1` | `done` (2026-05-31) | Inventory + gap matrix complete (see Inventory section). 18 tree files; gaps = PNR (Liberty/SDC/aux), Phase T linter, Phase U compiler/elaborator, Phase L annotation-closure, stimuli-signoff, rtl closure. |
| — | `TASKTREE-GOV.2` | `done` (2026-05-31) | Created 9 thin proposed-status skeleton trees for the gap lanes (PNR-LIBERTY/SDC/AUX-READERS, LINTER, COMPILER-ELABORATOR, ANNOT-CLOSURE, STIMULI-SIGNOFF, RTL-FE-CLOSURE, RTL-CE-CLOSURE) + registered them in TASK_TREE.md. Every roadmap lane now owned. |
| 1 | `TASKTREE-GOV.3` | `pending` (frontier) | Past-change audit annotations for pre-doctrine campaigns. Sequenced after inventory + coverage. |
| 4 | `TASKTREE-GOV.4` | `pending` | Tri-lock contract + drift check. Last, since it depends on the tree set being complete. |

## Inventory + Gap Matrix (`TASKTREE-GOV.1`, 2026-05-31)

**Existing tree files (`docs/tasks/*.md`, 18 incl. this one):**
`active`: SV-EXH-PROOF, TASKTREE-GOV. `done/complete`: INLINE-ALT-FIX, POST-SV-AUDIT, RGX-0084, RGX-0085, RGX-0086, RGX-0087, RGX-0087-FIX2, RGX-0088, RTL-CE-CONTRACT-BODY, RTL-CE-MDBOOK, RTL-FE-CONTRACT-BODY, RTL-FE-MDBOOK, SEMREF-SHAPED, SVPP-CONTRACT-BODY, SVPP-MDBOOK, VHDL-CONTRACT-BODY, VHDL-MDBOOK. (Plus historically-pruned trees referenced in TASK_TREE.md prose: DOC-ENVELOPE-0001, DOC-README-SHELL-0001, the original 9 + INLINE-ALT-FIX.)

**Roadmap lanes (SOTA roadmap phases A–V + LIVE capability rows) → owning tree:**

| Roadmap lane | LIVE status | Owning tree | Coverage |
| --- | --- | --- | --- |
| Phases A–R (foundational / annotation / frontend / SV / SVPP / regex / VHDL) | Done | done trees (SEMREF-SHAPED, POST-SV-AUDIT, RGX-*, SVPP-*, VHDL-*) + git history | COVERED |
| `systemverilog` main parser (Phase P) | Mostly Done | `SV-EXH-PROOF` (active) | COVERED |
| `systemverilog_preprocessor` (Phase Q) | Done | `SVPP-MDBOOK` + `SVPP-CONTRACT-BODY` | COVERED |
| `vhdl` parser family | Done | `VHDL-MDBOOK` + `VHDL-CONTRACT-BODY` | COVERED |
| `regex` parser family | Done | `RGX-008x` family | COVERED |
| `return_annotation` + cross-grammar shaping | Done / Mostly Done | `SEMREF-SHAPED`, `POST-SV-AUDIT` | COVERED |
| Phase V: per-parser standalone mdBooks | (planned) | `*-MDBOOK` trees (done for shipped families) | COVERED (extend per new family) |
| **Phase S: `rtl_const_expr` baseline evaluator** | Mostly Done | RTL-CE-{MDBOOK,CONTRACT-BODY} only (book/contract; NO parser-closure tree) | **PARTIAL — needs closure tree** |
| **Phase S: `rtl_frontend` synthesizable subset** | In Progress | RTL-FE-{MDBOOK,CONTRACT-BODY} only (NO closure tree) | **PARTIAL — needs closure tree** |
| **Phase S: PNR family — Liberty parser** | Not Started | NONE | **GAP** |
| **Phase S: PNR family — SDC parser** | Not Started | NONE | **GAP** |
| **Phase S: PNR aux readers (gate-level netlist / config / SDF)** | Not Started | NONE | **GAP** |
| **Phase T: Cross-Language Linter enablement** | planned | NONE | **GAP** |
| **Phase U: Compiler / Elaborator Workbench** | planned | NONE | **GAP** |
| **Phase L: Annotation 100% closure (return + semantic)** | Mostly Done | NONE explicit (`PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md` not tree-ized) | **GAP** |
| **Stimuli-generator-signoff vision** ([[project_stimuli_generator_signoff_vision]]) | new (user 2026-05-31) | NONE (SV-EXH-PROOF.7 feeds it but isn't the owner) | **GAP** |
| Parser-family exhaustive proof normalization | Mostly Done | `SV-EXH-PROOF` | COVERED |

**`.2` work-list (lanes needing a new owning tree):** PNR-LIBERTY, PNR-SDC, PNR-AUX-READERS, LINTER (Phase T), COMPILER-ELABORATOR (Phase U), ANNOT-CLOSURE (Phase L), STIMULI-SIGNOFF (generator vision), and closure trees for RTL-FE / RTL-CE parser-closure (distinct from their existing book/contract trees). Note: most are `Not Started`/`planned` long-horizon lanes — `.2` creates the OWNING SKELETONS (so nothing is untracked), not the implementation.

## Decisions

- `2026-05-31`: Created at user direction (doctrine re-affirmed + expanded: full-roadmap coverage, past-change audit, tri-lock). Sequencing chosen by user = "Governance tree first, now" — SV-EXH-PROOF pauses after its in-flight `.6` verification reports; this tree leads. Rationale: maximize doctrine compliance immediately.
- `2026-05-31` (`.1` done): inventory complete; gap matrix above. The doctrine ("whole roadmap task-tree-owned") is satisfied for all SHIPPED/active lanes; the GAPS are all `Not Started`/`planned`/`Mostly Done` long-horizon lanes (PNR, linter, compiler/elaborator, annotation-closure, stimuli-signoff, rtl closure) — `.2` will create owning skeletons so even un-started roadmap work is tracked.
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
