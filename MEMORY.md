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
- latest_commit: `PGEN-SV-DOLLAR-LRM-FIDELITY-0002` (`.2` CODE wave 1, grammar-only, session #21): **`SV-0029` wave 1 FIXED** — the 12 specify timing-check tokens now carry IEEE `$` spellings (SV release `1.0.158`→`1.0.159`, schema 13 unchanged); 12/12 LRM REJECT→ACCEPT + 12/12 mangled ACCEPT→REJECT both dialects; pins EXACT everywhere (zero re-pins: canonical `1328/2/1306/20` residual set-identical; union `1325/1`; v2005 `1138/2/809/327`, conformance 168/0 with 2 new locks). Session #21 chain: `-0001` (`.1` design/audit — wave-1 collision-freedom proven; `SV-0030` SECOND site `init_val`), `PGEN-VERILOG-2005-PROFILE-0017` (`.6.4` adjudication + family discovery).
- 🔴 OPEN (Fix In Progress): **`SV-0029`** remaining = SV-only group `$root`/`$unit`/severity×4/bare-`$` (7 tokens; LRM spellings REJECT under SV profiles; nonsense ACCEPTS incl. `sv_dollar_fatal;` under v2005 → needs gates; PEG choice-COMMIT steal hazard by `system_tf_identifier` — per-site order proofs required). **`SV-0030`** (TWO sites): `scalar_constant:4713` + UDP `init_val:2429` digit loss (`initial q = 1'b0;` REJECTS!) + `scalar_timing_check_condition:4720` eq-branches PEG-shadowed — shape-affecting fix (schema bump).
- ▶️ next_action: PNT — **`SV-DOLLAR-LRM-FIDELITY.3`** (CODE wave 2: restore the 10 LRM digit alternatives at BOTH sites — new sha1-named `1'b0`/`1'b1`/`1'B0`/`1'B1` tokens, remove the digit-less `1'b`/`1'B` with their last sites — + reorder eq/case_eq/ne/case_ne BEFORE bare `expression`; `e == 1'b0` gains `kind:"eq"` ⇒ schema 13→14 + shape samples + book/contract). Then `.4` (SV-only group + v2005 gates). Alternates: `VERILOG-2005-PROFILE.6.5`, `CERT-GEN-BUDGET.4`, `SV-0021..24`/`SV-0028` leaves.
- ⚠️ BINARY/GENERATED STATE: grammar at HEAD = `.2` state (census 1450, 12 `$`-literal tokens); `generated/systemverilog_parser.rs` + BOTH binaries fresh (Jul 2 23:15, wave-1 literals in). Other 9 generated parsers untouched (Jun 25/29 mtimes). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: SV cert canonical `1328/2/1306/UNKNOWN=20`, union `1325/UNKNOWN=1` (`context_member_method_call`); `verilog_2005` gate pins `1138/2/809/327` (294 NO-reach), matrix **168** (56 cases). VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail`. Disk ~100 GB free. cwd trap: compound `cd rust && …` persists — use absolute paths. Union-gate ~5 GB regen log recurs (port `prune_log` = `GRAMMAR-WELLFORMED.H.12.8.5.2`; pruned again this session).
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX intact: rtl_const_expr (default-entry canonical), json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar. EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0`. SV release `1.0.159`/schema 13.
- push: ~86 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
