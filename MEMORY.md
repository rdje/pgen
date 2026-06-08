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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0042` (GRAMMAR-WELLFORMED **`.H.5.1`**) — **LABELED the svpp witness-parseability residual: wired svpp `parse_detail` so cert-coverage shows the EXACT per-sample parse error** (was `"(no detail-capable parser registered)"`). Added `parse_with_systemverilog_preprocessor_detail_profile` adapter (`ParseDetailFn` shape, profile ignored) delegating to the existing `_detail` fn + flipped the svpp registry entry `parse_detail None → Some(...)`. ADJUDICATED (tools-backed): the labeled error is **`Parser did not consume full input at position 0`** on a sample leading with `` `define/***/ `` (no macro name); `pp_define`/`macro_*` are ALL `UNKNOWN` ⇒ the GENERATOR never emits a valid `` `define NAME body `` (under-fills required payload) — a generator-side deficiency, NOT a parser bug. Labeling-only (residual count unchanged). parser_registry 20/0; lib builds; clippy strict-SOURCE ok. `generated/` untracked ⇒ NO tracked-artifact change.
- prior milestone `H.2` (`-0041`): **every SHIPPED parser grammar now runs under cert-coverage** (json/regex/rtl_const_expr/svpp/rtl_frontend/SV/vhdl); only meta/annotation grammars (ebnf, return/semantic_annotation) unwired.
- ⚠️ OPEN follow-up `GRAMMAR-WELLFORMED.H.5.1.1` (owned STUB created in the task file): FIX the now-LABELED svpp residual so the generator emits well-formed `` `define NAME body `` etc. → svpp `sample_parse_failures` 24→0 + `pp_define`/`macro_*` UNKNOWN→0. LEAD (tools-observed, verify before coding): `macro_name := identifier := inline_trivia /regex/`, and `inline_trivia` GENERATES NEWLINES (isolation probe) → plausibly breaks the newline-terminated `` `define `` line. ⚠️ NOT a quick slice — `inline_trivia` is SHARED across grammars (global-regression risk), the LEXICAL-ANNOTATIONS class; first step = a focused parser TRACE of one labeled sample to PROVE the rejecting production, then strict fix-hierarchy (grammar/annotation before generator). This is the "task beyond a safe slice" boundary — deserves a focused/fresh effort.
- locked_program (director 2026-06-08): **ALL EXISTING PARSERS → `Done` (cert-coverage WIRED + clean / UNKNOWN=0 each).** Cert-coverage WIRED (ALL shipped done): json ✓fully_certified, regex ✓(UNKNOWN), rtl_const_expr ✓(UNKNOWN=7), svpp ✓(UNKNOWN=54 + 24 residual now LABELED), rtl_frontend ✓(UNKNOWN=133, clean), SV ✓(large UNKNOWN), vhdl ✓(UNKNOWN=85, clean). NOT wired: only the meta/annotation grammars (ebnf, return/semantic_annotation).
- active_work_unit: `GRAMMAR-WELLFORMED` → frontier `H`. next_action: **`H.5.1.1`** fix the svpp generator residual (now labeled) + **drive each wired grammar's UNKNOWN→0** (more/targeted witnesses + closed loop).
- in_flight_uncommitted: none. blockers: none.
- push: ~62 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
