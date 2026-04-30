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
    { "min": 3, "max": 3 },
    []
  ],
  "type": "piece"
}
```

`counted_quantifier`'s `-> $3` annotation lifts `counted_quantifier_body`'s typed `{min, max}` straight through, dropping the surrounding `{`, whitespace, and `}` tokens. The body's branch 2 (`digits ws? -> {min: $1, max: $1}`) duplicates the single source count into both fields.

## `a{2,5}` (range count)

```json
{
  "atom": "a",
  "quantifier": [
    { "min": 2, "max": 5 },
    []
  ],
  "type": "piece"
}
```

Branch 0 of `counted_quantifier_body` — `digits "," digits ws? -> {min: $1, max: $3}`.

## `a{2,}` (min only)

```json
{
  "atom": "a",
  "quantifier": [
    { "min": 2, "max": null },
    []
  ],
  "type": "piece"
}
```

Branch 1 of `counted_quantifier_body` — `digits "," ws? -> {min: $1, max: null}`. The unbounded upper bound is encoded as a typed JSON `null` (the `null` literal added in the same slice that introduced this typed shape).

## `a{,5}` (PCRE2 implicit min=0)

```json
{
  "atom": "a",
  "quantifier": [
    { "min": 0, "max": 5 },
    []
  ],
  "type": "piece"
}
```

Branch 3 of `counted_quantifier_body` — `"," ws? digits -> {min: 0, max: $3}`. The implicit `min=0` is a literal numeric in the annotation, not derived from the source.

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

## `a{2,5}?` and `a{2,5}+` (lazy / possessive range)

For `a{2,5}?`:

```json
{
  "atom": "a",
  "quantifier": [
    { "min": 2, "max": 5 },
    "lazy"
  ],
  "type": "piece"
}
```

For `a{2,5}+`:

```json
{
  "atom": "a",
  "quantifier": [
    { "min": 2, "max": 5 },
    "possessive"
  ],
  "type": "piece"
}
```

The counted-quantifier body is the same typed object regardless of greediness; the `quant_suffix?` slot independently carries `"lazy"` / `"possessive"` / `[]`.

## Consumer extraction

```rust
fn extract_quant(piece: &Value) -> Option<Quantifier> {
    let q = piece.get("quantifier")?.as_array()?;
    if q.is_empty() { return None; }

    let (min, max) = match &q[0] {
        Value::String(s) if s == "*" => (0u64, None),
        Value::String(s) if s == "+" => (1, None),
        Value::String(s) if s == "?" => (0, Some(1)),
        Value::Object(map) => {
            // counted_quantifier — typed {min, max} object directly.
            let min = map.get("min")?.as_u64()?;
            let max = match map.get("max") {
                Some(Value::Null) => None,            // unbounded `{n,}`
                Some(Value::Number(n)) => n.as_u64(), // bounded
                _ => return None,
            };
            (min, max)
        }
        _ => return None,
    };

    let greediness = match &q[1] {
        Value::String(s) if s == "lazy" => Greediness::Lazy,
        Value::String(s) if s == "possessive" => Greediness::Possessive,
        _ => Greediness::Greedy,
    };

    Some(Quantifier { min, max, greediness })
}
```

Once the remaining quantifier-subtree slices land (`quant_base`, then unified `quantifier` with `{type:"quantifier", min, max, greediness}`), this function collapses to a 4-line typed-field read.
