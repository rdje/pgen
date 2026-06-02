# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> This is the bounded layer-A resume pointer per `MEMORY_ARCHITECTURE.md`.
> OVERWRITE the "Current state" block each update — never append history here.
> (History is in git (layer D) + the task-tree logs (layer B); durable
> facts/decisions are in `docs/decisions/` (layer C).)

## How to resume
- Read `MEMORY_ARCHITECTURE.md` (the memory/continuity system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; the index is `docs/TASK_TREE.md`; follow `COMMIT.md`.
- Durable facts / standing disciplines / decisions live in `docs/decisions/` (see its `INDEX.md`).
- Live status: `LIVE_ACHIEVEMENT_STATUS.md`; changelog: `CHANGES.md`.

## Current state (OVERWRITE this block each update — do not append)
- latest_commit: `d60b73e3` PGEN-DIAG-SEVERITY-0003 (.3 error-by-reason). **PUSHED to origin/main 2026-06-02 (30-commit batch c6480943..d60b73e3); unpushed=0.** MEMORY-ARCH tree CLOSED.
- active_work_unit: two active trees. `DIAG-SEVERITY` (critical diagnostics fix) frontier `.3.1`; `SV-EXH-PROOF` frontier `.7.4.3` (depth-budget witness).
- next_action: suggested order `DIAG-SEVERITY.3.1` (canonical GenerationErrorReason enum + un-mask max_rule_visits + per-reason counts in gap-report JSON) → then `SV-EXH-PROOF.7.4.3` (depth-budget-aware minimal witnesses, MEASURED vs 888, director sign-off before the generation change lands). Also pending: DIAG-SEVERITY.4 (enforcement gate) + .5 (book); PARSE-SOTA director review of the §1 adoption backlog.
- in_flight_uncommitted: this MEMORY.md sync.
- blockers: none.

## Other open threads (not the active unit)
- **DIAG-SEVERITY** (NEW, critical): severity (warn/error/fatal) must NEVER be gated by trace verbosity. `.1` audit + `.2` Severity mechanism (always-on emit_diagnostic + pgen_warn!/error!/fatal!) + `.3` error-by-reason (depth_exceeded bucket + once-per-run pgen_warn!) DONE. `.3.1` canonical enum + max_rule_visits next; `.4` enforcement; `.5` book. The trace-masking is WHY the SV depth-exceeded cause was invisible all of `.7.2`.
- **SV-EXH-PROOF** literal-0: ROOT CAUSE pinned (`.7.4.3a`, -0140) = DEPTH-BUDGET exhaustion (deep-factored rules can't reach+complete within max_depth=24 from the top entry; ansi_port_declaration needs ≥10 depth; measured). Fix `.7.4.3` = per-target minimal witnesses with FRESH/adequate depth (root at/short-path to target + Purdom shortest subtree from `.7.4.2`'s min-length table + generalize the :4869 depth-slack), measured vs 888. NOT a global max_depth raise (blows up property_expr; reshapes the diverse pass).
- **PARSE-SOTA** (active, research `.1`–`.6` DONE): parser-gen path validated as a recognized published architecture; §1 adoption backlog awaits director review (Tier A: well-formedness check, ⭐shadowing lint, labeled failures, round-trip tests, ship `_meta`).
- Push: push IS release; default wait ~30 unpushed OR explicit "push". Restore tag `checkpoint/sv-exh-proof-3.2-clean` @ 41bef35e.
