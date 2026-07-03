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
- latest_commit: `PGEN-VERILOG-2005-PROFILE-0020` (`.6.8`, session #27): **`SV-0032` void boundary leak CLOSED** — grammar-only: two shape-preserving `_sv_only` lifts (`data_type_or_void_sv_only` :1838 + `void_cast_statement_sv_only` :5174, both `@profiles: ["sv_2017","sv_2023"]`) gate the SV-only `void` surface (return type + `void'(…)` cast) out of `verilog_2005`. `void` is categorically absent from IEEE 1364-2005 (§10.4; nowhere in Annex A). First of the three `.6.6`-surfaced leaks closed.
- verification: `function void f;` + `initial void'(f());` flip ACCEPT→REJECT under v2005, still ACCEPT under sv_2017/sv_2023; sv_2017 AST byte-identical (transparent passthrough, 0 wrapper keys). Both gates GREEN: conformance (orphans 0, matrix **198**/0, cert `1147/4/816/327` seeds 0/7/42 — `kw_void` genuine→NO-reach, NO-reach 295→296, hard pins unchanged); union re-pinned `1341→1343` (canonical `1343/2/1321/20`, union `1343/2/1340/1` residual `context_member_method_call`, both lifts witnessed under sv_2017). ast_shape 18/18, clippy source strict-clean, mdbook green; json `9/0` + regex `198/0` inert.
- ▶️ next_action: PNT — next: the remaining `.6.6` leaks `SV-0033` (dynamic-array `[]` — `unsized_dimension` in ≥3 dimension contexts, needs per-context adjudication) then `SV-0031` (`::` scope-resolution surface, broader), each its own `_sv_only` fix leaf; OR `VERILOG-2005-PROFILE.6.7` (certificate promotion + ALL gate re-pins; FIRST decision = conformance gate entry-universe wiring). Leaks-first yields a cleaner grammar for `.6.7`'s promotion.
- ⚠️ BINARY/GENERATED STATE: grammar census now **1465** (+2 void lifts); SV parser + both binaries fresh (Jul 3, post-`.6.8` rebuild). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: v2005 gate pins `1147/4/816/327` (296 NO-reach), matrix **198** (66 cases; 15 accept + 51 reject). union pins `1343/2/1321/1340`. v2005 open-waiver set now FOUR (`SV-0024`/`SV-0028`/`SV-0031`/`SV-0033`). VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail` (expected). cwd trap: compound `cd rust && …` persists — use absolute paths.
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX intact: rtl_const_expr (gate-locked), json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar (canonical UNKNOWN=20; union residual `context_member_method_call`). EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0`. SV release `1.0.161`/schema 15 (unchanged — `.6.8` SV profiles AST-byte-invariant).
- push: ~94 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
