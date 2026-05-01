# Examples: Character Classes

Concrete probe outputs for character class regexes. The `char_class` rule is **un-annotated** at this release — the AST emits the raw 4-element Sequence wrapper.

## Simple class — `[abc]`

```json
{
  "content": {
    "Json": {
      "pattern": [
        [[
          {
            "atom": [
              "[",
              [],                      // not negated
              [],                      // no initial-close
              [
                [["a"]],                // class_item for "a" — class_literal Terminal in nested-Alternative wrappers
                [["b"]],
                [["c"]]
              ],
              "]"
            ],
            "quantifier": [],
            "type": "piece"
          }
        ]],
        []
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 5 }
}
```

The `class_body` content (atom[3]) is a `Quantified-*` of `class_item` shapes. Each item for a plain literal is the un-annotated chain `class_item → class_literal → Terminal`, which serialises to a nested array per the un-annotated wrappers. Once those rules are annotated, the nesting collapses.

A consumer extracting class items:

```rust
fn extract_class_items(class_atom: &Value) -> Vec<&Value> {
    class_atom.as_array()
        .and_then(|a| a.get(3))
        .and_then(|b| b.as_array())
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}
```

For each item, classify by structural signature (per the [Atom Subtree](rules-atom.md) discriminator table).

## Negated class — `[^abc]`

```json
{
  "atom": [
    "[",
    "^",                        // negation matched — atom[1] is the literal "^"
    [],
    [
      [["a"]],
      [["b"]],
      [["c"]]
    ],
    "]"
  ],
  ...
}
```

`atom[1]` is `"^"` when negated, `[]` when not.

## Range — `[a-z]`

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      [
        [<class_atom for "a">],   // class_range starting atom
        [],                       // class_zero_width* prefix — empty
        "-",
        [],                       // class_zero_width* suffix — empty
        [<class_atom for "z">]    // class_range ending atom
      ]
    ],
    "]"
  ],
  ...
}
```

The `class_range` 5-element Sequence: `[<start>, <zw-prefix>, "-", <zw-suffix>, <end>]`. The two `class_zero_width*` slots are typically empty `[]`; they exist for PCRE2's `\E` / empty-`\Q\E` markers around the dash.

## Mixed range and literal — `[a-z0-9_]`

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      [<class_range a-z>],
      [<class_range 0-9>],
      [["_"]]                  // plain literal class_item
    ],
    "]"
  ],
  ...
}
```

Three `class_item`s: two ranges and one literal. The body's order matches the source order.

## Initial-close class — `[]a]`

PCRE2 quirk — a literal `]` as the FIRST class char is allowed:

```json
{
  "atom": [
    "[",
    [],                         // not negated
    "]",                        // class_initial_close matched
    [
      [["a"]]
    ],
    "]"
  ],
  ...
}
```

`atom[2]` distinguishes: `[]` (no initial close) vs `"]"` (initial close present).

## POSIX class — `[[:alpha:]]`

As of slice 8 (post-1.1.36, fixes [PGEN-RGX-0076](changelog-index.md)), POSIX classes inside character classes emit a fully typed `{type, name, negated}` shape:

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      {"type": "posix_class", "name": "alpha", "negated": []}
    ],
    "]"
  ],
  ...
}
```

`name` is the matched POSIX class name string. `negated` is the typed boolean `true` when the source has `^` after `[:`, or the empty array `[]` (un-matched `posix_negation?` slot — consumers map `[]` → `false`).

### `[[:^alpha:]]` — negated POSIX class

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      {"type": "posix_class", "name": "alpha", "negated": true}
    ],
    "]"
  ],
  ...
}
```

### `[[:alpha:][:digit:]]` — multiple POSIX classes

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      {"type": "posix_class", "name": "alpha", "negated": []},
      {"type": "posix_class", "name": "digit", "negated": []}
    ],
    "]"
  ],
  ...
}
```

Both POSIX classes typed and disambiguated.

### Migration from pre-1.1.37 (slice 8)

Before slice 8, the `posix_class` annotation `-> $1` extracted only the literal `"[:"` opener — the typed shape was just the bare string `"[:"`, which collapsed every POSIX class name (`alpha`/`digit`/`xdigit`/etc.) to the same value. Consumers who relied on a source-span fallback can drop that path; the typed shape now preserves the name and negation directly.

## Quoted class literal — `[\Qa-z\E]`

The PCRE2 class-quote form. The `class_item` matches `quoted_class_literal`:

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      [
        // class_item branch 3 → quoted_class_literal
        // = "\\Q" quoted_class_literal_char* "\\E"
        [
          "\\Q",
          [<chars: a, -, z>],
          "\\E"
        ]
      ]
    ],
    "]"
  ],
  ...
}
```

3-element Sequence `["\\Q", <chars-Quantified>, "\\E"]`.

## Class with escape — `[\d]`

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      [
        // class_item branch 5 → class_escape
        // class_escape = escape
        [
          "\\",
          [[[[[ "d" ]]]]]    // un-annotated escape_unit chain
        ]
      ]
    ],
    "]"
  ],
  ...
}
```

`class_escape` wraps `escape`'s 2-element Sequence.

## Stray `\E` inside class — `[\E]`

PCRE2 zero-width marker:

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      [
        // class_item branch 1 → stray_class_end_quote
        "\\E"
      ]
    ],
    "]"
  ],
  ...
}
```

`stray_class_end_quote = "\\E"` emits the bare terminal `"\\E"`.

## Future direction

The whole character-class subtree will be annotated in a future task #40 slice. Target:

```json
{
  "type": "char_class",
  "negated": false,
  "items": [
    { "type": "literal", "value": "a" },
    { "type": "range", "start": "b", "end": "z" },
    { "type": "escape", "kind": "shorthand", "name": "d" },
    { "type": "posix_class", "name": "alpha", "negated": false }
  ]
}
```

Until that lands, consumers walk the per-rule raw shapes documented above.
