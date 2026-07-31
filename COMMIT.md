# COMMIT.md

Last updated: 2026-05-14

## Purpose
Define the exact commit workflow for this project so a new AI instance can apply it consistently without re-reading chat history.

## When To Run
Run this workflow after each completed task/activity.

## Task-Tree Workflow Rule
When the completed activity belongs to a task-tree leaf (i.e. a leaf node from a file under `docs/tasks/`):

- Update the owning `docs/tasks/<TREE>.md` file: leaf status, verification log, commit log entries, frontier, decisions, blockers as applicable.
- Update `docs/TASK_TREE.md` Active Task Trees table only if the current frontier changes.
- The commit subject or first body line must name the leaf ID alongside the PGEN slice ID (example: `VHDL-MDBOOK-Slice-1 (PGEN-VHDL-MDBOOK-0001, leaf VHDL-MDBOOK.1): scaffold book.toml + SUMMARY`).
- One commit per completed leaf before selecting another leaf.

**Code-Change Doctrine (binding, non-negotiable — 2026-05-17):** it is strictly forbidden to make ANY code change (`grammars/*.ebnf`, Rust, codegen, generated artifacts, `rust/test_data/ast_shape_contract/*.json` manifests, or anything altering parser/codegen/generated behavior) unless it is first tracked/owned by a task-tree leaf. A task-tree leaf MUST exist and own the change before code is touched; then implement only that leaf and run this workflow. Rationale: task-tree ownership improved code review and code quality tremendously (user, 2026-05-17). See `docs/TASK_TREE.md` "Code-Change Doctrine".

When the activity is NOT a code change (pure live-docs/contracts/books/tracker/workflow-doc edits — a one-shot doc fix or single mechanical doc slice not promoted to a tree), the standard slice-ID convention `PGEN-<FAMILY>-<NNNN>` is sufficient and no `docs/tasks/` update is required. This carve-out does **not** apply to code changes — those always require a task-tree leaf first (the prior "a one-shot code fix may skip the tree" reading is superseded).

## Files Involved
- `README.md` (tracked)
  - Single project entrypoint and navigation hub — a **stable landing page**, governed by
    `docs/reference/PGEN_README_STABILITY_POLICY.md` and mechanically capped by the
    `README-STABILITY` doctrine (line cap AND byte cap) since 2026-07-30 (`README-POLICY.1`).
  - Update it only when its purpose, first-use path, top-level architecture, or canonical
    navigation changes.
  - ⛔ Ordinary feature work updates the CANONICAL DESTINATION, not the README: gate recipes ->
    `docs/book/src/gate-flow.md`, operational procedure ->
    `docs/book/src/operations-and-governance.md`, the repository path inventory ->
    `docs/book/src/developer-architecture.md`, family status -> the DONE-BAR register, history ->
    `CHANGES.md`. A cap is NEVER raised to land content.
- `rust/test_data/grammar_quality/done_bar_family_register_v0.json` (tracked) — **the live status CLAIM**
  - Each family's `claimed_status` is the authoritative, HAND-AUTHORED live progress claim. It moved
    here from `LIVE_ACHIEVEMENT_STATUS.md` in `LIVE-MEANS-LIVE.1a`, which was deleted in `.1c3`
    after reaching 1 547 057 B of which 94.7 % was a dated changelog. A schema-bounded field cannot
    rot that way; a free-form Markdown file with no cap is what let it.
  - ⛔ **NEVER auto-populate `claimed_status` from a gate.** It is one arm of a two-arm check — the
    three `*_parser_family_status_gate.sh` gates COMPUTE the status from proof surfaces and fail on
    disagreement. Generating the claim makes the comparison pass by construction.
  - Must use only `Done`, `Provisional (ceiling)`, `Provisional (corpus pending)`, `Mostly Done`, `In Progress`, and `Not Started` (the `Provisional` pair added 2026-07-29 — a SHIPPING tier for a family meeting legs 1-2 of the `Done` bar but not leg 3; `(ceiling)` = leg 3 unreachable by construction, a FINISHED row; `(corpus pending)` = a recognized corpus exists in the world and wiring it is outstanding. ⛔ Bare `Provisional` is incomplete, and `(ceiling)` requires the language be defined by PGEN itself — "could not find a corpus" is never a ceiling. The rules live in `docs/book/src/quality-and-closure-model.md`).
  - Must be reviewed and updated before every commit whenever actual closure or remaining scope changes.
  - The current live-status snapshot must be summarized in every user-facing completion message produced by the commit workflow.
  - If any live-status row changes, the completion message must also state how the task affected that snapshot.
