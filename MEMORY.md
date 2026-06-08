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
- latest_commit: (this commit) `PGEN-GRAMMAR-WELLFORMED-0048` (DOCS) — captured the director's GARBAGE/STORE-AWARE reframe + closed `H.5.1.3.1` (deferral implemented→insufficient→reverted, NO code landed). Prior: `-0046` (H.5.1.2) svpp `\b`-keyword fusion FIXED via `[>! /\w/]` lexical-annotation + a `collect_rule_body` frontend fix (svpp residual 8→1, lib 621/621, cross-family PASS); `-0047` H.5.1.3 root-cause checkpoint; `-0045` (H.5.1.1) whitespace-only greedy-tail guard.
- ⛔ DIRECTOR FEEDBACK (2026-06-08, emphatic, 3×): I SKIPPED `docs/book/` at startup (ramp-up item #2) → missed the `[>! …]` construct → root-caused into a generator heuristic. New STANDING [[feedback_full_startup_read_includes_mdbook]] + KM card `lexical-follow-restrictions`. READ THE BOOK at startup, exactly + completely; before any fix ask "is there already a declarative construct?".
- 🎯 DIRECTOR REFRAME (2026-06-08, key insight): the stimuli generator **outputs GARBAGE** (tool-backed: ~48% of every svpp sample is block-comment chars, ~9.4 comments/sample, + punctuation soup) because it is **NOT properly steered by the EBNF** — it samples context-free. The fix = **full semantic-fact-store support → "output text based on CONTEXT == semantic fact store"** (a `` `MACRO `` only after its `` `define ``, a type only after its typedef, etc.). This ELEVATES tree **STORE-AWARE-GEN** from the `fact_count_at_least` MVP to FULL context-aware generation (all predicates + `@emit_fact` emission + scope-awareness, all grammars). Honest scope: comment-density (a `*`-quantifier WEIGHTING knob) + the svpp `condition_text` lexical-context residual are ADJACENT facets, not store facts. See [[project_store_aware_generation]] AMENDMENT.
- `H.5.1.3`/`H.5.1.3.1` DONE (`-0048`, NO code landed): svpp residual-1 ROOT-CAUSED (generator injects a bare `\n` after `condition_text` when a `condition_atom` follows → strands a `` `" `` stringize). Class-only "line-rest" fix UNSOUND (breaks `[ \ta-z]+→Some("\n")`); successor-aware `\n` deferral IMPLEMENTED but tools-measured INSUFFICIENT (over-fires on comment-led successors — `condition_text` needs NO `\n` guard EVER, a GRAMMATICAL-CONTEXT fact no class check captures) → REVERTED (`stimuli_generator.rs` byte-identical to HEAD). Residual = lexical-context → **Phase C**.
- locked_program (director 2026-06-08): **ALL EXISTING PARSERS → `Done` (cert-coverage WIRED + clean / UNKNOWN=0 each).** WIRED: json ✓fully_certified, regex ✓(sample-fails 0), rtl_const_expr ✓, svpp ✓(residual 24→1, the last is the Phase-C lexical-context one), rtl_frontend ✓, SV ✓, vhdl ✓. NOT wired: only meta/annotation grammars.
- active_work_unit: `STORE-AWARE-GEN` (elevated to FULL context-aware generation) is now the strategic frontier per the director reframe; `GRAMMAR-WELLFORMED.H` UNKNOWN→0 drive continues. next_action: AWAIT director steer on prioritizing/scoping FULL store-aware generation (the answer to "garbage / not EBNF-steered").
- in_flight_uncommitted: none. blockers: director steer on the store-aware-generation priority/scope.
- push: ~67 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
