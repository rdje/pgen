# Examples: Escapes

Concrete probe outputs for PCRE2 escape sequences. The `escape` rule emits a 2-element Sequence `["\\", <escape_unit>]`. Sub-rules are un-annotated.

## Shorthand escapes — `\d`, `\w`, `\s`

These all use `simple_escape` (the catch-all branch of `escape_unit`).

For `\d`:

```json
{
  "atom": [
    "\\",
    [[[[[ "d" ]]]]]   // un-annotated escape_unit → simple_escape → any_char chain
  ],
  ...
}
```

The deeply-nested array on the right of `"\\"` is the un-annotated wrapping chain. Inner-most leaf is the matched char `"d"`.

For `\w`: same shape, leaf is `"w"`. For `\s`: leaf is `"s"`.

## Escaped metacharacters — `\.`, `\\`, `\(`, `\)`, etc.

Same shape as shorthand escapes — `simple_escape` matches any char after `\`:

For `\.`:

```json
{
  "atom": [
    "\\",
    [[[[[ "." ]]]]]
  ],
  ...
}
```

For `\\` (escaped backslash): leaf is `"\\"` (2-char string).

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
  "atom": [
    "\\",
    [
      "c",
      [<any_char shape for "A">]
    ]
  ],
  ...
}
```

2-element Sequence. The `any_char` is itself a regex `Terminal`.

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
  "atom": [
    "\\",
    "C"                          // single_byte_escape — Terminal "C"
  ],
  ...
}
```

PCRE2's `\C` matches one code unit. Inside `escape_unit`, the `single_byte_escape` branch matches `"C"`.

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
