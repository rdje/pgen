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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0066` (`H.11.3` investigation, pure docs) — **🚨 vhdl GENERATED-PARSER BUG: valid VHDL based literals REJECTED** (`2#1010#` rejects; `3374` + `16x"AB"` pass — over-generation AND lost-ordering hypotheses both probe-refuted). WHERE pinned (rule-scoped trace): inside `based_literal@40`, position JUMPS `41→58(EOF)` between `hash`'s trivia branch entry and the `white_space` body → the open `#` is sought at EOF → based_literal fails on ALL inputs. WHY open: suspects = tournament-arm stale `parser.position` restore OR memoized_call wrong cached end. NEXT STEP (NOT a fix yet, per why+where discipline): read the emitted tournament/trivia arm in `generated/vhdl_parser.rs` (probe `/tmp/vhdl_based_probe2.vhd` retained) + old-binary A/B (pre-`BRANCH-BROADCAST-FIX-0003` worktree regen) to bound the regression window. Fix = consumer-visible accept-set widening ⇒ vhdl release bump + ledger row + contract/book lockstep. PRIOR: `-0065` `H.11.1` tail-shape ENGINE FIX (vhdl UNKNOWN 30→4); `-0064` `C2.2` **regex = the 4TH FULLY-CERTIFIED grammar**; `-0063` `C2.1`.
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself"): (1-16) ✅ through `-0065`; (17) ✅ `H.11.3` investigation (`-0066`, this). NEXT = **`H.11.3` diagnostic→fix (HIGHEST priority: parser bug)**, then `H.11.4` white_space + `H.11.2` seed-7 spf (plausibly the same bug from the diverse side — check those samples for based literals) → vhdl fully_certified; then rtl_frontend (73) / SV (647 + 68 no-path/A2.1).
- 🧠 ENGINE RULE (standing, KM `memo-hit-transactional-replay`): a transactional per-rule parser record needs THREE legs — push on body entry, truncate in `try_parse`, AND delta-capture+replay in `memoized_call`. Two instances of the missing third leg so far (semantic store `.36.4`, coverage record `H.10.2.2`).
- ⚠️ OPERATIONAL (this session): cert-coverage numbers depend on the BINARY embedding the engine state (grammar is runtime-loaded; broadcast/atomicity/codegen is compiled) — compare binary mtime vs `git log -1` before declaring a regression. Also: Makefile `annotation_parsers`→`return_semantic_parsers` runs `clean` (wipes generated/ + cargo clean!) — use the direct `return_annotation_parser`/`semantic_annotation_parser` targets; regen after a MemoEntry-shape change needs the seed flow (rm `generated/ebnf.rs` → build → regen ebnf → regen rest).
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority.** 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×): PGEN is evolving toward bug-free/feature-complete; general parser-agnostic enhancements: discuss → own → design → implement. The stimuli generator is a BUG-FINDING ORACLE.
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = context-aware SOUND gating (reject provably-undeclared, accept unprovable); cross-unit binding/type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). Seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (32/32), **regex (the 4th, via C2)**. Remaining UNKNOWN: **vhdl 4** ([based_literal, based_value, hash, white_space] + the pre-existing seed-7 spf=2 = `H.11.2/.3/.4`), **rtl_frontend 73, SV 647 (+68 no-path/multi-entry, A2.1)**.
- active_work_unit: `GRAMMAR-WELLFORMED.H.11` vhdl drive → next leaf = `H.11.3` (based-literal over-generation, tools-first). in_flight_uncommitted: none. blockers: none.
- push: ~101 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
