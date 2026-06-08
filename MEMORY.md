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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0044` (GRAMMAR-WELLFORMED **`.H.5.1.1` INVESTIGATION**) — **PROVED the svpp witness-parseability root cause tools-first (byte-exact isolation + parser trace + closed-loop A/B); CORRECTS the H.5.1 adjudication.** WHERE (parser): rejecting production is `identifier` in `macro_name := identifier := inline_trivia /[a-zA-Z_]…/` — on the labeled sample the identifier regex is attempted at a bare `\n` and fails; as the FIRST `pp_item` fails (and all alts fail at 0), `pp_item*` matches zero → "did not consume full input at position 0". WHERE+WHY (generator): the bare `\n` is INJECTED by the faithful-spacing trailing guard `StimuliGenerator::regex_tail_greedy_blocker` (`stimuli_generator.rs:7314-7329`) — for the whitespace-only class `[ \t]+` it returns `Some("\n")` (the minimal separator the class can't absorb); in this line-oriented grammar `\n` terminates directives + is excluded from `inline_trivia`, so the macro name lands off-line. The macro name IS generated (DISPROVES H.5.1's "under-fills the name"). CLOSED-LOOP: full svpp 40 samples seed 0 re-parsed → default **24** fail, `--no-word-boundary-spacing` **8** fail ⇒ `\n`-injection causes 16/24; residual 8 are mostly keyword-fusion NEWLY introduced by disabling spacing (`\b`+wordchar) ⇒ fix must be SURGICAL. NO code change this slice (investigation/docs). `generated/` untracked ⇒ NO tracked-artifact change.
- ⚠️ NEXT (the actual `H.5.1.1` fix — SHARED generator code, GLOBAL-measurement required): in `regex_tail_greedy_blocker`, return `None` when the greedy tail class is WHITESPACE-ONLY (e.g. `[ \t]`) — whitespace self-separates so no anti-fusion guard is needed; this leaves `[^\n]*` (content classes, line comments) untouched. Parser-agnostic, one-thing-at-a-time, measure the cross-family stimuli gate + every grammar's cert-coverage (no regression) before landing. Acceptance: svpp `sample_parse_failures` 24→low/0; `pp_define`/`macro_*` leave UNKNOWN; no other grammar regresses.
- locked_program (director 2026-06-08): **ALL EXISTING PARSERS → `Done` (cert-coverage WIRED + clean / UNKNOWN=0 each).** Cert-coverage WIRED (ALL shipped done): json ✓fully_certified, regex ✓(UNKNOWN), rtl_const_expr ✓(UNKNOWN=7), svpp ✓(UNKNOWN=54 + 24 residual ROOT-CAUSED), rtl_frontend ✓(UNKNOWN=133, clean), SV ✓(large UNKNOWN), vhdl ✓(UNKNOWN=85, clean). NOT wired: only the meta/annotation grammars (ebnf, return/semantic_annotation).
- active_work_unit: `GRAMMAR-WELLFORMED` → frontier `H`. next_action: **`H.5.1.1` FIX** (the surgical `regex_tail_greedy_blocker` whitespace-only guard, global-measured) + drive each wired grammar's UNKNOWN→0.
- in_flight_uncommitted: none. blockers: none.
- push: ~63 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
