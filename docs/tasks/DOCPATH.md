# DOCPATH: Repo-relative path hygiene + enforcement for live surfaces

## Metadata

- Tree ID: `DOCPATH`
- Status: `active`
- Roadmap lane: cross-cutting docs / continuity hygiene — every repo-internal file
  path reference in a live/maintained surface must be repo-root-relative, never a
  checkout-specific absolute path that captures a local home directory
- Created: `2026-06-07`
- Owner: repo-local workflow

## Goal

Guarantee — by mechanical enforcement, not discipline — that no live/maintained
documentation surface carries a repo-**internal** absolute path (an absolute path rooted at
a local checkout, i.e. `<home>/.../pgen/...` expressed instead of a repo-root-relative
path). The user's only
window into the project is the docs/book; a leaked local path is non-portable noise that
breaks on any other clone, harness, or machine.

The principle was first applied by the bare slice `PGEN-DOCPATH-0001` (it relativized the
top-level book + user guide and added the first guard in
`scripts/check_diagnostics_and_docpaths.sh`, part 2). This tree extends that guard to the
**rest** of the live-surface set the director named — task-trees, decision records, the
Knowledge Map, and the live status tracker — and fixes the residual leaks those surfaces
still carry.

## Non-Goals

- **Append-only history** (`CHANGES.md`, `DEVELOPMENT_NOTES.md`): not rewritten.
  Rewriting the recovery/audit trail is itself an anti-pattern (`MEMORY_ARCHITECTURE.md`
  §12); those files record commands actually run and external-bug-report provenance.
- **Repo-EXTERNAL paths** (IEEE LRM PDFs under `~/Documents/github/*.pdf`, sibling repos
  `rgx`/`nexsim`/`specforge`, `~/Downloads/...`): these point **outside** the repo root,
  so they have **no** repo-relative equivalent. They are left as historical provenance.
  (Tool-backed split at creation: 313 tracked `/Users/richarddje` occurrences = 20
  repo-internal, 293 repo-external.)
- **Code / grammar / generated internal paths** (`grammars/*.ebnf` provenance headers,
  `rust/src/bin/pgen.rs`, `tools/lrm_optional_audit.py`, generated provenance JSON/JSONL,
  telemetry `.env`): out of scope for `.1`; the director chose "live surfaces only"
  (2026-06-07). Captured as `DOCPATH.2` (`deferred`) so the residual is tracked, not lost
  — these are code changes and would need their own task-tree leaf when prioritized.

## Acceptance Criteria

- The live-docs repo-relative guard in `scripts/check_diagnostics_and_docpaths.sh` (part 2)
  covers `docs/tasks/**`, `docs/decisions/**`, `KNOWLEDGE_MAP.md`, `docs/knowledge/**`,
  and `LIVE_ACHIEVEMENT_STATUS.md` in addition to its existing scope (`docs/book/src/**`,
  `docs/contracts/**`, `PGEN_USER_GUIDE.md`, `README.md`).
- The residual repo-internal absolute paths those new surfaces carry are relativized.
- The guard runs clean (`diagnostics+docpaths: OK`) on the tree.
- The book (`docs/book/src/developer-architecture.md`) documents the guard's docpath
  enforcement role and its surface coverage (previously only the severity role was noted).
- No append-only-history or repo-external reference is touched.

## Task Tree

- ID: `DOCPATH`
  Status: `active`
  Goal: mechanical guarantee that live/maintained surfaces carry no repo-internal
  absolute paths
  Children: `DOCPATH.1`, `DOCPATH.2`

- ID: `DOCPATH.1`
  Status: `done`
  Goal: extend the repo-relative guard to the full live-surface set (task-trees, decision
  records, Knowledge Map, live status) and relativize the residual leaks it surfaces
  Acceptance: guard scope expanded; the 2 decision-record leaks relativized; guard prints
  `diagnostics+docpaths: OK`; book documents the docpath role; history/external untouched
  Verification: see Verification Log
  Commit: `PGEN-DOCPATH-0002, leaf DOCPATH.1`

- ID: `DOCPATH.2`
  Status: `deferred`
  Goal: relativize the repo-internal absolute paths embedded in code/grammar/generated
  surfaces (`grammars/*.ebnf` provenance headers, `rust/src/bin/pgen.rs`,
  `tools/lrm_optional_audit.py`; regenerate generated provenance JSON/JSONL + telemetry
  `.env` with relative paths)
  Acceptance: those surfaces carry no repo-internal absolute path; generated artifacts are
  regenerated (not hand-edited) from a generator that emits relative paths
  Reason deferred: director chose "live surfaces only" (2026-06-07). These are code
  changes (grammars are code); activate as a dedicated leaf when prioritized.
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `DOCPATH.1` | `done` | completed 2026-06-07 |
| (deferred) | `DOCPATH.2` | `deferred` | director scoped out for now; activate on demand |

The frontier is empty for active work: `.1` is `done`, `.2` is `deferred` by director
decision. The tree stays `active` only as the home for `.2` should it be prioritized.

## Decisions

- `2026-06-07`: Scope = **live surfaces only** (director). Relativize repo-internal paths
  in live docs + extend the enforcement guard to the named surfaces; leave append-only
  history and repo-external references as historical provenance. Rationale: the
  repo-relative target only exists for repo-internal paths; rewriting history corrupts the
  recovery trail for no portability gain.
- `2026-06-07`: Couple the guard-scope extension with the residual-leak fix in one leaf —
  the guard runs in pre-commit, so the surfaces it newly covers must already be clean or
  the commit self-blocks. Verified tools-first that only the 2 decision-record leaks
  appear under the expanded scope before wiring it.
- `2026-06-07`: This guard is enforcement tooling, owned by a task tree (consistent with
  `MEMORY-ARCH` and `DIAG-SEVERITY`, which both created their gate scripts under a tree)
  rather than a bare doc slice.

## Open Questions

- None blocking. `DOCPATH.2` (code/grammar/generated internal paths) awaits director
  prioritization; it does not block `.1`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-07` | `DOCPATH.1` | `scripts/check_diagnostics_and_docpaths.sh` over expanded scope | `diagnostics+docpaths: OK` |
| `2026-06-07` | `DOCPATH.1` | `scripts/check_memory_architecture.sh` | `pending → recorded at commit` |
| `2026-06-07` | `DOCPATH.1` | guard grep over the 9 live surfaces shows 0 repo-internal `/pgen/` paths post-fix | `pending → recorded at commit` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `DOCPATH.1` | `PGEN-DOCPATH-0002, leaf DOCPATH.1` | guard scope + 2 leak fixes + book lockstep |

## Changelog

- `2026-06-07`: Created task tree; promoted the `PGEN-DOCPATH-0001` bare-slice family to a
  tree so the guard-script extension is task-tree-owned. Implemented `DOCPATH.1`; deferred
  `DOCPATH.2` per director scope decision.
