---
id: ebnf-supported-vs-aspirational-constructs
title: "Which EBNF constructs PGEN's codegen actually implements vs which the self-hosting meta-grammar only self-describes (aspirational)"
answers:
  - "which EBNF constructs does PGEN actually support"
  - "is parametric_rule / template_instantiation / lexer_mode / grammar inheritance / import implemented in pgen EBNF"
  - "does pgen EBNF support except / case control ~ / named captures / error productions / semantic predicates {? ?} / action blocks"
  - "why is a construct in ebnf.ebnf but does nothing in my grammar"
  - "is the @N% probability quantifier implemented"
  - "why does [a-z] not match a character class in my pgen grammar"
  - "what does element-level [ ... ] mean in a pgen EBNF rule"
  - "which rule operators := ::= = :- does pgen accept"
  - "what terminals and built-in matchers does pgen EBNF support (any_char ascii_char)"
  - "are the ebnf.ebnf extension constructs real or aspirational"
date: 2026-07-01
status: current
tags: [ebnf, meta-grammar, codegen, grammar-authoring, no-drift, ebnf-frontend, supported-surface]
evidence: "Self-hosting meta-grammar grammars/ebnf.ebnf (v2.0) self-DESCRIBES many extension constructs the codegen does NOT consume. SUPPORTED (verified): rule operators := ::= = :- all parse-interchangeable (ebnf_frontend.rs; ::= used 8x in systemverilog.ebnf, = is regex.ebnf's operator); string/char/raw/regex literals + escapes/unicode; native matchers any_char/builtin_any_char + ascii_char/builtin_ascii_char (ast_based_generator.rs; regex.ebnf idiom `( \"XX\" | !\"X\" builtin_any_char )* -> $text`); quantifiers ? * + {n} {n,m} {n,} {,m} (Layer-0 parse_quantifier_bounds in ast_pipeline/mod.rs); lookaheads & / ! (regex.ebnf 12 pos/39 neg); grouping (...); optional [ ... ] which ebnf_frontend.rs LOWERS to ( ... )? ('[' -> group_open, ']' -> group_close + '?'); return annotations $N/$text/$0/.field/[i]/arrays/objects/* /** /::n* ; include(...)/include_file(...)/include_dir(...); pragmas @stop_at_rule_boundary + the [>...]/[>!...] lexical follow-restriction. NOT-IMPLEMENTED (parsed-but-ignored / inert in ebnf.ebnf): except, case control ~/~i, named captures name:pattern, parametric_rule rule[..], template_instantiation rule<..>, lexer_mode, grammar inheritance extends, import statements, error productions error/sync/skip/panic, semantic predicates {? ?}, action blocks { }, epsilon as a construct. PARTIAL: probability quantifier @N% (parsed, not codegen'd); @inline/@memoize (limited). Documented for grammar authors in docs/ebnf_parser_book/ (rules-and-expressions.md NOT-implemented table; terminals.md the [..] footgun)."
reverify: "grep -nE \"'\\\\['\" rust/src/ebnf_frontend.rs   # the optional [..] -> (..)? lowering; and: rg -n 'builtin_any_char|any_char' grammars/regex.ebnf | head"
---

## The fact

`grammars/ebnf.ebnf` is a **self-hosting** EBNF meta-grammar: to parse grammars that use them, it
*self-describes* many extension constructs. **The PGEN code generators do not act on most of them.** A
grammar author (or a book author) must document/use only the surface the codegen actually consumes —
treating the meta-grammar's self-description as the supported list is **drift**.

- **SUPPORTED** (safe to use): the four rule operators (`:=` `::=` `=` `:-`, interchangeable); string /
  char / raw / regex literals; the native `any_char`/`builtin_any_char` + `ascii_char`/`builtin_ascii_char`
  matchers; quantifiers `?` `*` `+` `{n}` `{n,m}` `{n,}` `{,m}` (Layer-0 unified engine); lookaheads `&` /
  `!`; grouping `( … )`; optional `[ … ]`; the full return-annotation language; `include` / `include_file`
  / `include_dir`; the `@stop_at_rule_boundary` pragma and the `[> … ]` / `[>! … ]` lexical
  follow-restriction ([[lexical-follow-restrictions]]).
- **NOT IMPLEMENTED** (present in `ebnf.ebnf`, inert/defect in a real grammar — do **not** use): `except`,
  case control `~`/`~i`, named captures `name:pattern`, parametric rules `rule[…]`, template
  instantiation `rule<…>`, lexer modes, grammar inheritance `extends`, `import` statements, error
  productions (`error`/`sync`/`skip`/`panic`), semantic predicates `{? … ?}`, action blocks `{ … }`,
  epsilon as an explicit construct. Use the supported mechanism instead: `@predicate` for gating (not
  `{? ?}`), the [[ebnf-frontend-architecture | include system]] for composition (not `extends`/`import`).
- **PARTIAL**: the `@N%` probability quantifier is parsed but not codegen'd; `@inline`/`@memoize` have
  limited consumption.

## The `[ … ]` footgun

At the **grammar-element level**, `[ … ]` means **optional** — `ebnf_frontend.rs` lowers `[` → group-open
`(` and `]` → group-close `)` + `?`. It is **NOT** a character class. Character classes live **inside a
regex literal** (`/[a-z]/`); a return-annotation `[ … ]` (inside `-> …`) is an **array**. Three meanings,
disambiguated only by context.

## Canonical home

The grammar-author book `docs/ebnf_parser_book/` documents this in full (its `rules-and-expressions.md`
NOT-implemented table and `terminals.md` `[ … ]` footgun). Source of truth: `rust/src/ebnf_frontend.rs`,
`rust/src/ast_based_generator.rs`, `rust/src/ast_return_transform.rs`, and the shipped `grammars/*.ebnf`.

Related: [[ebnf-frontend-architecture]], [[ebnf-single-source-of-truth]], [[lexical-follow-restrictions]],
[[pgen-parsing-model]].
