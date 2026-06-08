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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0038` (GRAMMAR-WELLFORMED **`.4`**) — **Phase H cert-coverage WIRED for `rtl_const_expr`** (next-simplest after json/regex). Landed: `focus_rtl_const_expr` Makefile regen targets (mirror json) + `parse_and_cover_rtl_const_expr` in `parser_registry.rs` + the registry entry `Some(...)` + `main.rs` `run_certificate_coverage_report` now threads `--max-depth` (default 24 unchanged ⇒ json/regex/SV byte-identical). De-risked the H.2 mtime-staleness fear (tools-first): regen json from ebnf is byte-identical except `generated_at` ⇒ ZERO drift; parser regen only ADDS G.4.6 coverage methods. MEASURED (`--max-depth 32`, deterministic): `total=48 witness=41 UNKNOWN=7 (sample_parse_failures=0)` seed 0; seed 7 `witness=45 UNKNOWN=3`. parser_registry 20/0; lib `--features generated_parsers` builds; clippy strict-source 0 (generated-stage debt pre-existing, non-strict); json `fully_certified` + regex unaffected. `generated/` is gitignored ⇒ NO tracked-artifact change.
- locked_program (director 2026-06-08): **ALL EXISTING PARSERS → `Done` (cert-coverage WIRED + clean / UNKNOWN=0 for each).** Cert-coverage status: json ✓fully_certified, regex ✓runs(UNKNOWN residual), rtl_const_expr ✓runs(UNKNOWN=7), SV ✓runs(large UNKNOWN). NOT yet wired: svpp, rtl_frontend, vhdl, ebnf, return/semantic_annotation.
- active_work_unit: `GRAMMAR-WELLFORMED` → frontier `H` (per-grammar cert-coverage wiring). next_action: wire `parse_and_cover` for **svpp** then **rtl_frontend** (each: `focus_<grammar>` target + `parse_and_cover_<grammar>` registry fn; mechanical — the H.4 leaf documents the one-time bootstrap-ordering snag + the zero-drift regen recipe), then **vhdl** (now de-risked). Also drive each wired grammar's UNKNOWN→0.
- in_flight_uncommitted: none (this slice committed; `generated/rtl_const_expr_parser.rs` regenerated locally but gitignored).
- blockers: none.
- push: ~58 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
