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
- latest_commit: (this commit) `PGEN-INLINE-ACTIONS-0002` (ENGINE/CODEGEN, leaf `INLINE-ACTIONS.2`) — **branch-start inline semantic ACTION directives (`@emit_fact`/`@open_scope`/`@close_scope`) now FIRE for the winning branch.** `branch_effect_directives_for_rule_branch` accessor + conditionally-emitted `apply_branch_start_effect_directive` helper + per-rule-gated winner-branch application loop in `generate_or_logic` (rides the C3-B delta machinery: apply after predicates pass + before delta-capture; loser-non-leak is by construction — the loop is only in the winner block). Surface-stable, parser-agnostic. VERIFIED: all 10 grammars regen byte-identical (`helper=0 loop=0`, zero blast radius); 3 unit tests; lib 624/0 + 689/0; clippy source-strict clean; book `semantic-store.md` synced. `.3` (mid-sequence) deferred. END-TO-END firing proven by the consumer `SV-PARSE-STRICT.2`.
- 🧭 DIRECTOR DIRECTIVE 2026-06-09 (3×, standing): **PGEN is NOT bug-free / NOT feature-complete — it is evolving toward both.** When a general, parser-agnostic enhancement to semantic annotations / the pipeline is needed, **discuss → own → design carefully → implement** (the goal: capture the feature sets to parse as many languages as humanly possible — C, JS, HTML, Perl6, …). The `INLINE-ACTIONS` tree IS that path applied here. Director chose (AskUserQuestion) "**wire branch-start emit FIRST, then the SV fix**" over the helper-rule workaround.
- ⛓️ SEQUENCING: `INLINE-ACTIONS.2` ✅ DONE → **NOW `SV-PARSE-STRICT.2`** (the SV parser-bug fix): express piece-1 `wildcard_import_open` emit as a clean branch-start `@emit_fact` on the `pkg::*` `package_import_item` branch (`systemverilog.ebnf:3593`); gate the `nt a;` `net_declaration` branch (3279) with known-type/nettype OR wildcard-open; emit `type_name{nettype}` from the nettype decl. VERIFY corpus 14/14.
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic): **fix parser bugs ASAP = HIGHEST priority** — "no point on cert-cover if a clearly identifiable parser bug shows up." The stimuli generator is a BUG-FINDING ORACLE. Cert-coverage Defect A (`@sample`-hint literal-override drops `last_terminal_word_shaped` → `endprogram`+`module` fusion = SV `sample_parse_failures=3`) is DEFERRED behind the SV parser bug; owned by `GRAMMAR-WELLFORMED` (generate_rule 5196–5214).
- 🧭 DIRECTOR-CONFIRMED BOUNDARY 2026-06-09: SV parser = **context-aware, SOUND gating** (rejects *provably*-undeclared type id via current-TU + `--lib-in` facts; accepts when it can't prove, e.g. unresolved `import pkg::*`). ⇒ `endmodulemodule b;` IS a parser bug. SV fix design (`.2`): sound gate `has_fact(type_name,$1) OR wildcard_import_open` on the `nt a;` branch (`systemverilog.ebnf:3279`) + emit `wildcard_import_open` on `pkg::*` (3593, branch-start emit) + emit `type_name{nettype}` from the nettype decl. De-risk: corpus uses NO user nettypes; data_declaration type path already hard-gated; VERIFY corpus 14/14.
- locked_program (director 2026-06-08, now BEHIND the parser-bug + this feature): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). UNKNOWN (reach): rtl_const_expr 0 (fully_certified), vhdl 69, rtl_frontend 133, SV 1126, svpp 3, regex 98.
- active_work_unit: `SV-PARSE-STRICT.2` (the SV parser-bug fix; frontier). in_flight_uncommitted: none (this commit lands `INLINE-ACTIONS.2` — semantic_runtime.rs + ast_based_generator.rs + book + task/live docs; `generated/` regen is local/untracked). blockers: none.
- push: ~78 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
