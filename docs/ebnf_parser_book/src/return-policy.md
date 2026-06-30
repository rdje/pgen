# The Implicit / Passthrough Return Policy

Every generated parser returns an AST. When a rule carries an explicit `-> …`
[return annotation](return-annotations.md), that annotation shapes the result. But **what does a rule
return when it has no `-> …` at all?** That is the *implicit / passthrough* policy, and it is one of the
most useful things for a grammar author to understand — getting it wrong is the usual cause of "my AST
dropped a field" surprises.

This policy is tool-verified against the codegen
(`rust/src/ast_based_generator.rs`, `rust/src/ast_return_transform.rs`).

## The single-element synthetic `-> $1`

A rule or branch with **no** explicit `-> …` gets a **codegen-only synthetic `-> $1`** ("implicit
passthrough") **if and only if its body is a single element** — that is, a `Sequence` whose
`elements.len() <= 1`, OR a lone terminal, rule reference, lookahead, or `Or` (ordered choice).

```ebnf
# single rule reference → implicit -> $1: this rule returns whatever `expression` returns
parenthesized := expression          # behaves as: -> $1

# ordered choice of single elements → implicit -> $1: returns the chosen branch's value
literal := number | string | boolean
```

The synthetic `-> $1` is **codegen-only**: it never appears in the return-annotation inventory artifact
(which surfaces only annotations you actually wrote).

## Where the synthetic `-> $1` is deliberately NOT added

The codegen intentionally withholds the implicit `-> $1` in two cases, because a blind `$1` would lie
about the result:

### Multi-element sequences (≥ 2 elements)

```ebnf
# NO implicit annotation — a bare $1 would be '(' and silently drop `expr`
grouped := "(" expression ")"
```

If `grouped` had an implicit `-> $1`, it would return the literal `(` and throw away the payload. Because
the codegen refuses to guess, a multi-element sequence with no `-> …` passes its **raw** capture through
(see below) — but the moment you care about the shape, **write the annotation explicitly**:

```ebnf
grouped := "(" expression ")" -> $2          # return the inner expression
```

### Quantified bodies (`?` / `*` / `+`)

A rule whose body is a single **quantified** element is left as **raw passthrough**. The "natural reading"
of `$1` on a quantified body is "the whole capture group", and raw passthrough already yields that — but
raw passthrough of a quantified body yields the **structure** (the list of iterations), *not* the matched
text.

```ebnf
# raw passthrough: returns the list/structure of digit matches, not the joined text
digits := digit+
```

If you want the **matched text** of a quantified run rather than its structure, capture it explicitly with
`$text` / `$0` (see [Return Annotations](return-annotations.md) and [Terminals](terminals.md)):

```ebnf
digits := digit+ -> $text        # the matched substring, e.g. "1234"
```

## The passthrough rule, precisely

When a rule has no transform, `generate_passthrough` (the no-transform default) folds the captured
elements like this:

| Captures | Passthrough returns |
| --- | --- |
| 0 captures | an empty terminal `""` |
| 1 capture | that capture (the implicit `$1`) |
| N captures | the **last** capture |

That "N captures → last" fallback is exactly why you should not lean on passthrough for a multi-element
sequence — the result is the *last* element, which is rarely the field you want. Annotate it.

## Rule of thumb

- **One element, you want it verbatim?** Omit `-> …` — the implicit `$1` does the right thing.
- **Two or more elements?** Always write `-> …` naming the field(s) you mean.
- **A quantified run, and you want its text?** Write `-> $text` (or `-> $0`).
- **Anything structured (object/array/list)?** Write the explicit annotation; see
  [Return Annotations](return-annotations.md).

> Future direction (tracked, not yet landed): extend the policy so a terminal-only quantified body with no
> `-> …` defaults to `$text` (matched text) instead of raw structure, making the implicit model
> {single-element → `$1`, terminal-quantified → `$text`} fully principled. Until that lands, be explicit
> with `$text` for quantified text payloads.
