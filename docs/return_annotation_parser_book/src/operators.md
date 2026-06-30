# Extraction, Spread, and Access Operators

These operators reach *into* a capture and reshape it. They are listed here in the order the grammar
tries them.

## Extraction — `$N::target`

Extraction pulls one element out of a **quantified/group capture**:

```ebnf
extraction_expression := positional_reference '::' extraction_target spread_suffix?
-> {type: "extraction", base: $1, target: $3, spread: $4}

extraction_target := positive_integer   # 1-based index
                   | 'first'
                   | 'last'

@transform: str::parse::<usize>().unwrap_or(1)
positive_integer := /[1-9][0-9]*/

spread_suffix := '*'
-> "true"
```

- The **base** must be a positional reference (`$N`) — not an arbitrary expression.
- The **target** is a 1-based index (`positive_integer`, i.e. `1`, `2`, … — never `0`), or the keyword
  `first` or `last`.
- An optional trailing `*` marks the extraction as **spread** (`spread` becomes `"true"`).

| You write | Parsed node | Meaning |
| --- | --- | --- |
| `$2::2` | `{type: "extraction", base: $2, target: 2}` | element 2 of `$2` |
| `$2::first` | `{type: "extraction", base: $2, target: "first"}` | first element of `$2` |
| `$2::last` | `{type: "extraction", base: $2, target: "last"}` | last element of `$2` |
| `$2::2*` | `{…, target: 2, spread: "true"}` | element 2 of *each* tuple, then spread |

The canonical use is the **`X (sep X)*` flat-list idiom**: `[$1, $2::2*]` builds a flat list of the
`X`s (see [Objects and Arrays](objects-and-arrays.md)). Here `$2` is the list of `(sep, X)` tuples and
`::2` selects the `X` (index 1 = the separator, index 2 = the item).

## Spread — `$N*`

```ebnf
spread_expression := spreadable_expression '*'
-> {type: "spread", base: $1}
```

A trailing `*` **spreads** a capture's children into the surrounding accumulator (array or object
member list). `[$1, $2*]` produces a head `$1` followed by the children of `$2`. A plain spread builds
a **cons-shape** list (`[head, [next, …]]`) when applied to a repetition — contrast the flat
extraction-spread above.

```text
-> [$1, $2*]        # head then spread of $2 (cons-shape tail)
```

## Flatten-spread — `$N**`

```ebnf
flat_spread_expression := spreadable_expression '**'
-> {type: "flat_spread", base: $1}
```

`**` is like `*`, but it **additionally unwraps** any pushed child whose own `content` is a
`Sequence`/`Quantified`, pushing that wrapper's children inline. Use it when a child rule may return
*either* a single value *or* an array of values that must appear flat under the parent's accumulator.

The motivating case is `regex.ebnf`'s `piece` rule: the `\Q…\E quantifier?` branch returns a
`Sequence` of pieces that must flatten into `concatenation = piece+`'s output. A plain `*` would leave
the inner sequence nested; `**` flattens it.

> Ordering: the grammar lists `**` **before** `*`, so `$2**` is recognized as one flatten-spread rather
> than a spread of `$2*`.

## Property access — `$N.field`

```ebnf
property_access_expression := accessor_base '.' identifier
-> {type: "property_access", base: $1, property: $3}
```

Access a named property of a captured (shaped) value: `$1.value` →
`{type: "property_access", base: $1, property: "value"}`.

## Index access — `$N[i]`

```ebnf
array_access_expression := accessor_base '[' expression ']'
-> {type: "array_access", base: $1, index: $3}
```

Access an element by index: `$1[0]` → `{type: "array_access", base: $1, index: 0}`. The index is an
expression (usually a non-negative integer literal).

## Chaining access

```ebnf
accessor_base := positional_reference
               | property_access_expression
               | array_access_expression
               | '(' expression ')'
```

Because the accessor base may itself be a property/index access, the two access forms **chain**, and
they chain at **unbounded depth**:

```text
-> $1.value             # one hop
-> $matrix[0][1]        # two index hops
-> $a.b[0].c[1].d.e[2]  # mixed property + index, deep
```

This dotted-property + non-negative-integer indexing is a deliberate **subset** — it is *not* full
JSONPath (no wildcards, slices, filters, or descendant selectors).

## Known limitations of spread forms

A few real, tool-confirmed constraints to design around:

- **No mixed flatten-spread in an array.** `[$1**]` works, but the **mixed** forms `[a, $X**]` and
  `[$X**, a]` do **not** work. Workaround: avoid mixing a `**` element with sibling elements — keep the
  list either all-spread or restructure the grammar.
- **Binary-operator / op-chain lists need a named op-rule.** For a `next (OP next)*` op-chain, using an
  **inline alternation** as the iteration lead corrupts the positional model and yields
  `<invalid_sequence_access>`. The fix is to lift the operator to a *named* rule
  (e.g. `additive_op := plus | minus`, the `systemverilog.ebnf` `binary_operator` idiom) and capture a
  bare `rest: $2` — not `$2*` / `$2**` / `$2::2*`.
- **Pure lists use `::N*`.** For a plain `X (sep X)*` list, the flat idiom `[$1, $2::2*]` is the
  correct, supported form (Category A); see [Objects and Arrays](objects-and-arrays.md).
