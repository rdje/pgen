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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0065` (leaf `H.11.1`) — **🔧 ENGINE FIX: multi-token regex-terminal renders record a TAIL-aware word-shape (3rd H.8-class instance; helper renamed `tail_word_shaped`): vhdl cert-coverage `UNKNOWN 30→4`** (witness 187→213; the WHOLE keyword-fusion family `8 minto`/`msthen`/`nsand` closed; residual EXACTLY [based_literal, based_value, hash, white_space]; IDENTICAL seeds 0/7/42). regex STAYS fully_certified (198/198 spf=0); json/rtl_const_expr/svpp stay certified; rtl_frontend 73 + SV 647+68 byte-identical. **Decisive stash baseline: vhdl seed-7 diverse spf=2 is PRE-EXISTING (identical on pre-fix binary) → ticketed `H.11.2`.** Oracle + cross-family + mdbook gates PASS; lib 639/0 + 732/0 (+1 lock); strict clippy clean; NO regen/bump. PRIOR: `-0064` `C2.2` **regex = the 4TH FULLY-CERTIFIED grammar** (semantic-prelude reach, UNKNOWN 2→0); `-0063` `C2.1` design; `-0062`/`-0061` H.10.2 pool closure.
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-15) ✅ through `-0064`; (16) ✅ `H.11.1` (`-0065`, this). NEXT = **`H.11.3`** the vhdl based-literal family over-generation (generated `3374#BFCd#` — base outside 2..16 + digits beyond base — parser-rejected; grammar models base/digits independently → likely needs the grammar to constrain or the generator to honour the constraint; tools-first: read the vhdl parser's based-literal acceptance + the grammar rules, adjudicate the right level) + **`H.11.4`** white_space routing + **`H.11.2`** seed-7 spf=2 root-cause → vhdl fully_certified; then rtl_frontend (73) / SV (647 + 68 no-path/A2.1).
- 🧠 ENGINE RULE (standing, KM `memo-hit-transactional-replay`): a transactional per-rule parser record needs THREE legs — push on body entry, truncate in `try_parse`, AND delta-capture+replay in `memoized_call`. Two instances of the missing third leg so far (semantic store `.36.4`, coverage record `H.10.2.2`).
- ⚠️ OPERATIONAL (this session): cert-coverage numbers depend on the BINARY embedding the engine state (grammar is runtime-loaded; broadcast/atomicity/codegen is compiled) — compare binary mtime vs `git log -1` before declaring a regression. Also: Makefile `annotation_parsers`→`return_semantic_parsers` runs `clean` (wipes generated/ + cargo clean!) — use the direct `return_annotation_parser`/`semantic_annotation_parser` targets; regen after a MemoEntry-shape change needs the seed flow (rm `generated/ebnf.rs` → build → regen ebnf → regen rest).
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority.** 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×): PGEN is evolving toward bug-free/feature-complete; general parser-agnostic enhancements: discuss → own → design → implement. The stimuli generator is a BUG-FINDING ORACLE.
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = context-aware SOUND gating (reject provably-undeclared, accept unprovable); cross-unit binding/type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). Seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (32/32), **regex (the 4th, via C2)**. Remaining UNKNOWN: **vhdl 4** ([based_literal, based_value, hash, white_space] + the pre-existing seed-7 spf=2 = `H.11.2/.3/.4`), **rtl_frontend 73, SV 647 (+68 no-path/multi-entry, A2.1)**.
- active_work_unit: `GRAMMAR-WELLFORMED.H.11` vhdl drive → next leaf = `H.11.3` (based-literal over-generation, tools-first). in_flight_uncommitted: none. blockers: none.
- push: ~101 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
