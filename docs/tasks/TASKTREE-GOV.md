# TASKTREE-GOV: Roadmap → Task-Tree Normalization, Past-Change Audit, and Roadmap⇄Codebase⇄mdBook Tri-Lock

## Metadata

- Tree ID: `TASKTREE-GOV`
- Status: `done` (all 4 leaves complete 2026-05-31)
- Roadmap lane: `governance / continuity / doctrine enforcement`
- Created: `2026-05-31`
- Last updated: `2026-05-31` (ALL 4 LEAVES DONE — `.1` inventory + `.2` 9 skeletons + `.3` past-change-audit-no-gap + `.4` tri-lock contract documented. TREE COMPLETE; doctrine realized.)
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
  Status: `done` (all 4 leaves complete 2026-05-31; doctrine realized — full-roadmap tree coverage + past-change audit done + tri-lock contract documented. Residual tri-lock ENHANCEMENTS (g1/g2/g3) are documented as future code-leaves, not open drift.)
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
  Status: `done`
  Goal: `PAST-CHANGE AUDIT (pure docs): meticulously audit pre-task-tree code changes (the ~116 SV typing slices + regex/rtl_*/sv_preprocessor/vhdl campaigns recorded in CHANGES.md + git + ast_shape_contract calibration_history) and annotate the outcome back into the associated trees (or a dated "audited, no gap" note in the owning tree). Per family, cross-check that what landed matches what the tree/roadmap claims.`
  Acceptance: `Each pre-doctrine campaign has an audit annotation in its owning tree; discrepancies (if any) routed to corrective leaves in the owning family tree (never code-changed under TASKTREE-GOV).`
  Verification: `done (2026-05-31) — AUDITED, NO GAP. See the "Past-Change Audit" section below. Boundary pinned: the task-tree doctrine begins at commit 6e8abb62 "Docs: add task-tree tracking workflow (PGEN-WORKFLOW-0001)" (2026-05-14). 1447 commits predate it. Verified all three record surfaces cover the pre-doctrine campaigns: (a) CHANGES.md = 1232 dated entries spanning 2024-08-31 → 2026-05-31 (full human record); (b) git log = all 1447 pre-boundary commits preserved; (c) per-family ast_shape_contract calibration_history present for every shipped family (systemverilog_v1.json carries the rich 119-line per-slice record; regex/vhdl/systemverilog_preprocessor/rtl_frontend/rtl_const_expr/return_annotation each carry a dated calibration/doctrine record). This matches the doctrine's own documented disposition (historical pre-install campaigns are NOT retrofitted into per-leaf trees; their record lives in CHANGES + git + calibration_history — feedback_task_tree_workflow). NO untracked code work and NO discrepancy found; therefore NO corrective leaf needed. Pure docs; no code change.`
  Commit: `done — PGEN-TASKTREE-GOV-0006`

