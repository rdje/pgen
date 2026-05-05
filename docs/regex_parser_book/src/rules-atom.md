# Atom Subtree

`atom` is the most-frequently-walked rule in the regex parser — every regex element that doesn't have a quantifier inline (and even those that do, before the quantifier is attached) ends up here.

## `atom`

```ebnf
atom = literal
     | whitespace_literal
     | dot
     | anchor
     | backreference
     | quoted_literal
     | escape
     | posix_word_boundary_alias
     | char_class
     | subroutine_call
     | inline_modifiers
     | scoped_inline_modifiers
     | branch_reset_group
     | callout
     | conditional
     | lookaround
     | atomic_group
     | scan_substring_group
     | script_run_group
     | directive_verb
     | extended_class
     | code_block
     | comment_group
     | python_named_backreference
     | group
```

25-way Or rule. Currently **un-annotated**. Each branch's content varies by alternative.

### Current shape

The matched alternative's content appears as the atom's content directly — wrapped in an `Alternative` envelope at the ParseNode level, but `to_json_value()` unwraps it transparently.

So when walking the JSON output, the atom field of a piece simply IS the matched alternative's shape. There's no `{type: "atom", kind: "literal", ...}` wrapper.

### Future shape

When `atom` is annotated (planned in task #40), a target shape might be:

```json
{ "type": "atom", "kind": "<alternative-name>", "value": <alternative-content> }
```

— a discriminator on which alternative matched. Until then, consumers identify the atom kind by walking the `rule_name` of the ParseNode tree (when going through the legacy path) or by structurally pattern-matching the JSON shape (when going through the typed path).

## `literal`

```ebnf
literal = literal_char
```

Single-element wrapper around `literal_char`. Currently **un-annotated**. With the slice-36 codegen tightening, single-element-rule-body un-annotated rules still get the implicit `-> $1` default — so `literal` transparently returns whatever `literal_char` produced.

### Shape

`Terminal(<one-char>)` — visible as a bare string in JSON.

### Examples

For atom `a`: the `atom` content is `"a"` (JSON string).

## `literal_char`

```ebnf
literal_char = /([A-Za-z0-9!"#%&',\-\/:;<=>@\]{}_`~]|[^\x00-\x7F])/
```

A single-character regex literal. Captures one ASCII non-special char OR any non-ASCII byte.

### Shape

`Terminal(<one-char>)`.

### What's NOT a literal_char

These chars are excluded because they have regex-special meaning and would be matched by other atom alternatives or by escape rules: `\`, `(`, `)`, `[`, `*`, `+`, `?`, `.`, `^`, `$`, `|`, `{`, space, control characters.

## `whitespace_literal`

```ebnf
whitespace_literal = whitespace
```

A wrapper for whitespace as an atom. Mirrors `literal`'s pattern but matches `whitespace` (regex `/[ \t\n\r\f\v]/`).

### Shape

`Terminal(<one-whitespace-char>)`.

## `dot`

```ebnf
dot = "."
```

The `.` regex metacharacter (matches any character except newline by default).

### Shape

`Terminal(".")`. Visible as the string `"."`.

### Example

For atom `.`: the `atom` content is `"."`.

## `anchor`

```ebnf
anchor = "^"   -> {type: "anchor", kind: "start_of_line"}
       | "$"   -> {type: "anchor", kind: "end_of_line"}
       | "\\A" -> {type: "anchor", kind: "start_of_input"}
       | "\\Z" -> {type: "anchor", kind: "end_of_input_or_before_last_newline"}
       | "\\z" -> {type: "anchor", kind: "end_of_input"}
       | "\\b" -> {type: "anchor", kind: "word_boundary"}
       | "\\B" -> {type: "anchor", kind: "non_word_boundary"}
       | "\\G" -> {type: "anchor", kind: "match_start"}
       | "\\K" -> {type: "anchor", kind: "keep_out"}
```

PCRE2 anchor metacharacters. 9-way Or, **annotated** as of slice 7 (post-1.1.35).

### Shape

```json
{"type": "anchor", "kind": <kind-name>}
```

`<kind-name>` is one of: `start_of_line`, `end_of_line`, `start_of_input`, `end_of_input_or_before_last_newline`, `end_of_input`, `word_boundary`, `non_word_boundary`, `match_start`, `keep_out`.

### Example

For atom `\b`: `{"type": "anchor", "kind": "word_boundary"}`.

See [Examples: Anchors and Boundaries](examples-anchors.md) for the full set with consumer-side dispatch recipe.

## `backreference`

```ebnf
backreference = "\\" backreference_digits                              -> {type: "backreference", kind: "numeric",                index: $2}
              | "\\k" name_ref                                          -> {type: "backreference", kind: "named",                  ref:   $2}
              | "\\k" braced_name_ref                                   -> {type: "backreference", kind: "named_braced",           ref:   $2}
              | "\\g" "<" name ">"                                      -> {type: "backreference", kind: "subroutine_named",       ref:   $3}
              | "\\g" "<" signed_digits ">"                             -> {type: "backreference", kind: "subroutine_numeric",     ref:   $3}
              | "\\g" "'" name "'"                                      -> {type: "backreference", kind: "subroutine_named",       ref:   $3}
              | "\\g" "'" signed_digits "'"                             -> {type: "backreference", kind: "subroutine_numeric",     ref:   $3}
              | "\\g" "{" brace_ws? name brace_ws? "}"                  -> {type: "backreference", kind: "named_braced",           ref:   $4}
              | "\\g" "{" brace_ws? signed_digits brace_ws? "}"         -> {type: "backreference", kind: "numeric_backreference",  ref:   $4}
              | "\\g" signed_digits                                     -> {type: "backreference", kind: "numeric_backreference",  ref:   $2}
```

10-way Or, **annotated** as of slice 10 (initial 4-way) + PGEN-RGX-0081 fix (post-1.1.75 expanded to 10 branches with bracket-form discrimination).

### Shape

```json
{"type": "backreference", "kind": <form>, ...form-specific-fields...}
```

### Branches

| Branch | Form | Example | Output |
|---|---|---|---|
| 0 | `\1`, `\23` | `\1` | `{"type":"backreference","kind":"numeric","index":1}` (typed integer) |
| 1 | `\k<name>`, `\k'name'` | `\k<foo>` | `{"type":"backreference","kind":"named","ref":<name string>}` |
| 2 | `\k{name}` | `\k{foo}` | `{"type":"backreference","kind":"named_braced","ref":<name string>}` |
| 3 | `\g<NAME>` | `\g<foo>` | `{"type":"backreference","kind":"subroutine_named","ref":"foo"}` (PCRE2 subroutine call, named) |
| 4 | `\g<N>` | `\g<1>` | `{"type":"backreference","kind":"subroutine_numeric","ref":{"sign":[],"value":1}}` (PCRE2 subroutine call, numeric) |
| 5 | `\g'NAME'` | `\g'foo'` | Same as branch 3 (apostrophe variant) |
| 6 | `\g'N'` | `\g'1'` | Same as branch 4 (apostrophe variant) |
| 7 | `\g{NAME}` | `\g{foo}` | `{"type":"backreference","kind":"named_braced","ref":"foo"}` (back-reference per PCRE2; same kind as `\k{NAME}`) |
| 8 | `\g{N}` | `\g{1}` | `{"type":"backreference","kind":"numeric_backreference","ref":{"sign":[],"value":1}}` (numeric back-reference) |
| 9 | `\gN`, `\g+1`, `\g-2` | `\g42` | `{"type":"backreference","kind":"numeric_backreference","ref":{"sign":[],"value":42}}` (bare numeric back-reference) |

**PCRE2 semantics** (per `pcre2pattern(3)` § "Subroutine references and recursive patterns"):

- Angle/apostrophe forms (`\g<...>`, `\g'...'`) → **subroutine call** (re-execute the group)
- Brace/bare forms (`\g{...}`, `\gN`) → **back-reference** (match what the group captured)

The `kind` discriminator preserves this distinction — consumers can dispatch on `kind` to choose the right semantic. Pre-PGEN-RGX-0081 (parser releases 1.1.43 through 1.1.74) all `\g`-prefixed forms collapsed to `kind:"subroutine"`, losing this distinction.

`backreference_digits` is itself annotated (`@transform: str::parse::<usize>`), so branch 0's `index` is a typed integer directly. Named-ref branches (1, 2, 3, 5, 7) carry a clean `name` string. Numeric-ref branches (4, 6, 8, 9) carry a `{sign, value}` object from the typed `signed_digits` rule.

### Consumer extraction

```rust
fn classify_backreference(atom: &Value) -> Option<Backreference> {
    let obj = atom.as_object()?;
    if obj.get("type")?.as_str()? != "backreference" {
        return None;
    }
    match obj.get("kind")?.as_str()? {
        "numeric" => {
            // \1, \2, ...
            let index = obj.get("index")?.as_u64()?;
            Some(Backreference::Numeric(index))
        }
        "named" | "named_braced" => {
            // \k<NAME>, \k'NAME', \k{NAME}, \g{NAME}
            let name = obj.get("ref")?.as_str()?.to_string();
            Some(Backreference::Named(name))
        }
        "subroutine_named" => {
            // \g<NAME>, \g'NAME' — subroutine call (re-execute group by name)
            let name = obj.get("ref")?.as_str()?.to_string();
            Some(Backreference::SubroutineCallNamed(name))
        }
        "subroutine_numeric" => {
            // \g<N>, \g'N' — subroutine call (re-execute group N)
            let r = obj.get("ref")?;
            let sign = r.get("sign")?;
            let value = r.get("value")?.as_u64()?;
            Some(Backreference::SubroutineCallNumeric { sign: sign_str(sign), value })
        }
        "numeric_backreference" => {
            // \g{N}, \gN — numeric back-reference
            let r = obj.get("ref")?;
            let sign = r.get("sign")?;
            let value = r.get("value")?.as_u64()?;
            Some(Backreference::NumericBackref { sign: sign_str(sign), value })
        }
        _ => None,
    }
}
```

## `quoted_literal`

```ebnf
quoted_literal = "\\Q" quoted_literal_char* "\\E"
```

The full PCRE2 `\Q...\E` quoted-literal block as an atom. **Un-annotated**.

### Shape

3-element Sequence: `["\\Q", <Quantified of chars>, "\\E"]`.

### When this fires vs `piece_quoted_run_quantified`

- `piece_quoted_run_quantified` fires when `\Q...\E` is followed by a quantifier AND has at least 2 chars in the run.
- `quoted_literal` fires when `\Q...\E` is NOT followed by a quantifier, OR has 0 or 1 chars.

For `\Qab\E` (no trailing quantifier, 2 chars):

```json
{
  "atom": ["\\Q", ["a", "b"], "\\E"],
  "quantifier": [],
  "type": "piece"
}
```

For `\Qa\E{3}` (single-char run with quantifier — degenerate; falls through to atom path):

```json
{
  "atom": ["\\Q", ["a"], "\\E"],
  "quantifier": [<{3}>],
  "type": "piece"
}
```

For `\Q\E{2}` (empty run with quantifier — also degenerate; atom path):

```json
{
  "atom": ["\\Q", [], "\\E"],
  "quantifier": [<{2}>],
  "type": "piece"
}
```

These degenerate cases produce ONE piece (the whole `\Q...\E` as atom + the trailing quantifier) which is semantically correct because there's no quantifier-attachment ambiguity.

The non-degenerate case (multi-char run + quantifier, e.g. `\Qab*\E{2,}`) goes through `piece_quoted_run_quantified` instead and produces the multi-piece array — see [piece](rules-piece.md).

## `escape`

```ebnf
escape = "\\" escape_unit
```

The PCRE2 escape sequence wrapper. **Un-annotated**.

### Shape

2-element Sequence: `["\\", <escape_unit-shape>]`.

The `escape_unit` rule branches into the various escape forms; see [Escape Subtree](rules-escape.md).

### Example

For `\d`:

```json
{
  "atom": [
    "\\",
    [
      [
        [
          [
            "d"
          ]
        ]
      ]
    ]
  ],
  ...
}
```

The deeply-nested array structure on the right of `"\\"` is the un-annotated `escape_unit -> simple_escape -> any_char -> letter` chain. Each layer of un-annotated wrapping adds an array level. Once `escape_unit` is annotated, the nesting will collapse.

## `posix_word_boundary_alias`

```ebnf
posix_word_boundary_alias = "[[:<:]]" -> {type: "anchor", kind: "posix_word_start"}
                          | "[[:>:]]" -> {type: "anchor", kind: "posix_word_end"}
```

PCRE2's BSD-style word-boundary aliases. 2-way Or, **annotated** as of slice 9 (post-1.1.37). Joins the same typed anchor family as the `anchor` rule.

### Shape

```json
{"type": "anchor", "kind": "posix_word_start"}    // for [[:<:]]
{"type": "anchor", "kind": "posix_word_end"}      // for [[:>:]]
```

Treated as atomic units at the parser level — NOT character classes despite the syntactic resemblance. Consumers dispatching on `obj.type == "anchor"` handle these uniformly with the regular anchor variants. See [Examples: Anchors and Boundaries](examples-anchors.md).

## `char_class`

```ebnf
char_class = "[" negation? class_initial_close? class_body "]"
```

The full character class atom. **Un-annotated**. See [Character Class Subtree](rules-char-class.md) for the per-rule walk.

### Shape

4-element Sequence: `["[", <negation?>, <class_initial_close?>, <class_body>, "]"]`.

## `group`

```ebnf
group = capturing_group | noncapturing_group | named_group | python_named_group
```

Standard group forms. **Un-annotated**. See [Group Family](rules-groups.md).

## `subroutine_call`, `inline_modifiers`, `scoped_inline_modifiers`, `branch_reset_group`, `callout`, `conditional`, `lookaround`, `atomic_group`, `scan_substring_group`, `script_run_group`, `directive_verb`, `extended_class`, `code_block`, `comment_group`, `python_named_backreference`

All **un-annotated** atom alternatives. Each emits the raw matched shape per its sub-rule structure. Detailed per-rule shapes are documented in:

- [Group Family](rules-groups.md) — `subroutine_call`, `branch_reset_group`, `atomic_group`, `scan_substring_group`, `script_run_group`, `lookaround`, `conditional`, `python_named_backreference`.
- [Modifier Subtree](rules-modifiers.md) — `inline_modifiers`, `scoped_inline_modifiers`.
- [Anchors, Backreferences, and Misc](rules-misc.md) — `callout`, `directive_verb`, `extended_class`, `code_block`, `comment_group`.

## Identification table — what kind of atom is this?

When walking a piece's `atom` field, here's the structural signature for each kind (today's un-annotated state):

| Atom kind | Signature in JSON | Notes |
|---|---|---|
| `literal` | bare string, single ASCII non-special char or non-ASCII | `"a"`, `"x"` |
| `whitespace_literal` | bare string, single whitespace char | `" "`, `"\t"` |
| `dot` | bare string `"."` | exactly the `.` char |
| `anchor` | typed object `{"type":"anchor","kind":"<name>"}` | annotated in slice 7; dispatch on `obj.type == "anchor"` then read `obj.kind` |
| `backreference` | typed object `{"type":"backreference","kind":<form>,...}` | annotated in slice 10; expanded by PGEN-RGX-0081 fix (post-1.1.75). Dispatch on `obj.kind` (`numeric` / `named` / `named_braced` / `subroutine_named` / `subroutine_numeric` / `numeric_backreference`) |
| `quoted_literal` | 3-element array `["\\Q", <chars>, "\\E"]` | full quoted literal |
| `escape` | 2-element array starting with `"\\"` and not matching backreference form | `["\\", <escape_unit>]` |
| `posix_word_boundary_alias` | typed object `{"type":"anchor","kind":"posix_word_start"|"posix_word_end"}` | annotated in slice 9; same dispatch shape as `anchor` |
| `char_class` | 4-element array starting with `"["`, ending with `"]"` | square-bracket class |
| `group` | array starting with `"("` | various `(...)` forms |
| `lookaround` | array starting with `"(?="`, `"(?!"`, `"(?<="`, `"(?<!"` | etc. |
| `atomic_group` | array starting with `"(?>"` or `"(*atomic:"` | atomic group |
| `inline_modifiers` | array starting with `"(?"` followed by modifier_spec | `(?i)` |
| `scoped_inline_modifiers` | array starting with `"(?"` followed by modifier_spec then `:` then pattern | `(?i:foo)` |
| `branch_reset_group` | array starting with `"(?\|"` | `(?|...)` |
| `callout` | array starting with `"(?C"` | `(?C12)` |
| `conditional` | array starting with `"(?("` | `(?(1)yes\|no)` |
| `subroutine_call` | array starting with `"(?"` followed by subroutine_target | `(?P>name)`, `(?R)` |
| `code_block` | array starting with `"(?{"` | `(?{lua: ...})` |
| `comment_group` | array starting with `"(?#"` | `(?#comment)` |
| `python_named_backreference` | array starting with `"(?P="` | `(?P=name)` |
| `directive_verb` | array starting with `"(*"` (and not lookahead/lookbehind/atomic) | `(*UTF8)`, `(*ACCEPT)` |
| `extended_class` | array starting with `"(?["` | `(?[ ... ])` |
| `scan_substring_group` | array starting with `"(*scs:"` or `"(*scan_substring:"` | scan-substring |
| `script_run_group` | array starting with `"(*sr:"` or `"(*script_run:"` etc. | script run |

A robust consumer-side discriminator function:

```rust
fn classify_atom(atom: &Value) -> AtomKind {
    match atom {
        Value::String(s) if s == "." => AtomKind::Dot,
        Value::String(s) if s == "[[:<:]]" || s == "[[:>:]]" => AtomKind::PosixWordBoundary,
        Value::String(s) if matches!(s.as_str(), "^" | "$" | "\\A" | "\\Z" | "\\z" | "\\b" | "\\B" | "\\G" | "\\K") => AtomKind::Anchor,
        Value::String(s) if s.len() == 1 || (!s.starts_with('\\') && !s.starts_with('(') && !s.starts_with('[')) => AtomKind::Literal,
        Value::Array(arr) => {
            // Inspect arr[0] for the discriminating prefix
            match arr.first().and_then(|v| v.as_str()) {
                Some("\\Q") => AtomKind::QuotedLiteral,
                Some("\\") => /* backreference or escape — distinguish by arr[1] */,
                Some("[") => AtomKind::CharClass,
                Some(s) if s.starts_with("(") => /* group family — distinguish by full prefix */,
                Some(s) if s.starts_with("(*") => /* directive_verb / atomic_group / scan_substring */,
                _ => AtomKind::Unknown,
            }
        }
        _ => AtomKind::Unknown,
    }
}
```

Once `atom` is annotated, the discriminator becomes a clean `kind` field lookup. Until then, the structural-prefix dispatch above is the way.