- `docs/book/src/roadmap-and-live-status.md` (tracked) — **the published VIEW of that claim**
  - The human at-a-glance per-family table. `scripts/check_published_version_currency.sh`
    (`PUBLISHED-VERSION-CURRENCY`) holds it equal to the register family-by-family, in BOTH
    directions, so a status edit that updates only one of the two FAILS the commit.
  - ⇒ edit the register first, then this table.
- `git_message_brief.txt` (must remain untracked)
  - Short, concise commit message file.
  - Used with `git commit -F git_message_brief.txt`.
  - Must be cleared to 0 bytes after commit.
- `CHANGES.md` (tracked)
  - Changelog-style summary of completed work and validation.
  - Internal continuity / implementation-history surface, not primary public documentation.
- `DEVELOPMENT_NOTES.md` (tracked)
  - Detailed technical notes: root cause, implementation, validation.
  - Internal engineering continuity surface, not primary public documentation.
- `MEMORY.md` (tracked)
  - Live continuity file for resume/handoff.
  - Internal continuity only.
- `docs/reference/RUST_CODEBASE_ANALYSIS.md` (tracked)
  - Live Rust architecture/state assessment.
  - Must be reviewed and updated whenever a task materially changes Rust architecture, major subsystem boundaries, public integration seams, or the current high-level risk/steering picture.
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md` and `docs/contracts/PGEN_*_PARSER_INTEGRATION_CONTRACT.md` (tracked)
  - Versioned downstream handoff docs for parser families.
  - Must be reviewed and updated whenever a task changes a published parser family's stable integration surface, build/availability requirements, validation gate, or externally stated support boundary.
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` and `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (tracked)
  - Downstream parser support/bug-tracking workflow.
  - Must be reviewed and updated whenever the required bug-report bundle, release-support process, or released-parser bug state changes.
- `docs/book/` (`mdBook` source, tracked)
  - Primary public-facing documentation surface for users and developers.
  - Intended to become the comprehensive outward documentation system for the project.
  - Must be reviewed and updated whenever a task changes a user-facing command, workflow, contract, parser-family support boundary, developer architecture seam, rationale, or documentation map that is covered by an existing chapter.
  - `make -C rust SHELL=/bin/bash mdbook_docs_gate` is the maintained proof lane for this surface.
- `questions_keep_untracked.txt` (must remain untracked)
  - User backlog/questions for future UG work.
- `generated/` artifacts
  - Repository policy: the `generated/` tree is **not tracked in git** (it is `.gitignore`d) — it is regenerated locally from the grammars + codegen with the per-grammar `make` targets (e.g. `make -C rust focus_<grammar>`). This matches `docs/book/src/developer-architecture.md` (Repository Layout), which is where the path inventory lives since `README-POLICY.1`.
  - Because `generated/` is untracked, do **not** stage or commit generated artifacts:
    - never `git add generated/…` — regenerate locally to verify a codegen/grammar change, but the regenerated `generated/*` files are not part of the commit,
    - keep scratch logs and test-only temporary outputs out of `generated/`.
- Markdown path policy
  - Repo-internal paths mentioned in tracked `.md` files must be relative paths, never checkout-specific absolute paths.
  - This applies to links, prose path references, and command examples when they point into the PGEN repository.

## Required Commit Workflow (Exact Order)
1. Ensure task is complete and tested.
2. Run clippy flow when Rust/generated Rust files are amended:
   - `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change`
   - strict source lint must pass.
   - the generated-parser lint stage is **STRICT BY DEFAULT** since `GENERATED-LINT-CORRECTNESS.3`
     (`PGEN_CLIPPY_GENERATED_STRICT` defaults to `1`). `clippy::correctness` lints are
     deny-by-default, so this stage is what holds the generated-parser correctness floor at 0.
     - a failure here is a real defect: per the `-0001` adjudication the fix is to change the
       **emission at its codegen site**, never an `#[allow]` and never lowering the lint,
     - `PGEN_CLIPPY_GENERATED_STRICT=0` drops back to advisory mode; that is the posture that
       let 291 correctness errors accumulate unnoticed, so it is loud rather than silent.
   - the flow also runs `generated_clippy_correctness_gate.sh --policy-only`, which verifies the
     generated artifacts are present and that every lint pinned in
     `rust/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json` is still a
     member of `clippy::correctness`. That is the one thing the clippy run itself cannot see — a
     lint DEMOTED out of deny-by-default would silently stop being checked. It needs no cargo
     invocation, so it costs nothing.
   - full explicit-deny run (used by the hosted workflow, and available deliberately):
     - `make -C rust SHELL=/bin/bash generated_clippy_correctness_gate`
3. Update tracked docs as needed (`CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/reference/RUST_CODEBASE_ANALYSIS.md`, `README.md`, the DONE-BAR register + its book view, others touched by task).
   - Treat markdown synchronization as systematic, not optional:
     - always review the tracked continuity/workflow markdown surface before commit,
     - always update every relevant tracked `.md` file touched by the task or affected by its workflow/policy/command/documentation impact,
     - do not leave a relevant markdown file stale just because code/tests already passed.
   - `docs/reference/RUST_CODEBASE_ANALYSIS.md` review/update is mandatory before each commit when the task materially changes:
     - Rust architecture,
     - major subsystem boundaries,
     - public integration surfaces,
     - or the current high-level implementation/risk assessment of the Rust codebase.
   - `rust/test_data/grammar_quality/done_bar_family_register_v0.json` review/update (and the book view in lockstep) is mandatory before each commit whenever the task changes:
     - what is `Done`,
     - what is `Mostly Done`,
     - what is `In Progress`,
     - what is `Not Started`,
     - or what the next most important remaining gap is.
   - In every commit-workflow completion message:
     - display the current live-status snapshot from the DONE-BAR register,
     - make it clear whether the task changed that snapshot or left it unchanged.
   - When any live-status row changes:
     - update the register and its book view before commit,
     - summarize the changed status snapshot in the user-facing completion message,
     - explicitly state the effect of the completed task on the tracker.
   - When no live-status row changes:
     - still display the current live-status snapshot in the completion message,
     - explicitly say that the tracker is unchanged rather than implying a status update happened.
   - In every commit-workflow completion message, also display:
     - the commit ID,
     - the exact commit message,
     - the full list of tracked files included in that commit.
   - `README.md` sync is required when:
     - project objective/scope changes,
     - canonical generation flow changes,
     - key project paths or standard commands change,
     - markdown documentation map/ramp-up order changes.
   - `docs/book/` sync is required when:
     - a user-facing or developer-facing surface already represented by the book changes,
     - a new important surface deserves a new chapter or section,
     - the curated learning path or source-map bridge changes,
     - rationale or transparency around "what PGEN does and why" changed materially.
   - When `docs/book/` changes or when a task materially changes the curated book surface:
     - run `make -C rust SHELL=/bin/bash mdbook_docs_gate`
     - keep the book readable and curated rather than dumping raw repository notes into it,
     - but do not under-document important project behavior in the name of curation; the book is meant to become the comprehensive public surface.
   - While reviewing/updating markdown docs:
     - convert any repo-internal absolute checkout path to a relative path before commit,
     - do not leave checkout-specific absolute repo paths in tracked `.md` files.
4. Write concise commit message to `git_message_brief.txt`.
5. Stage only intended tracked files (`git add <files>`).
6. Commit with:
   - `git commit -F git_message_brief.txt`
7. Clear message file:
   - `: > git_message_brief.txt`
8. Confirm post-conditions:
   - `git ls-files --error-unmatch git_message_brief.txt` must fail (untracked).
   - `wc -c git_message_brief.txt` must be `0`.
   - `git status --short` must show expected state only.
9. In the user-facing completion message after commit, report:
   - the commit ID,
   - the exact commit message,
   - the list of tracked files included in the commit,
   - the current live-status snapshot,
   - and whether that snapshot changed or stayed unchanged.

## Pre-Commit Safety Rules
- Do not add `git_message_brief.txt` to git.
- Do not track `questions_keep_untracked.txt`.
- Do not use destructive git commands unless explicitly requested.

## Command Template
```bash
# 1) write concise commit message
cat > git_message_brief.txt <<'EOF'
<concise title>

- <brief bullet 1>
- <brief bullet 2>
EOF

# 2) run clippy flow when Rust/generated-Rust changed
make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change

# 3) stage intended files only
git add <tracked-file-1> <tracked-file-2> ...

# 4) commit
git commit -F git_message_brief.txt

# 5) clear message file
: > git_message_brief.txt

# 6) verify
wc -c git_message_brief.txt
git ls-files --error-unmatch git_message_brief.txt >/dev/null 2>&1; echo $?
git status --short
```