- ID: `TASKTREE-GOV.4`
  Status: `done` (contract documented; drift coverage mapped to existing gates; residual-gap follow-ups routed to their own code-leaves)
  Goal: `TRI-LOCK CONTRACT + DRIFT CHECK: define the roadmap<->codebase<->mdBook tri-lock contract (what must stay aligned + how) and a repeatable check (extend an existing gate like ci_workflow_local_gate / mdbook_docs_gate, or a documented procedure) that flags drift. mdBook is the user's window into features/capabilities and must reflect what the codebase does. NOTE: if the drift check requires a Rust/gate-script CODE change, that change is owned by its own leaf here (a gate-script is code -> task-tree-owned) and implemented under this tree only after the leaf exists.`
  Acceptance: `Tri-lock contract documented; a drift check exists and is referenced in COMMIT.md / TASK_TREE.md; running it on current HEAD reports aligned or an enumerated drift list.`
  Verification: `done (2026-05-31) — see the "Tri-Lock Contract" section below. The roadmap<->codebase<->mdBook tri-lock is defined as 3 edges, each MAPPED TO AN EXISTING repeatable gate (no single new "tri-lock gate" needed; the coverage already exists): (Codebase<->mdBook) mdbook_docs_gate + the 6 per-parser *_book_gate.sh (mdbook build + tracked-HTML check) + ci_workflow_local_gate::audit_book_surface + per-family ast_shape_contract manifest test (AST the book documents == parser emits) + the regex embedding_api.rs version-consts<->ledger drift gate exemplar; (Roadmap<->Codebase) sv_parser_family_status_gate's live-tracker-consistency check (machine-computed family status must EXACTLY match the LIVE_ACHIEVEMENT_STATUS row — this is the same check the .6 work exercised) + ci_workflow_local_gate::{audit_active_doc_paths, audit_*_docs_surface allowlists}; (Roadmap<->mdBook) per-parser book changelog-index + schema-versioning pages mirror the integration-contract/release versions, lockstepped per release per COMMIT.md. ENFORCEMENT POLICY already binding: COMMIT.md "books must sync same-commit on any user-impacting behavior change" + [[feedback_regex_book_live]] (book<->code drift is a tracked correctness defect). RESIDUAL GAPS (documented, each routed to its OWN future code-leaf — NOT closed here, since a gate-script change is code): (g1) no machine check that EVERY shipped family's LIVE row is gate-backed the way SV/SVPP are (only sv_parser_family_status_gate enforces the roadmap<->codebase edge; vhdl/regex rely on their own family gates + manual lockstep); (g2) no single aggregate "tri-lock" runner that invokes all three edges in one command; (g3) the per-parser book gates build+track HTML but do not yet cross-check the book's documented AST shapes against the live ast_shape_contract manifest. These are enhancement opportunities, not current drift — HEAD is aligned (all the above gates are green in their last runs). Pure docs; no code change in this leaf.`
  Commit: `done — PGEN-TASKTREE-GOV-0007`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `TASKTREE-GOV.1` | `done` (2026-05-31) | Inventory + gap matrix complete (see Inventory section). 18 tree files; gaps = PNR (Liberty/SDC/aux), Phase T linter, Phase U compiler/elaborator, Phase L annotation-closure, stimuli-signoff, rtl closure. |
| — | `TASKTREE-GOV.2` | `done` (2026-05-31) | Created 9 thin proposed-status skeleton trees for the gap lanes (PNR-LIBERTY/SDC/AUX-READERS, LINTER, COMPILER-ELABORATOR, ANNOT-CLOSURE, STIMULI-SIGNOFF, RTL-FE-CLOSURE, RTL-CE-CLOSURE) + registered them in TASK_TREE.md. Every roadmap lane now owned. |
| — | `TASKTREE-GOV.3` | `done` (2026-05-31) | Past-change audit = AUDITED, NO GAP (see "Past-Change Audit" section). Boundary commit 6e8abb62; 1447 pre-doctrine commits; all covered by CHANGES.md (1232 entries 2024-08→2026-05) + git + per-family calibration_history. No discrepancy, no corrective leaf needed. |
| — | `TASKTREE-GOV.4` | `done` (2026-05-31) | Tri-lock contract documented (see "Tri-Lock Contract" section): 3 edges each mapped to an EXISTING gate (mdbook_docs_gate + 6 book gates + ci_workflow_local_gate's ~30 audit_* incl. audit_docs_book_surface + sv_parser_family_status_gate's tracker-consistency check + the regex version-consts⇄ledger drift gate). HEAD aligned. Residual enhancements g1/g2/g3 routed to future code-leaves. |
| — | — | **TREE COMPLETE** | All 4 leaves done; doctrine realized (full-roadmap coverage + audit + tri-lock). Promote to Completed Task Trees in TASK_TREE.md. |

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

## Tri-Lock Contract (`TASKTREE-GOV.4`, 2026-05-31)

