# Examples: Quantifiers

Concrete probe outputs for the seven PCRE2 quantifier forms × three greediness modes. Truncated to the relevant parts of the AST; the outer envelope is always `regex` → `pattern` → `[[<concat-array>], []]`.

The `quantifier` field on each piece is the typed object `{type:"quantifier", min, max, greediness}` (post-1.1.34). The `greediness` field carries `"lazy"`/`"possessive"` for explicit suffixes, or `[]` (the un-matched `quant_suffix?` slot) for the greedy default — consumers map `[]` → `"greedy"`.

## `a*` (greedy zero-or-more)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 0, "max": null, "greediness": []},
  "type": "piece"
}
```

`min:0`, `max:null` (unbounded). `greediness: []` is the un-matched `quant_suffix?` slot — interpret as greedy.

## `a+` (greedy one-or-more)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 1, "max": null, "greediness": []},
  "type": "piece"
}
```

## `a?` (greedy zero-or-one)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 0, "max": 1, "greediness": []},
  "type": "piece"
}
```

## `a*?` (lazy zero-or-more)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 0, "max": null, "greediness": "lazy"},
  "type": "piece"
}
```

## `a*+` (possessive zero-or-more)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 0, "max": null, "greediness": "possessive"},
  "type": "piece"
}
```

## `a+?` (lazy one-or-more)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 1, "max": null, "greediness": "lazy"},
  "type": "piece"
}
```

## `a++` (possessive one-or-more)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 1, "max": null, "greediness": "possessive"},
  "type": "piece"
}
```

## `a??` (lazy zero-or-one)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 0, "max": 1, "greediness": "lazy"},
  "type": "piece"
}
```

## `a?+` (possessive zero-or-one)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 0, "max": 1, "greediness": "possessive"},
  "type": "piece"
}
```

## `a{3}` (exact count)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 3, "max": 3, "greediness": []},
  "type": "piece"
}
```

`counted_quantifier_body`'s branch 2 (`digits ws? -> {min: $1, max: $1}`) duplicates the single source count into both fields. `counted_quantifier`'s `-> $3` lifts the body's typed shape, then `quant_base`'s passthrough (`-> $1`) for the `counted_quantifier` branch carries it up, then `quantifier`'s annotation merges with the (empty) suffix slot.

## `a{2,5}` (range count)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 2, "max": 5, "greediness": []},
  "type": "piece"
}
```

## `a{2,}` (min only)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 2, "max": null, "greediness": []},
  "type": "piece"
}
```

The unbounded upper bound is encoded as a typed JSON `null`.

## `a{,5}` (PCRE2 implicit min=0)

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 0, "max": 5, "greediness": []},
  "type": "piece"
}
```

## `a{2,5}?` (lazy range) and `a{2,5}+` (possessive range)

For `a{2,5}?`:

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 2, "max": 5, "greediness": "lazy"},
  "type": "piece"
}
```

For `a{2,5}+`:

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 2, "max": 5, "greediness": "possessive"},
  "type": "piece"
}
```

## Consumer extraction

```rust
fn extract_quant(piece: &Value) -> Option<Quantifier> {
    let q = piece.get("quantifier")?;
    // Empty array `[]` means the un-matched `quantifier?` slot — no quantifier.
    if q.as_array().map_or(false, |a| a.is_empty()) {
        return None;
    }
    let obj = q.as_object()?;
    let min = obj.get("min")?.as_u64()?;
    let max = match obj.get("max") {
        Some(Value::Null) => None,
        Some(Value::Number(n)) => n.as_u64(),
        _ => None,
    };
    let greediness = match obj.get("greediness") {
        Some(Value::String(s)) if s == "lazy" => Greediness::Lazy,
        Some(Value::String(s)) if s == "possessive" => Greediness::Possessive,
        _ => Greediness::Greedy, // includes the un-matched `[]` case
    };
    Some(Quantifier { min, max, greediness })
}
```

The whole quantifier subtree is annotated as of slice 6: `digits` (typed integer), `quant_suffix` (typed enum string), `counted_quantifier_body` (typed `{min, max}`), `counted_quantifier` (passthrough), `quant_base` (typed `{min, max}` for every branch), and `quantifier` (typed `{type, min, max, greediness}`). Consumer code is a six-line typed-field read.

The `[]` → `Greediness::Greedy` mapping in the suffix lookup will be removed when the annotation language gains a coalesce operator and `quantifier`'s annotation can emit the literal string `"greedy"` directly.
