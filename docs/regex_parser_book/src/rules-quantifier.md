# Quantifier Subtree

Six grammar rules cover quantifier syntax. **As of parser release `1.1.35` / contract `1.1.37` (slice 6 closure), every rule in the quantifier subtree is annotated. The whole subtree emits a fully typed shape with no string-vs-object dispatch left for consumers.**

## `quantifier`

```ebnf
quantifier = quant_base quant_suffix?
-> {type: "quantifier", min: $1.min, max: $1.max, greediness: $2}
```

**Annotated.** Emits a typed `{type, min, max, greediness}` object directly.

### Shape

```json
{
  "type": "quantifier",
  "min": <usize>,
  "max": <usize | null>,
  "greediness": "lazy" | "possessive" | []
}
```

- `min` is always a non-negative integer.
- `max` is a non-negative integer (bounded) OR JSON `null` (unbounded — for `*`, `+`, `{n,}`).
- `greediness` is the typed string `"lazy"` (when source has `?` suffix) or `"possessive"` (when `+` suffix), or the empty array `[]` (no suffix matched). `[]` corresponds to PCRE2's "greedy" default; consumers map `[]` → `"greedy"`.

A future slice will introduce a coalesce operator in the annotation language so `greediness: $2 ?? "greedy"` becomes the literal string `"greedy"` directly. Until then, the `[]` convention holds.

### Examples

| Input | `quantifier` field on the corresponding piece |
|---|---|
| `a*` | `{"type":"quantifier","min":0,"max":null,"greediness":[]}` |
| `a+` | `{"type":"quantifier","min":1,"max":null,"greediness":[]}` |
| `a?` | `{"type":"quantifier","min":0,"max":1,"greediness":[]}` |
| `a*?` | `{"type":"quantifier","min":0,"max":null,"greediness":"lazy"}` |
| `a*+` | `{"type":"quantifier","min":0,"max":null,"greediness":"possessive"}` |
| `a{3}` | `{"type":"quantifier","min":3,"max":3,"greediness":[]}` |
| `a{2,5}` | `{"type":"quantifier","min":2,"max":5,"greediness":[]}` |
| `a{2,}` | `{"type":"quantifier","min":2,"max":null,"greediness":[]}` |
| `a{,5}` | `{"type":"quantifier","min":0,"max":5,"greediness":[]}` |
| `a{2,5}?` | `{"type":"quantifier","min":2,"max":5,"greediness":"lazy"}` |

For pieces with no quantifier (e.g. plain `a`), the piece's `quantifier` field is the empty array `[]` (the un-matched `?`-Quantified slot), NOT a typed-quantifier object.

## `quant_base`

```ebnf
quant_base = "*"                -> {min: 0, max: null}
           | "+"                -> {min: 1, max: null}
           | "?"                -> {min: 0, max: 1}
           | counted_quantifier -> $1
```

**Annotated, fully typed.** Every branch emits a `{min, max}` object — shorthand quantifiers expand to their PCRE2-equivalent bounds, and the counted-quantifier branch passes through the body's typed shape via `$1`.

The benefit: the parent `quantifier` rule can read `$1.min` / `$1.max` regardless of which branch matched. No string-vs-object dispatch.

### Shape per branch

| Source | Output |
|---|---|
| `*` | `{"min": 0, "max": null}` (unbounded zero-or-more) |
| `+` | `{"min": 1, "max": null}` (unbounded one-or-more) |
| `?` | `{"min": 0, "max": 1}` (zero-or-one) |
| `{n}`/`{n,}`/`{n,m}`/`{,m}` | passes through `counted_quantifier`'s typed `{min, max}` |

### Earlier form (pre-1.1.35)
Pre-slice-6, this rule used positional-passthrough `-> $1` on every branch, which emitted the bare terminal string for shorthand cases (`"*"`/`"+"`/`"?"`) and the typed object only for counted quantifiers. That heterogeneous shape forced consumers to dispatch on string-vs-object. Slice 6 unifies the output to typed `{min, max}` for every branch — Tier-2 break, contract bumped accordingly.

## `counted_quantifier`

```ebnf
counted_quantifier = "{" brace_ws? counted_quantifier_body brace_ws? "}"
-> $3
```

**Annotated.** The annotation `-> $3` lifts the typed `counted_quantifier_body` shape straight through, dropping the surrounding `{`, whitespace, and `}` tokens — they carry no semantic information beyond "this is a counted quantifier" (which the surrounding `quant_base` context already conveys).

**Whitespace is `brace_ws` (space + tab), not the full `ws` set** — release `1.1.85` (`REGEX-PCRE2-FIDELITY.3.18`, ledger `REGEX-0094`). PCRE2 allows exactly spaces and tabs inside quantifier braces (oracle-frozen: `a{\t65536\t}` is err 105, i.e. a quantifier; the same brace with a `\n`/`\f`/`\r`/`\v` anywhere inside compiles clean as LITERALS). Pre-`1.1.85` the rule used `ws?`, so `a{\n2\n}` mis-parsed as the quantifier `{2}` where PCRE2 sees five literal pieces.

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
| `{ 2 , 5 }` (spaces/tabs anywhere) | `{"min": 2, "max": 5}` |
| `{065535}` (leading zeros collapse) | `{"min": 65535, "max": 65535}` |