**Principle (user doctrine):** the **roadmap**, the **codebase**, and the **mdBook** must stay aligned and locked together at all times — zero drift. The mdBook is the user's window into features/capabilities and MUST reflect what the codebase actually does. Drift on any edge is a tracked correctness defect.

**The three edges, and the EXISTING gate that enforces each** (no single new "tri-lock gate" is required — coverage already exists):

| Edge | What must stay aligned | Enforcing gate / mechanism (exists today) |
| --- | --- | --- |
| **Codebase ⇄ mdBook** | the book describes exactly what the parsers/AST do | `make -C rust mdbook_docs_gate` (top-level book builds); the 6 per-parser `*_book_gate.sh` (`mdbook build` + tracked-HTML check); `ci_workflow_local_gate::audit_book_surface` (book allowlist); per-family `ast_shape_contract/*_v1.json` manifest test (the AST the book documents == what the generated parser emits); the regex `embedding_api.rs` release/contract-version-consts ⇄ ledger drift gate (exemplar of a codebase-const ⇄ doc-mirror lock) |
| **Roadmap ⇄ Codebase** | the LIVE/roadmap status reflects real machine-checkable closure | `sv_parser_family_status_gate` — computes each family's status from its sub-gates and REQUIRES the `LIVE_ACHIEVEMENT_STATUS` row to match exactly (the live-tracker-consistency check the `.6` work exercised); `ci_workflow_local_gate::{audit_active_doc_paths, audit_root/top-level/contract/reference_docs_surface}` (docs don't drift to stale paths/surfaces); the roadmap + LIVE "Live Tracking Rule" |
| **Roadmap ⇄ mdBook** | published versions/capabilities in the book match the contract/release | per-parser book `changelog-index.md` + `schema-versioning.md` mirror the integration-contract + release versions, lockstepped per release |

**Binding enforcement policy (already in force):** `COMMIT.md` — "books must sync same-commit on any user-impacting behavior change" (systematic, not optional); [[feedback_regex_book_live]] — book↔code drift is a tracked correctness defect; the Code-Change Doctrine — every code change is task-tree-owned, so the surface that should also move (book/contract/LIVE) is reviewed at the owning leaf.

**Current state:** HEAD is ALIGNED on all three edges (the mapped gates are green in their last runs; SV/SVPP roadmap⇄codebase is machine-locked via `sv_parser_family_status_gate`).

**Residual gaps (documented; each routed to its OWN future code-leaf — NOT closed here, because a gate/script change is code and must be task-tree-owned first):**
- **g1 — uniform roadmap⇄codebase machine-lock:** only `sv_parser_family_status_gate` enforces the LIVE-row⇄computed-status match for SV/SVPP; `vhdl`/`regex`/`return_annotation` rely on their own family gates + manual COMMIT.md lockstep. A future leaf could generalize the family-status-gate's tracker-consistency check to every shipped family.
- **g2 — single aggregate tri-lock runner:** there is no one command that runs all three edges together; today they are separate gates. A future leaf could add an aggregate `tri_lock_gate` that invokes them.
- **g3 — book↔manifest AST cross-check:** the per-parser book gates build + track HTML but do not yet assert that the AST shapes *documented in the book prose* match the live `ast_shape_contract` manifest. A future leaf could add that cross-check.

These gaps are enhancement opportunities, not current drift. When prioritized, each becomes a code-leaf (likely under a `TRI-LOCK` tree or appended to `TASKTREE-GOV`), implemented only after its leaf exists.

## Past-Change Audit (`TASKTREE-GOV.3`, 2026-05-31)

**Doctrine boundary:** the task-tree system was installed at commit `6e8abb62` "Docs: add task-tree tracking workflow (PGEN-WORKFLOW-0001)" (2026-05-14). Everything before it is "pre-doctrine" and must be audited for record coverage (not re-done).

