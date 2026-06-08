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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0040` (GRAMMAR-WELLFORMED **`.H.6`**) — **Phase H cert-coverage WIRED for `rtl_frontend`** (the ~5.5 MB synthesizable-RTL frontend parser). Mechanical mirror of H.5: `RTL_FRONTEND_*` vars + build rules + `focus_rtl_frontend` Makefile target + `parse_and_cover_rtl_frontend` registry fn + entry `Some(...)`. Runs at DEFAULT depth 24 (flat `design_item*` entry, no profile, NO `--max-depth`); NOTE `rtl_frontend.ebnf` needs `--features ebnf_dual_run` to parse. MEASURED (count 40, DETERMINISTIC seed 0 run#1==run#2==seed 7): `total=170 witness=37 UNKNOWN=133 (sample_parse_failures=0)` — CLEAN witness-parseability (like json/rtl_const_expr). parser_registry 20/0; lib `--features generated_parsers` builds; clippy strict-SOURCE ok; book Phase-H list + TASK_TREE synced; json(`fully_certified`)/regex/rtl_const_expr(`48/41/7`@d32)/svpp(`73/19/54`) UNAFFECTED. `generated/` gitignored ⇒ NO tracked-artifact change.
- ⚠️ OPEN follow-up `GRAMMAR-WELLFORMED.H.5.1`: svpp has a HIGH witness-parseability residual (24/40 sample_parse_failures) — the svpp stimuli generator over-produces structurally-INVALID directives (comment-dominated, `` `define `` w/o name/body, stray punctuation). Attribution-rule finding, ROUTED not accepted. Likely needs a svpp `parse_detail` adapter to label errors + generator/grammar root-cause.
- locked_program (director 2026-06-08): **ALL EXISTING PARSERS → `Done` (cert-coverage WIRED + clean / UNKNOWN=0 each).** Cert-coverage WIRED: json ✓fully_certified, regex ✓(UNKNOWN), rtl_const_expr ✓(UNKNOWN=7), svpp ✓(UNKNOWN=54 + high residual), rtl_frontend ✓(UNKNOWN=133, clean), SV ✓(large UNKNOWN). NOT wired: **vhdl (last shipped grammar — H.2)**, ebnf, return/semantic_annotation.
- active_work_unit: `GRAMMAR-WELLFORMED` → frontier `H`. next_action: **H.2 wire vhdl** (the LAST unwired shipped grammar; needs a `focus_vhdl` target + the full `ebnf_dual_run` frontend regen of `vhdl.json` then parser regen + the HEAVY vhdl conformance re-verify; de-risked by H.4/H.6's checkout-illusion zero-drift proof — verify vhdl's own drift first) → then drive each wired grammar's UNKNOWN→0 + `H.5.1` svpp residual.
- in_flight_uncommitted: none. blockers: none.
- push: ~60 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
