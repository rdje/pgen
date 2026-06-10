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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0063` (leaf `C2.1`) — **📐 C2 semantic-prelude reach ACTIVATED + count-gated-MVP DESIGN recorded (pure docs)** for the regex store-gated pair (LAST regex UNKNOWN residual). Design: plan-scoped TWO-PHASE in the plannable reach pass — phase 1 captures the count-gated rule's render + numeric v (prune bypassed, scoped); phase 2 injects v producer-steered quantified-body iterations (`()`×v at `concatenation`'s `piece+`) before the on-path iteration + replays the captured text ⇒ `()()…()\v` witnesses BOTH rules. Prelude spec defaults `None` ⇒ no-op everywhere else by construction; generator-only ⇒ NO regen/bump expected. PRIOR: `-0062` `H.10.2.3` (unicode_char `@sample: "é"`, regex 3→2); `-0061` `H.10.2.2` (ENGINE memo-hit coverage-delta replay, regex 5→3 + cross-grammar SV 738→647; KM `memo-hit-transactional-replay`).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself — do not involve me unless you can't decide"): (1-13) ✅ through `-0062`; (14) ✅ `C2.1` design (`-0063`, this). NEXT = **`C2.2` IMPLEMENT** the count-gated prelude MVP (engine, `stimuli_generator.rs` only; acceptance: regex cert-coverage `UNKNOWN 2→0` ⇒ `fully_certified` — the 4th — witness 198, spf=0, seeds 0/7/42; cross-grammar byte-identical; oracle PASS; unit locks; suites+clippy; book grammar-wellformedness arc `…→2→0` + tracker lockstep). Then the vhdl (30) / rtl_frontend (73) / SV (647 + 68 no-path) drives.
- 🧠 ENGINE RULE (standing, KM `memo-hit-transactional-replay`): a transactional per-rule parser record needs THREE legs — push on body entry, truncate in `try_parse`, AND delta-capture+replay in `memoized_call`. Two instances of the missing third leg so far (semantic store `.36.4`, coverage record `H.10.2.2`).
- ⚠️ OPERATIONAL (this session): cert-coverage numbers depend on the BINARY embedding the engine state (grammar is runtime-loaded; broadcast/atomicity/codegen is compiled) — compare binary mtime vs `git log -1` before declaring a regression. Also: Makefile `annotation_parsers`→`return_semantic_parsers` runs `clean` (wipes generated/ + cargo clean!) — use the direct `return_annotation_parser`/`semantic_annotation_parser` targets; regen after a MemoEntry-shape change needs the seed flow (rm `generated/ebnf.rs` → build → regen ebnf → regen rest).
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority.** 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×): PGEN is evolving toward bug-free/feature-complete; general parser-agnostic enhancements: discuss → own → design → implement. The stimuli generator is a BUG-FINDING ORACLE.
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = context-aware SOUND gating (reject provably-undeclared, accept unprovable); cross-unit binding/type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). spf=0 on ALL wired grammars; seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (32/32). Remaining UNKNOWN: **regex 2 (the store-gated pair ONLY), vhdl 30, rtl_frontend 73, SV 647 (+68 no-path/multi-entry, A2.1)**.
- active_work_unit: `GRAMMAR-WELLFORMED.C2` → frontier leaf `C2.2` (implement the count-gated prelude MVP per the `C2.1` design in the leaf). in_flight_uncommitted: none. blockers: none.
- push: ~99 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
