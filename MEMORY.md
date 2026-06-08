# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> This is the bounded layer-A resume pointer per `MEMORY_ARCHITECTURE.md`.
> OVERWRITE the "Current state" block each update — never append history here.
> (History is in git (layer D) + the task-tree logs (layer B); durable
> facts/decisions are in `docs/decisions/` (layer C).)

## How to resume
- Read `MEMORY_ARCHITECTURE.md` (the memory/continuity system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; the index is `docs/TASK_TREE.md`; follow `COMMIT.md`.
- Durable facts / standing disciplines / decisions live in `docs/decisions/` (see its `INDEX.md`).
- Live status: `LIVE_ACHIEVEMENT_STATUS.md`; changelog: `CHANGES.md`.

## Current state (OVERWRITE this block each update — do not append)
- latest_commit: (this commit) `PGEN-SVPP-EXPANSION-0002` (DOCS) — recorded the grammar-home design analysis in the `SVPP-EXPANSION` tree Open Questions per the director's design Q ("enhance svpp.ebnf vs a separate expression-parser EBNF?"): LEAN = **enhance `svpp.ebnf`** (expansion is ~90% transformation logic over what svpp already parses; the one grammar gap = macro-USAGE recognition in `non_directive_text`, a lexical enhancement; NO separate grammar — std SV `` `ifdef `` takes a macro identifier not a C-`#if` expression); confirm at `.1` SCOPING with IEEE 1800 §22 + slang/Verible/Verilator (parse-tree vs token-stream fork). Prior: `-0001` SVPP-EXPANSION tree created (NEXSIM release prerequisite, build-later, **sequenced AFTER the locked program**); `-0049` (H.5.1.3.2) svpp `condition_text -> $text` → **svpp cert-coverage now CLEAN** (sample_parse_failures 1→0 both seeds; svpp schema **3→4**, release **1.0.4→1.0.5**; shape-contract GREEN, lib 686/0, book gate GREEN, cross-family PASS).
- ⛔ STANDING [[feedback_full_startup_read_includes_mdbook]] (2026-06-08, emphatic 3×): READ `docs/book/` at startup, exactly+completely; before any fix ask "is there already a declarative construct?". (Honored here — `$text` is the declarative answer.)
- 🧭 DIRECTOR SESSION 2026-06-08 (key steer): chose "build FULL store-aware generation" → THEN, on my tool-backed evidence (SV cert-coverage: only **3 lexical** sample-fails + just **36/1160** rules store-`@predicate`-gated → semantic is ~3%-sufficient; the dominant levers are **reach/diversity + lexical**, NOT semantic), AGREED semantic barely matters for cert-coverage + DELEGATED ("do as you think is best"). Corrected me **svpp ≠ sv**. Then asked: do svpp+sv compose? **ANSWER (tool-verified): NO** — PGEN's svpp is a directive-STRUCTURE parser (NOT an expanding preprocessor); sv parses SV directly + tolerates directives inline via `compiler_directive` and does NOT expand includes/macros (`sv.ebnf:178`). No expansion stage connects them (a real composition gap if a true preprocess→parse pipeline is ever wanted).
- locked_program (director 2026-06-08): **ALL EXISTING PARSERS → `Done` (cert-coverage WIRED + clean / UNKNOWN=0 each).** All shipped grammars WIRED (Phase H complete). cert-coverage CLEAN (sample-fails 0): json✓(fully_certified) regex✓ rtl_const_expr✓ rtl_frontend✓ vhdl✓ SV✓ **svpp✓ (this commit)**. Remaining = UNKNOWN>0 (reach): rtl_const_expr 7, vhdl 85, rtl_frontend 133, SV 1142, svpp 4, regex 133.
- active_work_unit: `GRAMMAR-WELLFORMED.H` UNKNOWN→0 drive — the DOMINANT lever (tool-proven) is **reach/diversity + lexical**, NOT semantic store-aware. next_action: PNT — drive a wired grammar's UNKNOWN→0 (smallest: rtl_const_expr 7), and/or scope the reach/diversity capability (STIMULI-SIGNOFF k-path / GRAMMAR-WELLFORMED B2/C1/C2). `STORE-AWARE-GEN.4` stays owned but is ~3%-sufficient for the cert-coverage goal.
- in_flight_uncommitted: none (this commit lands H.5.1.3.2). blockers: none.
- push: ~69 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
