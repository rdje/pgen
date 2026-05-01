# Quantifier Subtree

Six grammar rules cover quantifier syntax. **At the parser release this book describes (current main HEAD, post-slices-1-through-5), `digits`, `quant_suffix`, `counted_quantifier_body`, `counted_quantifier`, and `quant_base` are annotated. Only the outer `quantifier` rule remains un-annotated; that closes in slice 6, which also carries the consolidated contract identity bump for slices 3-6.**

## `quantifier`

```ebnf
quantifier = quant_base quant_suffix?
```

**Un-annotated.** Output is the raw 2-element Sequence: `[<quant_base>, <quant_suffix?>]`.

### Current shape

```json
[<quant_base-content>, <quant_suffix?-content>]
```

`<quant_base-content>` is one of:
- `"*"`, `"+"`, `"?"` — for shorthand quantifiers (raw terminals).
- A typed `{min, max}` object — for counted quantifiers (`{n}`, `{n,}`, `{n,m}`, `{,m}`).

`<quant_suffix?-content>` is one of:
- `[]` — when no `?`/`+` suffix.
- `"lazy"` — typed string from the annotated `quant_suffix` rule.
- `"possessive"` — typed string from the annotated `quant_suffix` rule.

## `quant_base`

```ebnf
quant_base = ( "*" | "+" | "?" | counted_quantifier ) -> $1
```

**Annotated.** Single trailing `-> $1` annotation, broadcast to every alternative inside the parens-grouped Or. Each alternative emits its matched element via positional passthrough — shorthand branches pass through the literal terminal text; the counted-quantifier branch passes through the typed `{min, max}` object.

The factored form (single shared annotation) works thanks to task #38's parens-grouped-Or trailing-annotation broadcast support. The earlier per-branch form (`"*" -> $1 | "+" -> $1 | ...`) produced identical output but was needed pre-#38 to work around a codegen bug where only branch 0 of a parens-grouped Or received the trailing annotation.

This slice didn't change the consumer-visible JSON output — pre-annotation the same bytes were emitted via codegen defaults. The annotation locks the rule's emission into the typed-shape contract (Tier-2 stable) instead of relying on default fall-through.

### Current shape

