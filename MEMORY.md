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
- latest_commit: (this commit) `PGEN-SV-PARSE-STRICT-0001` (DOCS, leaf `SV-PARSE-STRICT.1`) — SCOPING/DESIGN of a **director-elevated SV PARSER BUG** found by the stimuli generator. The cert-coverage diverse pass emitted `module a; endmodulemodule b; endmodule`; the SV parser ACCEPTS it (tools-proven: the ungated `net_declaration` `nt a;` branch `systemverilog.ebnf:3279` takes a bare `declaration_identifier` as a user nettype — A/B + AST dump + `--trace debug`). New tree `SV-PARSE-STRICT` owns the fix. `.1` done (pure docs); `.2` (the sound-gate fix) is the frontier.
- 🚨 DIRECTOR PRINCIPLE 2026-06-09 (emphatic): **fix parser bugs ASAP = HIGHEST priority** — "no point on cert-cover if a clearly identifiable parser bug shows up." The stimuli generator is a BUG-FINDING ORACLE (this bug was found by it). The cert-coverage generator-faithfulness fix (Defect A: `@sample`-hint literal-override drops `last_terminal_word_shaped` → `endprogram`+`module` fusion = the `sample_parse_failures=3`) is DEFERRED behind this parser bug; root-caused, owned by `GRAMMAR-WELLFORMED` (generate_rule 5196–5214).
- 🧭 DIRECTOR-CONFIRMED BOUNDARY 2026-06-09 (AskUserQuestion): SV parser = **context-aware, SOUND gating**. SV is not context-free (type-name feedback / lexer-hack), so the parser MUST consult the store; it REJECTS a *provably*-undeclared type identifier (current-TU + `--lib-in` facts) and ACCEPTS when it can't prove (unresolved `import pkg::*` in scope) — sound-not-complete. ⇒ `endmodulemodule b;` IS a parser bug. Elaboration owns cross-unit binding + type/width/param/generate semantics.
- FIX DESIGN (`.2`, grammar-only): sound gate `has_fact(type_name,$1) OR wildcard_import_open` on the `nt a;` branch (the ungated branch is TODAY the de-facto unresolved-import escape — a hard gate would be UNSOUND, would reject `import pkg::*; foo_type b;`); + emit `wildcard_import_open` on `pkg::*` (3593) + emit `type_name{nettype}` from `nettype_declaration`. De-risk: corpus uses NO user nettypes (friscv's is `` `default_nettype `` directive); the parallel `data_declaration` type path is ALREADY hard-gated (`checked_type_identifier`) + corpus 14/14 → gate consistently. VERIFY: corpus 14/14 (`sv_external_corpus_triage_gate`) + bad-inputs reject + declared/wildcard accept + lib + shape-contract.
- locked_program (director 2026-06-08, now BEHIND the parser bug): ALL EXISTING PARSERS → `Done` (cert-coverage clean + UNKNOWN=0). Cert-cov sample-fails 0 everywhere EXCEPT SV (the `=3` is Defect A). UNKNOWN (reach): rtl_const_expr 0 (fully_certified), vhdl 69, rtl_frontend 133, SV 1126, svpp 3, regex 98.
- active_work_unit: `SV-PARSE-STRICT.2` (the fix). in_flight_uncommitted: none (this commit lands `.1` — docs only: SV-PARSE-STRICT.md + TASK_TREE.md + live docs). blockers: none.
- push: ~76 unpushed (push at ~200 per `feedback_push_pacing`; do NOT push yet).
