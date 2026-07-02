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
- latest_commit: `PGEN-VERILOG-2005-PROFILE-0011` (`VERILOG-2005-PROFILE.6.1`, INVESTIGATION — adjudication + ledger/docs, ZERO code change): the `verilog_2005` profiled-cert **310-UNKNOWN residual fully adjudicated tools-first** (3-step protocol + `PGEN_REACH_PATH_DUMP` + parse/AST probes + scoped predicate trace). 33 genuine-gap candidates = 17 store-gated SV-only use-sites (producers profile-gated; `🚫 has_fact [type_name,…]` trace) + 8 spurious reach paths (BFS skips gated MANDATORY siblings) + **2 LEAKS ledgered `SV-0025`** (`delay_value` `1step`/`time_literal` un-gated: `wire #1step w;`/`wire #10ns w;` ACCEPT) **+ `SV-0026`** (`description → package_item` `$unit` surface: top-level `wire w;`/`reg r;` ACCEPT) + 6 in-profile ratchet targets + 1 canonical residual. Prior: `-0010` (afab44e7, `.5` union-gate re-baseline), `-0009` (d11c08d7, `.4.3` closure surface `1138/2/826/310/spf=0`).
- 🧭 SESSION CONTEXT (2026-07-02, session #19): startup read done (core docs direct + 3 Explore digests: full book, roadmap/LIVE/COMMIT, codebase). Book digest flagged candidate BOOK DRIFT (grammar-wellformedness.md narrative tops at release `1.0.151`/schema 4 vs actual `1.0.158`/13; diagnosing-unknowns.md:63 stale-illustrative cert line) — needs independent verification before edits; PNT candidate (docs slice).
- ▶️ next_action: PNT — **`VERILOG-2005-PROFILE.6.2`** (fix `SV-0025`: branch-lift SV-only `delay_value` alternatives + corpus reject rows + contract re-baseline; acceptance template in the leaf). Then `.6.3` (`SV-0026` `$unit` gate, EXPECTS cert re-pin), `.6.4` (witness ratchet: delay-ident/hier-tf/randomize/scalar_constant routes), `.6.5` (per-profile proof accounting DESIGN: 277 + class-A/B + planner off-profile scaffolds). Alternates: `SV-0021..24` fix leaves; Phase-S Liberty/SDC fork.
- ⚠️ BINARY/GENERATED STATE: grammar + all 7 generated parsers UNTOUCHED this session (release `1.0.158`, schema `13`); binaries current. COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: SV cert canonical `UNKNOWN=20` (`total=1324`), union `UNKNOWN=1` (`context_member_method_call`); `verilog_2005` gate pins `1138/2/826/310` UNCHANGED (leak reject-rows land with `.6.2`/`.6.3` fixes + re-baseline). Adjudication evidence archived in session scratchpad (`v2005_cert_{dumpall,probes,reachpaths}_seed0.txt`, `leakprobes/`) — durable copy = tree `.6.1` Findings. VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail`.
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX: rtl_const_expr, json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar. EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0` (profiles: sv_2017/sv_2023/verilog_2005/vhdl_1076_2019/regex_default).
- push: ~76 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
