# Return Annotations

A **return annotation** (`-> …`) shapes the AST a rule produces. It is the normative mechanism for
building a typed tree out of positional captures. This chapter is the grammar-author overview; the deep,
exhaustive reference — including the parsed annotation envelope, every operator, and the
Bootstrap-vs-Generated backends — is the dedicated
[return_annotation parser book](../../return_annotation_parser_book/src/welcome.md) and
`docs/RETURN_ANNOTATIONS_REFERENCE.md`.

## Captures

Inside a sequence, each element is captured positionally, `$1`, `$2`, `$3`, … left to right. **Non-consuming
elements still occupy a position** — count [lookaheads](lookaheads.md) when you number your captures.

```ebnf
port := direction data_type identifier ";" -> {dir: $1, type: $2, name: $3}
```

## References

| Form | Returns |
| --- | --- |
| `$N` | the value captured at position `N` (`$1`, `$2`, …) |
| `$text` | the **whole matched substring** of the rule |
| `$0` | alias for `$text` (the Perl5 whole-match convention) |
| `$1.field` | property access into a captured object, chainable: `$1.a.b` |
| `$1[0]` | non-negative integer index into a captured list, chainable and mixable: `$1[0].name` |

```ebnf
keyword_run := keyword+ -> $text        # the matched text, not the list
min_bound   := bound -> $1.min          # reach into a produced object
```

## Literals and structures

| Form | Builds |
| --- | --- |
| `"…"`, numbers, `true`/`false`, `null` | a literal value |
| `[ … ]` | an **array** (e.g. `[ $1, $2 ]`) |
| `{ key: …, … }` | an **object** (keys are identifiers or strings) |

```ebnf
pair := key ":" value -> { key: $1, value: $3 }
empty_quantifier := slot? -> { quantifier: [] }        # [] is the empty array
```

> Reminder from [Terminals](terminals.md): `[ … ]` means three different things by context — a return
> **array** here, an element-level **optional** in a rule body, and a **character class** only inside a
> `/…/` regex literal.

## List operators (for quantified captures)

A quantified element (`X*`, `X+`, `( … )*`) captures a **list** of iterations. These operators collect and
flatten it:

| Operator | Effect |
| --- | --- |
| `$N*` (also `$N+`, `$N?`) | spread the quantified capture's elements into the surrounding array |
| `$N**` | **flatten-spread** — spread one level deeper (recursive-spread) |
| `$N::n*` | **extraction-spread** — take element `n` (or `first` / `last`) from each iteration, then spread |

The canonical "first, then the rest" list idiom — used across the shipped grammars — is:

```ebnf
items := item ( "," item )* -> [ $1, $2* ]          # [first, ...rest]
```

`$N**` and `$N::n*` exist for the `X (sep X)*` binop/op-chain shapes where the separator and operand need
to be lifted out of the per-iteration grouping; see the return_annotation book's operators chapter for the
worked patterns and the exact rules on which form to use where.

## Per-branch annotations

Each non-last branch of an ordered choice may carry its own `-> …` (placed before the `|`); the last
branch's trailing `-> …` is the rule-level annotation:

```ebnf
unary := "-" operand -> {op: "neg", value: $2}
       | "!" operand -> {op: "not", value: $2}
       |     operand -> $1
```

## Where to go deep

- [return_annotation parser book](../../return_annotation_parser_book/src/welcome.md) — the full language,
  the parsed envelope, the operators chapter, and the backend split.
- `docs/RETURN_ANNOTATIONS_REFERENCE.md` — the reference, including the bootstrap-mode flat-structure
  limits (see [The Bootstrap Path](bootstrap.md)).
- [The Implicit / Passthrough Return Policy](return-policy.md) — what happens with **no** `-> …`.
