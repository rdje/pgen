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
- latest_commit: `7cced916` PGEN-SV-EXH-PROOF-0137 (.7.3 research) — then PGEN-PARSE-SOTA-0001 (this commit, PARSE-SOTA tree creation). Ahead of origin ~22; push at ~30. MEMORY-ARCH tree CLOSED (`-0001..0006`).
- active_work_unit: `SV-EXH-PROOF` → frontier `.7.4` (literature-grounded literal-0, commissioned by `.7.3`). Also opening a new `PARSE-SOTA` tree to ground the EBNF→parser-generator path in literature (director ask 2026-06-02).
- next_action: (a) create the `PARSE-SOTA` tree + run its literature research; (b) begin `.7.4.1` (pin the coverage criterion `replay_target_count` measures + reachable-target-set under the PEG guard, pure docs). `.7.4` lands phased + measured per the no-regression discipline; director sign-off before any generation-behavior change.
- in_flight_uncommitted: none after the `-0137` commit.
- blockers: none.

## Other open threads (not the active unit)
- `SV-EXH-PROOF` literal-0: `.7.3` (-0137) RESEARCH overturned accept-at-888 — literal-0 IS systematically attainable via DECOUPLE (diverse background + per-residual Purdom minimal witnesses + our existing PEG-forcing); 3 prior steering regressions = textbook mode collapse. Design: docs/tasks/SV-EXH-PROOF-7.3-literature-grounded-literal-zero-design.md. Frontier `.7.4`. Best-known residual still 888 (generation byte-identical to the 888 blob) until `.7.4.3` lands witnesses.
- Push: ⛔ no-push override; push at ~30 unpushed or on explicit request. Restore tag `checkpoint/sv-exh-proof-3.2-clean` @ 41bef35e.
