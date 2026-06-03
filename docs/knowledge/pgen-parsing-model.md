---
id: pgen-parsing-model
title: What PGEN is w.r.t. PEG / Packrat / data-dependent grammars (precise placement)
answers:
  - "what is PGEN with respect to PEG and Packrat"
  - "is PGEN a PEG or a CFG parser generator"
  - "what is PEG / what is Packrat / what is a data-dependent grammar"
  - "does PGEN handle left recursion automatically or must the author eliminate it"
  - "is PGEN stateless or stateful packrat"
  - "where does PGEN stand in the parsing literature"
tags: [parser, architecture, peg, packrat, parsing-model]
date: 2026-06-03
status: current
evidence: "docs/tasks/PARSE-SOTA-research-synthesis.md; Ford POPL 2004 (PEG) + ICFP 2002 (packrat); SPEG/Nez; Yakker (data-dependent); Laurent & Mens SLE 2016 (stateful delta); code: ast_pipeline/mod.rs:1488/1523/1588/1640, main.rs:309/900/1828"
reverify: grep -n "eliminate_left_recursion\|eliminate_left_recursive_patterns" rust/src/ast_pipeline/mod.rs rust/src/main.rs
---

**PGEN is a packrat-memoized, *data-dependent (stateful)* PEG parser-generator**, with
automatic left-recursion elimination, synthesized-attribute return annotations, and a
semantic store. Squarely in the **SPEG / Nez / data-dependent-grammar** family.

Definitions:
- **PEG** (Ford, POPL 2004): recognition-based grammar; **ordered choice** (first match
  commits → no ambiguity), greedy/possessive repetition, syntactic predicates `&`/`!`
  (unlimited zero-consumption lookahead), deterministic. *Not* a CFG (yacc/ANTLR allow
  ambiguity).
- **Packrat** (Ford, ICFP 2002): the **linear-time implementation** of a PEG via
  memoization of every `(rule, position) → result`; eliminates exponential backtracking;
  **assumes a PURE parse function**.
- **Data-dependent / stateful PEG** (SPEG SLE 2017; Nez; Yakker POPL 2010): PEG + parse-time
  state (a symbol table gates rules) → context-sensitivity (e.g. SV type-vs-expression).

PGEN component-by-component: ordered choice + `&`/`!` + greedy quantifiers = faithful PEG;
`MemoEntry` = packrat; `@emit_fact` + `@predicate`-gates-a-rule + scope tree = stateful /
data-dependent PEG; **memo entry carries a semantic *delta* replayed on each hit** = Laurent
& Mens SLE 2016 (the published-correct fix because the store makes the parse function
impure); `-> {kind, field:$N}` = synthesized attributes (Knuth 1968).

**Left recursion is eliminated AUTOMATICALLY by the AST pipeline** — a transform-time pass
`eliminate_left_recursive_patterns` (default-on: `eliminate_left_recursion: true`, toggle
`--eliminate-left-recursion`; handles direct + indirect/chain) + runtime
`mutual_recursion_handler` cycle-breaking. It is **transparent to the EBNF author**: you
write the natural left-recursive form (`expr := expr "+" term | term`), the pipeline
rewrites it. (Authoring-time manual elimination would be impractical — corrected fact;
earlier docs wrongly said "authoring time".) PGEN does **not** use Warth runtime seed-grow.

Consequences (map to the sign-off pillars, [[parser-signoff-four-pillars]]):
- PEG ⇒ alternative *order is semantic* (the `A := a | ab` shadowing class = PARSE-SOTA A2).
- Packrat ⇒ aims linear, **but the store violates packrat's purity assumption** → the
  linear-time guarantee is NOT free → [[stateful-packrat-not-linear]] / PARSE-TERMINATION.
- Data-dependent ⇒ parses genuinely context-sensitive constructs cleanly (a strength).
