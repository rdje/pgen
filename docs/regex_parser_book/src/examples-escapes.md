# Examples: Escapes

Concrete probe outputs for PCRE2 escape sequences. As of slice 16 (post-1.1.46), the `escape` rule is a transparent wrapper (`-> $2`) and 6 of the 7 `escape_unit` sub-rules emit typed `{type:"escape", kind:<form>, ...}` objects directly: simple, single_byte, control (slice 14), hex, unicode (slice 15), octal (slice 16). Only `property_escape` still emits a raw shape pending the follow-up slice.

## Shorthand escapes — `\d`, `\w`, `\s`, `\.`, `\\`, etc.

These all flow through `simple_escape` (the catch-all branch of `escape_unit`):

```ebnf
simple_escape  = any_char -> {type: "escape", kind: "shorthand", char: $1}
```

For `\d`:

```json
{
  "atom": {"type": "escape", "kind": "shorthand", "char": "d"},
  ...
}
```

For `\w`: same shape, `char:"w"`. For `\s`: `char:"s"`. For `\.`: `char:"."`. For `\\`: `char:"\\"` (the actual backslash char). Any letter/symbol after `\` produces a `kind:"shorthand"` typed object with the char in the `char` field.

(Pre-slice-14 the same input emitted `["\\", [[[[[ "d" ]]]]]]` — a 5-level un-annotated chain wrapping the matched char Terminal. Consumers walking that chain via `to_json_value()` saw deeply-nested arrays. Post-slice the typed shape is a single field read.)

## Escaped metacharacters — `\.`, `\\`, `\(`, `\)`, etc.

Same `kind:"shorthand"` shape as shorthand classes — `simple_escape` matches any char after `\`:

For `\.`:

```json
{
  "atom": {"type": "escape", "kind": "shorthand", "char": "."},
  ],
  ...
}
```

For `\\` (escaped backslash): `char:"\\"` (the literal backslash char).

## Hex escape (1-2 digit form) — `\xFF`

```json
{
  "atom": {"type": "escape", "kind": "hex", "digits": "FF"},
  ...
}
```

For `\xF` (single digit): `digits:"F"`. The `hex_escape_short_payload` regex literal `/([0-9A-Fa-f]{1,2})/` accepts 1 or 2 hex digits.

## Hex escape (braced form) — `\x{1F}`

```json
{
  "atom": {"type": "escape", "kind": "hex", "digits": "1F"},
  ...
}
```

The braced form accepts any number of hex digits via `hex_digits = /([0-9A-Fa-f]+)/`, with optional whitespace inside the braces (PCRE2's `brace_ws` allowance).

## Hex escape (long codepoint) — `\x{1F600}` (😀)

```json
{
  "atom": {"type": "escape", "kind": "hex", "digits": "1F600"},
  ...
}
```

Consumers parse the codepoint with `usize::from_str_radix(obj.digits, 16)`.

## `digits` is a string, not an int

The hex/unicode `digits` field carries the raw hex string. Decode to a numeric codepoint yourself:

```rust
let cp = usize::from_str_radix(obj["digits"].as_str().unwrap(), 16).unwrap();
```

PGEN's `@transform` machinery is currently hard-coded to `str::parse::<TYPE>().unwrap_or(DEFAULT)`-style and can't express `from_str_radix(s, 16)`. Extending it is a separate codegen-feature slice.

## Unicode escape — `\u{1F600}`

```json
{
  "atom": {"type": "escape", "kind": "unicode", "digits": "1F600"},
  ...
}
```

**Validator note:** PGEN's host-side compile validator currently rejects `\u{...}` escapes ("unsupported regex escape `\u`"). The annotation IS in place and correct when the validator allows the escape through; for inputs the validator rejects, no AST is produced. That validator behavior is pre-existing and tracked separately from the slice 15 shape work.

## Octal escape (braced) — `\o{777}`

```json
{
  "atom": {"type": "escape", "kind": "octal", "digits": "777"},
  ...
}
```

For `\o{7777}` (longer): `digits:"7777"`. For `\o{1}`: `digits:"1"`.

## Octal escape (bare 1-3 digit) — `\377`

**At atom-level**, bare `\NNN` is parsed as a numeric backreference (PEG-ordering: the `backreference` branch in `atom` precedes `escape`):

```json
{
  "atom": {"type": "backreference", "kind": "numeric", "index": 377},
  ...
}
```

This is pre-existing behavior — PCRE2 disambiguates `\NNN` between numeric backref and bare octal contextually ("if NNN ≤ 9 OR there are NNN capture groups, treat as backref; else octal"), which PEG cannot express directly. Disambiguation is left to consumers via post-parse semantic analysis if/when atom-level bare-octal support is needed.

**Inside character classes**, the `class_range_escape_unit` path reaches the bare-octal branch and emits the typed shape:

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      {"type": "escape", "kind": "octal", "digits": "377"}
    ],
    "]"
  ],
  ...
}
```

