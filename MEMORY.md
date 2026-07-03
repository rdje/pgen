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
- latest_commit: `PGEN-SV-DOLLAR-LRM-FIDELITY-0004` (`.4` CODE wave 3, session #23): **`SV-0029` CLOSED in full — tree `SV-DOLLAR-LRM-FIDELITY` COMPLETE** — 7 anchor literals fixed (`$root`/`$unit`/severity×4/bare-`$`) + `system_tf_call` steal guards (`!($root .)`/`!($unit ::)`, annotation re-based $3/$4 — a lookahead OCCUPIES a $N slot, codegen-proven) + 3 sv-only lifts (incl. load-bearing `name:{body:"$unit"}` — unresolvable predicate $refs hard-reject) + `!( identifier )`-guarded `rooted_tf_call_sv_only` carrier (the runtime BROADCASTS per-branch predicates rule-wide, `semantic_runtime.rs:735` — open engine question in the tree). SV release `1.0.160`→`1.0.161`, schema `14`→`15`; census 1463.
- verification: 28-probe matrix (LRM ACC/ACC/REJ everywhere; anchors dump-proven, zero system_tf residue; 24/24 mangled+control byte-identical); canonical cert `1341/2/1319/20 spf=0` seeds 0/7/42 residual SET-IDENTICAL; v2005 `1147/4/816/327` deterministic (bare-`$` → NO-reach 295; `$root`/`$unit` → witness-to-PROOF); conformance GREEN 192/0 (64 cases, +4 locks); union GREEN re-pinned (`1341/2/1319/20`, union `1338/1`); shape 18/18 (31 samples, +6 + 6 arms); corpus 10 non-uvm pass + 4 documented uvm mem-cap rows; clippy source-clean; books+ledger (`SV-0029`→`Released`)+contract (`1.0.161`/15, incl. fixing the `.3`-stale line-62 v2005 pin) lockstep.
- ▶️ next_action: PNT — alternates: `VERILOG-2005-PROFILE.6.5` (per-profile proof accounting design), `CERT-GEN-BUDGET.4` (gate-pin the rce canonical cert), `SV-0021`..`SV-0024`/`SV-0028` fix leaves, `GRAMMAR-WELLFORMED.H.12.8.5.2` (union-gate `prune_log` port — the ~5 GB regen log recurred + was pruned again this session).
- ⚠️ BINARY/GENERATED STATE: grammar at HEAD = `.4` state (census 1463); `generated/systemverilog_parser.rs` + BOTH binaries fresh (Jul 3 ~02:18-02:30). Other 9 generated parsers untouched (Jun 25/29 mtimes). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: v2005 gate pins `1147/4/816/327` (295 NO-reach), matrix **192** (64 cases). VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail` (uvm_pkg/uvm_compat_pkg × 2 profiles — expected). Disk ~76 GB free. cwd trap: compound `cd rust && …` persists — use absolute paths (bit AGAIN this session via a relative redirect). Background tasks got externally killed once mid-gates — re-run sequentially when it happens.
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX intact: rtl_const_expr, json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar (canonical UNKNOWN=20; union residual `context_member_method_call`). EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0`. SV release `1.0.161`/schema 15.
- push: ~88 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
