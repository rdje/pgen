# Repo-Local Task Tree Workflow (PGEN)

This document defines the repo-local task-tree workflow used by PGEN.
For a step-by-step setup guide reusable by another project, read
[docs/TASK_TREE_README.md](TASK_TREE_README.md).

## Purpose

Use a task tree when a top-level task is too broad to finish safely as one
signoff-level slice, or when a task is expected to discover subtasks and
sub-subtasks over time.

The goal is not to create a second roadmap. The roadmap (split across
`README.md`, `LIVE_ACHIEVEMENT_STATUS.md`, and the per-parser integration
contracts under `docs/contracts/`) states the high-level workstream direction.
A task tree owns the recursive breakdown, current frontier, acceptance
criteria, blockers, decisions, validation, and completion evidence for one
top-level task.

PGEN already uses per-slice IDs of the form `PGEN-<FAMILY>-<NNNN>` in commit
subjects (e.g. `PGEN-SVP-0114`, `PGEN-VHDL-0001`, `PGEN-PIP-001`). Those
remain the unit of commit traceability. When a slice belongs to a task-tree
leaf, the commit subject or first body line also names the leaf ID
(e.g. `VHDL-MDBOOK.2`), so the slice ID and the tree-node ID coexist on the
same commit.

## Active Task Trees

The eight `*-MDBOOK` / `*-CONTRACT-BODY` trees and the
`systemverilog`-book `DOC-ENVELOPE-0001` closeout are complete
(`2026-05-16`). The inline-alternation parser-correctness lane is now
decomposed into the `INLINE-ALT-FIX` task tree (multi-slice, structured,
released-parser-behavior). The remaining `DOC-README-SHELL-0001`
README/COMMIT.md `SHELL=` reconciliation stays a non-task-tree
single-slice lane (`PGEN-WORKFLOW-<NNNN>`). See
`LIVE_ACHIEVEMENT_STATUS.md` for the live snapshot.

| Tree | Status | Roadmap lane | Current frontier | File |
| --- | --- | --- | --- | --- |
| `INLINE-ALT-FIX` | `active` | parser-correctness (released-parser defect class) | `INLINE-ALT-FIX.2` | [docs/tasks/INLINE-ALT-FIX.md](tasks/INLINE-ALT-FIX.md) |

## Proposed Task Trees

Proposed trees record accepted backlog direction, but they are not
PNT-eligible until explicitly activated.

