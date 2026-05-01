# Character Class Subtree

PCRE2 character classes (`[abc]`, `[a-z]`, `[^a-z]`, `[[:alpha:]]`, etc.) live under `char_class`. Many sub-rules. None currently annotated — all emit raw envelope shapes.

## `char_class`

```ebnf
char_class = "[" negation? class_initial_close? class_body "]"
```

The 4-element Sequence of `[`, optional `^`, optional initial `]`, body, `]`.

### Shape

```json
[
  "[",
  <negation? — empty array or "^" terminal>,
  <class_initial_close? — empty array or "]" terminal>,
  <class_body — Quantified-* of class_item nodes>,
  "]"
]
```

### Example — input `[abc]`

```json
"atom": [
  "[",
  [],          // not negated
  [],          // no initial close
  [
    [...class_item for "a"...],
    [...class_item for "b"...],
    [...class_item for "c"...]
  ],
  "]"
]
```

## `negation`

```ebnf
@generate: "^" if $1 else ""
@semantic_value: $1 != null
negation = "^"
```

Single-char `^`. Carries semantic annotations but no return annotation. Emits `Terminal("^")`.

## `class_initial_close`

```ebnf
class_initial_close = "]"
```

Allows a literal `]` as the first class char (PCRE2 quirk). Emits `Terminal("]")`.

## `class_body`

```ebnf
@generate: build_class_items_list($1)
@semantic_value: flatten($1)
@validate: all($1, item => is_valid_class_item(item))
class_body = class_item*
```

Quantified-`*` of class items. **Un-annotated** (the `@generate`/`@semantic_value`/`@validate` directives are semantic annotations, not return annotations).

### Shape

A `Quantified` of class_item nodes. JSON: array of class_item shapes.

## `class_item`

```ebnf
class_item = posix_class
           | stray_class_end_quote
           | class_range
           | quoted_class_literal
           | class_literal
           | class_escape
```

6-way Or, **un-annotated**. Each branch's content varies.

### Branches

| Branch | Form | Shape (current) |
|---|---|---|
| 0 | `[:alpha:]`, etc. | nested `posix_class` Sequence |
| 1 | stray `\E` (PCRE2 zero-width marker) | `Terminal("\\E")` |
| 2 | `a-z`, etc. | nested `class_range` Sequence |
| 3 | `\Q...\E` inside class | nested `quoted_class_literal` Sequence |
| 4 | single literal char | `Terminal(<char>)` |
| 5 | `\d`, `\w`, etc. | nested `class_escape` shape |

### Walking a class_item

```rust
fn classify_class_item(item: &Value) -> ClassItemKind {
    match item {
        Value::String(s) if s == "\\E" => ClassItemKind::StrayEndQuote,
        Value::String(s) if s.len() == 1 => ClassItemKind::Literal(s),
        Value::Array(arr) => {
            match arr.first() {
                Some(Value::String(s)) if s == "[:" => ClassItemKind::PosixClass,
                Some(Value::String(s)) if s == "\\Q" => ClassItemKind::QuotedLiteral,
                Some(Value::String(s)) if s == "\\" => ClassItemKind::Escape,
                _ if arr.len() >= 5 => ClassItemKind::Range,  // class_range has class_atom-class_atom shape
                _ => ClassItemKind::Unknown,
            }
        }
        _ => ClassItemKind::Unknown,
    }
}
```

## `posix_class`

```ebnf
@generate: generate_posix_class_check($3)
@semantic_value: {type: "posix", name: $3, negated: $2 != null}
posix_class = "[:" posix_negation? posix_name ":]"
-> {type: "posix_class", name: $3, negated: $2}

posix_negation = "^" -> true
```

**Annotated** as of slice 8 (post-1.1.36) — fixes [PGEN-RGX-0076](changelog-index.md). Pre-fix the rule used `-> $1` which extracted only the literal `"[:"` opener, silently discarding the POSIX class name and negation marker.

### Shape

```json
{"type": "posix_class", "name": <name>, "negated": <true | []>}
```

- `name` is the matched POSIX class name as a string (`"alpha"`, `"digit"`, `"xdigit"`, etc. — one of the 14 names accepted by `posix_name`).
- `negated` is the typed boolean `true` when the source has `^` after `[:` (e.g. `[[:^alpha:]]`), or the empty array `[]` when no `^` was matched. Consumers map `[]` → `false`. Same convention as `quantifier.greediness`. A future coalesce-operator slice will let the rule emit a bare `false` instead of `[]`.

### Examples

| Source | Output |
|---|---|
| `[[:alpha:]]`     | `class_body[0] = {"type":"posix_class","name":"alpha","negated":[]}` |
| `[[:^alpha:]]`    | `class_body[0] = {"type":"posix_class","name":"alpha","negated":true}` |
| `[[:digit:]]`     | `class_body[0] = {"type":"posix_class","name":"digit","negated":[]}` |
| `[[:xdigit:]]`    | `class_body[0] = {"type":"posix_class","name":"xdigit","negated":[]}` |
| `[[:alpha:][:digit:]]` | 2-element class_body — both POSIX classes typed and disambiguated |

### Consumer extraction

```rust
fn extract_posix_class(class_item: &Value) -> Option<(String, bool)> {
    let obj = class_item.as_object()?;
    if obj.get("type")?.as_str()? != "posix_class" {
        return None;
    }
    let name = obj.get("name")?.as_str()?.to_string();
    let negated = obj.get("negated").map(|v| v.as_bool().unwrap_or(false)).unwrap_or(false);
    Some((name, negated))
}
```

