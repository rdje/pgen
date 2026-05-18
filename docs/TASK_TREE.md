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

## Code-Change Doctrine (binding, non-negotiable — 2026-05-17)

**It is strictly forbidden to make any code change unless that change is
first tracked by, or owned by, a task-tree leaf.** This is the standing
doctrine going forward — no compromise, non-negotiable.

- "Code change" means any edit to: `grammars/*.ebnf`, Rust sources
  (`rust/`), codegen, generated artifacts, shape-contract manifests
  (`rust/test_data/ast_shape_contract/*.json`), or anything that alters
  parser/codegen/generated behavior. The grammar `.ebnf` files are code.
- Before touching code, a task-tree leaf must exist that owns the change
  (create/extend a tree, or add a leaf to an active one). The leaf —
  its goal, acceptance, verification, and commit — is the unit of
  review. Then implement only that leaf and run the full `COMMIT.md`
  workflow.
- **Rationale (user, 2026-05-17):** task-tree ownership improved code
  review and code quality *tremendously*. The tree's explicit
  goal/acceptance/verification/blocker structure forces the change to
  be scoped, justified, independently verified, and lock-stepped with
  docs before it lands.
- Pure non-code changes (live-docs, contracts, the books, tracker
  files, this workflow doc itself) may still use the
  `PGEN-<FAMILY>-<NNNN>` single-slice convention without a task tree —
  the doctrine governs **code** changes specifically. When in doubt
  (a change touches both), treat it as a code change and require a
  task-tree leaf.
- This supersedes the looser "a one-shot code fix not promoted to a
  tree may use the slice-ID convention" reading: one-shot code fixes
  now require a task-tree leaf too.

This doctrine is mirrored in `COMMIT.md` (Task-Tree Workflow Rule),
`DEVELOPMENT_NOTES.md`, the live-book
(`docs/book/src/quality-and-closure-model.md`), and the auto-memory
(`feedback_task_tree_workflow`).

## Active Task Trees

