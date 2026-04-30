# Top-Level Rules: regex, pattern, alternation, alternative, concatenation

These five rules together describe how the top-level structure of a regex is shaped. Reading order: outermost (`regex`) inward to where pieces live (`concatenation`).

## `regex`

```ebnf
regex = pattern?
-> {type: "regex", pattern: $1}
```

The entry rule. Always emits `Json(Object({"type": "regex", "pattern": <pattern-content>}))`.

### Shape

| Field | Type | Notes |
|---|---|---|
| `type` | `"regex"` (string literal) | Discriminator. |
| `pattern` | array | The structural body. See `pattern` below. For an empty input, this is `[]`. |

### Example — input `a`

```json
{
  "content": {
    "Json": {
      "type": "regex",
      "pattern": [[[{ "atom": "a", "quantifier": [], "type": "piece" }]], []]
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 1 }
}
```

The `pattern` field's array nesting is documented in the per-rule sections below.

## `pattern`

```ebnf
pattern = alternation
-> $1
```

A passthrough — `pattern` returns whatever `alternation` returns.

### Shape

Whatever shape `alternation` produces. With `alternation` currently un-annotated, this is a 2-element `Sequence` (in raw envelope form) or a 2-element JSON array (in serialised form): `[<first-alternative>, <rest>]`.

The `pattern` field of the top-level `regex` Json object IS this 2-element array.

### Example

For `a`:

```json
"pattern": [
  [[{ "atom": "a", "quantifier": [], "type": "piece" }]],
  []
]
```

- `pattern[0]` is the first alternative's content.
- `pattern[1]` is the (zero-or-more)-tail of additional `|`-separated alternatives, empty when there's no `|`.

## `alternation`

```ebnf
alternation = alternative ("|" alternative)*
```

Currently **un-annotated** — emits the raw 2-element Sequence shape.

### Shape

```text
Sequence([
  <alternative-node>,        // the first alternative
  <Quantified-* of pairs>    // the ("|" alternative)* tail
])
```

In JSON form: a 2-element array.

### When alternation is empty (no `|` operator)

`pattern[1]` is `[]` (the empty `*` group).

### When there's one `|` (e.g. `a|b`)

`pattern[1]` is a 1-element array, where the element is itself a `["|", <alternative>]` pair.

### When there's many `|` (e.g. `a|b|c|d`)

`pattern[1]` is an N-element array, each element a `["|", <alternative>]` pair.

### Example — input `a|b`

```json
"pattern": [
  [[{ "atom": "a", "quantifier": [], "type": "piece" }]],   // first alt
  [
    [
      "|",
      [[{ "atom": "b", "quantifier": [], "type": "piece" }]]   // second alt
    ]
  ]
]
```

The consumer extraction pattern for collecting all alternatives:

```rust
fn extract_alternatives(pattern: &Value) -> Vec<&Value> {
    let mut alts = vec![];
    let arr = pattern.as_array().unwrap();
    alts.push(&arr[0]);                            // first alternative
    if let Some(rest) = arr.get(1).and_then(|v| v.as_array()) {
        for pair in rest {
            // pair is [", <alternative>] — extract index 1
            if let Some(p) = pair.as_array() {
                alts.push(&p[1]);
            }
        }
    }
    alts
}
```

## `alternative`

```ebnf
alternative = concatenation?
```

Currently **un-annotated**. Body is a Quantified-`?` over `concatenation`. With the codegen tightening from slice 36 (2026-04-30), Quantified-bodied rules without an explicit annotation no longer get the implicit `-> $1` default — so `alternative` emits the raw `Quantified([<concat>], "?")` shape.

### Shape

In JSON form: a 1-element array containing the `concatenation`'s typed shape (when matched), or an empty array (when the alternative was empty — e.g. in `a|`).

### Example

For `a` (the first alt of input `a`):

```json
[[{ "atom": "a", "quantifier": [], "type": "piece" }]]
```

That outer single-element array is the `Quantified-?` carrier. The inner `[{...piece...}]` array is the `concatenation`'s output (next).

For an empty alternative (e.g. the empty side of `a|`):

```json
[]
```

## `concatenation`

```ebnf
concatenation = piece+
-> [$1**]
```

**Annotated** with `[$1**]`, the flatten-spread of `piece+`. Emits a flat array of piece objects.

### Shape

`Json(Array([<piece-object>, <piece-object>, ...]))` — a flat list of pieces. The `**` flatten-spread operator on `$1` (the piece+ Quantified) iterates each piece node and, if any piece's content is itself a Sequence (e.g. from the `piece_quoted_run_quantified` branch which produces `Sequence([prefix-pieces..., last-piece-with-quantifier])`), unwraps one level so the children appear inline.

In JSON form: a single array of piece objects.

### Example — input `abc`

```json
"pattern": [
  [[
    { "atom": "a", "quantifier": [], "type": "piece" },
    { "atom": "b", "quantifier": [], "type": "piece" },
    { "atom": "c", "quantifier": [], "type": "piece" }
  ]],
  []
]
```

The flat array of 3 piece objects is the `concatenation` output, lifted into `pattern[0][0]` through the alternation/alternative wrappers.

### Example — input `\Qab*\E{2,}`

The `piece_quoted_run_quantified` branch emits a `Sequence` of pieces, which `concatenation`'s `**` flatten-spread lifts inline:

```json
"pattern": [
  [[
    { "atom": "a", "quantifier": [],          "type": "piece" },
    { "atom": "b", "quantifier": [],          "type": "piece" },
    { "atom": "*", "quantifier": [<{2,}>],    "type": "piece" }
  ]],
  []
]
```

Same flat shape as `abc` — the `\Q...\E` quoted-run is invisible in the output, exactly as PCRE2 semantics dictate (the runtime behavior of `\Qab*\E{2,}` and `ab\*{2,}` is identical).

## Putting it together — the navigation pattern

A consumer that wants to iterate the pieces of a non-alternated concatenation does:

```rust
fn pieces_of(regex_json: &Value) -> Vec<&Value> {
    let pattern = regex_json.get("pattern")?.as_array()?;
    // pattern[0] is the first (and only, when no |) alternative
    let alt = pattern.get(0)?.as_array()?;
    // alt[0] is the concatenation array (when present)
    alt.get(0)?.as_array().cloned().unwrap_or_default()
}
```

For alternation-bearing regexes, walk both `pattern[0]` and `pattern[1][N][1]` for each top-level branch.

## Why the `pattern` field's nesting is what it is

Each layer of nesting corresponds to a grammar rule:

```
regex
└── pattern (-> $1, transparent)
    └── alternation (un-annotated, 2-element Sequence)
        ├── alternative (un-annotated, Quantified-?)
        │   └── concatenation (-> [$1**], flat list)
        │       └── piece+ (the actual piece objects)
        └── ("|" alternative)*  (the rest)
```

Today the alternation/alternative layers visibly nest because they're un-annotated. Once those rules carry their own typed annotations (planned in task #40), the `pattern` shape will collapse to something like:

```json
"pattern": { "type": "alternation", "alternatives": [
  { "type": "concatenation", "pieces": [...] },
  ...
]}
```

— a single alternation object whose children are the alternative-concatenations directly. Until then, consumers walk the current 2-deep `[<concat-array>, <rest-array>]` pattern field.
