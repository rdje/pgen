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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0064` (leaf `C2.2`) — **🏁 SEMANTIC-PRELUDE REACH LANDED (count-gated MVP, engine, stimuli_generator.rs only): regex cert-coverage `UNKNOWN 2→0` ⇒ `fully_certified` — REGEX IS THE 4TH FULLY-CERTIFIED GRAMMAR** (198/198, spf=0, seeds 0/7/42 × counts 1/40/200, deterministic re-run). Cross-grammar UNCHANGED (json 9/9, rtl_const_expr 48/48, svpp 74/74 certified; vhdl 30, rtl_frontend 73, SV 647+68-no-path; spf=0 everywhere); oracle + cross-family + mdbook gates PASS; lib 638/0 + 731/0 (+2 locks); strict clippy clean; NO regen/bump. Live nuance: phase-1 probe PARSES (RGX-0084 octal degrade → ParsedNotWitnessed) — arming is outcome-independent. PRIOR: `-0063` `C2.1` design; `-0062` `H.10.2.3`; `-0061` `H.10.2.2` (KM `memo-hit-transactional-replay`).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-14) ✅ through `-0063`; (15) ✅ `C2.2` (`-0064`, this). NEXT = the remaining per-grammar `UNKNOWN` drives per the GRAMMAR-WELLFORMED frontier: **vhdl (30) → rtl_frontend (73) → SV (647 + 68 no-path/A2.1)** — adjudicate each residual per the attribution rule (generator-reach vs dead-rule) with the full pass arsenal (constructive-reach + plannable + semantic-prelude); tools-first probes before any fix.
- 🧠 ENGINE RULE (standing, KM `memo-hit-transactional-replay`): a transactional per-rule parser record needs THREE legs — push on body entry, truncate in `try_parse`, AND delta-capture+replay in `memoized_call`. Two instances of the missing third leg so far (semantic store `.36.4`, coverage record `H.10.2.2`).
- ⚠️ OPERATIONAL (this session): cert-coverage numbers depend on the BINARY embedding the engine state (grammar is runtime-loaded; broadcast/atomicity/codegen is compiled) — compare binary mtime vs `git log -1` before declaring a regression. Also: Makefile `annotation_parsers`→`return_semantic_parsers` runs `clean` (wipes generated/ + cargo clean!) — use the direct `return_annotation_parser`/`semantic_annotation_parser` targets; regen after a MemoEntry-shape change needs the seed flow (rm `generated/ebnf.rs` → build → regen ebnf → regen rest).
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority.** 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×): PGEN is evolving toward bug-free/feature-complete; general parser-agnostic enhancements: discuss → own → design → implement. The stimuli generator is a BUG-FINDING ORACLE.
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = context-aware SOUND gating (reject provably-undeclared, accept unprovable); cross-unit binding/type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). spf=0 on ALL wired grammars; seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (32/32), **regex (the 4th, via C2)**. Remaining UNKNOWN: **vhdl 30, rtl_frontend 73, SV 647 (+68 no-path/multi-entry, A2.1)**.
- active_work_unit: `GRAMMAR-WELLFORMED.H` per-grammar drives → next = the vhdl-30 residual (tools-first adjudication probe pass). in_flight_uncommitted: none. blockers: none.
- push: ~100 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