The 9 prior trees + the `INLINE-ALT-FIX` parser-correctness tree are
complete (`2026-05-16`→`17`); `DOC-ENVELOPE-0001` (5 books),
`DOC-README-SHELL-0001` (closed — hypothesis empirically falsified),
and `POST-SV-AUDIT` (TaskList #49 holistic AST-shape audit,
`2026-05-17`) are likewise done. On `2026-05-17` the user selected the
largest open parser-family debt from the post-`POST-SV-AUDIT`
strategic fork, and **`SV-EXH-PROOF`** was created + **activated**,
**re-scoped** (`PGEN-SV-EXH-PROOF-0001`: trio-port hypothesis
falsified), then its **`.1` measured baseline** (`PGEN-SV-EXH-PROOF-0002`,
[docs/SV_EXH_PROOF_BASELINE.md](SV_EXH_PROOF_BASELINE.md)) locked the
honest scope: SV-main static syntax-closure is already healthy
(falsification holds), but the external-corpus parse surface is
genuinely `0/14` (not the `10/14` the tracker claimed — corrected
same-commit), `sv_formal_exhaustive_closure_gate.sh:245` hard-codes
the closure-green literal, and a prerequisite **preprocessor
preprocessor proof-stack regression cascade** (lockstep defect from
this session's POST-SV-AUDIT.2.1/INLINE-ALT-FIX.1) blocks the SV
umbrella. `PGEN-SV-EXH-PROOF-0003`/`-0004` (leaves `.2.1`/`.2.2`) remediated
the syntax-closure (A1) + `pp_if_keyword` quality-coverage (A2) +
reachable-branch-universe-drift mis-spec (A3') facets — all
evidence-grounded + verified, not masked; a further facet (`.2.3`,
preprocessor closed-loop self-rejects 3 generated directive stimuli)
remains the frontier — `PGEN-SV-EXH-PROOF-0005` proved via the exact
diffs that `.2.3` is **not** campaign-caused (both campaign edits are
generatively inert; root cause = a separate not-yet-identified
non-grammar / seed-sensitive generator⊋parser asymmetry, bisect
pending). The tree (6 leaves, `.2` split into `.2.1`/`.2.2` done +
`.2.3`) re-earns `Done` for the SV main-parser family honestly. See
`LIVE_ACHIEVEMENT_STATUS.md` for the live snapshot and the other
roadmap-level workstreams (which require user prioritization — they
are large multi-week efforts, not bounded PNT slices).

| Tree | Status | Roadmap lane | Current frontier | File |
| --- | --- | --- | --- | --- |
| `RGX-0085` | `active` (**user-directed priority** — ordered first) | released-parser bug remediation (`regex`; downstream `PGEN-RGX-0085`) | `RGX-0085.1` — root cause PINNED from code (recursion ceiling exists & returns a clean error but at 4096 frames ≈ 512 nesting > 64 MiB-worker debug capacity ⇒ OS SIGABRT first; inline ≤16 on caller stack; serialize shares worker stack). Design locked: embedding-API configurable PCRE2-parity (250) pre-parse paren-nesting ceiling + lower inline threshold; global engine recursion guard UNTOUCHED ⇒ zero SV/VHDL risk | [docs/tasks/RGX-0085.md](tasks/RGX-0085.md) |
| `RGX-0086` | `proposed` (ordered second, after `RGX-0085`) | released-parser metadata-integrity (`regex`; downstream `PGEN-RGX-0086`) | stale `embedding_api.rs` `REGEX_PARSER_RELEASE_VERSION`/`…CONTRACT_VERSION` consts (`1.1.29`/`1.1.31`) vs ledger latest; sync to the ledger-latest (which `RGX-0085` will advance) + add a drift gate. Activated after `RGX-0085` closes | [docs/tasks/RGX-0086.md](tasks/RGX-0086.md) |
| `SV-EXH-PROOF` | `active` (**PAUSED for the RGX priority interrupt**; `.3.1` root cause pinned+committed `2b9b50b2`, resumes after `RGX-0085`+`RGX-0086`) | parser-family exhaustive-proof normalization (last open parser-family proof debt) | **`.2` CLOSED** (`PGEN-SV-EXH-PROOF-0011`): preprocessor regression family fully remediated — `.2.1` A1+A2 ✓, `.2.2` A3' mis-spec ✓, `.2.3.1` `SVPP-0002` grammar bug ✓ (3→2), `.2.3.2` parser/EBNF-agnostic closed-loop generator hardening ✓ (2→0) → `sv_preprocessor_zero_plausible_gap_proof_gate` verdict **GREEN** gate-verified FRESH, cross-parser no-regression, 2 downstream proof contracts re-baselined in-slice (non-masking), full lockstep. **Frontier → `SV-EXH-PROOF.3`**: SV-main grammar hardening (external-corpus 0/14 → green + replay-shadow Finding A3) — a large multi-slice workstream | [docs/tasks/SV-EXH-PROOF.md](tasks/SV-EXH-PROOF.md) |

## Proposed Task Trees

Proposed trees record accepted backlog direction, but they are not
PNT-eligible until explicitly activated.

| Tree | Status | Roadmap lane | Proposed first leaf | File |
| --- | --- | --- | --- | --- |
| _(none proposed)_ | | | | |

## Completed Task Trees

| Tree | Status | Roadmap lane | Completed frontier | File |
| --- | --- | --- | --- | --- |
| `RGX-0084` | `done` | released-parser bug remediation (`regex`; downstream `PGEN-RGX-0084`) | all leaves `.1`–`.3` `done` (`2026-05-18`); **PGEN-RGX-0084 FIXED** — bare `\NN…` octal-vs-backref PCRE2 disambiguation at parse time (single-digit always backref; N≥10 gated by groups-opened-so-far incl. named groups) via the parser-agnostic semantic-annotation mechanism; fix `b5036c4e`, full books↔code lockstep, `REGEX-0083` ledger (release 1.1.76/contract 1.1.78, schema stays 1); single-digit over-gating caught by the no-regression gate + corrected pre-commit | [docs/tasks/RGX-0084.md](tasks/RGX-0084.md) |
| `SEMREF-SHAPED` | `done` | AST-pipeline/semantic-runtime resolver correctness (shared engine; parser-agnostic) | all leaves `.1`–`.3` `done` (`2026-05-18`); parser-agnostic shaped-structure semantic-ref resolution (`$name` on a `->` rule resolves against its produced object; no-`->` byte-identical); engine `79dc494e`, no-regression `annotation_contract_gate` 41✅ + differential, behaviorally proven via the RGX-0084 consumer; unblocked RGX-0084 | [docs/tasks/SEMREF-SHAPED.md](tasks/SEMREF-SHAPED.md) |
| `VHDL-MDBOOK` | `done` | vhdl deliverables | all leaves `.1`–`.6` `done` (`2026-05-16`) | [docs/tasks/VHDL-MDBOOK.md](tasks/VHDL-MDBOOK.md) |
| `RTL-FE-MDBOOK` | `done` | rtl_frontend deliverables | all leaves `.1`–`.6` `done` (`2026-05-16`) | [docs/tasks/RTL-FE-MDBOOK.md](tasks/RTL-FE-MDBOOK.md) |
| `RTL-CE-MDBOOK` | `done` | rtl_const_expr deliverables | all leaves `.1`–`.6` `done` (`2026-05-16`); .4 surfaced PGEN-RTL-0002 | [docs/tasks/RTL-CE-MDBOOK.md](tasks/RTL-CE-MDBOOK.md) |
| `SVPP-MDBOOK` | `done` | sv_preprocessor deliverables | all leaves `.1`–`.6` `done` (`2026-05-16`); .4 surfaced SVPP-0001 | [docs/tasks/SVPP-MDBOOK.md](tasks/SVPP-MDBOOK.md) |
| `VHDL-CONTRACT-BODY` | `done` | vhdl deliverables | all leaves `.1`–`.4` `done` (`2026-05-16`); VHDL book DOC-ENVELOPE-0001 closed in lockstep | [docs/tasks/VHDL-CONTRACT-BODY.md](tasks/VHDL-CONTRACT-BODY.md) |
| `RTL-FE-CONTRACT-BODY` | `done` | rtl_frontend deliverables | all leaves `.1`–`.4` `done` (`2026-05-16`); rtl_frontend book DOC-ENVELOPE-0001 closed in lockstep (7 chapters) | [docs/tasks/RTL-FE-CONTRACT-BODY.md](tasks/RTL-FE-CONTRACT-BODY.md) |
| `RTL-CE-CONTRACT-BODY` | `done` | rtl_const_expr deliverables | all leaves `.1`–`.3` `done` (`2026-05-16`); rtl_const_expr book DOC-ENVELOPE-0001 closed in lockstep (7 chapters, Slice-2); .3 added literal/identifier shapes + Companion Documentation + Gate Recipe + Glossary | [docs/tasks/RTL-CE-CONTRACT-BODY.md](tasks/RTL-CE-CONTRACT-BODY.md) |
| `SVPP-CONTRACT-BODY` | `done` | sv_preprocessor deliverables | all leaves `.1`–`.4` `done` (`2026-05-16`); sv_preprocessor book DOC-ENVELOPE-0001 closed in lockstep (8 chapters, Slice-2); .2 AST Envelope + pp_item dispatch, .3 conditional tree + macro fragments, .4 Companion Documentation + Gate Recipe + Glossary | [docs/tasks/SVPP-CONTRACT-BODY.md](tasks/SVPP-CONTRACT-BODY.md) |
| `INLINE-ALT-FIX` | `done` | parser-correctness (released-parser defect class) | all leaves `.1`–`.3` `done` (`2026-05-16`→`17`); systemic inline-alternation-`$N` class fully resolved: `.1` SVPP-0001 (sv_preprocessor, +pp_if_keyword, 64→66), `.2` RTL-FE-0001 (rtl_frontend, 5 un-annotated op-rules, 156/74 unchanged), `.3` VHDL-0001 (vhdl, named {kind} op-rules, 249→256); all schema 1→2 / release 1.0.1→1.0.2, contract+book+ledger lockstep | [docs/tasks/INLINE-ALT-FIX.md](tasks/INLINE-ALT-FIX.md) |
| `POST-SV-AUDIT` | `done` | shape audit (TaskList #49) | all leaves `.1`/`.2.1`–`.2.4b`/`.3` `done` (`2026-05-17`); deferred holistic post-campaign AST-shape audit complete. `.1` classified ledger; `.2` per-grammar Cat-A/inline-alt fixes (svpp macro_formals; rtl_frontend 15+RTL-FE-0002; vhdl 17; sv net_alias+5-number-defensive+11-structured); `.3` Cat-C/benign/already-correct/not-an-iteration confirmed + close. Every `{first/lhs..rest:$N}` occurrence across 6 product grammars FIXED / CONFIRMED-CORRECT / RECORDED-ACCEPTED. TaskList #49 closed | [docs/tasks/POST-SV-AUDIT.md](tasks/POST-SV-AUDIT.md) |

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
