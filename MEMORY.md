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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0039` (GRAMMAR-WELLFORMED **`.5`**) — **Phase H cert-coverage WIRED for `systemverilog_preprocessor` (svpp)**. Mechanical mirror of H.4 (`focus_systemverilog_preprocessor` Makefile targets + `parse_and_cover_systemverilog_preprocessor` registry fn + entry `Some(...)`; no `main.rs` change — svpp `pp_item*` is flat so default depth 24 suffices). MEASURED (count 40, deterministic): seed 0 `total=73 witness=19 UNKNOWN=54 (sample_parse_failures=24)`; seed 7 `witness=7 UNKNOWN=66 (sample_parse_failures=32)`. parser_registry 20/0; lib builds; clippy strict-source 0; mdbook green; json/regex/rtl_const_expr unaffected. `generated/` gitignored ⇒ NO tracked-artifact change.
- ⚠️ OPEN follow-up `GRAMMAR-WELLFORMED.H.5.1`: svpp has a HIGH witness-parseability residual (24/40 sample_parse_failures) — the svpp stimuli generator over-produces structurally-INVALID directives (comment-dominated, `` `define `` w/o name/body, stray punctuation). Attribution-rule finding, ROUTED not accepted. Likely needs a svpp `parse_detail` adapter to label errors + generator/grammar root-cause.
- locked_program (director 2026-06-08): **ALL EXISTING PARSERS → `Done` (cert-coverage WIRED + clean / UNKNOWN=0 each).** Cert-coverage WIRED: json ✓fully_certified, regex ✓(UNKNOWN), rtl_const_expr ✓(UNKNOWN=7), svpp ✓(UNKNOWN=54 + high residual), SV ✓(large UNKNOWN). NOT wired: rtl_frontend, vhdl, ebnf, return/semantic_annotation.
- active_work_unit: `GRAMMAR-WELLFORMED` → frontier `H`. next_action: **H.6 wire rtl_frontend** (5.5MB parser; same recipe: `focus_rtl_frontend` target + `parse_and_cover_rtl_frontend` registry fn; bootstrap-ordering = direct regen first; check if its expr grammar needs `--max-depth` like rtl_const_expr) → then **vhdl** (de-risked) → then drive each wired grammar's UNKNOWN→0 + `H.5.1` svpp residual.
- in_flight_uncommitted: none. blockers: none.
- push: ~59 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