That is `[\377]` parses with the bare-octal escape typed inside the class body.

## `digits` is a string for octal too

Same convention as hex/unicode — consumers parse with `usize::from_str_radix(obj.digits, 8)`.

## Control escape — `\cA`

```json
{
  "atom": {"type": "escape", "kind": "control", "char": "A"},
  ...
}
```

Typed `{type:"escape", kind:"control", char:<C>}` — the `c` prefix is dropped, the matched control letter is in the `char` field. For `\cZ`: `char:"Z"`. For `\cz`: `char:"z"` (case-sensitive).

## Property escape (braced) — `\p{Lu}`

```json
{
  "atom": [
    "\\",
    [
      "p{",
      [<prop_name shape — chars "L", "u">],
      "}"
    ]
  ],
  ...
}
```

3-element Sequence. `prop_name` is a Quantified-`+` of `prop_name_chars`.

For `\P{Lu}`: opening is `"P{"` instead of `"p{"`.

## Short property escape — `\pL`, `\PN`

```json
{
  "atom": [
    "\\",
    [
      "p",
      "L"                       // short_prop_letter
    ]
  ],
  ...
}
```

2-element Sequence with `"p"` (or `"P"`) followed by single short-property letter.

## Single-byte escape — `\C`

```json
{
  "atom": {"type": "escape", "kind": "single_byte"},
  ...
}
```

PCRE2's `\C` matches one code unit. Typed `{type:"escape", kind:"single_byte"}` — no `char` field since the char is fixed (always uppercase `C`).

## Identifying escape kind from the AST shape

The `escape_unit` is an Or with 7 branches. As of slice 16, 6 of those branches emit typed `{type:"escape", kind:<form>, ...}` objects directly — consumers can dispatch on `kind`. The remaining `property_escape` branch still emits a raw structural shape; consumers fall through to a structural-prefix check.

```rust
fn classify_escape(escape_atom: &Value) -> Option<EscapeKind> {
    // Typed branches: simple, single_byte, control, hex, unicode, octal
    if let Some(obj) = escape_atom.as_object() {
        if obj.get("type").and_then(|v| v.as_str()) == Some("escape") {
            return match obj.get("kind").and_then(|v| v.as_str())? {
                "shorthand"   => Some(EscapeKind::Simple),
                "single_byte" => Some(EscapeKind::SingleByte),
                "control"     => Some(EscapeKind::Control),
                "hex"         => Some(EscapeKind::Hex),
                "unicode"     => Some(EscapeKind::Unicode),
                "octal"       => Some(EscapeKind::Octal),
                _             => None,
            };
        }
    }

    // Untyped raw-shape branch: property
    let arr = escape_atom.as_array()?;
    match arr.first() {
        Some(Value::String(s)) if s == "p{" || s == "P{" => {
            Some(EscapeKind::Property(PropertyForm::Braced))
        }
        Some(Value::String(s)) if s == "p" || s == "P" => {
            Some(EscapeKind::Property(PropertyForm::Short))
        }
        _ => None,
    }
}
```

## Future direction

The remaining task #40 escape-subtree slice will type `property_escape`. Once that lands, the structural fallback above will be removed and the dispatcher reduces to a single object-shape check on `kind`.
