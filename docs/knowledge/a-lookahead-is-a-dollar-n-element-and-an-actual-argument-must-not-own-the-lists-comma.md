---
id: a-lookahead-is-a-dollar-n-element-and-an-actual-argument-must-not-own-the-lists-comma
title: A lookahead is a $n element and an actual argument must not own the list's comma — compare a legal control's kind multiset before generating, give actual-argument positions a comma-free twin, and read Annex A's footnotes as grammar
answers:
  - "my grammar edit kept every verdict right but the AST lost nodes (initialisers vanished) — do lookaheads count in $n numbering"
  - "how do I check a grammar change is shape-neutral before regenerating the parser (kind multiset HEAD vs new)"
  - "a property/checker/sequence port default followed by the next port fails at the = (clk = $inferred_clock, rst = $inferred_disable)"
  - "module top #(type T = int, T x = 0) fails — the type list swallowed T"
  - "PGEN accepts `a;` or `y = 1;` at module level (implicit-type declaration without var) — Annex A footnote 6"
  - "PGEN accepts `localparam X;` — Annex A footnote 18"
  - "a block whose first statement is an assignment parses as a declaration (block_data > implicit_data)"
  - "an empty file with only /* parses — who consumes an unterminated block comment"
  - "the canonical certificate UNKNOWN count rose after a grammar tightening although every pin passes — a lookahead the generator cannot honour"
tags: [grammar-authoring, lookahead, ast-shape, sv-corpus, annex-a, footnotes, systemverilog, over-acceptance]
date: 2026-09-04
status: current
evidence: "SV-CORPUS-GRAD.3.35 (PGEN-SV-CORPUS-GRAD-0364, release 1.0.210, ledger SV-0083): four grammar corrections (event_expression_no_comma; the type-list lookahead; footnote 6 var-gated implicit arms; footnote 18 with_default lists); the first cut's &var lookahead shifted $4/$5 and the kind-multiset comparison caught it; 13 pins; ENGINE-UNIVERSAL-SERVICES.93 for the layout skipper"
reverify: "python3 stimuli/sv/run_adjudication_repros.py | tail -1   # the 13 .3.35 pins hold on sv_2017 + sv_2023, failures=0"
---

**A parenthesised or positive lookahead counts as a `$n` element in a node template, and the verdicts will
not tell you when your fields shift — compare a legal control's kind multiset before generating. An actual
argument is one item of a list, so its expression grammar must not own the separator. And Annex A's
footnotes are grammar.**

## The rules

- **Count lookaheads.** `!( … )` and `&x` are `$n` elements (`conditional_expression := cond_predicate
  &question question … -> {attributes: $4}`); a bare `!x` is not (`type_assignment_sv_2017 := !kw_type id …
  -> {name: $1}`). When a guard must not shift the fields, make the guarded element MANDATORY instead of
  `&x ( x )?`, or pass the payload through with `-> $2`.
- **Verdicts are not trees.** The first cut passed every pin and had dropped the initialisers of `var x =
  1;`. Dump the interpreter's raw tree (`--interpret-parse-ast-json`) for each legal control under the
  previous and the new grammar and compare the multiset of `kind` values; identical means shape-neutral,
  a difference names exactly what moved (here, only the intended `block_data → statement` correction).
- **An actual argument must not own the list's comma.** `event_expression ::= … | event_expression ,
  event_expression` is right inside `@( … )` and wrong as a bare actual argument, where the comma belongs
  to the port list; `event_expression_no_comma` at the actual-argument position keeps the same node
  shape and leaves the parenthesised form its commas.
- **Read the footnotes.** Footnote 6 (an implicit data type needs `var`) and footnote 18 (a `localparam`
  carries its default) are normative grammar; enforcing them closed six over-acceptances and corrected the
  derivation of every block that begins with a statement.
- **A parse-only lookahead is not a grammar constraint.** The certificate sampler generates from the same
  EBNF; a guard the generator cannot see leaves it emitting text the parser refuses (`localparam X;`),
  and the canonical UNKNOWN count is the tell (13 → 20 here, the whole `localparam` family). Restate a
  tightening as a positive production with the same node shape, and read the seed-0 census before the gate.
- **A greedy list stops at a two-identifier item.** `type T = int, T x = 0`: after the comma, an identifier
  followed by another identifier is a new typed parameter, never a continued type list.
- **Layout is engine.** An unterminated `/*` is consumed by the generated layout skipper, not by any
  grammar rule; route it (`ENGINE-UNIVERSAL-SERVICES.93`) rather than patching the grammar.

## The measurement

See `docs/tasks/SV-CORPUS-GRAD.md` `.3.35` and `.13y`, ledger `SV-0083`, register row `1.0.210`.
