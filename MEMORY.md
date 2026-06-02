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
- latest_commit: `14524a18` — "PGEN-MEMORY-ARCH-0003 (MEMORY-ARCH.2): docs/decisions layer C + migrate 54 records" (ahead of origin: ~17; push at ~30)
- active_work_unit: `MEMORY-ARCH` → frontier leaf: `MEMORY-ARCH.3` (in progress — demote root MEMORY.md to this bounded pointer)
- next_action: finish `MEMORY-ARCH.3` (this file), then `MEMORY-ARCH.4` (enforcement kit: check script + .githooks + core.hooksPath + CI + bootstrap pointers), then `.5` (verify gates bite + close).
- in_flight_uncommitted: this MEMORY.md rewrite (being committed as `MEMORY-ARCH.3`).
- blockers: none.

## Other open threads (not the active unit)
- `SV-EXH-PROOF` (active, paused at a logged checkpoint): reach campaign `.7.2` consolidated in its `.7.2 CAMPAIGN SUMMARY`. Best-known stimuli residual = 888 (down from 1273 dead-hook / ~2660 initial); 3 steering-intensification attempts all regressed via diversity collapse → steering exhausted; residual is 100% reachable (not structural). Open decision `.7.2.21`: accept-with-evidence at 888 vs commission a new (non-steering, diversity-preserving) mechanism — director call. SV main parser = Mostly Done; generation path byte-identical to the 888 blob.
- Push: ⛔ no-push override; push at ~30 unpushed or on explicit request. Restore tag `checkpoint/sv-exh-proof-3.2-clean` @ 41bef35e.
