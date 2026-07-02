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
- latest_commit: `PGEN-SV-DOLLAR-LRM-FIDELITY-0003` (`.3` CODE wave 2, session #22): **`SV-0030` CLOSED both sites** — ten IEEE digit alternatives at `scalar_constant`+UDP `init_val` (8 new `\b` tokens, digit-less removed) + compare branches REACHABLE via NEW precedence-restricted lhs (`scalar_timing_check_compare_lhs`, Table-11-2-rows-3–7 operator tier) + `!binary_operator`/`!tick` guards; SV release `1.0.159`→`1.0.160`, schema `13`→`14`. KEY design fact: the `.1`-sketched reorder was tool-proven INERT (greedy `expression` lhs eats `== 1'b0`); fallbacks byte-identical on 9 guard probes.
- verification: canonical cert `1337/2/1315/20 spf=0` seeds 0/7/42 residual SET-IDENTICAL; v2005 `1147/2/819/326` (sole diff = `scalar_constant` ratchet EARNED, NO-reach 294 identical); conformance GREEN 180/0 (60 cases, +4 locks) re-pinned; union GREEN re-pinned (`1337/2/1315/20`, union `1334/1`); shape 18/18 (25 samples, +4 + 4 dispatch arms); books+ledger (`SV-0030`→`Released`)+contract lockstep.
- ▶️ next_action: PNT — **`SV-DOLLAR-LRM-FIDELITY.4`** (CODE wave 3, the most delicate: `$root`×2 / `$unit` / severity×4 / bare-`$` literal fixes + per-site PEG-order proofs — `system_tf_identifier` token-level steal hazard, `.1`(d) — + `verilog_2005` gates incl. closing the live `sv_dollar_fatal;` v2005 nonsense-ACCEPT; `$root` fix ⇒ `kind:"system_tf"`→hierarchical kinds = schema adjudication). Alternates: `VERILOG-2005-PROFILE.6.5`, `CERT-GEN-BUDGET.4`, `SV-0021`..`SV-0024`/`SV-0028` leaves.
- ⚠️ BINARY/GENERATED STATE: grammar at HEAD = `.3` state (census 1459); `generated/systemverilog_parser.rs` + BOTH binaries fresh (Jul 3 ~00:30, digits+compare rules in). Other 9 generated parsers untouched (Jun 25/29 mtimes). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: v2005 gate pins `1147/2/819/326` (294 NO-reach), matrix **180** (60 cases). VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail`. Disk ~93 GB free. cwd trap: compound `cd rust && …` persists — use absolute paths (bit again this session). Union-gate ~5 GB regen log recurs (port `prune_log` = `GRAMMAR-WELLFORMED.H.12.8.5.2`; pruned again this session).
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX intact: rtl_const_expr, json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar. EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0`. SV release `1.0.160`/schema 14.
- push: ~87 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