## `counted_quantifier_body`

```ebnf
counted_quantifier_body = quant_bound_number brace_ws? "," brace_ws? quant_bound_number brace_ws?  -> {min: $1, max: $5}
                      | quant_bound_number brace_ws? "," brace_ws?               -> {min: $1, max: null}
                      | quant_bound_number brace_ws?                       -> {min: $1, max: $1}
                      | "," brace_ws? quant_bound_number                   -> {min: 0,  max: $3}
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

## `quant_bound_number` (and `quant_bound_number_body` / `quant_bound_core`)

```ebnf
@transform: str::parse::<usize>().unwrap_or(0)
quant_bound_number      = quant_bound_number_body
quant_bound_number_body = "0"+ quant_bound_core? | quant_bound_core
quant_bound_core        = "6553" ("0" | "1" | "2" | "3" | "4" | "5")
                        | "655" ("0" | "1" | "2") digit
                        | "65" ("0" | "1" | "2" | "3" | "4") digit digit
                        | "6" ("0" | "1" | "2" | "3" | "4") digit digit digit
                        | ("1" | "2" | "3" | "4" | "5") digit digit digit digit
                        | nonzero_digit digit digit digit
                        | nonzero_digit digit digit
                        | nonzero_digit digit
                        | nonzero_digit
```

**Annotated (`@transform` on the wrapper).** Release `1.1.85` (`REGEX-PCRE2-FIDELITY.3.18`, ledger `REGEX-0092`): the counted-quantifier bound is VALUE-bounded to `[0, 65535]` (PCRE2 err 105) **structurally** — the [`callout_number`](rules-misc.md) `[0, 255]` idiom scaled up. `quant_bound_number_body` admits leading zeros plus an optional nonzero-led core whose 9-branch cascade covers exactly `1..=65535` (the default longest-match tournament picks the full value over any prefix), so an out-of-range digit run has no fully-consuming quantifier parse — and generation draws in-range bounds **by construction**. The wrapper carries the same `@transform` span parse `digits` used, so `min`/`max` stay typed ints (`"065535"` → `65535`) and the AST shape is unchanged.

### Shape

`Json(Number(<usize>))` in `0..=65535` — exactly what the former `digits` slot emitted for in-range values. Out-of-range values (`{65536}`, `{4294967296}` — the former u32-overflow hole that skipped every check) no longer parse at all: the quantifier fails AND the [`literal_open_brace` guard](rules-atom.md) blocks the literal fallback, so the pattern REJECTS err-105-faithfully.

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

- Inside `version_number = digits ("." digits)?`.
- Inside `recursion_condition = "R" digits?`.
- Inside `signed_digits = sign? digits`.
- NOT inside `counted_quantifier_body` since release `1.1.85` — the `min`/`max` slots are the value-bounded `quant_bound_number` (same typed-int carrier; see above).
- NOT inside `callout_arg` since release `1.1.83`-era `.3.16` — the numeric callout arg is the value-bounded `callout_number` (same idiom, `[0, 255]`).

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

Walking a piece's `quantifier` slot:

```rust
fn extract_quantifier(piece: &Value) -> Option<Quantifier> {
    let q = piece.get("quantifier")?;
    if q.as_array().map_or(false, |a| a.is_empty()) {
        return None; // empty Quantified-? slot — no quantifier
    }
    let obj = q.as_object()?; // typed {type, min, max, greediness}
    let min = obj.get("min")?.as_u64()?;
    let max = match obj.get("max") {
        Some(Value::Null) => None,
        Some(Value::Number(n)) => n.as_u64(),
        _ => None,
    };
    let greediness = match obj.get("greediness") {
        Some(Value::String(s)) if s == "lazy" => Greediness::Lazy,
        Some(Value::String(s)) if s == "possessive" => Greediness::Possessive,
        _ => Greediness::Greedy, // including the `[]` no-suffix-matched case
    };
    Some(Quantifier { min, max, greediness })
}
```

Six lines of typed-field reads. No Sequence-wrapper digging, no string-vs-object dispatch. The `[]` → `Greediness::Greedy` mapping in the suffix slot will be removed once the annotation language gains a coalesce operator and `quantifier`'s annotation can emit the literal string `"greedy"` directly.

## Quantifier-subtree campaign — closed

All six rules in the quantifier subtree are now annotated:

- `digits` — typed `usize` integer (slice 1, post-1.1.31).
- `quant_suffix` — typed `"lazy"`/`"possessive"` strings (slice 2, post-1.1.32).
- `counted_quantifier_body` — typed `{min, max}` (slice 3, post-1.1.33).
- `counted_quantifier` — `-> $3` lifts body's typed shape (slice 4, post-1.1.33).
- `quant_base` — typed `{min, max}` for every branch (slice 5+6, post-1.1.34).
- `quantifier` — typed `{type, min, max, greediness}` (slice 6, post-1.1.34, contract `1.1.37`).

Next focus areas for the broader task #40 campaign: atom subtree, character class subtree, group family.
