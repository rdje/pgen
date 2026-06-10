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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0061` (leaf `H.10.2.2`) — **🔧 ENGINE FIX: memo-hit coverage-delta replay** (the memoization × transactional-record composition gap, the `.36.4` class on the certifying-linter WITNESS record): `MemoEntry.coverage_delta` captured at body time + replayed on every cache hit. Tools-first: 2 of the mislabeled "terminal-selection trio" were THIS defect (probes parsed with the construct in the AST, the record lost it on memo hits). **regex cert-coverage `UNKNOWN 5→3`** (witness 193→195, spf=0, seeds 0/7/42) + **CROSS-GRAMMAR vhdl `31→30`, rtl_frontend `75→73`, SV `738→647` (−91)**, spf=0 everywhere; oracle gate PASS ⇒ NO bump; all 10 parsers regenerated; suites 683/0+757/0+798/0; KM card `memo-hit-transactional-replay`; book + tracker synced. PRIOR: `BRANCH-BROADCAST-FIX` tree complete (`-0001..-0005`).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-11) ✅ through `-0005`; (12) ✅ `H.10.2.2` (`-0061`, this). NEXT = **`H.10.2.3`** (unicode_char declarative `@sample` witnessing literal — generation-side gap, tool-proven: `builtin_any_char` has no grammar def + no generator special-case ⇒ `Missing rule` error; `Lookahead → Ok("")` is guard-blind; leaf already opened+designed in the tree). Then the store-gated pair (B2/C1/C2 lane) + the vhdl (30) / rtl_frontend (73) / SV (647 + 68 no-path) drives.
- 🧠 ENGINE RULE (standing, KM `memo-hit-transactional-replay`): a transactional per-rule parser record needs THREE legs — push on body entry, truncate in `try_parse`, AND delta-capture+replay in `memoized_call`. Two instances of the missing third leg so far (semantic store `.36.4`, coverage record `H.10.2.2`).
- ⚠️ OPERATIONAL (this session): cert-coverage numbers depend on the BINARY embedding the engine state (grammar is runtime-loaded; broadcast/atomicity/codegen is compiled) — compare binary mtime vs `git log -1` before declaring a regression. Also: Makefile `annotation_parsers`→`return_semantic_parsers` runs `clean` (wipes generated/ + cargo clean!) — use the direct `return_annotation_parser`/`semantic_annotation_parser` targets; regen after a MemoEntry-shape change needs the seed flow (rm `generated/ebnf.rs` → build → regen ebnf → regen rest).
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority.** 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×): PGEN is evolving toward bug-free/feature-complete; general parser-agnostic enhancements: discuss → own → design → implement. The stimuli generator is a BUG-FINDING ORACLE.
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = context-aware SOUND gating (reject provably-undeclared, accept unprovable); cross-unit binding/type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). spf=0 on ALL wired grammars; seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (32/32). Remaining UNKNOWN: **regex 3 (unicode_char + store-gated pair), vhdl 30, rtl_frontend 73, SV 647 (+68 no-path/multi-entry, A2.1)**.
- active_work_unit: `GRAMMAR-WELLFORMED.H.10.2.3` (queued leaf, design recorded in the tree). in_flight_uncommitted: none. blockers: none.
- push: ~97 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
