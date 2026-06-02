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
- latest_commit: `f833fa91` PGEN-DIAG-SEVERITY-0006 (DIAG-SEVERITY tree CLOSED). Pushed through d60b73e3 (unpushed ~5 since). MEMORY-ARCH + DIAG-SEVERITY trees CLOSED.
- active_work_unit: `SV-EXH-PROOF` frontier `.7.4.3` (depth-budget-aware minimal witnesses — the actual residual fix).
- next_action: `SV-EXH-PROOF.7.4.3` — per-target minimal witnesses with FRESH/adequate depth (root at/short-path to target + Purdom shortest subtree from `.7.4.2` min-length table + generalize the :4869 depth-slack), now VERIFIABLE via the new depth/rule-visit telemetry (DIAG-SEVERITY.3/.3.1). MEASURED vs 888; **director sign-off required before the generation change lands**. Also pending: PARSE-SOTA director review of the §1 adoption backlog; the proposed live-docs-path enforcement is already done (DIAG-SEVERITY.4 folded it in).
- in_flight_uncommitted: this MEMORY.md sync.
- blockers: none.

## Other open threads (not the active unit)
- **DIAG-SEVERITY** (CLOSED `-0001..0006`): severity (warn/error/fatal) never gated by verbosity — Severity enum + always-on emit_diagnostic + pgen_warn!/error!/fatal!; error-by-reason classification (canonical GenerationErrorReason enum; depth_exceeded + rule_visit_limit un-masked + surfaced); enforcement gate (scripts/check_diagnostics_and_docpaths.sh, pre-commit + CI); book section. This fixed the masking that hid the SV depth cause all of `.7.2`. Also reclaimed ~24 GB (stale logs + incremental caches) + relativized live-doc paths (PGEN-DOCPATH-0001).
- **SV-EXH-PROOF** literal-0: ROOT CAUSE pinned (`.7.4.3a`, -0140) = DEPTH-BUDGET exhaustion (deep-factored rules can't reach+complete within max_depth=24 from the top entry; ansi_port_declaration needs ≥10 depth; measured). Fix `.7.4.3` = per-target minimal witnesses with FRESH/adequate depth (root at/short-path to target + Purdom shortest subtree from `.7.4.2`'s min-length table + generalize the :4869 depth-slack), measured vs 888. NOT a global max_depth raise (blows up property_expr; reshapes the diverse pass).
- **PARSE-SOTA** (active, research `.1`–`.6` DONE): parser-gen path validated as a recognized published architecture; §1 adoption backlog awaits director review (Tier A: well-formedness check, ⭐shadowing lint, labeled failures, round-trip tests, ship `_meta`).
- Push: push IS release; default wait ~30 unpushed OR explicit "push". Restore tag `checkpoint/sv-exh-proof-3.2-clean` @ 41bef35e.
