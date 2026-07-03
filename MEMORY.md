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
- latest_commit: `PGEN-VERILOG-2005-PROFILE-0019` (`.6.6`, session #26, 1st commit): **the READ-ONLY machine residual-classification LANDED** — P1 `profile_entry_positively_live` + P2 `classify_profile_residual` (pure, parser-agnostic) in `grammar_wellformedness.rs` + cert-report print behind `PGEN_CERT_RESIDUAL_CLASSIFICATION=1`; default output diff-proven byte-identical; on v2005 the machine split `292/17/18 = 327` reconciled the manual adjudication EXACTLY and corrected it twice tools-first (`hierarchical_tf_identifier` stale note; `class_scoped_tf_call` negative-gated escape) → THREE new ledgered v2005 leaks **`SV-0031`** (`::` surface: `p::f()`/`p::C::f()`/`p::X` accept), **`SV-0032`** (`void` return type), **`SV-0033`** (dynamic `[]`) — all `Root Caused`, own fix leaves. Full record: "`.6.6` Findings".
- verification: v2005 + canonical+union seed-0 outputs byte-identical pre↔post (env unset); classification md5-identical seeds 0/7/42; union gate GREEN (`20`/union `1`), conformance gate GREEN (192 matrix + pins `1147/4/816/327`); fully-certified six byte-identical + rce gate GREEN; 39/39 module unit tests; mdbook + km + clippy green.
- ▶️ next_action: PNT — next: `VERILOG-2005-PROFILE.6.7` (certificate promotion + ALL gate re-pins same-slice; FIRST decision = the conformance gate's entry-universe wiring — its cert command has no `--cert-union-config`, and a single-entry universe would falsely prove the library cohort dead) OR the ledgered fix leaves `SV-0021`..`SV-0024`/`SV-0028`/`SV-0031`..`SV-0033`.
- ⚠️ BINARY/GENERATED STATE: grammar at HEAD unchanged (census 1463); SV parser fresh; both binaries fresh (Jul 3, post-`.6.6` rebuild). COLD-START regen: `make -C rust SHELL=/bin/bash regex_parser_bootstrap` → `make -C rust focus_{json,vhdl,systemverilog_preprocessor,rtl_const_expr,rtl_frontend,systemverilog}` → rebuild both binaries.
- ⚠️ OPERATIONAL: v2005 gate pins `1147/4/816/327` (295 NO-reach), matrix **192** (64 cases). v2005 open-waiver set now FIVE (`SV-0024`/`SV-0028`/`SV-0031`..`33`). VHDL strict floor `75`. uvm on THIS 24 GB host: 4 triage rows mem-cap `parse_fail` (expected). cwd trap: compound `cd rust && …` persists — use absolute paths.
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX intact: rtl_const_expr (gate-locked), json, svpp, regex, vhdl, rtl_frontend. SV = the only non-fully-certified shipped grammar (canonical UNKNOWN=20; union residual `context_member_method_call`). EBNF meta-grammar self-hosting 12/12. Embedding API `1.3.0`. SV release `1.0.161`/schema 15.
- push: ~91 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
