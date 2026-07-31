# COMPILER-ELABORATOR: Compiler & Elaborator Workbench Enablement (Phase U)

## Metadata

- Tree ID: `COMPILER-ELABORATOR`
- Status: `proposed`
- Roadmap lane: `Phase U — Compiler And Elaborator Workbench (PGEN-as-front-end-substrate for downstream compilers/elaborators)`
- Created: `2026-05-31`
- Owner: repo-local workflow

## Goal

Make PGEN a front-end **workbench** that materially accelerates compiler/
elaborator creation (NOT whole-compiler auto-generation): lossless/shaped
front-end products with source spans + stable node ids, generated
visitors/walkers/typed query helpers, semantic-bundle export reused as
compiler/elaborator substrate, and elaborator-oriented handoff scaffolding
(constant-expression capture, substitution, dependency, connectivity).
Detailed plan: `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`.

## Non-Goals

- PGEN owns front-end structure/provenance/seeds/helpers/handoff surfaces ONLY.
  Downstream passes still own binding, typing, optimization, scheduling,
  back-end logic.
- No ad hoc language-specific magic — cross-language workbench infrastructure.

## Acceptance Criteria

- "PGEN as front-end workbench" doctrine frozen.
- Front-end bundle shape (CST/token fidelity + shaped AST + spans + stable node
  ids) strengthened.
- Generated visitors/walkers/typed query helpers.
- First elaborator scaffolding (const-expr capture/substitution/dependency/
  connectivity handoff) — note `rtl_const_expr` is a natural first proving input.
- Each completed leaf committed through `COMMIT.md`; any code change owns a leaf.

## Task Tree

- ID: `COMPILER-ELABORATOR`
  Status: `proposed`
  Goal: `PGEN as front-end workbench for downstream compilers/elaborators (Phase U).`
  Children: `COMPILER-ELABORATOR.1`

- ID: `COMPILER-ELABORATOR.1`
  Status: `pending`
  Goal: `SCOPING (pure docs): turn PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md milestones into an ordered leaf plan; map which front-end products (shaped AST, spans, node ids, _meta carrier, semantic bundles) already exist vs. missing. Shares substrate with LINTER (Phase T) — note the overlap. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Ordered milestone→leaf plan + existing-vs-missing front-end-product analysis + LINTER overlap noted.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `COMPILER-ELABORATOR.1` | `pending` (proposed) | Scoping precedes any code; lane is planned/not-started; shares substrate with LINTER. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2` so the Phase U (PGEN-as-service-provider) lane is task-tree-tracked. Not started; activate when prioritized. Shares the semantic-seed/provenance substrate with `LINTER` (Phase T) — sequence them together.

## Open Questions

- How much front-end-product infrastructure (stable node ids, traversal helpers) already exists in the AST pipeline vs. needs building? (resolve in `.1`)

## Blockers

- None (planned).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `COMPILER-ELABORATOR.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `COMPILER-ELABORATOR.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
