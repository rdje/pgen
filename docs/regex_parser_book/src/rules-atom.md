# Atom Subtree

`atom` is the most-frequently-walked rule in the regex parser — every regex element that doesn't have a quantifier inline (and even those that do, before the quantifier is attached) ends up here.

## `atom`

```ebnf
atom = literal
     | whitespace_literal
     | dot
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

24-way Or rule. The `atom` rule itself carries no annotation — the matched
alternative's shape passes through directly — but **every alternative except the
plain-text ones is itself typed**, so in practice a piece's `atom` field is always
one of exactly two things:

- a **bare string** — a literal char, the dot `"."`, or a whitespace literal; or
- a **typed object** carrying its own discriminator: `{"type": "atom", "kind": ...}`
  for the structural constructs, `{"type": "escape", "kind": ...}` for the escape
  subtree, or `{"type": "backreference", "kind": ...}` for the backreference family.

There is no extra `atom`-level wrapper: `to_json_value()` unwraps the internal
`Alternative` envelope transparently, so the atom field simply IS the matched
alternative's typed shape. Consumer dispatch is therefore: string ⇒ literal text;
object ⇒ read `type` + `kind` (the [identification table](#identification-table--what-kind-of-atom-is-this)
below lists every shape).

Since release `1.1.82` (`REGEX-PCRE2-FIDELITY.3.13`), `anchor` is **not** an `atom` alternative — anchors are non-quantifiable in PCRE2, so they form their own `piece` branch (see [piece](rules-piece.md)). The anchor's typed `{type:"anchor", kind}` shape inside a piece is unchanged.

## `literal`

```ebnf
literal = literal_char | literal_open_brace
```

Two disjoint branches (release `1.1.85`): `literal_char` covers every literal character except `{`, and `literal_open_brace` covers the guarded literal `{`. **Un-annotated** — the matched branch's shape passes straight through.

### Shape

`Terminal(<one-char>)` — visible as a bare string in JSON.

### Examples

For atom `a`: the `atom` content is `"a"` (JSON string). For the `{` in `a{}`: the `atom` content is `"{"` — byte-identical to pre-`1.1.85` releases.

## `literal_char`

```ebnf
literal_char = letter | digit | '!' | '"' | '#' | '%' | '&' | "'" | ',' | '-' | '/' | ':' | ';' | '<' | '=' | '>' | '@' | ']' | '}' | '_' | '`' | '~' | unicode_char
```

A single-character regex literal: one ASCII non-special char OR any non-ASCII char (`unicode_char`). Native alternation since the self-hosting conversion (no Rust-regex body).

### Shape

`Terminal(<one-char>)`.

### What's NOT a literal_char

Chars with regex-special meaning, matched by other atom alternatives or escape rules: `\`, `(`, `)`, `[`, `*`, `+`, `?`, `.`, `^`, `$`, `|`, space, control characters. Since release `1.1.85`, `{` is also not a `literal_char` — it lives in the guarded `literal_open_brace` branch below. An unmatched `}` IS a `literal_char` (PCRE2 treats it as literal in default mode).

## `literal_open_brace`

```ebnf
literal_open_brace = !( "{" brace_ws? ( digit+ brace_ws? ( "," brace_ws? ( digit+ brace_ws? )? )? | "," brace_ws? digit+ brace_ws? ) "}" ) "{" -> $2
```

**Annotated (`-> $2`).** Release `1.1.85` (`REGEX-PCRE2-FIDELITY.3.18`, ledger `REGEX-0093`): PCRE2's brace tokenization model — a brace whose text is syntactically-valid quantifier shape (digits with spaces/tabs anywhere inside, in the four forms `{n}` `{n,}` `{n,m}` `{,m}`) is ALWAYS a quantifier, never a literal. The inline negative lookahead encodes exactly that syntax (digit runs value-UNBOUNDED, whitespace = `brace_ws` = space + tab), so:

- a valid-syntax brace at a **non-repeatable position** (`{2,5}` at pattern start, `x|{2,5}`, `a{2}{3}`) can neither quantify nor fall back to literal ⇒ the pattern REJECTS (PCRE2 err 109);
- a valid-syntax brace with an **out-of-range value** (`a{65536}`, `a{4294967296}`) fails the quantifier ([`quant_bound_number`](rules-quantifier.md)) and is blocked here too ⇒ REJECTS (err 105);
- a **non-quantifier-shaped** brace (`{}`, `{,}`, `{ }`, `{a}`, `a{1,2,3}b`, a `\n`/`\f`/`\r`/`\v` inside, unterminated `a{65536`) matches this branch and stays a literal `{`, exactly as PCRE2 compiles it.

The `-> $2` annotation drops the zero-width guard slot and emits the bare `"{"` terminal — byte-identical to the shape the former `literal_char` arm produced.

### Shape

`Terminal("{")`.

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
              | "\\k" "'" name "'"                                      -> {type: "backreference", kind: "named",                  ref:   $3}
              | "\\k" braced_name_ref                                   -> {type: "backreference", kind: "named_braced",           ref:   $2}
              | "\\g" "<" name ">"                                      -> {type: "backreference", kind: "subroutine_named",       ref:   $3}
              | "\\g" "<" signed_digits ">"                             -> {type: "backreference", kind: "subroutine_numeric",     ref:   $3}
              | "\\g" "'" name "'"                                      -> {type: "backreference", kind: "subroutine_named",       ref:   $3}
              | "\\g" "'" signed_digits "'"                             -> {type: "backreference", kind: "subroutine_numeric",     ref:   $3}
              | "\\g" "{" brace_ws? name brace_ws? "}"                  -> {type: "backreference", kind: "named_braced",           ref:   $4}
              | "\\g" "{" brace_ws? signed_digits brace_ws? "}"         -> {type: "backreference", kind: "numeric_backreference",  ref:   $4}
              | "\\g" signed_digits                                     -> {type: "backreference", kind: "numeric_backreference",  ref:   $2}
```

11-way Or, **annotated** as of slice 10 (initial 4-way) + PGEN-RGX-0081 fix (post-1.1.75 expanded to 10 branches with bracket-form discrimination) + **REGEX-PCRE2-FIDELITY.4.2** (release 1.1.93: the explicit `\k'name'` quote branch — same `kind:"named"` as the angle form; previously `\k'name'` silently decomposed to a `simple_escape` shorthand + literals, a WRONG-shape accept — and `\k` is now always a named-backref introducer via the `!"k"` guard on `simple_escape`). Also as of `.4.2` the `name` rule is bounded `{0,127}` ⇒ every capture/backref NAME is ≤ 128 code-points (PCRE2 err 148), so a ≥ 129-code-unit name hard-REJECTs at the grammar layer.

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
quoted_literal = "\\Q" quoted_literal_char+ "\\E"  -> {type: "atom", kind: "quoted_literal", body: $2}
```

A NON-empty TERMINATED PCRE2 `\Q...\E` quoted-literal block, as an atom (`char+` since release `1.1.89`, `REGEX-PCRE2-FIDELITY.3.23` — the empty case `\Q\E` moved to `zero_width`). Everything between `\Q` and `\E` is literal, including structural metacharacters (`\Q(?:\E`, `\Q**\E`, `\Q)\E` all accept). `body` is the array of matched chars (each `quoted_literal_char` is a single-char string; an escaped char like `\d` is the 2-element `["\\","d"]`).

### Shape — annotated

```json
{ "type": "atom", "kind": "quoted_literal", "body": [<chars>] }
```

### When this fires vs the sibling `\Q` rules

- `piece_quoted_run_quantified` (see [piece](rules-piece.md)) fires when a TERMINATED `\Q...\E` (≥1 char) is followed by a quantifier — the quantifier binds the LAST char (`\Qab\E*` = `a`, `b*`).
- `quoted_literal` fires for a non-empty terminated `\Q...\E` NOT followed by a quantifier.
- `empty_quoted_literal` (the `zero_width` in [piece](rules-piece.md#zero_width--the-transparent-stray-e-and-empty-qe)) handles `\Q\E`: bare `\Q\E` accepts (unchanged shape), `\Q\E*` REJECTS (err 109 — an empty quote is unrepeatable).
- `unterminated_quoted_literal` (below) handles a `\Q…` with no `\E`.

For `\Qab\E` (terminated, no trailing quantifier):

```json
{
  "atom": { "type": "atom", "kind": "quoted_literal", "body": ["a", "b"] },
  "quantifier": [],
  "type": "piece"
}
```

## `unterminated_quoted_literal`

```ebnf
unterminated_quoted_literal = "\\Q" quoted_literal_char*  -> {type: "atom", kind: "quoted_literal", body: $2}
```

Release `1.1.89` (`REGEX-PCRE2-FIDELITY.3.23`). A `\Q…` with NO closing `\E` quotes everything to END-OF-PATTERN as literal — so `\Q)`, `\Q(`, `\Q[`, `\Q^`, `\Q(?:`, `\Q**` are ACCEPTED as one literal run (before `1.1.89` these REJECTED, because `\Q` was mis-parsed as a shorthand escape and the tail as live regex). The empty tail (`char*` = 0) is bare `\Q` at end-of-pattern.

It is consumed ONLY by the standalone [piece](rules-piece.md#piece) branch `unterminated_quoted_literal -> {type:"piece", atom:$1, quantifier:[]}` — never as a quantifiable `atom`. That is deliberate: a non-atom greedy-to-end piece can never be the ABSORPTION branch's atom, which keeps the stimuli generator from emitting `\Q\E<quantifier>` (a duality break). Its `body` uses the same `{type:"atom", kind:"quoted_literal", body}` carrier as `quoted_literal`.

For `\Q)` (unterminated, `)` literal):

```json
{
  "atom": { "type": "atom", "kind": "quoted_literal", "body": [")"] },
  "quantifier": [],
  "type": "piece"
}
```

## `escape`

```ebnf
escape = "\\" escape_unit
```

The PCRE2 escape sequence wrapper. **Transparent** (`-> $2`, slice 17): the wrapper
contributes nothing to the output — the typed `escape_unit` object IS the atom.

### Shape

The typed escape object emitted by the matched `escape_unit` branch:

```json
{ "type": "escape", "kind": <form>, ... }
```

with `kind` ∈ `shorthand` / `control` / `hex` / `octal` / `unicode` / `property`
(payload fields per kind — see [Escape Subtree](rules-escape.md)).

### Example

For `\d` (exact probe output):

```json
{
  "atom": { "type": "escape", "kind": "shorthand", "char": "d" },
  "quantifier": [],
  "type": "piece"
}
```

The same single-object shape appears in every escape position — as an atom, inside a
character class body, and inside `class_range` endpoints. (In the pre-slice-14 era
this emitted the 2-element `["\\", <nested chain>]` sequence; that shape no longer
exists in any released artifact.)

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

The full character class atom. **Typed** (slice 26, post-1.1.56):

```json
{ "type": "atom", "kind": "char_class", "negated": <bool or []>,
  "initial_close": <bool or []>, "body": [<class items>] }
```

See [Character Class Subtree](rules-char-class.md) and
[Examples: Character Classes](examples-char-class.md) for the item shapes.

## `group`

```ebnf
group = capturing_group | noncapturing_group | named_group | python_named_group
```

Standard group forms. The `group` Or itself carries no annotation — each branch emits
its own typed `{type:"atom", kind:...}` carrier. See [Group Family](rules-groups.md).

## `subroutine_call`, `inline_modifiers`, `scoped_inline_modifiers`, `branch_reset_group`, `callout`, `conditional`, `lookaround`, `atomic_group`, `scan_substring_group`, `script_run_group`, `directive_verb`, `extended_class`, `code_block`, `comment_group`, `python_named_backreference`

All **typed** atom alternatives — each emits its own `{type, kind, ...}` object (the
identification table below lists every shape). Detailed per-rule documentation:

- [Group Family](rules-groups.md) — `subroutine_call`, `branch_reset_group`, `atomic_group`, `scan_substring_group`, `script_run_group`, `lookaround`, `conditional`, `python_named_backreference`.
- [Modifier Subtree](rules-modifiers.md) — `inline_modifiers`, `scoped_inline_modifiers`.
- [Anchors, Backreferences, and Misc](rules-misc.md) — `callout`, `directive_verb`, `extended_class`, `code_block`, `comment_group`.

## Identification table — what kind of atom is this?

When walking a piece's `atom` field, dispatch is: **bare string ⇒ literal text;
object ⇒ read `type` then `kind`.** The complete live-verified signature table:

| Construct | Signature in JSON | Example source |
|---|---|---|
| `literal` | bare string (single char; `"a"`, `"x"`, non-ASCII) | `a` |
| `whitespace_literal` | bare string, single whitespace char | `" "`, `"\t"` |
| `dot` | bare string `"."` | `.` |
| `anchor` (piece-level since `1.1.82`) | `{"type":"anchor","kind":"<name>"}` (`start_of_line`, `end_of_line`, `word_boundary`, `non_word_boundary`, …) | `^`, `$`, `\b`, `\B` |
| `posix_word_boundary_alias` | `{"type":"anchor","kind":"posix_word_start"\|"posix_word_end"}` | `[[:<:]]`, `[[:>:]]` |
| `escape` | `{"type":"escape","kind":"shorthand"\|"control"\|"hex"\|"octal"\|"unicode"\|"property"\|"single_byte", ...}` | `\d`, `\cA`, `\x41`, `\o{101}`, `\p{Lu}`, `\C` |
| `backreference` | `{"type":"backreference","kind":"numeric"\|"named"\|"named_braced"\|"numeric_backreference"\|"subroutine_named"\|"python_named", "ref"\|"index": ...}` | `\1`, `\k<n>`, `\k{n}`, `\g{2}`, `\g<n>`, `(?P=n)` |
| `quoted_literal` | `{"type":"atom","kind":"quoted_literal","body":[<chars>]}` | `\Qab\E`, unterminated `\Q…`, empty `\Q\E` |
| `char_class` | `{"type":"atom","kind":"char_class","negated":…,"initial_close":…,"body":[…]}` | `[a-z]`, `[^\d]` |
| `capturing_group` | `{"type":"atom","kind":"capturing_group","body":<pattern>}` | `(abc)` |
| `noncapturing_group` | `{"type":"atom","kind":"noncapturing_group","body":<pattern>}` | `(?:abc)` |
| `named_group` | `{"type":"atom","kind":"named_group","name":<str>,"body":<pattern>}` | `(?<n>a)`, `(?'n'a)` |
| `python_named_group` | `{"type":"atom","kind":"python_named_group","name":<str>,"body":<pattern>}` | `(?P<n>a)` |
| `atomic_group` | `{"type":"atom","kind":"atomic_group","body":<pattern>}` | `(?>ab)`, `(*atomic:ab)` |
| `branch_reset_group` | `{"type":"atom","kind":"branch_reset_group","body":<pattern>}` | `(?\|a\|b)` |
| `lookaround` | `{"type":"atom","kind":"lookahead"\|"lookbehind"\|"non_atomic_lookahead"\|"non_atomic_lookbehind","positive":<bool>,"body":<pattern>}` or `{"kind":"alpha_lookaround","name":<str>,"body":<pattern>}` | `(?=x)`, `(?<!x)`, `(?*x)`, `(*nla:x)` |
| `inline_modifiers` | `{"type":"atom","kind":"inline_modifiers","spec":<spec>}` | `(?i)`, `(?^x)` |
| `scoped_inline_modifiers` | `{"type":"atom","kind":"scoped_inline_modifiers","spec":<spec>,"body":<pattern>}` | `(?i-mx:foo)` |
| `callout` | `{"type":"atom","kind":"callout","arg":<[] \| int \| {payload,quote}>}` | `(?C12)`, `(?C"str")` |
| `conditional` | `{"type":"atom","kind":"conditional","condition":…,"yes_branch":…,"no_branch":…}` | `(?(1)y\|n)` |
| `subroutine_call` | `{"type":"atom","kind":"subroutine_call","target":{…}}` | `(?R)`, `(?&n)`, `(?1)` |
| `code_block` | `{"type":"atom","kind":"code_block","lang":<str\|null>,"content":[<chars>]}` | `(?{lua: …})` |
| `comment_group` | `{"type":"atom","kind":"comment","text":<str>}` | `(?#comment)` |
| `directive_verb` | `{"type":"atom","kind":"directive_verb","body":{…}}` | `(*ACCEPT)`, `(*UTF8)` |
| `extended_class` | `{"type":"atom","kind":"extended_class","body":[…]}` | `(?[ … ])` |
| `scan_substring_group` | `{"type":"atom","kind":"scan_substring_group","name":<str>,"captures":[…],"body":<pattern>}` | `(*scs:(1)ab)` |
| `script_run_group` | `{"type":"atom","kind":"script_run_group","name":<str>,"body":<pattern>}` | `(*sr:ab)` |

A robust consumer-side discriminator is now a `kind` lookup:

```rust
fn classify_atom(atom: &Value) -> AtomKind {
    match atom {
        // Bare string = literal text (a literal char, the dot, or whitespace).
        Value::String(s) if s == "." => AtomKind::Dot,
        Value::String(_) => AtomKind::Literal,
        Value::Object(obj) => {
            match obj.get("type").and_then(|v| v.as_str()) {
                Some("anchor") => AtomKind::Anchor,           // read obj["kind"]
                Some("escape") => AtomKind::Escape,           // read obj["kind"] + payload
                Some("backreference") => AtomKind::Backreference,
                Some("atom") => {
                    // the structural constructs — dispatch on kind
                    match obj.get("kind").and_then(|v| v.as_str()) {
                        Some("char_class") => AtomKind::CharClass,
                        Some("quoted_literal") => AtomKind::QuotedLiteral,
                        Some("capturing_group") | Some("noncapturing_group")
                        | Some("named_group") | Some("python_named_group") => AtomKind::Group,
                        Some("lookahead") | Some("lookbehind")
                        | Some("non_atomic_lookahead") | Some("non_atomic_lookbehind")
                        | Some("alpha_lookaround") => AtomKind::Lookaround,
                        Some(k) => AtomKind::Other(k.to_string()),
                        None => AtomKind::Unknown,
                    }
                }
                _ => AtomKind::Unknown,
            }
        }
        _ => AtomKind::Unknown,
    }
}
```

> **Migration note.** Pre-typed-era consumers dispatched on structural prefixes
> (`arr[0] == "\\Q"`, `"("`-prefix walks, 2-element `["\\", …]` escapes). Those raw
> shapes no longer exist in any released artifact; the string-vs-object + `type`/`kind`
> dispatch above fully replaces them.
