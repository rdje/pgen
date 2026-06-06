# Lexical Annotations — the 4th pillar (surface faithfulness)

> **Status: landing incrementally** (tree [`LEXICAL-ANNOTATIONS`]). The derived obligations (A
> intra-token anchor honouring, B inter-token boundary separation) and the **before-rule declarative
> follow-restriction** `[> … ]` / `[>! … ]` are implemented and consumed by the stimuli generator (see
> *Notation* and *What the generator does with it* below). Still to come: the inline (per-element) form
> and the cross-grammar verification sweep. *Lexical annotations* is the preferred name; *layout
> annotations* is an accepted synonym.

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

## What the generator does with it

A before-rule directive binds the **rule that immediately follows it** (exactly like a `@…` directive —
see [Annotation reference][annot-binds]). The directive is recorded per-rule, so the stimuli generator
can consume it (per-rule and per-branch annotations are generator-visible; per-element ones are not).

- **`[>! LIST ]` (forbid)** is enforced during generation. Whenever the generator emits the rule's
  surface, it **self-terminates** that surface with the *minimal* separator the forbidden set cannot
  absorb — a single space, escalating to a newline only if a space would itself be a forbidden follow.
  Because the separator is baked into the rule's rendered text before anything is appended after it, the
  rule's last token can never fuse with whatever comes next, no matter how the surrounding rules
  concatenate. This is what lets a grammar forbid a *distinct-longer-token* fusion that derivation
  cannot infer — e.g. a fixed literal `<` immediately followed by another `<` becoming `<<`:

  ```ebnf
  [>! "<"]            # the `<` token must never be directly followed by another `<`
  lt := "<"
  #  doc := lt lt   →  "< <"  (faithful)   rather than   "<<"  (a different token)
  ```

- **`[> LIST ]` (require)** is recorded for the **parsing** direction (follow restrictions also
  disambiguate scannerless parsing). Generation performs no insertion for it: faithfulness violations
  come from *fusion* (which `forbid` prevents), and the generator cannot force a *successor* token
  locally. So `require` is carried and available, but a documented generation no-op today.

Enforcement is part of the **lexical-faithfulness mode**, which is **on by default**. Negative-test
generation — which deliberately produces malformed lexical surface — opts out, and then follow
restrictions (and all other faithfulness guards) are not applied. The certificate-coverage gate
re-parses every generated sample, so any faithfulness gap surfaces as a parse failure.

On the command line, `ast_pipeline … --generate-stimuli` (and `--generate-stimuli-module`) is now
**faithful by default** — you no longer pass a flag to get faithful output. To opt out (for negative-test
generation), pass `--no-word-boundary-spacing`:

```bash
# faithful by default — generated samples re-lex to their intended tokens
ast_pipeline grammar.ebnf --generate-stimuli --output samples.txt

# opt out — allow malformed lexical surface (negative-test generation)
ast_pipeline grammar.ebnf --generate-stimuli --no-word-boundary-spacing --output samples.txt
```

The legacy `--enforce-word-boundary-spacing` flag is still accepted (it now just forces the default on)
and is redundant; prefer relying on the default and using `--no-word-boundary-spacing` only when you
deliberately want malformed output.

[annot-binds]: ./annotation-system.md
