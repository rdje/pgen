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

## North star (⛔ POINTERS ONLY — the curated list lives in [[project_north_star]], which is authoritative)
- Goal + the TWO non-negotiables (parser-neutrality, peak speed — costs are REJECTED, not traded), the first-tier-only `Done` bar, accuracy-before-speed, **SV is 100 % LRM-compliant by default** (over-acceptance is a defect), the EBNF as sole source of truth, prior-art-before-design, every-finding-is-FIXED (routing decides WHEN), instruments need ground truth, data stays on the repo volume → **read [[project_north_star]]** for the 15 authoritative record links.

## Current state (OVERWRITE this block each update — do not append)
> ⛔ **DERIVED, never stored** (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3): a field a command answers EXACTLY is not written here — it is looked up. `git log -1 --oneline` = last commit · `git rev-list --count origin/main..HEAD` = unpushed (cadence **300** or an explicit order → [[feedback_push_pacing]]) · `docs/TASK_TREE.md` = every tree, its frontier, and the routed queue. A stored copy of these drifts silently; the push counter did, and was wrong BY CONSTRUCTION (writing it takes a commit, which increments it).
- **active_work_unit**: ⭐⭐ **MAKE THE SV PARSER RELEASE-READY — ALL AXES. ⛔ LANE LOCK: do NOT leave SV until it is RELEASED to Nexsim** (director 2026-08-08 ×3) → [[project_nexsim_sv_signoff_delivery_focus]] (**read its 2026-08-08 section first**). A finding in another family is ROUTED to a parked leaf, never worked; the only exception is a defect that BLOCKS the SV release. ⛔ PARKED, not abandoned: `CORPUS-GRAD-ALL.2` VHDL (31.6 %, worklist queued), `LIVE-DOC-CONTAINMENT.4`, `LIVE-MEANS-LIVE`, `SV-EXH-PROOF.7.4.6.18`.
- **next_action**: next `.3` family from the REGENERATED live worklist (**284 rows/177 sigs/7 families**) on a *token-level tell* — ⛔ NOT the bucketer, NOT one signature (`.3.23`: both undercount one construct 7/9). ⭐ `.3.24` ANSWERED the sizing: **95.1 % of the backlog is PARSER work**, adjudicator-from-metadata is 0 — so keep cutting construct leaves. Coverage half = `.7c` (**92.3 %/104 gaps**), never `.7a`; reports SELF-DATE — re-hash before quoting. Axis 1 is literal 0 over a FROZEN universe. ⛔⛔ [[a-rising-pass-rate-is-not-evidence-of-correctness]]; no family is at 100 % on axis 2.
- **in_flight_uncommitted**: none.
- **blockers**: none. ⏳ 3 director calls open (⛔ all OBJECTIVE/SCOPE → [[feedback_answer_your_own_technical_questions]]): `LESSON-RETRIEVAL.2` scope; a priced `schedule:` lane for `sota_exit_gate`; the ANVIL README-policy note.
- ⚠️ **standing tripwires — 8 live traps, and every one of them fails SILENTLY or in the PASSING direction** (column-0 comment DELETES grammar alternatives; corpus timeout ≠ parser verdict; `TASK-ACCEPTANCE` vacuous on `- ID:` trees; 3 gates RED/blind on HEAD; rule-census move ⇒ re-baseline the cert contracts SAME commit; `clippy_on_rust_change` prints ✅ and SKIPS; sweep traps): ⛔ **read [[project_standing_tripwires]] BEFORE acting** — it names each owning leaf.
- **live doctrine anchor** — a DELIBERATE class-(b) pinned duplicate, legal because `REGEX-ORACLE-ANCHOR-SYNC` fails on drift: regex oracle real tuple **`2189/1879/262/48`**.
- **flow health**: `sota_exit_gate` not re-proven end-to-end since `CI-PARITY-GATE-ROT.7`; automatic CI tier = the **17** doctrines per push + 3 cheap gate targets, the other ~120 `make` targets operator-invoked.
