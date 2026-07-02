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
- latest_commit: `PGEN-VERILOG-2005-PROFILE-0009` (`VERILOG-2005-PROFILE.4.3`, CLOSURE — no code-classified change): **the `verilog_2005` closure surface landed** — 46-file conformance corpus promoted to `rust/test_data/grammar_quality/verilog_2005_conformance/`; repo-standard **`verilog_2005_conformance_gate`** (contract JSON + script + Makefile: 0-orphan lint lock + 138-check 3-profile matrix + 2 alias probes + profiled cert baseline `total=1138 proof=2 witness=826 UNKNOWN=310 spf=0`, deterministic seeds 0/7/42 — 277/310 = profile-unreachable-by-design); downstream SV-contract § "Dialect Profile — verilog_2005" + stale embedding-baseline `1.2.0`→`1.3.0` fixed; **LIVE dialect block `In Progress`→`Mostly Done`** (not `Done`: curated corpus + `SV-0024` known leak). Prior: `-0008` (9f3180c5, `.4.2` lifts), `-0007` (cd1dfa14, `.4.1` coherence).
- 🧭 SESSION CONTEXT (2026-07-02, session #18): full mandated startup read done (core docs + 2 Explore digests). `.4.3` executed: scratch corpora RESCUED from prior sessions' `/private/tmp` scratchpads (still present) → matrix re-measured on HEAD before pinning (reproduced `.4.1`/`.4.2` adjudications exactly) → gate authored on the union-gate template. Ops finding: `focus_systemverilog` streams a ~5 GB log — the gate prunes stage logs >10 MB to last-2000-lines.
- ▶️ next_action: **`VERILOG-2005-PROFILE.5`** — re-baseline `sv_cert_recognized_union_gate` count pins (`systemverilog_recognized_cert_union_contract.json`: pinned `1304/1/1283+1302` → actual `1324/2/1302+1321`; semantic invariants intact; root cause git-traced in tree `.2` Findings). Then PNT: `.6` (profiled-cert ratchet, proposed) / `SV-0021..24` fix leaves / BIG-LANE FORK: Phase-S Liberty/SDC; deferred-hard SV literal `UNKNOWN=0`.
- ⚠️ BINARY/GENERATED STATE: grammar + all 7 generated parsers UNTOUCHED this session (release `1.0.158`, schema `13`); binaries current (gate rebuilt both). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: SV cert canonical `UNKNOWN=20` (`total=1324`; union-gate pins stale → `.5`). Sound 4-config union `UNKNOWN=1` (`context_member_method_call`). New gate: `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate`. VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail` (PARSE-TERMINATION.4 posture; non-uvm 10/10 green). Cert-inflation caveat: [[project_cert_coverage_tournament_loser_leak]].
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX: rtl_const_expr, json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar. EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0` (profiles: sv_2017/sv_2023/verilog_2005/vhdl_1076_2019/regex_default).
- push: ~75 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
