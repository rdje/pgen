# Examples: Quantifiers

Concrete probe outputs for the seven PCRE2 quantifier forms × three greediness modes. Truncated to the relevant parts of the AST; the outer envelope is always `regex` → `pattern` → `[[<concat-array>], []]`.

## `a*` (greedy zero-or-more)

```json
{ "atom": "a", "quantifier": ["*", []], "type": "piece" }
```

`quantifier[0]` is the bare `quant_base` terminal `"*"`. `quantifier[1]` is the empty `quant_suffix?` slot `[]`.

## `a+` (greedy one-or-more)

```json
{ "atom": "a", "quantifier": ["+", []], "type": "piece" }
```

## `a?` (greedy zero-or-one)

```json
{ "atom": "a", "quantifier": ["?", []], "type": "piece" }
```

## `a*?` (lazy zero-or-more)

```json
{ "atom": "a", "quantifier": ["*", "lazy"], "type": "piece" }
```

`quantifier[1]` is the typed string `"lazy"` from `quant_suffix`'s annotation.

## `a*+` (possessive zero-or-more)

```json
{ "atom": "a", "quantifier": ["*", "possessive"], "type": "piece" }
```

## `a+?` (lazy one-or-more)

```json
{ "atom": "a", "quantifier": ["+", "lazy"], "type": "piece" }
```

## `a++` (possessive one-or-more)

```json
{ "atom": "a", "quantifier": ["+", "possessive"], "type": "piece" }
```

## `a??` (lazy zero-or-one)

```json
{ "atom": "a", "quantifier": ["?", "lazy"], "type": "piece" }
```

## `a?+` (possessive zero-or-one)

```json
{ "atom": "a", "quantifier": ["?", "possessive"], "type": "piece" }
```

## `a{3}` (exact count)

```json
{
  "atom": "a",
  "quantifier": [
    [
      "{",
      [],                       // optional ws — empty
      [<digits=3>, [], []],     // counted_quantifier_body branch 0
      [],                       // trailing optional ws
      "}"
    ],
    []                          // no suffix
  ],
  "type": "piece"
}
```

The counted_quantifier shape is the raw 5-element Sequence. The body at `quantifier[0][2]` is the un-annotated `counted_quantifier_body` Sequence; for `{3}` (just `digits`, no comma), the shape is `[<digits>, <ws?>, <optional sub-group>]` with the sub-group `[]` (not matched).

`<digits=3>` is the typed integer `3` (from the annotated `digits` rule).

## `a{2,5}` (range count)

```json
{
  "atom": "a",
  "quantifier": [
    [
      "{",
      [],
      [
        <digits=2>,
        [],
        [                       // sub-group: ["," ws? digits?]
          ",",
          [],
          [<digits=5>]           // digits? slot — matched
        ]
      ],
      [],
      "}"
    ],
    []
  ],
  "type": "piece"
}
```

The sub-group at body index 2 is now present: `[",", <ws>, <digits-Quantified>]`. The inner `[<digits=5>]` is the `Quantified-?` slot containing the second count.

## `a{2,}` (min only)

```json
{
  "atom": "a",
  "quantifier": [
    [
      "{",
      [],
      [
        <digits=2>,
        [],
        [
          ",",
          [],
          []                     // digits? slot — empty (no upper bound)
        ]
      ],
      [],
      "}"
    ],
    []
  ],
  "type": "piece"
}
```

The sub-group is matched (comma is present), but the inner `digits?` slot is empty — meaning unbounded.

## `a{,5}` (PCRE2 implicit min=0)

```json
{
  "atom": "a",
  "quantifier": [
    [
      "{",
      [],
      [                          // counted_quantifier_body branch 1
        ",",
        [],
        <digits=5>
      ],
      [],
      "}"
    ],
    []
  ],
  "type": "piece"
}
```

This is **branch 1** of `counted_quantifier_body` — starts with comma directly (no leading digits). The 3-element shape is `[",", <ws>, <digits>]`. Distinguish from branch 0 by inspecting the FIRST element: if it's a Number (typed integer), branch 0; if it's the string `","`, branch 1.

## `a{2,5}?` (lazy range)

```json
{
  "atom": "a",
  "quantifier": [
    [<counted_quantifier shape with min=2 max=5>],
    "lazy"
  ],
  "type": "piece"
}
```

The `quant_suffix?` slot now carries the typed string `"lazy"`.

## `a{2,5}+` (possessive range)

```json
{
  "atom": "a",
  "quantifier": [
    [<counted_quantifier shape with min=2 max=5>],
    "possessive"
  ],
  "type": "piece"
}
```

## Consumer extraction

```rust
fn extract_quant(piece: &Value) -> Option<Quantifier> {
    let q = piece.get("quantifier")?.as_array()?;
    if q.is_empty() { return None; }

    let (min, max) = match &q[0] {
        Value::String(s) if s == "*" => (0u64, None),
        Value::String(s) if s == "+" => (1, None),
        Value::String(s) if s == "?" => (0, Some(1)),
        Value::Array(seq) => {
            // counted_quantifier — Sequence ["{", ws, body, ws, "}"]
            let body = seq.get(2)?.as_array()?;
            // Determine branch:
            //   body[0] is a typed integer  → branch 0 (digits-first)
            //   body[0] is the string ","   → branch 1 (comma-first)
            match body.first() {
                Some(Value::Number(_)) => {
                    let min = body[0].as_u64()?;
                    let sub = body.get(2)?;
                    // sub is the optional ("," ws? digits?)? Quantified
                    // []     → no comma → exact {n}
                    // [...]  → comma present → look at inner [2] for max
                    if let Some(sub_arr) = sub.as_array() {
                        if sub_arr.is_empty() {
                            return Some(Quantifier { min, max: Some(min), greediness: Greediness::Greedy });
                        }
                        // sub_arr is the inner Sequence [",", ws?, digits?]
                        let max_slot = sub_arr.get(2)?.as_array()?;
                        // digits? — Quantified-?
                        let max_val = max_slot.first().and_then(|d| d.as_u64());
                        return Some(Quantifier { min, max: max_val, greediness: Greediness::Greedy });
                    }
                    None
                }
                Some(Value::String(s)) if s == "," => {
                    // Branch 1: {,m}
                    let max = body.get(2)?.as_u64()?;
                    Some(Quantifier { min: 0, max: Some(max), greediness: Greediness::Greedy })
                }
                _ => None,
            }
        }
        _ => return None,
    }?;

    let greediness = match &q[1] {
        Value::String(s) if s == "lazy" => Greediness::Lazy,
        Value::String(s) if s == "possessive" => Greediness::Possessive,
        _ => Greediness::Greedy,
    };

    Some(Quantifier { min, max, greediness })
}
```

Once future PGEN slices land typed `counted_quantifier_body` and unified `quantifier` shapes, this function collapses to a few `value.get("min").as_u64()` lookups.
