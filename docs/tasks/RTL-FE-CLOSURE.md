# RTL-FE-CLOSURE: rtl_frontend Parser-Family Closure (Phase S)

## Metadata

- Tree ID: `RTL-FE-CLOSURE`
- Status: `proposed`
- Roadmap lane: `Phase S — rtl_frontend synthesizable-RTL subset: parser-family closure (distinct from the existing RTL-FE-MDBOOK / RTL-FE-CONTRACT-BODY book/contract trees)`
- Created: `2026-05-31`
- Last updated: `2026-05-31`
- Owner: repo-local workflow

## Goal

Drive the `rtl_frontend` family from its current `In Progress` LIVE status to
PGEN closure: generated-grammar exhaustiveness + elaboration-facing closure,
beyond the already-landed generated-contract/handwritten-parity work. The
existing `RTL-FE-MDBOOK` + `RTL-FE-CONTRACT-BODY` trees (done) cover the book +
integration-contract surfaces; THIS tree owns the parser/elaboration CLOSURE
the LIVE row says is still open.

## Non-Goals

- Not the book or integration-contract surfaces (owned by the existing RTL-FE
  trees) — this is parser/elaboration closure.
- Per Phase S rule: handwritten `parse_design` baseline is scaffolding;
  generated-parser exhaustiveness is the closure bar.

## Acceptance Criteria

- Generated `rtl_frontend` grammar exhaustiveness proven.
- Elaboration-facing closure proven (the `rtl_frontend_generated_contract_gate`
  + elaboration replay layer extended to the closure bar).
- LIVE "rtl_frontend synthesizable subset baseline" row → Done with evidence.
- Each completed leaf committed through `COMMIT.md`; any code change owns a leaf.

## Task Tree

- ID: `RTL-FE-CLOSURE`
  Status: `proposed`
  Goal: `rtl_frontend generated-grammar exhaustiveness + elaboration closure → LIVE Done.`
  Children: `RTL-FE-CLOSURE.1`

- ID: `RTL-FE-CLOSURE.1`
  Status: `pending`
  Goal: `SCOPING (pure docs): read the LIVE rtl_frontend row + README rtl_frontend notes + rtl_frontend_generated_contract_gate to pin the exact remaining exhaustiveness/elaboration gap to Done; produce an ordered leaf plan. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Remaining-to-Done gap + ordered leaf plan recorded.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RTL-FE-CLOSURE.1` | `pending` (proposed) | Must pin the remaining closure gap (LIVE = In Progress) before closure code. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2` to own the rtl_frontend parser/elaboration CLOSURE gap, distinct from the done book/contract trees. Not started; activate when prioritized (after SV-EXH-PROOF).

## Open Questions

- Exact generated-exhaustiveness + elaboration-parity bar for Done? (resolve in `.1`)

## Blockers

- None (In Progress baseline; closure deferred).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `RTL-FE-CLOSURE.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RTL-FE-CLOSURE.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
