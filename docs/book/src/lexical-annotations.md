# Lexical Annotations — the 4th pillar (surface faithfulness)

> **Status: design in progress** (tree [`LEXICAL-ANNOTATIONS`]). This chapter introduces the concept
> and why it exists; the concrete annotation surface and engine mechanism are being designed and will
> be documented here as they land. *Lexical annotations* is the preferred name; *layout annotations* is
> an accepted synonym.

## Three pillars describe the tree; a fourth describes the text

PGEN describes a language with three declarative pillars you have already met:

| Pillar | Governs | Direction |
|---|---|---|
| **EBNF grammar** | structure — which token sequences are valid | text → tree |
| **Return annotations** | structure → data — the AST/output shape | tree → data |
| **Semantic annotations** | meaning — types, scopes, the store | tree ⟷ meaning |

Together they pin down the **tree** and what it *means*. But they say nothing about how that tree turns
back into **characters** — how tokens are separated, where a newline is mandatory, how layout works.
That is a distinct, fourth concern:

> **Lexical annotations** govern **surface faithfulness**: the characters emitted must **re-lex (and
> re-parse) to exactly the tokens they were meant to be.**

## Why you only notice it when generating

Parsing never seems to need this, because the lexer uses **maximal munch** (it always takes the longest
match). That is *why* `endprogrammodule` is read as one identifier, not `endprogram` followed by
`module` — and the parser handles all of this automatically.

**Generation is the inverse, and it has no maximal munch to lean on.** When PGEN's stimuli generator
turns a tree back into text, it must *actively* insert enough separation that the result re-lexes to the
tokens it intended. If it doesn't, it produces text that is no longer valid:

```text
# the generator meant two tokens: `endprogram` then `module`
endprogrammodule          # ✗ lexes as ONE identifier — the keywords fused

# the generator meant a comment, then `;` to be parsed
timeunit 1 ps //note ;    # ✗ the // comment with no newline swallows the `;` to end-of-line
```

Both are valid *trees* that render to *invalid text*. Pillars 1–3 cannot prevent this; the lexical
pillar is what does.

So, in one line: **lexical annotations can steer both parsing and generation, but in PGEN today they
are needed for generation far more often.** The constraints are a property of the lexical interface that
*both* directions cross — the same kind of rule can disambiguate parsing (as it does in scannerless
parsers) and enforce faithful rendering in generation. PGEN's parser simply already covers most of its
side automatically (maximal munch), while the generator has no such discipline — so generation is where
they bite today. Parsing also *verifies* the result: the certificate-coverage gate re-parses every
generated sample, so a faithfulness violation is caught immediately. Today: enforcer = the generator;
honourer + verifier = the parser.

## The shape of the solution (being designed)

The goal is a single invariant, handled generally and parser-agnostically — never as a per-case patch:

> `render(t)` must re-lex/re-parse to `t`.

Most of it can be **derived automatically** from the token definitions the grammar already contains
(the generator can tell, from two adjacent tokens' regexes, whether they would fuse or whether a
comment needs a real newline, and insert the minimal separator). A small **declarative follow-
restriction** annotation will cover the rare cases derivation cannot infer — that declarative surface is
"lexical annotations" proper. The existing `enforce_word_boundary_spacing` behaviour is an early,
single-case prototype of this pillar and will be subsumed by it.

This is grounded in established work — maximal-munch lexical disambiguation and SDF2 *follow
restrictions* on the parsing side, and Oppen/Wadler unparsing (pretty-printing) on the generation side;
see the tree's [SOTA research synthesis](../../tasks/LEXICAL-ANNOTATIONS-research-synthesis.md).

## Notation

When derivation can't infer a boundary rule, you declare it. A lexical annotation sits **above the
rule** (like `@…` directives) and reads:

- `[> LIST ]` — the rule's token **must be followed by** one of `LIST`.
- `[>! LIST ]` — the rule's token **must not be followed by** any of `LIST`.

`LIST` is one or more **items**, each a `/regex/` or a `"string"`, freely mixed and separated by
whitespace and/or commas; the list is a **union** ("any of these"). The enclosing `[ … ]` bounds the
list. (`[< … ]` / `[<! … ]` are reserved for "preceded by", should a grammar ever need lookbehind.)

```ebnf
[>! /\w/]                        # an identifier must not be followed by another word char
identifier := /[A-Za-z_]\w*/

[> /\n|$/]                       # a line comment must be followed by newline-or-EOF
line_comment := /\/\/[^\n]*(\n|$)/

[>! "(", "[", /\d/]              # this token must not be directly followed by ( or [ or a digit
some_token := /.../
```

Although `[ … ]` is also "optional" inside a rule *body*, `[>` / `[>!` is unambiguous: no rule
expression can begin with `>`, so `[>` is never an optional. Most grammars need **no** lexical
annotations at all — the regex-derived rules cover the common cases; this is the explicit escape hatch
for the rest.
