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

## Inner whitespace inside `{...}` — `a{ 1 , 2 }`, `a{1 ,2}`, `a{1, 2}` (PCRE2 default mode)

PCRE2 default mode allows whitespace at any position inside `{...}` per `pcre2pattern(3) §"Repetition"` — "spaces and tabs may appear at any point inside the curly brackets". Pre-1.1.72 (PGEN-RGX-0080), only outer-boundary whitespace worked; whitespace abutting the comma caused the rule to fail and the parser to fall back to per-character literal pieces. Fixed in 1.1.72.

```json
{
  "atom": "a",
  "quantifier": {"type": "quantifier", "min": 1, "max": 2, "greediness": []},
  "type": "piece"
}
```

All five whitespace variants produce the same shape:
- `a{1,2}` — no whitespace
- `a{ 1,2 }` — outer whitespace
- `a{ 1 , 2 }` — whitespace around the comma (was 10 literal pieces pre-fix)
- `a{1 ,2}` — whitespace before the comma (was 7 literal pieces pre-fix)
- `a{1, 2}` — whitespace after the comma (was 7 literal pieces pre-fix)

PCRE2 conformance test suite testinput1:6679 (`/a{ 1 , 2 }/`) covers this case.

**Since release `1.1.85`, "whitespace" here is exactly space + tab** (the oracle-frozen PCRE2 set — `brace_ws`). Tab-spaced forms like `a{\t1\t,\t2\t}` parse to the same quantifier shape; a `\n`/`\f`/`\r`/`\v` anywhere inside the brace makes the whole brace a LITERAL (see the brace-model section below), matching PCRE2's tokenizer. Pre-`1.1.85` the rule used the full 6-char whitespace set, so `a{\n2\n}` mis-parsed as a quantifier.

## The PCRE2 brace tokenization model — quantifier vs literal vs reject (release 1.1.85)

A brace whose text is **syntactically-valid quantifier shape** — digits with spaces/tabs anywhere inside, in the four forms `{n}` `{n,}` `{n,m}` `{,m}` — is ALWAYS a quantifier in PCRE2, never a literal. PGEN encodes this with the value-structural [`quant_bound_number`](rules-quantifier.md) (bounds ≤ 65535 by construction) and the [`literal_open_brace`](rules-atom.md) negative-lookahead guard (a valid-syntax brace can never fall back to literal pieces). Oracle: `pcre2test` 10.47, `REGEX-PCRE2-FIDELITY.3.18`, ledger `REGEX-0092`/`0093`/`0094`.

| Input | Verdict (PGEN `1.1.85` = PCRE2) | Why |
|---|---|---|
| `a{2,5}` · `a{ 2 , 5 }` · `a{\t2\t,\t5\t}` | ACCEPT (quantifier) | valid syntax, in-range, repeatable atom precedes |
| `a{0000000000065535}` | ACCEPT (quantifier `{65535,65535}`) | the bound is the VALUE, not the digit count |
| `a{65536}` · `a{4294967296}` · `a{0,65536}` | REJECT (err-105 class) | value > 65535; `1.1.84` wrongly accepted the >u32 forms |
| `a{5,2}` · `a{\t5\t,\t2\t}` | REJECT (err-104 class) | min > max (validator-owned; tab-aware since `1.1.85`) |
| `{2,5}` at start · `x\|{2,5}` · `a{2}{3}` | REJECT (err-109 class) | valid-syntax brace at a non-repeatable position; `1.1.84` wrongly accepted these as literals |
| `a{}` · `a{,}` · `a{ }` · `{a}` · `a{1,2,3}b` · `a{65536` (unterminated) | ACCEPT (literal pieces) | not quantifier syntax — literal `{` exactly as PCRE2 |
| `a{\n2\n}` · `a{\n5,2\n}` | ACCEPT (literal pieces) | a `\n`/`\f`/`\r`/`\v` inside the brace makes it literal; `1.1.84` wrongly order-rejected `a{\n5,2\n}` |

For an accepted literal brace, each character is its own `piece` with a bare-string atom — e.g. `a{}`:

```json
[
  {"atom": "a", "quantifier": [], "type": "piece"},
  {"atom": "{", "quantifier": [], "type": "piece"},
  {"atom": "}", "quantifier": [], "type": "piece"}
]
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

The whole quantifier subtree is annotated as of slice 6: `quant_bound_number` (typed integer in `0..=65535` — the value-structural `digits` replacement since release `1.1.85`), `quant_suffix` (typed enum string), `counted_quantifier_body` (typed `{min, max}`), `counted_quantifier` (passthrough), `quant_base` (typed `{min, max}` for every branch), and `quantifier` (typed `{type, min, max, greediness}`). Consumer code is a six-line typed-field read.

The `[]` → `Greediness::Greedy` mapping in the suffix lookup will be removed when the annotation language gains a coalesce operator and `quantifier`'s annotation can emit the literal string `"greedy"` directly.