| Tree | Status | Roadmap lane | Proposed first leaf | File |
| --- | --- | --- | --- | --- |
| `POST-SV-AUDIT` | `proposed` | shape audit | `POST-SV-AUDIT.1` | (not created — see TaskList #49 placeholder) |

## Completed Task Trees

| Tree | Status | Roadmap lane | Completed frontier | File |
| --- | --- | --- | --- | --- |
| `VHDL-MDBOOK` | `done` | vhdl deliverables | all leaves `.1`–`.6` `done` (`2026-05-16`) | [docs/tasks/VHDL-MDBOOK.md](tasks/VHDL-MDBOOK.md) |
| `RTL-FE-MDBOOK` | `done` | rtl_frontend deliverables | all leaves `.1`–`.6` `done` (`2026-05-16`) | [docs/tasks/RTL-FE-MDBOOK.md](tasks/RTL-FE-MDBOOK.md) |
| `RTL-CE-MDBOOK` | `done` | rtl_const_expr deliverables | all leaves `.1`–`.6` `done` (`2026-05-16`); .4 surfaced PGEN-RTL-0002 | [docs/tasks/RTL-CE-MDBOOK.md](tasks/RTL-CE-MDBOOK.md) |
| `SVPP-MDBOOK` | `done` | sv_preprocessor deliverables | all leaves `.1`–`.6` `done` (`2026-05-16`); .4 surfaced SVPP-0001 | [docs/tasks/SVPP-MDBOOK.md](tasks/SVPP-MDBOOK.md) |
| `VHDL-CONTRACT-BODY` | `done` | vhdl deliverables | all leaves `.1`–`.4` `done` (`2026-05-16`); VHDL book DOC-ENVELOPE-0001 closed in lockstep | [docs/tasks/VHDL-CONTRACT-BODY.md](tasks/VHDL-CONTRACT-BODY.md) |
| `RTL-FE-CONTRACT-BODY` | `done` | rtl_frontend deliverables | all leaves `.1`–`.4` `done` (`2026-05-16`); rtl_frontend book DOC-ENVELOPE-0001 closed in lockstep (7 chapters) | [docs/tasks/RTL-FE-CONTRACT-BODY.md](tasks/RTL-FE-CONTRACT-BODY.md) |
| `RTL-CE-CONTRACT-BODY` | `done` | rtl_const_expr deliverables | all leaves `.1`–`.3` `done` (`2026-05-16`); rtl_const_expr book DOC-ENVELOPE-0001 closed in lockstep (7 chapters, Slice-2); .3 added literal/identifier shapes + Companion Documentation + Gate Recipe + Glossary | [docs/tasks/RTL-CE-CONTRACT-BODY.md](tasks/RTL-CE-CONTRACT-BODY.md) |
| `SVPP-CONTRACT-BODY` | `done` | sv_preprocessor deliverables | all leaves `.1`–`.4` `done` (`2026-05-16`); sv_preprocessor book DOC-ENVELOPE-0001 closed in lockstep (8 chapters, Slice-2); .2 AST Envelope + pp_item dispatch, .3 conditional tree + macro fragments, .4 Companion Documentation + Gate Recipe + Glossary | [docs/tasks/SVPP-CONTRACT-BODY.md](tasks/SVPP-CONTRACT-BODY.md) |

## Coverage Note

The SV typing campaign (~116 slices, completed before this workflow landed)
is intentionally NOT retrofitted into a task tree. Its history lives in
`CHANGES.md`, the per-slice commit log, and the calibration_history field of
`rust/test_data/ast_shape_contract/systemverilog_v1.json`. Only future
multi-slice lanes adopt task-tree decomposition.

The remaining typing campaigns (regex/SV/SV-preprocessor/VHDL/rtl_const_expr/
rtl_frontend) likewise stay outside the task-tree ledger — they completed
ahead of this workflow installation. Their slice IDs and CHANGES.md entries
remain the canonical record.

## Directory Layout

```text
docs/TASK_TREE_README.md
docs/TASK_TREE.md
docs/tasks/
  TEMPLATE.md
  <TREE>.md
```

`docs/TASK_TREE.md` is the workflow and active-tree index.
Each top-level task owns one file in `docs/tasks/`.
`docs/tasks/TEMPLATE.md` is copied when creating a new top-level tree.

## Definitions

- Task tree: the recursive decomposition of one top-level task.
- Node: one item in that tree.
- Container node: a node with children. It is not directly executable.
- Leaf node: a node with no children. It is the only unit PNT may implement.
- Current frontier: the ordered set of leaf nodes that are eligible to be
  picked next.
- Slice: one completed leaf task plus its tests, docs, live-doc updates, and
  commit workflow.
- Evidence: the validation output, changed-doc summary, and git commit subject
  that prove a leaf was completed.

## ID Rules

Each task tree has a stable top-level ID.

```text
<TREE>
<TREE>.1
<TREE>.1.1
<TREE>.1.1.1
```

Rules:

- `<TREE>` uses uppercase letters, digits, and hyphens.
- Child IDs append dot-separated positive integers.
- IDs are permanent once published.
- Never renumber closed nodes.
- If a new ordering is needed, add new IDs and mark old nodes `superseded` or
  `deferred` with a reason.
- A commit that completes a task-tree leaf must identify the leaf ID in the
  commit subject or in the first body line, alongside the slice ID where
  applicable.

## Status Vocabulary

Use only these statuses.

| Status | Meaning |
| --- | --- |
| `proposed` | Captured but not yet accepted into the active tree. |
| `active` | The top-level tree is open, or a container has unfinished children. |
| `pending` | Ready to be selected once it reaches the current frontier. |
| `in_progress` | Currently being implemented in the worktree. |
| `blocked` | Cannot proceed without a named blocker and unblock condition. |
| `done` | Completed, validated, documented, and committed. |
| `deferred` | Deliberately postponed with an explicit consequence. |
| `superseded` | Replaced by another node, with the replacement ID named. |

## Required Task File Sections

Every top-level task file must contain:

- Metadata: tree ID, status, roadmap lane, created date, last updated date.
- Goal: the user-visible or project-visible outcome.
- Non-goals: what this tree deliberately does not try to solve.
- Acceptance criteria: concrete conditions that close the top-level task.
- Task tree: all known nodes, with status and short result intent.
- Current frontier: ordered leaf nodes that PNT may select next.
- Decisions: accepted technical decisions and their rationale.
- Open questions: unresolved questions that do not block the whole tree yet.
- Blockers: blockers with unblock conditions.
- Verification log: checks run for completed leaves.
- Commit log: leaf IDs mapped to completion commit subjects.
- Changelog: dated edits to the tree itself.

## Node Rules

Every node must be one of these two shapes.

Container node:

```text
- ID: <TREE>.<n>
  Status: active
  Goal: ...
  Children: <TREE>.<n>.1, <TREE>.<n>.2
```

Leaf node:

```text
- ID: <TREE>.<n>
  Status: pending
  Goal: ...
  Acceptance: ...
  Verification: pending
  Commit: pending
```

A node with children must not be marked `done` until every child is `done`,
`deferred`, or `superseded`, and every non-`done` child has a recorded reason.

## Current Frontier Rules

The current frontier is the only list PNT uses when selecting work from a task
tree.

Rules:

- The frontier contains only leaf nodes.
- The frontier is ordered by intended priority.
- A container never appears in the frontier.
- A blocked node stays out of the frontier until unblocked.
- When a leaf is split, remove that leaf from the frontier, mark it `active`,
  add children, and place the first executable child or children in the
  frontier.
- When a leaf completes, remove it from the frontier and add the next eligible
  leaf or leaves.

## PNT Selection Rules

When PNT is asked to continue and at least one active task tree exists:

1. Read `docs/TASK_TREE.md`.
2. Read the active task file named in the `Active Task Trees` table.
3. Pick the first eligible leaf in that file's `Current Frontier`.
4. Implement only that leaf.
5. If the leaf is too broad, split it before implementation and commit the
   tree update as the leaf's honest outcome.
6. Run the required validation for the leaf.
7. Update the task file, live docs, and roadmap if status changed.
8. Run the full commit workflow before selecting another leaf.

If several active trees exist, choose the first active tree in the table unless
the user names another tree or live-status names a different immediate lane.

Slice-level mechanical work (e.g. annotating N similar rules in one grammar)
does NOT have to be promoted into a task tree if it fits as a single slice
with one PGEN-`<FAMILY>`-`<NNNN>` ID. Task-tree decomposition is for
multi-slice lanes where structure helps.

## Splitting Rules

Split a node when any of these are true:

- It cannot be completed to signoff quality in one slice.
- It mixes design, implementation, diagnostics, tests, and docs in ways that
  can be reviewed independently.
- It hides an unresolved policy choice behind implementation wording.
- It would require touching unrelated ownership areas in one commit.
- It discovers a lower-level dependency that should be solved first.

Do not split merely to create vague placeholders. Every child must have a
clear goal and a way to verify completion.

## Completion Rules

A leaf is complete only when all of the following are true:

- Implementation or documentation work for that leaf is finished.
- Focused checks passed, and broader checks ran when warranted.
- The owning task file records the result, validation, and commit subject.
- `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
  `LIVE_ACHIEVEMENT_STATUS.md` are updated when the leaf changes project
  state.
- The commit workflow in `COMMIT.md` has completed.
- `git_message_brief.txt` (if used) has been cleared after commit.

Commit hashes are intentionally not required inside the same task-file update:
the final hash cannot be known until after the commit exists. The stable
join key is the leaf ID in the commit subject or first body line. Later status
refreshes may backfill hashes if useful.

## Blocker Rules

A blocked node must record:

- the exact blocker,
- why it blocks the node,
- the unblock condition,
- and the next task that should run instead, if any.

Do not leave a node as `blocked` only because it is large or unclear. Large or
unclear work should be split until a real blocker is visible.

## Relationship To Live Docs

The task tree is the detailed execution ledger.

- `LIVE_ACHIEVEMENT_STATUS.md` remains the canonical high-level workstream
  status.
- `MEMORY.md` remains the recovery/handoff continuity log.
- `CHANGES.md` remains the chronological technical history.
- `DEVELOPMENT_NOTES.md` remains design rationale.
- The per-parser-family contracts under `docs/contracts/` remain the
  downstream-consumer integration surface.
- The per-parser mdBooks under `docs/<grammar>_parser_book/src/` remain the
  user-facing reference.

Do not duplicate the whole task tree into those files. Link to the task tree
and summarize only the part that changes live project state.

## Slice ID + Leaf ID Convention

Commits associated with task-tree leaves follow this form:

```text
<short-subject> (PGEN-<FAMILY>-<NNNN>, leaf <TREE>.<path>)

<long body explaining what was done, validation, etc.>
```

The PGEN slice ID stays the unit of commit-log indexing. The leaf ID joins
the task tree to the slice.