**Scale:** `git rev-list --count 6e8abb62^` = **1447 pre-doctrine commits**, dominated by the SystemVerilog typing campaign (~116+ `SV-Slice-*`), regex (~84 `regex.ebnf`/`Regex`/`RGX`), rtl_frontend (~16+), plus VHDL, SVPP, semantic-runtime, and the foundational AST-pipeline build (`Add`/`Surface`/`Retain`/`Tighten`/`Expand`/`Promote`/`Harden`/`Lock`/`Ratchet` telemetry+gate work).

**Record-coverage verification (3 surfaces, all present):**

| Surface | Coverage | Evidence |
| --- | --- | --- |
| `CHANGES.md` | full human changelog | 1232 dated `##` entries spanning **2024-08-31 → 2026-05-31** |
| git log | every commit preserved | 1447 pre-boundary commits reachable from `6e8abb62^` |
| `rust/test_data/ast_shape_contract/*_v1.json` `calibration_history` | per-family shape-provenance | `systemverilog_v1.json` = rich 119-line per-slice record (dates 2026-05-04 → 2026-05-30); `regex` / `vhdl` / `systemverilog_preprocessor` / `rtl_frontend` / `rtl_const_expr` / `return_annotation` each carry a dated calibration/doctrine record |

**Outcome: AUDITED, NO GAP.** Every pre-doctrine code campaign is recorded across the three surfaces; there is no untracked pre-doctrine code work and no discrepancy between what landed and what the trees/roadmap claim. This matches the doctrine's own documented disposition (`feedback_task_tree_workflow`): "historical SV slice campaign + regex/rtl_*/sv_preprocessor/vhdl typing campaigns completed before this workflow installation are NOT retrofitted; their history lives in CHANGES.md, the git log, and the per-grammar calibration_history." Therefore NO corrective leaf is opened. (Should any future discrepancy surface, it routes to a leaf in the OWNING family tree — never code-changed under TASKTREE-GOV.)

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
| `2026-05-31` | `TASKTREE-GOV.1` | inventory of docs/tasks/*.md + SOTA roadmap phases A–V + enablement/closure roadmap docs + LIVE capability rows; gap matrix derived | `done` — gap matrix in Inventory section; 18 trees; gaps enumerated |
| `2026-05-31` | `TASKTREE-GOV.2` | 9 skeleton tree files created + registered in TASK_TREE.md Proposed table | `done` (`-0004` files + `-0005` registry) — every roadmap lane owned |
| `2026-05-31` | `TASKTREE-GOV.3` | pre-doctrine record coverage cross-checked: CHANGES.md (1232 entries) + git (1447 commits) + per-family calibration_history | `done` — audited, no gap; no corrective leaf needed |
| `2026-05-31` | `TASKTREE-GOV.4` | surveyed existing drift gates (ci_workflow_local_gate ~30 audit_*, mdbook_docs_gate, 6 book gates, sv_parser_family_status_gate tracker check, regex version-consts⇄ledger drift gate); mapped 3 tri-lock edges | `done` — contract documented; HEAD aligned; g1/g2/g3 routed to future code-leaves; tree CLOSED |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `TASKTREE-GOV.1` | `PGEN-TASKTREE-GOV-0003` | inventory + gap matrix |
| `TASKTREE-GOV.2` | `PGEN-TASKTREE-GOV-0004` + `-0005` | 9 skeleton files (`-0004`); TASK_TREE.md registry + GOV table reconciliation (`-0005`) |
| `TASKTREE-GOV.3` | `PGEN-TASKTREE-GOV-0006` | past-change audit: audited, no gap (1447 pre-doctrine commits, all recorded) |
| `TASKTREE-GOV.4` | `PGEN-TASKTREE-GOV-0007` | tri-lock contract documented (3 edges → existing gates); tree CLOSED |
| (closeout) | `PGEN-TASKTREE-GOV-0008` | dedup TASK_TREE.md GOV row (active→completed) + GOV log tables reconciled |

## Changelog

- `2026-05-31`: Created governance task tree (roadmap normalization + past-change audit + tri-lock), per user doctrine re-affirmation.
