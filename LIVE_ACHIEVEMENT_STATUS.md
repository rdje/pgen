# Live Achievement Status

Last updated: 2026-03-14

## Purpose
Provide a precise, always-current progress surface for the project using exactly four status levels:
- `Done`
- `Mostly Done`
- `In Progress`
- `Not Started`

This file is the authoritative live tracking view for "where we are now".

## Status Rules
- `Done`: exit criteria for the tracked area are implemented, validated, and no material roadmap gap remains for that area.
- `Mostly Done`: the core implementation is landed and validated, but bounded follow-up work is still required before closure.
- `In Progress`: meaningful implementation has started, but core capabilities or validation are still missing.
- `Not Started`: no meaningful implementation has landed yet.

## Update Policy
- Review and update this file before every commit when a task changes actual project closure, remaining scope, or the next most important gap.
- Use only the four statuses above.
- Keep "Evidence" concrete and "Left To Close" explicit.

## Live Snapshot

### Major Roadmap Phases

| Area | Status | Evidence | Left To Close |
|---|---|---|---|
| Phases A-R | Done | Roadmap phases `A` through `R` currently show only completed checklist items. | Nothing material inside the currently tracked phase checklists. |
| Phase S overall: RTLSyn parser stack minimum viable coverage | In Progress | `rtl_const_expr` and `rtl_frontend` are active, executable crates with ongoing implementation and passing tests. | Close the remaining frontend/elaboration gaps and start the still-missing companion parser crates. |

### Phase S Detailed Breakdown

| Area | Status | Evidence | Left To Close |
|---|---|---|---|
| `rtl_const_expr` baseline evaluator | Mostly Done | Standalone crate exists, is integrated into `rtl_frontend`, and supports literals, operators, ternary, dotted identifiers, and package-qualified names. | Broaden constant-expression coverage to the remaining RTL cases needed by the planned frontend/elaboration stack. |
| `rtl_frontend` synthesizable subset baseline | Mostly Done | Current subset covers modules, params/localparams, ANSI ports, typedef/package scope, instantiations, generate, aggregate types, procedural forms, typed/concatenated assignment targets, structured assignment values, and elaboration-time validation. | Expand the remaining parser/elaboration surface, especially richer mixed operator/value-expression forms and deeper procedural/dataflow semantics. |
| Liberty parser crate | Not Started | Roadmap item still open; no crate/worktree implementation is tracked yet. | Add the crate and land the minimum timing/Boolean/area extraction subset. |
| SDC parser crate | Not Started | Roadmap item still open; no crate/worktree implementation is tracked yet. | Add the crate and land the planned minimum constraint subset. |
| Later auxiliary readers (`gate-level` netlist reader, config reader, optional SDF) | Not Started | Still listed as later/non-day-1 items in Phase S only. | Start only after the core parser-stack MVP is materially closer to closure. |

### Immediate Next Gap

| Area | Status | Evidence | Left To Close |
|---|---|---|---|
| `rtl_frontend` post-structured-RHS work | In Progress | Assignment parsing/validation now covers typed and concatenated LHS targets plus structured RHS values (signal/member/select/concat/repeat forms). | Broaden the remaining dataflow expression surface, especially mixed operator expressions over structured/member/select operands. |
