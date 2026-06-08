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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0041` (GRAMMAR-WELLFORMED **`.H.2`**) — **Phase H cert-coverage WIRED for `vhdl`, the LAST unwired SHIPPED grammar. MILESTONE: every shipped parser grammar now runs under cert-coverage** (json/regex/rtl_const_expr/svpp/rtl_frontend/SV/vhdl). Mechanical mirror of H.6: `VHDL_*` vars + build rules + `focus_vhdl` Makefile target + `parse_and_cover_vhdl` registry fn (cfg `has_generated_vhdl_parser`; mirrors svpp) + entry `Some(...)`. DEFAULT depth 24 (flat `vhdl_file := design_unit*`, no profile, NO `--max-depth`); `vhdl.ebnf` needs `--features ebnf_dual_run` (106 `/.../`). **H.2 STALENESS BLOCK DISCHARGED:** fresh `vhdl.json` from current ebnf = byte-identical to on-disk modulo metadata (`diff` empty) ⇒ ZERO drift ⇒ behaviour-preserving regen ⇒ no heavy conformance run. MEASURED (count 40, DETERMINISTIC seed 0 run#1==run#2): seed 0 `total=217 witness=132 UNKNOWN=85 (sample_parse_failures=0)`; seed 7 `129/88/2`. parser_registry 20/0; lib `--features generated_parsers` builds; clippy strict-SOURCE ok; json(`fully_certified`)/regex/rtl_const_expr(`48/41/7`@d32) UNAFFECTED. `generated/` untracked ⇒ NO tracked-artifact change.
- ⚠️ OPEN follow-up `GRAMMAR-WELLFORMED.H.5.1`: svpp has a HIGH witness-parseability residual (24/40 sample_parse_failures) — the svpp stimuli generator over-produces structurally-INVALID directives (comment-dominated, `` `define `` w/o name/body, stray punctuation). Attribution-rule finding, ROUTED not accepted. Likely needs a svpp `parse_detail` adapter to label errors + generator/grammar root-cause.
- locked_program (director 2026-06-08): **ALL EXISTING PARSERS → `Done` (cert-coverage WIRED + clean / UNKNOWN=0 each).** Cert-coverage WIRED (ALL shipped done): json ✓fully_certified, regex ✓(UNKNOWN), rtl_const_expr ✓(UNKNOWN=7), svpp ✓(UNKNOWN=54 + high residual), rtl_frontend ✓(UNKNOWN=133, clean), SV ✓(large UNKNOWN), vhdl ✓(UNKNOWN=85, clean). NOT wired: only the meta/annotation grammars (ebnf, return/semantic_annotation).
- active_work_unit: `GRAMMAR-WELLFORMED` → frontier `H`. next_action: **drive each wired grammar's UNKNOWN→0** (full certification: more/targeted witnesses + closed loop) + **`H.5.1`** svpp witness-parseability residual root-cause (svpp `parse_detail` adapter). Per-grammar wiring of the shipped set is COMPLETE.
- in_flight_uncommitted: none. blockers: none.
- push: ~61 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
