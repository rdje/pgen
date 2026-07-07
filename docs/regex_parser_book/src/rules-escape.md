# Escape Subtree

PCRE2 escape sequences (`\d`, `\w`, `\xFF`, `\u{1F}`, `\o{777}`, `\cA`, `\p{Lu}`, etc.). None currently annotated. All emit raw envelope shapes.

## `escape`

```ebnf
@type: "context_sensitive_escape_sequence"
escape = "\\" escape_unit
```

2-element Sequence: `["\\", <escape_unit shape>]`. The `@type`/`@description`/`@effect`/etc. are semantic annotations, not return annotations.

## `escape_unit`

```ebnf
escape_unit = single_byte_escape
            | hex_escape
            | unicode_escape
            | octal_escape
            | control_escape
            | property_escape
            | simple_escape
```

7-way Or, **un-annotated**.

### Branches and shapes

| Branch | Form | Example | Shape |
|---|---|---|---|
| 0 (`single_byte_escape`) | `\C` | `\C` | `Terminal("C")` (just the unit; outer `escape` adds the `\\`) |
| 1 (`hex_escape`) | `\xFF`, `\x{FFFF}` | `\xFF` | nested hex_escape Sequence |
| 2 (`unicode_escape`) | `\u{FFFF}` | `\u{1F}` | nested unicode_escape Sequence |
| 3 (`octal_escape`) | `\o{777}`, `\377` | `\o{777}` | nested octal_escape Sequence |
| 4 (`control_escape`) | `\cA` | `\cA` | nested control_escape Sequence |
| 5 (`property_escape`) | `\p{Lu}`, `\P{Lu}`, `\pL`, `\PL` | `\pL` | nested property_escape Sequence |
| 6 (`simple_escape`) | `\<any-char>` (catch-all) | `\d`, `\.`, `\\` | `Terminal(<char>)` (single char after backslash) |

