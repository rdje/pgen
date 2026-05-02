# Examples: Character Classes

Concrete probe outputs for character class regexes. As of slice 26 (post-1.1.56), `char_class` emits typed `{type:"atom", kind:"char_class", negated:<bool>, initial_close:<bool>, body:<class_body>}` objects. Inner items already typed by earlier slices (`posix_class` from slice 8, `class_range_escape` family) propagate transparently inside `body`. `class_body` per-rule typing (the `class_item*` Quantified-of-items shape) is its own concern.

## Simple class — `[abc]`

```json
{
  "atom": {"type": "atom", "kind": "char_class", "negated": [], "initial_close": [], "body": ["a", "b", "c"]},
  "quantifier": [],
  "type": "piece"
}
```

`body` is the array of class_items. For plain literals the items are bare strings (after slice 15's `class_literal` regex-literal rewrite). `negated:[]` (un-matched optional slot — consumer maps to `false`); `initial_close:[]` (same).

A consumer extracting class items:

```rust
fn extract_class_items(class_atom: &Value) -> Vec<&Value> {
    class_atom.as_object()
        .and_then(|o| o.get("body"))
        .and_then(|b| b.as_array())
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}
```

For each item, classify by structural signature (per the [Atom Subtree](rules-atom.md) discriminator table).

## Negated class — `[^abc]`

```json
{
  "atom": {"type": "atom", "kind": "char_class", "negated": true, "initial_close": [], "body": ["a", "b", "c"]},
  ...
}
```

`negated:true` (matched) or `[]` (un-matched — consumer maps to `false`). Real boolean via `negation -> true`.

## Range — `[a-z]`

```json
{
  "atom": {
    "type": "atom",
    "kind": "char_class",
    "negated": [],
    "initial_close": [],
    "body": [{"type": "class_range", "start": "a", "end": "z"}]
  },
  ...
}
```

`class_range` is typed `{type:"class_range", start, end}` (slice 29). The two `class_zero_width*` slots (rare PCRE2 `\E`/`\Q\E` markers around the dash) are dropped from the typed shape; consumers needing them can fall back to the raw `class_range` shape.

## Mixed range and literal — `[a-z0-9_]`

```json
{
  "atom": {
    "type": "atom",
    "kind": "char_class",
    "body": [
      {"type": "class_range", "start": "a", "end": "z"},
      {"type": "class_range", "start": "0", "end": "9"},
      "_"
    ]
  },
  ...
}
```

Three `class_item`s: two typed ranges and one bare literal. The body's order matches the source order.

## Hex-escape range — `[\xA-\xFF]`

```json
{
  "atom": {
    "type": "atom",
    "kind": "char_class",
    "body": [{
      "type": "class_range",
      "start": {"type": "escape", "kind": "hex", "digits": "A"},
      "end": {"type": "escape", "kind": "hex", "digits": "FF"}
    }]
  },
  ...
}
```

End-to-end typed — the typed escape_unit shape (slice 15) surfaces directly inside `class_range.start`/`end` via slice 29's `class_range_escape -> $2` passthrough.

## Initial-close class — `[]a]`

PCRE2 quirk — a literal `]` as the FIRST class char is allowed:

```json
{
  "atom": {"type": "atom", "kind": "char_class", "negated": [], "initial_close": true, "body": ["a"]},
  ...
}
```

`initial_close:true` (matched) or `[]` (un-matched — consumer maps to `false`). Real boolean via `class_initial_close -> true`.

## POSIX class — `[[:alpha:]]`

As of slice 8 (post-1.1.36, fixes [PGEN-RGX-0076](changelog-index.md)), POSIX classes inside character classes emit a fully typed `{type, name, negated}` shape:

```json
{
  "atom": {
    "type": "atom", "kind": "char_class", "negated": [], "initial_close": [],
    "body": [
      {"type": "posix_class", "name": "alpha", "negated": []}
    ]
  },
  ...
}
```

`name` is the matched POSIX class name string. `negated` is the typed boolean `true` when the source has `^` after `[:`, or the empty array `[]` (un-matched `posix_negation?` slot — consumers map `[]` → `false`).

### `[[:^alpha:]]` — negated POSIX class

```json
{
  "atom": {
    "type": "atom", "kind": "char_class", "negated": [], "initial_close": [],
    "body": [
      {"type": "posix_class", "name": "alpha", "negated": true}
    ]
  },
  ...
}
```

### `[[:alpha:][:digit:]]` — multiple POSIX classes

```json
{
  "atom": {
    "type": "atom", "kind": "char_class", "negated": [], "initial_close": [],
    "body": [
      {"type": "posix_class", "name": "alpha", "negated": []},
      {"type": "posix_class", "name": "digit", "negated": []}
    ]
  },
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
