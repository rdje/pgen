# Quantifiers

A quantifier repeats the element immediately to its left. PGEN implements a **single, unified quantifier
engine** (the "Layer-0" engine, `rust/src/ast_pipeline/mod.rs::parse_quantifier_bounds` +
`rust/src/ast_based_generator.rs`), so every quantifier form has the same per-iteration semantics.

## The forms

| Form | Meaning | `(min, max)` |
| --- | --- | --- |
| `?` | zero or one (optional) | `(0, 1)` |
| `*` | zero or more | `(0, ∞)` |
| `+` | one or more | `(1, ∞)` |
| `{n}` | exactly `n` | `(n, n)` |
| `{n,m}` | between `n` and `m` | `(n, m)` |
| `{n,}` | at least `n` | `(n, ∞)` |
| `{,m}` | at most `m` | `(0, m)` |

All seven are first-class and consumed by the codegen. The bounded forms (`{n}`, `{n,m}`, `{n,}`, `{,m}`)
are used throughout `grammars/regex.ebnf`.

```ebnf
optional_sign  := ( "+" | "-" )?
zero_or_more   := digit*
one_or_more    := digit+
exactly_four   := hex_digit{4}
two_to_eight   := hex_digit{2,8}
at_least_two   := item{2,}
up_to_ten      := item{,10}
```

## Per-iteration atomicity (Layer-0)

The unified engine wraps **every** iteration of a quantifier in an atomic try: if an iteration fails
partway, the parser cleanly backtracks to the position just before that iteration. For the minimum-count
forms (`+`, `{n}`, `{n,…}`), the whole quantifier rolls back to before the quantifier on a min-count
failure. This makes the repetition forms behave uniformly — there is no asymmetry between the first and
subsequent iterations.

A zero-length guard prevents an inner rule that matches the empty string from looping forever, and a large
safety limit backstops pathological repetition.

## Quantifying a group

A quantifier binds to the single element on its left — use a group to repeat more than one element:

```ebnf
# repeat the (',' expression) pair — the classic comma-separated-list idiom
arg_list := expression ( "," expression )*

# WRONG: '+' binds only to 'b'
ab := a | b+        # == a, OR one-or-more b
# RIGHT:
ab := ( a | b )+    # == one-or-more of (a or b)
```

## Returning a quantified group

A quantified element produces a **list** of its iterations. The return-annotation language has dedicated
operators to collect, spread, and flatten those lists — `$N*`, `$N**`, and the extraction-spread `$N::n*`
— described in [Return Annotations](return-annotations.md). The common pattern:

```ebnf
items := item ( "," item )* -> [ $1, $2* ]      # [first, ...rest]
```

## Not implemented: the probability quantifier `@N%`

The meta-grammar `grammars/ebnf.ebnf` describes a probability quantifier (`@75%`). It is **parsed but not
consumed by the codegen** — it has no effect on a generated parser or stimuli generator. Do not use it;
steer stimuli weighting through the [semantic-annotation](semantic-annotations.md) `@…` directives
instead.
