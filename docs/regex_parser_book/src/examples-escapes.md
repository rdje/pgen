# Examples: Escapes

Concrete probe outputs for PCRE2 escape sequences. As of slice 14 (post-1.1.43), the `escape` rule is a transparent wrapper (`-> $2`) and 3 of the 7 `escape_unit` sub-rules emit typed `{type:"escape", kind:<form>, ...}` objects directly. Hex/unicode/octal/property branches still emit raw shapes pending follow-up slices.

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
  "atom": [
    "\\",
    [
      "x",
      "F",
      [["F"]]   // hex_digit? Quantified-?, optional second digit
    ]
  ],
  ...
}
```

3-element Sequence inside the `escape_unit`: `["x", <hex_digit>, <hex_digit?>]`.

## Hex escape (braced form) — `\x{1F}`

```json
{
  "atom": [
    "\\",
    [
      "x{",
      [],                       // brace_ws? — empty
      [["1"], ["F"]],           // hex_digits — Quantified of digit terminals
      [],                       // brace_ws? — empty
      "}"
    ]
  ],
  ...
}
```

5-element Sequence. The `hex_digits` at index 2 is a Quantified-`+` of single-char hex_digit terminals; consumers concatenate to recover the hex string.

## Hex escape (long codepoint) — `\x{1F600}` (😀)

```json
{
  "atom": [
    "\\",
    [
      "x{",
      [],
      [["1"], ["F"], ["6"], ["0"], ["0"]],
      [],
      "}"
    ]
  ],
  ...
}
```

5 hex_digits. Concatenate to `"1F600"`, parse as hex int = 128512 = U+1F600 = 😀.

## Unicode escape — `\u{1F600}`

```json
{
  "atom": [
    "\\",
    [
      "u{",
      [<hex_digits>],
      "}"
    ]
  ],
  ...
}
```

3-element Sequence (no whitespace allowance — note the differences from `\x{...}` shape).

## Octal escape (braced) — `\o{777}`

```json
{
  "atom": [
    "\\",
    [
      "o{",
      [],
      [<octal_digits — chars: "7", "7", "7">],
      [],
      "}"
    ]
  ],
  ...
}
```

5-element Sequence.

## Octal escape (bare 1-3 digit) — `\377`

```json
{
  "atom": [
    "\\",
    [
      "3",                     // first octal digit (REQUIRED)
      ["7"],                   // second (optional, Quantified-?)
      ["7"]                    // third (optional, Quantified-?)
    ]
  ],
  ...
}
```

3-element Sequence inside `escape_unit`. Each octal_digit is a single char.

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

The `escape_unit` is an Or with 7 branches. At this release, none are annotated, so the consumer dispatches by structural signature on the inner content (the second element of the outer `["\\", <unit>]` Sequence):

```rust
fn classify_escape(escape_atom: &Value) -> Option<EscapeKind> {
    let arr = escape_atom.as_array()?;
    let unit = arr.get(1)?;

    match unit {
        // single_byte_escape: Terminal "C"
        Value::String(s) if s == "C" => Some(EscapeKind::SingleByte),

        Value::Array(seq) => {
            match seq.first() {
                // hex_escape — starts with "x" or "x{"
                Some(Value::String(s)) if s == "x" => Some(EscapeKind::Hex(HexForm::Short)),
                Some(Value::String(s)) if s == "x{" => Some(EscapeKind::Hex(HexForm::Braced)),

                // unicode_escape — starts with "u{"
                Some(Value::String(s)) if s == "u{" => Some(EscapeKind::Unicode),

                // octal_escape — starts with "o{" or with a digit
                Some(Value::String(s)) if s == "o{" => Some(EscapeKind::Octal(OctalForm::Braced)),
                Some(Value::String(s)) if s.chars().all(|c| ('0'..='7').contains(&c)) => {
                    Some(EscapeKind::Octal(OctalForm::Bare))
                }

                // control_escape — starts with "c"
                Some(Value::String(s)) if s == "c" => Some(EscapeKind::Control),

                // property_escape — starts with "p{", "P{", "p", or "P"
                Some(Value::String(s)) if s == "p{" || s == "P{" => {
                    Some(EscapeKind::Property(PropertyForm::Braced))
                }
                Some(Value::String(s)) if s == "p" || s == "P" => {
                    Some(EscapeKind::Property(PropertyForm::Short))
                }

                // simple_escape — anything else; deeply nested, leaf is the char
                _ => Some(EscapeKind::Simple),
            }
        }

        _ => None,
    }
}
```

The simple_escape's char is recovered by descending the nested Alternative wrappers until reaching a string leaf — see [Walking the AST](walking-the-ast.md) for the descent pattern.

## Future direction

Future task #40 escape-subtree slice will produce shapes like `{type: "escape", kind: "simple", char: "d"}`, `{type: "escape", kind: "hex", value: 31}`, etc. Until then, the structural-prefix dispatch above is the way.
