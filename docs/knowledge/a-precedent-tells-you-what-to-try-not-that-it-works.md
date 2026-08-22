---
id: a-precedent-tells-you-what-to-try-not-that-it-works
title: A sibling grammar shipping the exact shape is a LEAD, not a verdict — close the facet matrix before adopting it
answers:
  - "another grammar solved this with one directive — can I just copy it"
  - "how do I decide whether @whitespace_sensitive is the right fix for my grammar"
  - "what do the terminals / regex_tokens / trailing facets actually change"
  - "my whitespace rule is never witnessed even though it is referenced from the entry"
  - "why is my comment rule witnessed but my whitespace rule is not"
  - "how do I test a grammar-level directive without regenerating the parser"
  - "is a rule that the layout skipper consumes a dead rule or a reach gap"
  - "when should I proof-promote a rule that can never be witnessed"
tags: [layout, whitespace, directives, grammar-wellformedness, certificate-coverage, instruments, interpreter]
date: 2026-08-22
status: current
evidence: GRAMMAR-WELLFORMED.H.16.4 (`PGEN-GRAMMAR-WELLFORMED-0167`). `ebnf.ebnf`'s `whitespace := /(\s+)/` sits in `grammar_file`'s alternation beside `comment` and can never be witnessed — the emitted `consume_layout_for_regex` gates every comment arm on `regex_token_matches_at_cursor(pattern)` but runs `consume_optional_whitespace()` unconditionally. `systemverilog_preprocessor.ebnf:23` fixes the same shape with `@whitespace_sensitive: { regex_tokens: true }`; on `ebnf.ebnf` all four settings of the three-facet directive break reading `grammars/json.ebnf`, and only two even witness the rule.
reverify: "printf '    ' > /tmp/ws.txt && ./rust/target/debug/ast_pipeline grammars/ebnf.ebnf --interpret-parse /tmp/ws.txt --interpret-parse-ast-json /tmp/ws.json && python3 -c \"import json;print(json.load(open('/tmp/ws.json'))['content'])\"   # elements: [] , span 0..0 — the * ran ZERO iterations"
---

**Finding that a sibling grammar already solved your problem is the right move, and it is where the
work starts — not where it ends.**

`grammars/ebnf.ebnf` declares `grammar_file := (… | comment | whitespace)*` with
`whitespace := /(\s+)/`. `comment` is witnessed. `whitespace` never is. Both are alternatives of the
same quantified choice, so the asymmetry is the diagnosis, and it is twelve lines apart in one
emitted function:

```rust
fn consume_layout_for_regex(&mut self, can_match_empty: bool, pattern: &str) {
    …
    loop {
        let before = self.position;
        self.consume_optional_whitespace();                            // UNCONDITIONAL — no guard
        …
        if bytes[self.position] == b'#' {
            if self.regex_token_matches_at_cursor(pattern) { break; }  // GUARDED
```

Every **comment** arm first asks *"would the token I am about to match consume these bytes itself?"*.
The **whitespace** skip never asks. `comment`'s branch survives because its own first terminal is a
comment introducer and the dynamic guard protects it; `whitespace`'s bytes are gone before any guard
runs. Measured on an input of four spaces: `accepted=true furthest_position=0`, typed AST
`{"elements": [], "type": "grammar_file"}`, span **0..0** — the `*` ran **zero** iterations, so
nothing was mis-routed and the reach planner is not the suspect.

## The precedent, and the control that refuted it

`grammars/systemverilog_preprocessor.ebnf:23` declares `@whitespace_sensitive: { regex_tokens: true }`;
its `space_or_tab := /[ \t]+/` **is** witnessed and the family reads `74/0/74/0 fully_certified=true`.
Same shape, same problem, one declarative line.

Applying it to `ebnf.ebnf` — measured on a scratch copy through `--interpret-parse`, so no
regeneration was needed — gives a **closed** matrix over the directive's three booleans, scored on two
arms:

| `@whitespace_sensitive:` | target rule commits? | a REAL input (`grammars/json.ebnf`) still parses? |
|---|---|---|
| *(none — shipped baseline)* | no | **yes** |
| `{ regex_tokens: true }` | **yes** | **no** |
| `{ terminals: true }` | no | **no** |
| `{ trailing: true }` | no — strictly worse | **no** |
| `true` | **yes** | **no** |

**No setting satisfies both arms, and two of the four do not even witness the rule.** The matrix also
yields the *reason*, which is the part worth carrying: the directive is **grammar-wide**, disabling
layout skipping before *every* regex terminal. A preprocessor grammar structurally owns all of its
whitespace and can afford that; `ebnf.ebnf`'s `rule_name := /([a-zA-Z_][a-zA-Z0-9_]*)/` and its other
regex terminals **rely** on preceding layout being skipped. ⇒ the property needed is **per-terminal**
and the directive only offers **per-grammar**.

## Three habits this is a good argument for

**Close the facet matrix.** Three booleans is four meaningful settings and minutes of work. A closed
matrix is a refutation; a spot check is an opinion — and the closed one hands you the mechanism.

**Use the interpreter for grammar variants.** `--interpret-parse` reads a `.ebnf` directly and mirrors
codegen's layout skipping byte-for-byte, so N variants cost no regeneration, no `rustc`, no registry
edit — and the tracked grammar is never modified. Reaching for codegen first turns a minutes-long
question into an hours-long one.

**Look for the working neighbour before reading the code.** The unguarded line had been there all
along. What made it findable was a *paired case* — two rules in the same alternation, one witnessed
and one not. A defect with a correctly-behaving sibling is far cheaper to diagnose than one without.

## And name the residual rather than forcing it into an existing bucket

Such a rule is **not dead** (it is referenced and has a reach path) and **not a reach gap** (the
planner routes to it and its sample parses). It is **layout-shadowed**, which makes it a candidate for
a *proof* certificate rather than a witness. ⛔ Do not promote it while a plausible fix is
un-adjudicated: that records a fixable defect as a design fact.
