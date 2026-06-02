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
- latest_commit: `9af539a5` PGEN-PARSE-SOTA-0009 (A1 wiring corrected + A2 --lint-grammar). Pushed through d60b73e3; unpushed ~14. MEMORY-ARCH + DIAG-SEVERITY trees CLOSED.
- active_work_unit: rolling `PARSE-SOTA` (Tier-A) per director "roll to exhaustion". A1 (.8/.8.1) + A2 (.9/.9.1) DONE+VERIFIED (via `make focus_regex` + --lint-grammar on all 8 grammars = 0 non-terminating). `SV-EXH-PROOF.7.4.3` DONE+KEPT.
- next_action: PARSE-SOTA `.10` (A4 round-trip/determinism + golden-file AST tests) + `.11` (A5 ship the approved `_meta` carrier). Also: `SV-EXH-PROOF.7.4.4` (context-gating tail — the 63 "other" witness failures; verify cause then reach-from-context); full canonical 888->X gate run (def. number for .7.4.3). VERIFY GRAMMAR/PARSER WORK VIA THE Makefile targets (make focus_<grammar>), per director — not ad-hoc ast_pipeline invocations.
- in_flight_uncommitted: this MEMORY.md sync.
- blockers: none.

## Other open threads (not the active unit)
- **SV-EXH-PROOF.7.4** literal-0: `.7.4.1` criterion + `.7.4.2` Purdom min-length table + `.7.4.3a` depth-budget ROOT CAUSE + `.7.4.3` witness pass (KEPT: monotone-additive, real-SV 72/150 resolved, 0 depth/visit failures = depth fix validated). Frontier `.7.4.4` = context-gating tail (63 "other" failures = rules whose @predicate needs ancestor store facts → reach-from-context, not standalone).
- **PARSE-SOTA** (active): research `.1`-`.7` done; A1 (`.8`, non-terminating detection — LR-reject dropped, PGEN handles LR) + A2 (`.9`, shadowing lint) ANALYSES done in grammar_wellformedness.rs; wiring `.8.1`/`.9.1` next; A4/A5 pending.
- **DIAG-SEVERITY** (CLOSED `-0001..0006`): severity never gated by verbosity; canonical GenerationErrorReason; enforcement gate (pre-commit+CI); book. Fixed the masking that hid the SV depth cause. Also reclaimed ~24 GB + relativized live-doc paths (PGEN-DOCPATH-0001).
- **SV-EXH-PROOF** literal-0: ROOT CAUSE pinned (`.7.4.3a`, -0140) = DEPTH-BUDGET exhaustion (deep-factored rules can't reach+complete within max_depth=24 from the top entry; ansi_port_declaration needs ≥10 depth; measured). Fix `.7.4.3` = per-target minimal witnesses with FRESH/adequate depth (root at/short-path to target + Purdom shortest subtree from `.7.4.2`'s min-length table + generalize the :4869 depth-slack), measured vs 888. NOT a global max_depth raise (blows up property_expr; reshapes the diverse pass).
- **PARSE-SOTA** (active, research `.1`–`.6` DONE): parser-gen path validated as a recognized published architecture; §1 adoption backlog awaits director review (Tier A: well-formedness check, ⭐shadowing lint, labeled failures, round-trip tests, ship `_meta`).
- Push: push IS release; default wait ~30 unpushed OR explicit "push". Restore tag `checkpoint/sv-exh-proof-3.2-clean` @ 41bef35e.