## `class_range`

```ebnf
@generate: "ch >= '" + escape_char($1) + "' && ch <= '" + escape_char($5) + "'"
@semantic_value: {type: "range", start: $1, end: $5}
@validate: ord($1) <= ord($5)
class_range = class_atom class_zero_width* "-" class_zero_width* class_atom
```

5-element Sequence: `[<start_atom>, <zero-width-prefix*>, "-", <zero-width-suffix*>, <end_atom>]`. **Un-annotated** at the return-annotation level.

### Shape

```json
[
  <class_atom for start>,
  <Quantified of class_zero_width markers>,
  "-",
  <Quantified of class_zero_width markers>,
  <class_atom for end>
]
```

For `a-z`:

```json
[<class_atom for "a">, [], "-", [], <class_atom for "z">]
```

The empty Quantifieds at indices 1 and 3 are the optional `\E` / empty-`\Q\E` markers that PCRE2 allows around the `-`.

## `class_atom`

```ebnf
@generate: extract_char_value($1)
@semantic_value: {type: "atom", value: $1}
class_atom = quoted_class_range_atom | class_range_escape | class_literal
```

3-way Or. Branches:

| Branch | Form | Shape |
|---|---|---|
| 0 | `\Q<char>\E` (single-char quoted literal) | nested `quoted_class_range_atom` Sequence `["\\Q", <char>, "\\E"]` |
| 1 | `\<escape>` (escaped char) | nested `class_range_escape` Sequence `["\\", <escape-unit>]` |
| 2 | plain literal char | `Terminal(<char>)` |

## `class_literal`

```ebnf
@generate: "ch == '" + escape_char($1) + "'"
@semantic_value: {type: "literal", char: $1}
@optimize: group_literals_for_switch($1)
class_literal = /([A-Za-z0-9 \t\n\r\f\v\[!@#$%\^&*()\-+={}|:;"'<>,.?\/`~_]|[^\x00-\x7F])/
```

Single-character regex. Emits `Terminal(<char>)`.

## `class_escape`

```ebnf
@generate: resolve_escape_pattern($1)
@semantic_value: {type: "escape", pattern: $1}
class_escape = escape
```

Single-element wrapper around `escape`. **Un-annotated**. Emits whatever `escape` produces — see [Escape Subtree](rules-escape.md).

## `quoted_class_literal`

```ebnf
quoted_class_literal = "\\Q" quoted_class_literal_char* "\\E"
```

Inside-class `\Q...\E` block. 3-element Sequence: `["\\Q", <chars-Quantified>, "\\E"]`.

## `quoted_class_range_atom`

```ebnf
quoted_class_range_atom = "\\Q" quoted_class_literal_char "\\E"
```

A single-character `\Q<char>\E` used as a range endpoint. 3-element Sequence: `["\\Q", <char>, "\\E"]`.

## `stray_class_end_quote`

```ebnf
stray_class_end_quote = "\\E"
```

A bare `\E` inside a character class — PCRE2 treats this as a zero-width marker. Emits `Terminal("\\E")`.

## `empty_quoted_class_literal`

```ebnf
empty_quoted_class_literal = "\\Q" "\\E"
```

The empty `\Q\E` zero-width sequence. 2-element Sequence: `["\\Q", "\\E"]`.

## `class_zero_width`

```ebnf
class_zero_width = stray_class_end_quote | empty_quoted_class_literal
```

2-way Or wrapping the two zero-width forms. Emits the matched alternative's shape.

## `class_safe_special`

```ebnf
class_safe_special = /([\[!@#$%\^&*()\-+={}|:;"'<>,.?\/`~_])/
```

Single-character regex matching characters that are safe to use inside a class without escaping. `Terminal(<char>)`.

## `class_range_escape`

```ebnf
class_range_escape = "\\" class_range_escape_unit
```

The 2-element Sequence `["\\", <unit>]` for escapes that can act as range endpoints.

## `class_range_escape_unit`

```ebnf
class_range_escape_unit = hex_escape
                        | unicode_escape
                        | octal_escape
                        | control_escape
                        | class_range_simple_escape
```

5-way Or — restricted set of escapes valid as range endpoints. The matched alternative's shape appears.

## `class_range_simple_escape`, `class_range_any_char_no_orphan_quote_end`, `class_range_literal_escape_letter`

Inner sub-rules controlling which characters can act as escaped class-range endpoints. All emit Terminal/Sequence shapes per their match. Consumers rarely need to walk these directly — the outer `class_range_escape` is sufficient for identifying the escape-as-endpoint shape.

## `posix_negation`, `posix_name`, `letter_no_upper_e`

Inner POSIX sub-rules. Emit raw Terminal/Sequence shapes.

## Walking a `[abc-z]` example

For `[abc-z]`:

```json
"atom": [
  "[",
  [],
  [],
  [
    [<class_item: "a">],
    [<class_item: "b">],
    [<class_range: c-z>]
  ],
  "]"
]
```

A consumer extracting the items:

```rust
fn class_items(class_atom: &Value) -> Vec<&Value> {
    let arr = class_atom.as_array().unwrap();
    let body = arr.get(3).and_then(|b| b.as_array()).unwrap();
    body.iter().collect()
}
```

For each item, classify by structural signature (per the Atom Subtree dispatch table — see [Atom Subtree](rules-atom.md)).