- Branch 0 (`*`): JSON string `"*"`.
- Branch 1 (`+`): JSON string `"+"`.
- Branch 2 (`?`): JSON string `"?"`.
- Branch 3 (`counted_quantifier`): the typed `{min, max}` object directly (since `counted_quantifier` itself lifts the body's typed shape via `-> $3`).

## `counted_quantifier`

```ebnf
counted_quantifier = "{" ws? counted_quantifier_body ws? "}"
-> $3
```

**Annotated.** The annotation `-> $3` lifts the typed `counted_quantifier_body` shape straight through, dropping the surrounding `{`, whitespace, and `}` tokens — they carry no semantic information beyond "this is a counted quantifier" (which the surrounding `quant_base` context already conveys).

### Current shape

```json
{ "min": <usize>, "max": <usize | null> }
```

A typed `{min, max}` object, identical to whatever `counted_quantifier_body` emitted.

| Source input | Resulting shape at quant_base position |
|---|---|
| `{2}` | `{"min": 2, "max": 2}` |
| `{2,}` | `{"min": 2, "max": null}` |
| `{2,5}` | `{"min": 2, "max": 5}` |
| `{,5}` | `{"min": 0, "max": 5}` |

## `counted_quantifier_body`

```ebnf
counted_quantifier_body = digits "," digits ws?  -> {min: $1, max: $3}
                        | digits "," ws?         -> {min: $1, max: null}
                        | digits ws?             -> {min: $1, max: $1}
                        | "," ws? digits         -> {min: 0,  max: $3}
```

**Annotated.** Four explicit branches, one per logical case (`{n,m}`, `{n,}`, `{n}`, `{,m}`). Each branch carries its own per-branch annotation producing the same `{min, max}` shape. PEG-ordered alternation tries each branch in order; the first match wins. The most specific shapes come first so `{2,5}` matches the range form before falling through to `{2,}` or `{2}`.

### Current shape

```json
{ "min": <usize>, "max": <usize | null> }
```

`min` is always a typed integer (`Number`). `max` is either a typed integer (`Number`) or `null` (when the source uses the unbounded `{n,}` form).

### Why the four branches

The original rule was 2 branches with 4 logical cases compressed inside an optional sub-group of branch 1, which made consumer-side branch detection awkward. Splitting into 4 explicit branches lets each case carry its own annotation, so the output shape is identical regardless of which branch matched.

The book entry for [\Q...\E Quoted Literals](examples-quoted-literal.md) and [Quantifiers](examples-quantifiers.md) shows this typed shape in worked examples.

## `quant_suffix`

```ebnf
quant_suffix = "?" -> "lazy"
             | "+" -> "possessive"
```

**Annotated.** Per-branch annotations emit semantic strings.

### Shape

| Branch | Source | Output |
|---|---|---|
| 0 | `"?"` | `Json(String("lazy"))` |
| 1 | `"+"` | `Json(String("possessive"))` |

When the parent `quant_suffix?` slot doesn't match (no suffix in the input), the slot is `[]` (empty array), produced by the un-annotated `?`-Quantified parent.

### Examples

| Input | `quant_suffix?` slot |
|---|---|
| `a*` | `[]` (no suffix) |
| `a*?` | `"lazy"` (matched `?`) |
| `a*+` | `"possessive"` (matched `+`) |
| `a+?` | `"lazy"` |
| `a{2,5}?` | `"lazy"` |
| `a{2,5}+` | `"possessive"` |

## `digits`

```ebnf
@transform: str::parse::<usize>().unwrap_or(0)
digits = /([0-9]+)/
```

**Annotated.** Emits a typed integer via `@transform`.

### Shape

`Json(Number(<usize>))` — a non-negative integer.

### Where it appears

- Inside `counted_quantifier_body` — the `min` and `max` digit slots, e.g. `{2,5}` produces typed integers `2` and `5`.
- Inside `version_number = digits ("." digits)?`.
- Inside `recursion_condition = "R" digits?`.
- Inside `callout_arg = digits | callout_string`.
- Inside `signed_digits = sign? digits`.

The `digit` (per-char) rule used by `backreference_digits = nonzero_digit digit*` is a DIFFERENT rule and is not annotated — each digit is still a per-char Terminal.

## Related rules: `digit`, `hex_digits`, `octal_digits`, `nonzero_digit`, `hex_digit`, `octal_digit`

| Rule | Form | Annotated? | Shape |
|---|---|---|---|
| `digit` | `/([0-9])/` | NO | `Terminal(<char>)` |
| `hex_digit` | `/([0-9A-Fa-f])/` | NO | `Terminal(<char>)` |
| `octal_digit` | `/([0-7])/` | NO | `Terminal(<char>)` |
| `nonzero_digit` | `/([1-9])/` | NO | `Terminal(<char>)` |
| `hex_digits` | `hex_digit+` | NO | Quantified of digit terminals |
| `octal_digits` | `octal_digit+` | NO | Quantified of digit terminals |
| `digits` | `/([0-9]+)/ + @transform` | **YES** | typed integer |

`digits` is the lone annotated leaf in this group. The rest are un-annotated.

## Putting it together

Walking a piece's `quantifier` slot at this release:

```rust
fn extract_quantifier(piece: &Value) -> Option<Quantifier> {
    let q = piece.get("quantifier")?;
    let arr = q.as_array()?;
    if arr.is_empty() {
        return None; // no quantifier
    }
    // arr[0] = quant_base (raw "*", "+", "?", or typed {min, max} object)
    // arr[1] = quant_suffix? — empty array, "lazy", or "possessive"
    let base = &arr[0];
    let suffix = &arr[1];

    let (min, max) = match base {
        Value::String(s) if s == "*" => (0, None),
        Value::String(s) if s == "+" => (1, None),
        Value::String(s) if s == "?" => (0, Some(1)),
        Value::Object(map) => {
            // counted_quantifier — typed {min, max} object directly.
            let min = map.get("min")?.as_u64()?;
            let max = map.get("max").and_then(|v| match v {
                Value::Number(n) => n.as_u64(),
                Value::Null => None,
                _ => None,
            });
            // Note: max == None if the JSON value is null OR the field is missing.
            // For counted_quantifier_body the field is always present; null marks
            // the unbounded `{n,}` form.
            let max = match map.get("max") {
                Some(Value::Null) => None,
                Some(Value::Number(n)) => n.as_u64(),
                _ => None,
            };
            (min, max)
        }
        _ => return None,
    };

    let greediness = match suffix {
        Value::String(s) if s == "lazy" => Greediness::Lazy,
        Value::String(s) if s == "possessive" => Greediness::Possessive,
        _ => Greediness::Greedy,
    };

    Some(Quantifier { min, max, greediness })
}
```

The dispatch is now: shorthand quantifiers come through as bare strings; counted quantifiers come through as typed `{min, max}` objects. No structural digging into Sequence wrappers required.

## Future direction

One slice remains in the quantifier-subtree campaign:

- `quantifier` annotation. Goal: combine `quant_base` and `quant_suffix?` into a single typed object:

```json
{
  "type": "quantifier",
  "min": 2,
  "max": null,
  "greediness": "greedy"
}
```

When that final slice lands, the consumer's `extract_quantifier` walker collapses to a 4-line field read. The slice is tracked under task #40 in PGEN's tracker.
