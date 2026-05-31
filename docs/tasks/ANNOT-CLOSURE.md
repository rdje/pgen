# ANNOT-CLOSURE: Annotation 100% Closure (Return + Semantic) (Phase L)

## Metadata

- Tree ID: `ANNOT-CLOSURE`
- Status: `proposed`
- Roadmap lane: `Phase L — Annotation 100% Closure (return + semantic)`
- Created: `2026-05-31`
- Last updated: `2026-05-31`
- Owner: repo-local workflow

## Goal

Drive return-annotation + semantic-annotation support to documented 100%
closure across the platform, per
`docs/reference/PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md`. LIVE status:
`return_annotation` = Done; "Cross-grammar return-AST shaping adoption" =
Mostly Done — this tree owns the remaining closure gap to 100%.

## Non-Goals

- Not a re-do of the shipped annotation system; this closes the documented
  remaining-percentage gap.
- New annotation features only when existing primitives are proven insufficient
  (per the no-workarounds hierarchy).

## Acceptance Criteria

- The annotation closure roadmap's remaining items each map to a leaf and land.
- `return_annotation_support_gate` + `annotation_contract_gate` +
  `annotation_stimuli_quality_gate` + `semantic_full_contract_gate` remain green.
- Cross-grammar return-AST shaping adoption reaches its documented 100% bar.
- Each completed leaf committed through `COMMIT.md`; any code change owns a leaf.

## Task Tree

- ID: `ANNOT-CLOSURE`
  Status: `proposed`
  Goal: `Annotation (return + semantic) 100% closure per Phase L roadmap.`
  Children: `ANNOT-CLOSURE.1`

- ID: `ANNOT-CLOSURE.1`
  Status: `pending`
  Goal: `SCOPING (pure docs): read PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md + the current LIVE "Cross-grammar return-AST shaping adoption: Mostly Done" row; enumerate the precise remaining closure items (which grammars/constructs are short of 100%) into an ordered leaf plan. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Remaining-closure-item list + ordered leaf plan recorded.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `ANNOT-CLOSURE.1` | `pending` (proposed) | Must enumerate the exact remaining-to-100% items before any closure code. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2`. The annotation system is largely shipped (return_annotation Done); this tree owns only the documented residual gap to 100%.

## Open Questions

- Which specific grammars/constructs are short of the 100% shaping bar? (resolve in `.1`)

## Blockers

- None (Mostly Done; residual closure work).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `ANNOT-CLOSURE.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `ANNOT-CLOSURE.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
