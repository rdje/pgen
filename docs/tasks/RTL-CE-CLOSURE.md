# RTL-CE-CLOSURE: rtl_const_expr Parser-Family Closure (Phase S)

## Metadata

- Tree ID: `RTL-CE-CLOSURE`
- Status: `proposed`
- Roadmap lane: `Phase S — rtl_const_expr baseline evaluator: parser-family closure (distinct from the existing RTL-CE-MDBOOK / RTL-CE-CONTRACT-BODY book/contract trees)`
- Created: `2026-05-31`
- Owner: repo-local workflow

## Goal

Drive the `rtl_const_expr` family from its current `Mostly Done` LIVE status to
PGEN closure (constant-expression parser/evaluator exhaustiveness + the
remaining documented gap). The existing `RTL-CE-MDBOOK` + `RTL-CE-CONTRACT-BODY`
trees (done) cover book + integration-contract surfaces; THIS tree owns the
remaining parser/evaluator CLOSURE.

## Non-Goals

- Not the book or integration-contract surfaces (owned by the existing RTL-CE
  trees) — this is parser/evaluator closure.
- Per Phase S rule: generated-parser closure is the bar; handwritten evaluator
  is scaffolding.

## Acceptance Criteria

- `rtl_const_expr` generated-parser + evaluator closure proven to the documented bar.
- LIVE "rtl_const_expr baseline evaluator" row → Done with evidence.
- Each completed leaf committed through `COMMIT.md`; any code change owns a leaf.

## Task Tree

- ID: `RTL-CE-CLOSURE`
  Status: `proposed`
  Goal: `rtl_const_expr parser/evaluator closure → LIVE Done.`
  Children: `RTL-CE-CLOSURE.1`

- ID: `RTL-CE-CLOSURE.1`
  Status: `pending`
  Goal: `SCOPING (pure docs): read the LIVE rtl_const_expr row + README rtl_const_expr notes to pin the exact remaining closure gap to Done; ordered leaf plan. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Remaining-to-Done gap + ordered leaf plan recorded.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RTL-CE-CLOSURE.1` | `pending` (proposed) | Must pin the remaining closure gap (LIVE = Mostly Done) before closure code. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2` to own the rtl_const_expr parser/evaluator CLOSURE gap, distinct from the done book/contract trees. Not started; activate when prioritized.

## Open Questions

- Exact closure bar for Done (which const-expr forms remain)? (resolve in `.1`)

## Blockers

- None (Mostly Done; closure deferred).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `RTL-CE-CLOSURE.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-CE-CLOSURE.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
