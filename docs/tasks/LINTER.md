# LINTER: Cross-Language Linter Enablement (Phase T)

## Metadata

- Tree ID: `LINTER`
- Status: `proposed`
- Roadmap lane: `Phase T — Cross-Language Linter Enablement (PGEN-as-substrate for downstream linters)`
- Created: `2026-05-31`
- Last updated: `2026-05-31`
- Owner: repo-local workflow

## Goal

Turn PGEN's annotation-capable EBNF pipeline into a trustworthy front-end
**substrate** for serious linters: grammar-driven semantic-seed emission,
provenance-carrying facts/events, and stable export/embedding APIs that a
downstream lint rule-engine consumes. HDL (SV/VHDL) is the first proving
ground, generalizing to any PGEN grammar. Detailed plan:
`docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`.

## Non-Goals

- PGEN does NOT own lint rules or global attribution — it emits LOCAL semantic
  seeds; the downstream linter computes global meaning + runs rules.
- No brand-new annotation language unless the existing semantic-annotation
  system is proven insufficient (prefer widening it).
- Not HDL-specific magic — infrastructure must stay cross-language.

## Acceptance Criteria

- Shared semantic-seed schema (with language overlays) frozen.
- Provenance-bearing fact + event records emitted by the runtime.
- Stable embedding + CLI export APIs for semantic bundles.
- Pilot seeds (declaration/scope/process/assignment/pragma/waiver) on
  constrained SV + VHDL subsets.
- Each completed leaf committed through `COMMIT.md`; any code change owns a leaf.

## Task Tree

- ID: `LINTER`
  Status: `proposed`
  Goal: `PGEN as trustworthy front-end substrate for downstream linters (Phase T).`
  Children: `LINTER.1`

- ID: `LINTER.1`
  Status: `pending`
  Goal: `SCOPING (pure docs): turn PGEN_LINTER_ENABLEMENT_ROADMAP.md milestones into an ordered leaf plan; identify which already-shipped semantic-store/annotation primitives ([[feedback_universal_semantic_store]]) satisfy the seed/provenance needs vs. what is genuinely missing. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Ordered milestone→leaf plan + existing-capability-vs-gap analysis recorded.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LINTER.1` | `pending` (proposed) | Scoping precedes any enablement code; lane is planned/not-started. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2` so the Phase T (PGEN-as-service-provider) lane is task-tree-tracked. Not started; activate when prioritized.

## Open Questions

- How much of the seed/provenance need is already met by the existing semantic store + `@export_to_library` machinery? (resolve in `.1`)

## Blockers

- None (planned; lower priority than active SV-EXH-PROOF + governance work).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `LINTER.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `LINTER.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
