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
- latest_commit: `0ed9d83c` — "PGEN-MEMORY-ARCH-0005 (MEMORY-ARCH.4): enforcement kit E1–E4" (ahead of origin ~20; push at ~30) — being superseded by the MEMORY-ARCH.5 close commit
- active_work_unit: none in flight — `MEMORY-ARCH` tree CLOSED (`-0001..0006`, durable memory architecture adopted, E1–E4 enforced, hooks armed via core.hooksPath).
- next_action: pick the next thread. Primary open item: SV-EXH-PROOF `.7.2.21` director decision (accept-with-evidence at 888 vs commission a new non-steering coverage mechanism). Backlog: 9 proposed skeleton trees (PNR/linter/compiler-elaborator/...).
- in_flight_uncommitted: the MEMORY-ARCH.5 close (this file + LIVE/CHANGES/TASK_TREE + tree node) being committed now.
- blockers: none.

## Other open threads (not the active unit)
- `SV-EXH-PROOF` (active, paused at a logged checkpoint): reach campaign `.7.2` consolidated in its `.7.2 CAMPAIGN SUMMARY`. Best-known stimuli residual = 888 (down from 1273 dead-hook / ~2660 initial); 3 steering-intensification attempts all regressed via diversity collapse → steering exhausted; residual is 100% reachable (not structural). Open decision `.7.2.21`: accept-with-evidence at 888 vs commission a new (non-steering, diversity-preserving) mechanism — director call. SV main parser = Mostly Done; generation path byte-identical to the 888 blob.
- Push: ⛔ no-push override; push at ~30 unpushed or on explicit request. Restore tag `checkpoint/sv-exh-proof-3.2-clean` @ 41bef35e.
