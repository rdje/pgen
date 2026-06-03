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
- latest_commit: `af0e18f3` PGEN-PARSE-SOTA-0012 + (this commit) PGEN-SV-EXH-PROOF-0142 (leaf .7.4.4.1). PARSE-SOTA A1/A2/A4/A5-design done; MEMORY-ARCH + DIAG-SEVERITY CLOSED. unpushed ~19.
- active_work_unit: `SV-EXH-PROOF.7.4.4` (literal-0 residual tail), RE-SCOPED by `.7.4.4.1`. PARSE-SOTA Tier-A (`.8`-`.11`) parked: A5 (`.11`) impl is a delicate two-surface codegen change deferred to a focused session per its design doc.
- next_action: `SV-EXH-PROOF.7.4.4` (RE-SCOPED) — the witness "other" tail is NOT context-gating; it is SLOW GENERATION (other=0 proven on 30- & 150-target real-SV samples; failures = target_timeout on deeply-factored rules). FIX = Purdom min-length-guided greedy witness expansion (reuse `.7.4.2`'s compute_min_terminal_lengths table) so slow witnesses converge fast; measure vs 888; MONOTONE. Director sign-off before the generation change lands. VERIFY VIA Makefile targets (make focus_<grammar>), not ad-hoc invocations.
- in_flight_uncommitted: none (this commit includes MEMORY.md sync).
- blockers: none. (Pre-existing: full-workspace `cargo test` RED from stale GlobalOptions ctors at parseability_probe.rs:738/754 — unrelated; use `cargo test --lib`.)

## Other open threads (not the active unit)
- **SV-EXH-PROOF.7.4** literal-0: `.7.4.1` criterion + `.7.4.2` Purdom min-length table + `.7.4.3a` depth-budget ROOT CAUSE + `.7.4.3` witness pass (KEPT) + `.7.4.4.1` (-0142) reason-split classifier + bounded WHY+WHERE sample → PROVED the "other" tail = SLOW GENERATION not context-gating (other=0 on 30- & 150-target real-SV; failures = target_timeout). Frontier `.7.4.4` RE-SCOPED: Purdom min-length-guided greedy witness expansion so slow witnesses converge fast (NOT reach-from-context — that targeted the disproven context-gating cause).
- **PARSE-SOTA** (active): research `.1`-`.7` done; A1 (`.8`, non-terminating detection — LR-reject dropped, PGEN handles LR) + A2 (`.9`, shadowing lint) ANALYSES done in grammar_wellformedness.rs; wiring `.8.1`/`.9.1` next; A4/A5 pending.
- **DIAG-SEVERITY** (CLOSED `-0001..0006`): severity never gated by verbosity; canonical GenerationErrorReason; enforcement gate (pre-commit+CI); book. Fixed the masking that hid the SV depth cause. Also reclaimed ~24 GB + relativized live-doc paths (PGEN-DOCPATH-0001).
- **SV-EXH-PROOF** literal-0: ROOT CAUSE pinned (`.7.4.3a`, -0140) = DEPTH-BUDGET exhaustion (deep-factored rules can't reach+complete within max_depth=24 from the top entry; ansi_port_declaration needs ≥10 depth; measured). Fix `.7.4.3` = per-target minimal witnesses with FRESH/adequate depth (root at/short-path to target + Purdom shortest subtree from `.7.4.2`'s min-length table + generalize the :4869 depth-slack), measured vs 888. NOT a global max_depth raise (blows up property_expr; reshapes the diverse pass).
- **PARSE-SOTA** (active, research `.1`–`.6` DONE): parser-gen path validated as a recognized published architecture; §1 adoption backlog awaits director review (Tier A: well-formedness check, ⭐shadowing lint, labeled failures, round-trip tests, ship `_meta`).
- Push: push IS release; default wait ~30 unpushed OR explicit "push". Restore tag `checkpoint/sv-exh-proof-3.2-clean` @ 41bef35e.
