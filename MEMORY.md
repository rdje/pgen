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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0062` (leaf `H.10.2.3`) — **🧪 unicode_char witnessed via declarative `@sample: "é"`** (grammar-only; the rule was UNGENERATABLE — `builtin_any_char` parser-side primitive, `Missing rule` error, guard-blind lookahead): **regex cert-coverage `UNKNOWN 3→2`** (witness 196, spf=0, seeds 0/7/42); oracle PASS ⇒ NO bump; shape-contract 14/14 (inventory stays 186); dual-feature lib 729/0; engine capability ticketed in STIMULI-SIGNOFF. PRIOR: `-0061` `H.10.2.2` **ENGINE FIX memo-hit coverage-delta replay** (regex 5→3 + CROSS-GRAMMAR vhdl 31→30, rtl_frontend 75→73, SV 738→647; KM `memo-hit-transactional-replay`); `BRANCH-BROADCAST-FIX` tree complete (`-0001..-0005`).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-11) ✅ through `-0005`; (12) ✅ `H.10.2.2` (`-0061`); (13) ✅ `H.10.2.3` (`-0062`, this). NEXT = **the regex store-gated pair** `numeric_backreference`/`backreference_digits` (the LAST regex UNKNOWN residual — witnessing needs ≥N capture groups BEFORE the backref; the generation-side store honours the predicate (STORE-AWARE-GEN), so the REACH PASS needs a fact-emitting PRELUDE; B2/C1/C2 constructive lane — likely an ENGINE design leaf, own it before coding). Then the vhdl (30) / rtl_frontend (73) / SV (647 + 68 no-path) drives.
- 🧠 ENGINE RULE (standing, KM `memo-hit-transactional-replay`): a transactional per-rule parser record needs THREE legs — push on body entry, truncate in `try_parse`, AND delta-capture+replay in `memoized_call`. Two instances of the missing third leg so far (semantic store `.36.4`, coverage record `H.10.2.2`).
- ⚠️ OPERATIONAL (this session): cert-coverage numbers depend on the BINARY embedding the engine state (grammar is runtime-loaded; broadcast/atomicity/codegen is compiled) — compare binary mtime vs `git log -1` before declaring a regression. Also: Makefile `annotation_parsers`→`return_semantic_parsers` runs `clean` (wipes generated/ + cargo clean!) — use the direct `return_annotation_parser`/`semantic_annotation_parser` targets; regen after a MemoEntry-shape change needs the seed flow (rm `generated/ebnf.rs` → build → regen ebnf → regen rest).
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority.** 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×): PGEN is evolving toward bug-free/feature-complete; general parser-agnostic enhancements: discuss → own → design → implement. The stimuli generator is a BUG-FINDING ORACLE.
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = context-aware SOUND gating (reject provably-undeclared, accept unprovable); cross-unit binding/type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). spf=0 on ALL wired grammars; seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (32/32). Remaining UNKNOWN: **regex 2 (the store-gated pair ONLY), vhdl 30, rtl_frontend 73, SV 647 (+68 no-path/multi-entry, A2.1)**.
- active_work_unit: `GRAMMAR-WELLFORMED.H.10.2` (pool = the store-gated pair; next leaf to open = the fact-emitting-prelude reach capability). in_flight_uncommitted: none. blockers: none.
- push: ~98 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
