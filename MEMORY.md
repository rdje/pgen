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
- latest_commit: `PGEN-VERILOG-2005-PROFILE-0010` (`VERILOG-2005-PROFILE.5`, RE-BASELINE — contract-pin + docs): **union-gate count pins re-baselined** `1304/1/1283/1302`→`1324/2/1302/1321` (`systemverilog_recognized_cert_union_contract.json`; stale-pin gap from the SV-AST-SHAPE-FIDELITY + verilog_2005 named-lift campaigns) — `sv_cert_recognized_union_gate` **RED→GREEN** fresh (semantic invariants byte-identical: canonical `UNKNOWN=20`, union `UNKNOWN=1`, residual `context_member_method_call`, `spf=0`, seeds 0/7/42); book + SV-contract stale pins updated with provenance. Prior: `-0009` (d11c08d7, `.4.3` closure surface — 46-file corpus + `verilog_2005_conformance_gate` + profiled cert baseline `1138/2/826/310/spf=0` + contract § + LIVE `In Progress`→`Mostly Done`), `-0008` (9f3180c5, `.4.2` lifts), `-0007` (cd1dfa14, `.4.1` coherence).
- 🧭 SESSION CONTEXT (2026-07-02, session #18): startup read done (core docs + 2 Explore digests). `.4.3`: scratch corpora RESCUED from prior sessions' `/private/tmp` scratchpads → matrix re-measured on HEAD before pinning (reproduced `.4.1`/`.4.2` exactly) → gate authored on the union-gate template (stage logs >10 MB pruned to last-2000-lines — `focus_systemverilog` streams ~5 GB). `.5`: pins-only re-baseline, one concern per commit. Follow-up noted: port `prune_log` to `sv_cert_recognized_union_gate.sh` (own surface).
- ▶️ next_action: PNT — **`VERILOG-2005-PROFILE.6`** (profiled-cert baseline ratchet, proposed: adjudicate 310 UNKNOWN = 277 profile-unreachable-by-design + ~33 genuine) OR the `SV-0021..24` ledgered fix leaves (each needs its own leaf/tree before code) OR BIG-LANE FORK: Phase-S Liberty/SDC (`PNR-LIBERTY`/`PNR-SDC` proposed trees, `.1` scoping); deferred-hard SV literal `UNKNOWN=0`.
- ⚠️ BINARY/GENERATED STATE: grammar + all 7 generated parsers UNTOUCHED this session (release `1.0.158`, schema `13`); binaries current (gate rebuilt both). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: SV cert canonical `UNKNOWN=20` (`total=1324`; union-gate pins stale → `.5`). Sound 4-config union `UNKNOWN=1` (`context_member_method_call`). New gate: `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate`. VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail` (PARSE-TERMINATION.4 posture; non-uvm 10/10 green). Cert-inflation caveat: [[project_cert_coverage_tournament_loser_leak]].
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX: rtl_const_expr, json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar. EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0` (profiles: sv_2017/sv_2023/verilog_2005/vhdl_1076_2019/regex_default).
- push: ~75 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