The branches are tried in order; `simple_escape` is the catch-all that matches any unrecognized `\<char>` escape. **Profile note (REGEX-PCRE2-FIDELITY.3.1):** branch 2 (`unicode_escape`, `\u{…}`) is `@profiles: ["relaxed"]` — active only under the `relaxed` profile; branch 6 (`simple_escape`) splits into a strict default variant and a `relaxed` variant. In the default (`pcre2`) profile the six PCRE2-unsupported escape letters `\i \F \l \L \u \U` are rejected. See [Profiles — strict default vs `relaxed`](#profiles--strict-default-vs-relaxed).

## `single_byte_escape`

```ebnf
single_byte_escape = "C"
```

PCRE2's `\C` — match one code unit. `Terminal("C")`.

## `simple_escape`

```ebnf
simple_escape = simple_escape_strict | simple_escape_relaxed
```

The catch-all single-char escape. Emits the typed shorthand object `{type: "escape", kind: "shorthand", char: <char>}` — the character that follows the backslash. It is a transparent alternation over a **strict** (default / `pcre2` profile) variant and a `@profiles: ["relaxed"]` variant (see [Profiles — strict default vs `relaxed`](#profiles--strict-default-vs-relaxed) below).

For `\d`: the inner shape is `{type:"escape",kind:"shorthand",char:"d"}` — the standard PCRE2 metacharacter is just text from the parser's perspective; semantic interpretation (`\d` = digit-class) is downstream.

- `simple_escape_strict` (always active): the catch-all **excluding** the four brace-form leads (`\o{`/`\x{`/`\p{`/`\P{`), the digit guards (`\0`–`\9`, PGEN-RGX-0087), **and** the six PCRE2-unsupported escape letters `\i \F \l \L \u \U` (REGEX-PCRE2-FIDELITY.3.1). Positional ref `char: $21`.
- `simple_escape_relaxed` (`@profiles: ["relaxed"]` — active only under the `relaxed` profile): the historical catch-all without the six-letter exclusion (re-admits `\i \F \l \L \u \U`). Positional ref `char: $15`.

## Profiles — strict default vs `relaxed`

**REGEX-PCRE2-FIDELITY.3.1.** PGEN's regex parser is **PCRE2-faithful by default**. The six escape letters PCRE2 does not support — `\i \F \l \L \u \U` (and the braced `\u{…}` form) — are **rejected in the default (`pcre2`) profile** and re-admitted only under the opt-out **`relaxed`** profile. This rule lives entirely in the grammar (the EBNF is the single source of truth); it was previously enforced by an out-of-band host validator.

Three escape catch-alls carry the strict/relaxed split so the six letters are rejected in **every** context (PCRE2 rejects them in all three — verified against `pcre2test` 10.47):

| Context | Rule | Default (`pcre2`) | `relaxed` |
|---|---|---|---|
| atom (`\u`) | `simple_escape` | REJECT | accept (`{kind:"shorthand",char}`) |
| char class (`[\u]`) | `class_simple_escape` | REJECT | accept |
| class range bound (`[\u-\x7f]`) | `class_range_literal_escape_letter` | REJECT | accept |
| braced (`\u{41}`) | `unicode_escape` | REJECT | accept (`{kind:"unicode",digits}`) |

Selecting the profile:

```bash
# default — strict PCRE2: \u rejected
ast_pipeline grammars/regex.ebnf --generate-stimuli ...
parseability_probe --parse regex pattern.re

# relaxed opt-out: \u / \u{41} / \i / \F / \l / \L / \U accepted
parseability_probe --parse regex pattern.re --profile relaxed
```

> **Where the default is declared (DEFAULT-PROFILE.2, 2026-07-07).** The "unspecified profile =
> strict `pcre2`" resolution is declared **in the grammar** — `grammars/regex.ebnf` carries the
> grammar-level `@default_profile: pcre2` directive, and the generated parser embeds it (a
> `DEFAULT_GRAMMAR_PROFILE` constant; the constructor starts on it, and `set_grammar_profile(None)`
> restores it). Observable behavior is unchanged from the prior engine-side default; it is now
> grammar-declared rather than engine-hard-coded, per the EBNF-single-source-of-truth doctrine.

> **Downstream note (RGX):** the stable `pgen::embedding_api` host surface currently exposes only the strict `regex_default` profile, so embedded consumers get PCRE2-faithful default behavior. A rejected `\u` (etc.) surfaces as diagnostic code `E_PARSE_FAILURE` with a machine-localizable location — match on the **code**, not the message text. Selecting `relaxed` through the embedding API is a planned follow-on.

> **Scope.** This covers exactly the six letters above. PCRE2 also rejects other unrecognized `\<letter>` escapes (e.g. `\I`, `\J`) that the default profile still accepts — the full recognized-escape whitelist is tracked separately (REGEX-PCRE2-FIDELITY.3.11).

## `hex_escape`

```ebnf
hex_escape = "x" hex_digit hex_digit?
           | "x{" brace_ws? hex_digits brace_ws? "}"
```

2 branches. Hex escapes have two PCRE2 forms.

| Form | Shape |
|---|---|
| `\xFF` (1-2 digit) | 3-element Sequence `["x", <hex_digit>, <hex_digit?>]` |
| `\x{...}` (braced, any-length) | 5-element Sequence `["x{", <ws?>, <hex_digits>, <ws?>, "}"]` |

For `\x{1F}`:

```json
[
  "\\",
  [
    "x{",
    [],
    [<hex_digits: ["1", "F"]>],
    [],
    "}"
  ]
]
```

`hex_digits` is itself a Quantified-`+` of `hex_digit` terminals. Consumers concatenate to get the hex string, then parse to int.

## `unicode_escape`

```ebnf
@profiles: ["relaxed"]
unicode_escape = "u{" hex_digits "}"
```

Typed object `{type: "escape", kind: "unicode", digits: <hex_digits>}`.

**Profile-gated (REGEX-PCRE2-FIDELITY.3.1):** `\u{…}` is PCRE2-unsupported in the default (`pcre2`) profile (`pcre2test` 10.47 → error 137; only `PCRE2_ALT_BSUX` gives it meaning), so `unicode_escape` is tagged `@profiles: ["relaxed"]` and is **only active under the `relaxed` profile**. In the default profile it backtracks, and `\u{41}` is rejected (the strict `simple_escape` excludes `\u`, so the whole pattern fails to parse). Under `relaxed`, `\u{1F600}` yields `{type:"escape",kind:"unicode",digits:"1F600"}`.

## `octal_escape`

```ebnf
octal_escape = "o{" brace_ws? octal_digits brace_ws? "}"
             | octal_digit octal_digit? octal_digit?
```

2 branches:

| Form | Shape |
|---|---|
| `\o{...}` braced | 5-element Sequence `["o{", <ws?>, <octal_digits>, <ws?>, "}"]` |
| `\NNN` 1-3 digit | 3-element Sequence `[<octal_digit>, <octal_digit?>, <octal_digit?>]` |

## `octal_digits`

```ebnf
octal_digits = octal_digit+
```

Quantified-`+` of octal_digit terminals. Currently un-annotated; see [Quantifier Subtree](rules-quantifier.md) for `digits`-style typing roadmap.

## `control_escape`

```ebnf
control_escape = "c" any_char
```

2-element Sequence: `["c", <any_char>]`.

For `\cA`: `["c", "A"]` (under the outer `escape`).

## `property_escape`

```ebnf
property_escape = "p{" prop_name "}"
                | "P{" prop_name "}"
                | "p" short_prop_letter
                | "P" short_prop_letter
```

4 branches, distinguishing braced vs short-form and lower vs upper `p`.

| Form | Shape |
|---|---|
| `\p{Lu}` | 3-element Sequence `["p{", <prop_name>, "}"]` |
| `\P{Lu}` | 3-element Sequence `["P{", <prop_name>, "}"]` |
| `\pL` (short) | 2-element Sequence `["p", <short_prop_letter>]` |
| `\PL` (short) | 2-element Sequence `["P", <short_prop_letter>]` |

## `short_prop_letter`

```ebnf
short_prop_letter = 'C' | 'L' | 'M' | 'N' | 'P' | 'S' | 'Z'
                  | 'c' | 'l' | 'm' | 'n' | 'p' | 's' | 'z'
```

14-way Or. `Terminal(<letter>)`.

## `prop_name`

```ebnf
prop_name = prop_name_chars+
```

Quantified-`+` of `prop_name_chars`. Concatenate to recover the name string.

## `prop_name_chars`

```ebnf
prop_name_chars = letter | digit | whitespace | '_' | ':' | '-' | '=' | '&' | '^'
```

9-way Or, each emitting a Terminal of the matched char.

## Walking a `\d+` example

For input `\d+`:

```json
{
  "atom": [
    "\\",
    [
      [
        [
          [
            [
              "d"
            ]
          ]
        ]
      ]
    ]
  ],
  "quantifier": {"type": "quantifier", "min": 1, "max": null, "greediness": []},
  "type": "piece"
}
```

The deeply-nested array on the right of `"\\"` is the un-annotated chain `escape_unit -> simple_escape -> any_char -> letter`, with each layer adding an Alternative wrapping that doesn't unwrap when serialised raw.

A consumer extracting the escape character:

```rust
fn extract_simple_escape_char(escape_atom: &Value) -> Option<&str> {
    // escape_atom = ["\\", <escape_unit shape>]
    let arr = escape_atom.as_array()?;
    let unit = arr.get(1)?;
    // simple_escape branch: descend until we find a string
    let mut cur = unit;
    loop {
        match cur {
            Value::String(s) => return Some(s),
            Value::Array(a) if a.len() == 1 => cur = &a[0],
            _ => return None,
        }
    }
}
```

(That's the un-annotated walk; once `escape_unit` is annotated, the walk collapses to a single field lookup.)

## Walking a `\x{1F}` example

```json
{
  "atom": [
    "\\",
    [
      "x{",
      [],
      [["1"], ["F"]],
      [],
      "}"
    ]
  ],
  "quantifier": [],
  "type": "piece"
}
```

The hex digits are a Quantified-`+` of single-char Terminals (each digit wrapped in an Alternative). To extract the hex string:

```rust
fn extract_hex_digits(seq: &Value) -> String {
    // seq is a Quantified array; each element wraps a single hex digit
    let mut s = String::new();
    if let Some(arr) = seq.as_array() {
        for elem in arr {
            // each elem may be an Alternative-wrapped or bare-string digit
            let mut cur = elem;
            loop {
                match cur {
                    Value::String(c) => { s.push_str(c); break; }
                    Value::Array(a) if a.len() == 1 => cur = &a[0],
                    _ => break,
                }
            }
        }
    }
    s
}
```

## Future direction

Future task #40 escape-subtree slice will likely produce shapes like:

- `simple_escape` → `{type: "escape", kind: "simple", char: <char>}`.
- `hex_escape` → `{type: "escape", kind: "hex", value: <int>}`.
- `unicode_escape` → `{type: "escape", kind: "unicode", codepoint: <int>}`.
- `octal_escape` → `{type: "escape", kind: "octal", value: <int>}`.
- `control_escape` → `{type: "escape", kind: "control", char: <char>}`.
- `property_escape` → `{type: "escape", kind: "property", name: <str>, negated: <bool>}`.

These are not yet implemented; consumers walk the raw shapes today.
