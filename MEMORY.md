# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> This is the bounded layer-A resume pointer per `MEMORY_ARCHITECTURE.md`.
> OVERWRITE the "Current state" block each update — never append history here.
> (History is in git (layer D) + the task-tree logs (layer B); durable
> facts/decisions are in `docs/decisions/` (layer C).)

## How to resume
- Read `MEMORY_ARCHITECTURE.md` (memory/continuity), `README.md` (project), and `TOOLBOX.md` (debug toolbox — USE FIRST).
- Work is tracked in task-trees under `docs/tasks/`; index `docs/TASK_TREE.md`; follow `COMMIT.md`.
- Durable facts / standing disciplines / decisions: `docs/decisions/` (see `INDEX.md`). Live status: `LIVE_ACHIEVEMENT_STATUS.md`; changelog: `CHANGES.md`.
- ⛔ A code change MUST pass the **acceptance checklist** (ROOT CAUSE + ADDRESSED + NO REGRESSION, evidence-backed) in its task leaf — enforced by `scripts/check_doctrines.sh` via `.githooks/pre-commit` (run `git config core.hooksPath .githooks` once per clone).

## Current state (OVERWRITE this block each update — do not append)
- latest_commit: `PGEN-VERILOG-2005-PROFILE-0018` (`.6.5`, session #25, 1st commit): **per-profile proof accounting DESIGNED + DECIDED (staged)** — the three profile-gated v2005 residual classes SHALL become sound `proof` entries: P1 = entry-UNIVERSE unreachability over the active tree w/ satisfiability-honest edges (the 295 NO-reach set is HETEROGENEOUS — the entry-relative `library_text` cohort must never be branded dead), P2 = unproducible-mandatory-POSITIVE-store-gate fixpoint (kind-level, `lacks_fact` excluded, inert under live `@import_from_library`). Staged: `.6.6` read-only env-gated classification (headlines byte-identical, `reach_hops_pass` untouched) → `.6.7` certificate promotion + ALL gate re-pins same-slice. Contract `baseline_note` prose fixed (326→327 + dup phrase + missing `.4` hop); pins untouched; conformance gate re-run GREEN. Full design in `VERILOG-2005-PROFILE.md` "`.6.5` Findings".
- verification: fresh seed-0 `DUMP_ALL` cert reproduced pins byte-identical (`1147/4/816/327` spf=0); mechanisms pinned to file:line (`grammar_wellformedness.rs:306-313`, `systemverilog.ebnf:1018-5481`, `stimuli_generator.rs:6869`/`:7120`); `verilog_2005_conformance_gate` GREEN end-to-end post-edit; doctrine enforcer PASS.
- ▶️ next_action: PNT — next: `VERILOG-2005-PROFILE.6.6` (the read-only residual-classification CODE leaf — P1+P2 pure fns in `grammar_wellformedness.rs` + `PGEN_CERT_RESIDUAL_CLASSIFICATION=1` cert-report surface; verify machine classes vs the `.6.5` recorded 18A/7B/1E/6-drift split), alternates `SV-0021`..`SV-0024`/`SV-0028` fix leaves.
- ⚠️ BINARY/GENERATED STATE: grammar at HEAD unchanged (census 1463); SV parser + both binaries fresh (Jul 3). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: v2005 gate pins `1147/4/816/327` (295 NO-reach), matrix **192** (64 cases). VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail` (expected). cwd trap: compound `cd rust && …` persists — use absolute paths.
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX intact: rtl_const_expr (gate-locked), json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar (canonical UNKNOWN=20; union residual `context_member_method_call`). EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0`. SV release `1.0.161`/schema 15.
- push: ~90 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
