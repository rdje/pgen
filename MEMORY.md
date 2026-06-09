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
- latest_commit: (this commit) `PGEN-INLINE-ACTIONS-0001` (DOCS, leaf `INLINE-ACTIONS.1`) — SCOPING/DESIGN/PROPOSAL of a parser-agnostic engine feature: **inline semantic ACTION directives (`@emit_fact`/`@open_scope`/`@close_scope`) on a BRANCH must fire at parse time.** Empirically proven gap (probe grammar → `ast_pipeline --generate-parser`): `ebnf.ebnf` supports inline annotations by design, and rule-level actions + branch-local PREDICATES are live, but **branch-start actions are a no-op** (tournament loop acts only on `Predicate`; `EmitFact` → `=> {}`) and **mid-sequence actions are dropped** (`branch_mid_sequence_semantic_annotations` never compiled). New tree `INLINE-ACTIONS` owns it. `.1` done (pure docs); `.2` (wire branch-start actions) is the frontier; `.3` mid-sequence deferred.
- 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×, standing): **PGEN is NOT bug-free / NOT feature-complete — it is evolving toward both.** When a general, parser-agnostic enhancement to semantic annotations / the pipeline is needed, **discuss → own → design carefully → implement** (the goal: capture the feature sets to parse as many languages as humanly possible — C, JS, HTML, Perl6, …). The `INLINE-ACTIONS` tree IS that path applied here. Director chose (AskUserQuestion) "**wire branch-start emit FIRST, then the SV fix**" over the helper-rule workaround.
- ⛓️ SEQUENCING: `INLINE-ACTIONS.2` (engine: branch-start action wiring; ride the C3-B delta machinery — apply a branch's `is_effect()` directives after predicates pass + before delta-capture → winner-replay/loser-rollback unchanged; surface-stable, zero blast radius when unused) → THEN `SV-PARSE-STRICT.2` (the SV parser-bug fix, now expressing piece-1 `wildcard_import_open` emit as a clean branch-start `@emit_fact`).
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic): **fix parser bugs ASAP = HIGHEST priority** — "no point on cert-cover if a clearly identifiable parser bug shows up." The stimuli generator is a BUG-FINDING ORACLE. Cert-coverage Defect A (`@sample`-hint literal-override drops `last_terminal_word_shaped` → `endprogram`+`module` fusion = SV `sample_parse_failures=3`) is DEFERRED behind the SV parser bug; owned by `GRAMMAR-WELLFORMED` (generate_rule 5196–5214).
- 🧭 DIRECTOR-CONFIRMED BOUNDARY 2026-06-09: SV parser = **context-aware, SOUND gating** (rejects *provably*-undeclared type id via current-TU + `--lib-in` facts; accepts when it can't prove, e.g. unresolved `import pkg::*`). ⇒ `endmodulemodule b;` IS a parser bug. SV fix design (`.2`): sound gate `has_fact(type_name,$1) OR wildcard_import_open` on the `nt a;` branch (`systemverilog.ebnf:3279`) + emit `wildcard_import_open` on `pkg::*` (3593, branch-start emit) + emit `type_name{nettype}` from the nettype decl. De-risk: corpus uses NO user nettypes; data_declaration type path already hard-gated; VERIFY corpus 14/14.
- locked_program (director 2026-06-08, now BEHIND the parser-bug + this feature): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). UNKNOWN (reach): rtl_const_expr 0 (fully_certified), vhdl 69, rtl_frontend 133, SV 1126, svpp 3, regex 98.
- active_work_unit: `INLINE-ACTIONS.2` (engine, frontier) → then `SV-PARSE-STRICT.2`. in_flight_uncommitted: none (this commit lands `INLINE-ACTIONS.1` — docs only: INLINE-ACTIONS.md + TASK_TREE.md + SV-PARSE-STRICT.md note + live docs). blockers: none.
- push: ~77 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
