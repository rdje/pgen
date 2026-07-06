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
- latest_commit: `PGEN-SEM-FINDINGS-0002` (`SEM-FINDINGS.1`, session #50, DOC-only): **F5 effects-timing NORMATIVE** — new spec section *Effects Timing (Normative)* (effects commit on rule success INCLUDING zero-length; rollback transactional never structural — the quantifier guard is not a transaction boundary; consume-≥1-byte idiom for no-effect-when-empty) + top-book ROLLBACK note + sem-ann book disambiguation (the old "iff committed parse survived" was ambiguous); all cite the `sem_zero_len_emit` pin; both book gates GREEN. Earlier #50: F4+F2+F6 trees CLOSED (semantic suite 24/24; lint hard-gates undefined refs 13/13 grammars clean; names textual).
- ▶️ next_action: **`SEM-FINDINGS.2`** (close-out: verify tree-level acceptance §6 — all six findings fixed-or-documented, `parse_harness_semantic_gate` pins post-fix semantics 24/24, SV release policy handled (all NO-bump, verified per-tree) — then mark the dispatch tree `complete`). THEN pick the next PNT target (parked/optional: `MEMO-STORE-SOUNDNESS.3` perf headroom; PARSE-HARNESS `.7`/`.8`/`.9`; `.6.1`'s 3 findings; `.5.5` director PENDING; `.5.3`; `.5.1` `@whitespace_sensitive`). ⛔ VERILOG-AMS PARKED. ⛔ SV union `context_member_method_call` reach-gap HARD-BLOCKED (do NOT pick).
- ⚠️ BINARY/GENERATED STATE: `generated/*` current (#49 cold-start regen; scratch slot restored + `focus_scratch` re-run in #50 after a probe detour — NOTE the probe trap: the release probe embeds scratch at COMPILE time, rebuild it after any slot swap). Release probe rebuilt post-F2-engine-change (#50). Debug lib/test binaries current (gates rebuilt them). Oracle workdirs warm. Gate runs: semantic ~56 s / combinator ~48 s / equivalence ~20 s.
- ⚠️ OPERATIONAL: canonical cert `1343/10/1321/12`, union `1343/10/1332/1` residual `context_member_method_call` — SV byte-identical through the `.2` regen (re-verified `sv_cert_recognized_union_gate`). Differential-equiv CERTIFIED 11 (json/semantic_annotation/rtl_frontend/vhdl/**systemverilog (sv_2017)**/scratch/regex/systemverilog_preprocessor/ebnf/return_annotation/rtl_const_expr; DEFERRED empty; EXCLUDED → builtin_*). v2005 pins **`1115/328/773/14`**; conformance matrix 240. VHDL strict floor 75. cwd trap: use absolute paths. Clippy: generated stage has pre-existing `eq_op` errors (non-strict by design); parse_harness* modules clippy-clean.
- locked_program (director 2026-06-08): all parsers → Done (UNKNOWN=0). Fully-certified SIX intact (rtl_const_expr, json, svpp, regex, vhdl, rtl_frontend). SV = only non-fully-certified shipped grammar (canonical UNKNOWN=12; union residual `context_member_method_call`). Roadmap family Verilog-AMS: PARKED (reaffirmed director 2026-07-04). SV release **`1.0.167`**/schema **`16`**.
- push: ~140 unpushed after this commit — next push at ~200 per `feedback_push_pacing` unless director says "push now".
