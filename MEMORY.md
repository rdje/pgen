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
- latest_commit: `PGEN-SV-DOLLAR-LRM-FIDELITY-0001` (`.1` DESIGN/AUDIT docs-only, session #21). Before it: `PGEN-VERILOG-2005-PROFILE-0017` (`.6.4` adjudication: class-D ratchet not-earnable — trio already witnessed via `.6.3.2` carrier-div; `identifier_list` D→B; `hierarchical_tf_identifier` `$root`-mandatory; discovery of the family below).
- 🔴 OPEN DEFECT FAMILY (Root Caused, fix designed): **`SV-0029`** — 19 `kw_sv_dollar_*` tokens (`systemverilog.ebnf:6248-6284`) match mangled text `sv_dollar_*`, NOT the LRM `$*`: LRM `$setup(…)` etc. REJECT everywhere (core 1364-2005 §15!), `$unit::`/module-level severity REJECT under SV profiles, nonsense ACCEPT (severity nonsense ACCEPTS under v2005 too — wave-3 gates needed), `$root.` mis-routes (`system_tf`). **`SV-0030`** (TWO sites) — `scalar_constant:4713` AND UDP `init_val:2429` digit loss (UDP `initial q = 1'b0;` REJECTS!) + `scalar_timing_check_condition:4720` eq-branches PEG-shadowed. `.1` proved wave 1 collision-free (`specify_item` has no system-TF route; tokens referenced only by the 12 timing rules) and named the wave-3 hazard (PEG choice-COMMIT steal by `system_tf_identifier` on `$root`/`$unit`).
- ▶️ next_action: PNT — **`SV-DOLLAR-LRM-FIDELITY.2`** (CODE wave 1: swap the 12 timing-check literals `/sv_dollar_X\b/`→`/\$X\b/`, KEEP token names [pinned-list comparability]; release bump; conformance LRM-accept + mangled-reject locks; cert/union/v2005 re-pins with set-diff-proven deltas). Then `.3` (SV-0030 digits×2 sites + eq reorder ⇒ schema bump), `.4` (SV-only group + v2005 gates + per-site PEG-order proofs). Alternates: `VERILOG-2005-PROFILE.6.5`, `CERT-GEN-BUDGET.4`, `SV-0021..24`/`SV-0028` leaves.
- ⚠️ BINARY/GENERATED STATE: grammar at HEAD = `.6.3` state (census 1450); `generated/systemverilog_parser.rs` + BOTH binaries fresh (Jul 2, incl. `.6.3.2` engine fix). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: SV cert canonical `1328/2/1306/UNKNOWN=20`, union `1325/UNKNOWN=1` (`context_member_method_call`); `verilog_2005` gate pins `1138/2/809/327` (294 NO-reach), matrix 162. VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail`. Disk ~100 GB free. cwd trap: compound `cd rust && …` persists — use absolute paths. Union-gate ~5 GB regen log recurs (port `prune_log` = `GRAMMAR-WELLFORMED.H.12.8.5.2`).
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX intact: rtl_const_expr (default-entry canonical), json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar. EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0`. SV release `1.0.158`/schema 13.
- push: ~85 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
