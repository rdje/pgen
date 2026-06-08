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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0045` (GRAMMAR-WELLFORMED **`.H.5.1.1` FIX**) — **SURGICAL generator fix for the svpp witness-parseability residual, GLOBAL-measured, zero cross-grammar regression.** `StimuliGenerator::regex_tail_greedy_blocker` (`rust/src/ast_pipeline/stimuli_generator.rs`) now returns `None` when the greedy unbounded tail class is WHITESPACE-ONLY (new helper `regex_class_is_whitespace_only`) — a trailing whitespace run needs no anti-fusion guard, so the old `Some("\n")` escalation for `[ \t]+` (pointless + harmful in line-oriented grammars) is gone. Makes the generator MORE faithful to the EBNF (already-correct grammar); parser-agnostic + general. VERIFIED: svpp cert-cov (count40 seed0) `sample_parse_failures` **24→8**, `witness` **19→66**, `UNKNOWN` **54→7**; DECISIVE stash-baseline A/B (json/regex/rtl_const_expr/vhdl/rtl_frontend byte-identical pre/post — ONLY svpp changed, matching the static proof that svpp's `[ \t]+` is the sole `\n`-excluding greedy ws class); new unit test `whitespace_only_greedy_tail_gets_no_separator`; lib 621/621; clippy source-strict clean; **cross-family stimuli gate PASS**. No regen / no `generated/` change (runtime generator, not codegen).
- prior `-0044` (H.5.1.1 INVESTIGATION): PROVED the root cause tools-first (parser trace + byte-exact isolation) and CORRECTED the H.5.1 "generator under-fills the name" adjudication (the name IS generated; faithful-spacing injects a `\n`).
- ⚠️ NEXT: svpp **residual-8** (a different, PRE-EXISTING class, NOT this fix's target): (a) `\b`-keyword↔word-char macro-name spacing gap (`` `ifndefR7Sh ``, `` `timescale63_ `` — deferred join-rule space not landing for a `\b`-terminated keyword); (b) deeper structural (`` `define NAME(formals)`` macro_formals, `` `timescale … ms`` time_literal). Plus drive each wired grammar's UNKNOWN→0.
- locked_program (director 2026-06-08): **ALL EXISTING PARSERS → `Done` (cert-coverage WIRED + clean / UNKNOWN=0 each).** Cert-coverage WIRED (ALL shipped done): json ✓fully_certified, regex ✓(UNKNOWN), rtl_const_expr ✓(UNKNOWN=3), svpp ✓(UNKNOWN 54→7, residual 24→8), rtl_frontend ✓(UNKNOWN=133, clean), SV ✓(large UNKNOWN), vhdl ✓(UNKNOWN=85, clean). NOT wired: only the meta/annotation grammars (ebnf, return/semantic_annotation).
- active_work_unit: `GRAMMAR-WELLFORMED` → frontier `H`. next_action: svpp residual-8 (`\b`-keyword spacing + macro_formals/time_literal) + drive each wired grammar's UNKNOWN→0.
- in_flight_uncommitted: none. blockers: none.
- push: ~65 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
