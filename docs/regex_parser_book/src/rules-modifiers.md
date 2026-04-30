# Modifier and Inline-Modifier Subtree

PCRE2 inline modifiers like `(?i)`, `(?-mx)`, `(?^x:...)` etc. None currently annotated. All emit raw envelope shapes.

## `inline_modifiers`

```ebnf
inline_modifiers = "(?" modifier_spec? ")"
```

3-element Sequence: `["(?", <modifier_spec?>, ")"]`. Modifier-spec absent means a bare `(?)`.

For `(?i)`:

```json
"atom": [
  "(?",
  [<modifier_spec for "i">],
  ")"
]
```

## `scoped_inline_modifiers`

```ebnf
scoped_inline_modifiers = "(?" modifier_spec ":" pattern? ")"
```

5-element Sequence. The `:` separates the modifier from the scoped pattern.

For `(?i:foo)`:

```json
"atom": [
  "(?",
  <modifier_spec for "i">,
  ":",
  [<pattern for "foo">],
  ")"
]
```

## `modifier_spec`

```ebnf
modifier_spec = "^" modifier_seq?
              | modifier_seq
```

2-way Or. The `^` form resets all modifiers; the bare form modifies in-place.

| Branch | Form | Shape |
|---|---|---|
| 0 | `^[seq]?` | 2-element Sequence `["^", <modifier_seq?>]` |
| 1 | `[seq]` | the `modifier_seq` directly |

## `modifier_seq`

```ebnf
modifier_seq = modifier_group ("-" modifier_group)?
             | "-" modifier_group
```

2 branches:

- Branch 0: positive modifiers, optionally followed by `-` and negative modifiers.
- Branch 1: just `-` and negative modifiers (no leading positives).

## `modifier_group`

```ebnf
modifier_group = modifier_item+
```

Quantified-`+` of modifier items. Emits an array of `modifier_item`s.

## `modifier_item`

```ebnf
modifier_item = "a" ascii_restrict_modifier?
              | "x" "x"?
              | modifier_char
```

3-way Or:

- Branch 0: `a` followed by optional ASCII-restrict modifier (`D`, `S`, `W`, `P`, `T`).
- Branch 1: `x` or `xx`.
- Branch 2: a single `modifier_char` (one of `i`, `m`, `s`, `U`, `J`, `n`, `r`).

## `ascii_restrict_modifier`

```ebnf
ascii_restrict_modifier = "D" | "S" | "W" | "P" | "T"
```

5-way Or, emits `Terminal(<letter>)`.

## `modifier_char`

```ebnf
modifier_char = "i" | "m" | "s" | "U" | "J" | "n" | "r"
```

7-way Or, emits `Terminal(<letter>)`.

## Walking a `(?im)` example

For input `(?im)`:

```json
"atom": [
  "(?",
  [
    [
      [<modifier_item for "i">],
      [<modifier_item for "m">]
    ]
  ],
  ")"
]
```

Each modifier_item is a 1-element Sequence wrapping its modifier_char terminal. Concatenate to recover the modifier string.

## Walking a `(?i-mx:foo)` example

For input `(?i-mx:foo)`:

```json
"atom": [
  "(?",
  [
    [<modifier_group for "i">],
    "-",
    [<modifier_group for "mx">]
  ],
  ":",
  [<pattern for "foo">],
  ")"
]
```

## Future shape

Eventually `inline_modifiers` and `scoped_inline_modifiers` will be annotated to produce shapes like:

```json
{ "type": "modifiers", "set": ["i"], "unset": ["m", "x"] }
```

and

```json
{ "type": "scoped_modifiers", "set": [...], "unset": [...], "body": <pattern> }
```

Until then, consumers walk the per-rule Sequence shapes documented above.
