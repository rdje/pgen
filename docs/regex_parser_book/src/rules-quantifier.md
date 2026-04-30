# Quantifier Subtree

Six grammar rules cover quantifier syntax. **At the parser release this book describes (post-PGEN-RGX-0074, after the cold-clone bootstrap landed), `digits` and `quant_suffix` are annotated. The remaining four (`quantifier`, `quant_base`, `counted_quantifier`, `counted_quantifier_body`) are not yet — they emit raw envelope shapes and will be annotated in successive future slices.**

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
- A `Sequence` of `["{", ws?, <counted_quantifier_body>, ws?, "}"]` — for counted quantifiers.

`<quant_suffix?-content>` is one of:
- `[]` — when no `?`/`+` suffix.
- `"lazy"` — typed string from the annotated `quant_suffix` rule.
- `"possessive"` — typed string from the annotated `quant_suffix` rule.

## `quant_base`

```ebnf
quant_base = "*" | "+" | "?" | counted_quantifier
```

**Un-annotated.** Each branch emits the raw matched text or the un-annotated `counted_quantifier` Sequence.

### Current shape

- Branch 0 (`*`): `Terminal("*")`. Visible in JSON as the bare string `"*"`.
- Branch 1 (`+`): `Terminal("+")`. Visible as `"+"`.
- Branch 2 (`?`): `Terminal("?")`. Visible as `"?"`.
- Branch 3 (`counted_quantifier`): nested counted quantifier shape — see below.

## `counted_quantifier`

```ebnf
counted_quantifier = "{" ws? counted_quantifier_body ws? "}"
```

**Un-annotated.** Output is the raw 5-element Sequence: `["{", ws?, body, ws?, "}"]`.

## `counted_quantifier_body`

```ebnf
counted_quantifier_body = digits ws? ("," ws? digits?)?
                        | "," ws? digits
```

**Un-annotated** at this release. Two branches with four logical cases compressed inside the optional sub-group of branch 1. Each branch emits the raw Sequence of its body.

### Current shape — Branch 0

```json
[
  <digits>,                  // first count, typed integer (digits IS annotated)
  <ws-content>,              // optional whitespace
  <optional [",", ws, digits?] sub-group>
]
```

For `{n}` (no comma): branch 0 matches with the optional sub-group empty, so the third slot is `[]`.
For `{n,}` (comma, no second count): branch 0 matches with the sub-group present but its inner `digits?` empty.
For `{n,m}` (full range): branch 0 matches with both digits present.

### Current shape — Branch 1

```json
[",", <ws-content>, <digits>]
```

For `{,m}`: branch 1 matches.

### Walking the body shape

Because `digits` IS annotated to a typed integer at this release, the digit values inside `counted_quantifier_body` are direct `serde_json::Value::Number` integers — no concatenation needed.

A consumer extracting count bounds:

```rust
fn extract_count_bounds(body: &Value) -> Option<(u64, Option<u64>)> {
    let arr = body.as_array()?;
    // Branch detection by first element shape:
    //   Number → branch 0 (starts with digits).
    //   String "," → branch 1 (starts with comma, {,m} form).
    if matches!(arr.first(), Some(Value::String(s)) if s == ",") {
        // Branch 1: {,m}
        let max = arr.get(2)?.as_u64()?;
        return Some((0, Some(max)));
    }
    // Branch 0 — first element is digits (a typed integer)
    let min = arr.first()?.as_u64()?;
    // arr[2] is the optional ("," ws? digits?)? sub-group
    let sub = arr.get(2)?;
    if let Some(sub_arr) = sub.as_array() {
        if sub_arr.is_empty() {
            return Some((min, Some(min))); // {n} — no comma
        }
        // sub_arr is the [",", ws?, digits?] inner Sequence
        let inner_digits_slot = sub_arr.get(2)?;
        // inner_digits_slot is the digits?'s Quantified-? slot
        if let Some(inner_arr) = inner_digits_slot.as_array() {
            if inner_arr.is_empty() {
                return Some((min, None));     // {n,}
            }
            let max = inner_arr.first()?.as_u64()?;
            return Some((min, Some(max)));     // {n,m}
        }
    }
    None
}
```

(The branching here is what motivated splitting `counted_quantifier_body` into 4 explicit branches in a future slice — when that lands, the consumer code becomes a single-line field lookup.)

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
    // arr[0] = quant_base (raw "*", "+", "?", or counted_quantifier Sequence)
    // arr[1] = quant_suffix? — empty array, "lazy", or "possessive"
    let base = &arr[0];
    let suffix = &arr[1];

    let (min, max) = match base {
        Value::String(s) if s == "*" => (0, None),
        Value::String(s) if s == "+" => (1, None),
        Value::String(s) if s == "?" => (0, Some(1)),
        Value::Array(seq) => {
            // counted_quantifier — Sequence shape: ["{", ws?, body, ws?, "}"]
            // Body is at index 2.
            let body = seq.get(2)?;
            extract_count_bounds(body)?
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

## Future direction

Future PGEN releases will annotate the remaining quantifier-subtree rules. Target shape for the unified `quantifier` rule:

```json
{
  "type": "quantifier",
  "min": 2,
  "max": null,           // null = unbounded
  "greediness": "greedy" // "greedy" | "lazy" | "possessive"
}
```

The slices to land that target shape are tracked under task #40 in PGEN's tracker. When RGX bumps PGEN to a release containing them, this book chapter will be updated. Until then, the per-rule walking above is operative.
