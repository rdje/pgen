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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0067` (`H.11.3` DONE) — **🔧 vhdl RELEASED-PARSER BUG FIXED: based literals parse (release `1.0.3`→`1.0.4`, schema stays 3, ledger `VHDL-0002`)**. WHY: the emitted layout skipper `consume_layout_for_regex` hard-codes `#`-to-EOL comment skipping (EBNF convention; VHDL `#` = based-literal delimiter) with NO introducer guard on the regex side (string-terminal side always had one) → `hash := trivia /#/` lost its own `#`. FIX (engine, `ast_based_generator.rs`): comment arms stand down when the active pattern matches at the introducer (`regex_token_matches_at_cursor`); skipper moved into the `uses_match_regex` conditional; all parsers regenerated. vhdl UNKNOWN 4→1 (seeds 0/7/42; residual = white_space = `H.11.4`); rtl_frontend 73→71; SV 647→645; suites 639/0 + 733/0 (+1 lock); oracle + cross-family gates PASS; clippy clean.
- 🎫 TICKETS from the `-0067` sweep (each needs its own leaf before code): **`H.11.5`** SV generator `/`↔comment fusion (canonical seed-0 spf 0→1 — pre-existing over-generation EXPOSED by grammar-faithful comment ownership; parser right to reject; fix generator-side); **ebnf `**` gap** (generated ebnf parser can't parse flatten-spread — `ebnf.ebnf` `quantified_marker` single-marker; `ebnf_frontend_dual_run_gate` red on the regex flow, stash-proven pre-existing); **rtl_frontend contract gate red** (`always_ff_well_formed` missing `module_declaration`, stash-proven pre-existing, comment-free sample). `H.11.2` same-bug conjecture REFUTED (seed-7 spf=2 persists; no based literals in the 2 failing samples).
- 🧭 PNT batch (2026-06-09/10, director: "PNT yourself"): (1-17) ✅ through `-0066`; (18) ✅ `H.11.3` fix (`-0067`, this). NEXT = **`H.11.4`** (white_space, the LAST vhdl UNKNOWN — adjudicate reachable-vs-routed) → vhdl fully_certified; then `H.11.5`/`H.11.2` (generator residuals), the two ticketed red gates, rtl_frontend (71) / SV (645 + 68 no-path/A2.1).
- 🧠 ENGINE RULE (standing, KM `memo-hit-transactional-replay`): a transactional per-rule parser record needs THREE legs — push on body entry, truncate in `try_parse`, AND delta-capture+replay in `memoized_call`. Two instances of the missing third leg so far (semantic store `.36.4`, coverage record `H.10.2.2`).
- ⚠️ OPERATIONAL: cert-coverage canonical invocations — count 40 seed 0 (+7/42); rtl_const_expr needs `--max-depth 32`; SV needs `--grammar-profile 2017` (total=1342; profile-less = 1458, NOT comparable). Makefile `return_semantic_parsers` runs `clean` — use direct targets. Engine-emit changes: rebuild ast_pipeline → regen ebnf.rs → rebuild → focus_* the rest.
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic, standing): **fix parser bugs ASAP = HIGHEST priority.** 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×): PGEN is evolving toward bug-free/feature-complete; general parser-agnostic enhancements: discuss → own → design → implement. The stimuli generator is a BUG-FINDING ORACLE.
- 🧭 DIRECTOR-CONFIRMED BOUNDARY (standing): SV parser = context-aware SOUND gating (reject provably-undeclared, accept unprovable); cross-unit binding/type/width/param/generate = elaboration.
- locked_program (director 2026-06-08): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). Seed-robust UNKNOWN=0: rtl_const_expr, json, svpp (32/32), regex. Remaining UNKNOWN: **vhdl 1** (`H.11.4` white_space; + seed-7 spf=2 `H.11.2`), **rtl_frontend 71, SV 645 (+68 no-path/multi-entry, A2.1)**.
- active_work_unit: `GRAMMAR-WELLFORMED.H.11` vhdl drive → next leaf = `H.11.4`. in_flight_uncommitted: none. blockers: none.
- push: ~102 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
