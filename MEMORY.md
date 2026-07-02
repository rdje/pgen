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
- latest_commit: `PGEN-VERILOG-2005-PROFILE-0017` (`VERILOG-2005-PROFILE.6.4`, INVESTIGATION docs-only, session #21): the class-D witness ratchet ADJUDICATED not-earnable (trio ALREADY witnessed inside the `809/327` pins via the `.6.3.2` carrier-div `module m(a,b);` rescue; `identifier_list` reclassified D→B — gated mandatory `constraint_block` tail; `hierarchical_tf_identifier` = `$root.`-mandatory SV-only, AST-proven `system_tf` mis-route; `scalar_constant` blocked on `SV-0030`).
- 🔴 MAJOR DISCOVERY (ledgered, fix tree spawned): **`SV-0029`** — 19 `kw_sv_dollar_*` tokens (`systemverilog.ebnf:6248-6284`) match mangled literal text `sv_dollar_*`, NOT the LRM `$*` spellings: LRM `$setup(…)` specify timing checks REJECT under EVERY profile (core 1364-2005 §15!), `$unit::y` + module-level `$fatal;` REJECT under sv_2017, nonsense spellings ACCEPT, `$root.` mis-routes. Provenance: `tools/extract_systemverilog_lrm_profiles.py:315` name-canonicalization leaked into token literals; snapshots `$`-faithful; no pre-parse rewrite (grep rust/src = 0). **`SV-0030`** — `scalar_constant` digit loss (`1'b` vs LRM `1'b0…`) + `scalar_timing_check_condition` eq-branches PEG-shadowed by the bare-`expression` first branch.
- ▶️ next_action: PNT — **`SV-DOLLAR-LRM-FIDELITY.1`** (DESIGN/audit: per-token × per-profile consequence matrix over the 19 tokens, PEG-collision audit [`$setup` currently lexes as `system_tf_identifier` in expr contexts], AST-shape/schema impact, gating map: 12 timing checks = BOTH dialects; `$root`/`$unit`/severity = SV-only → v2005 gates post-fix). Then `.2`/`.3` CODE waves. Alternates: `VERILOG-2005-PROFILE.6.5`, `CERT-GEN-BUDGET.4` (rce cert gate-pin), `SV-0021..24`/`SV-0028` fix leaves, book-drift docs slice (`grammar-wellformedness.md` stale 1.0.151/schema-4 narrative).
- ⚠️ BINARY/GENERATED STATE: grammar at HEAD = `.6.3` state (census 1450); `generated/systemverilog_parser.rs` + BOTH binaries fresh (Jul 2, incl. `.6.3.2` engine fix). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: SV cert canonical `1328/2/1306/UNKNOWN=20`, union `1325/UNKNOWN=1` (`context_member_method_call`); `verilog_2005` gate pins `1138/2/809/327` (294 NO-reach), matrix 162. VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail`. Disk ~100 GB free. cwd trap: compound `cd rust && …` persists — use absolute paths. Union-gate ~5 GB regen log recurs (port `prune_log` = `GRAMMAR-WELLFORMED.H.12.8.5.2`).
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX intact: rtl_const_expr (default-entry canonical), json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar. EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0`. SV release `1.0.158`/schema 13.
- push: ~84 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
