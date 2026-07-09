# Character Class Subtree

PCRE2 character classes (`[abc]`, `[a-z]`, `[^a-z]`, `[[:alpha:]]`, `[]x]`, …) live under
`char_class`, which emits a **typed** `{type: "atom", kind: "char_class", negated,
initial_close, body}` object. Since release `1.1.84` (`REGEX-PCRE2-FIDELITY.3.15`, ledger
`REGEX-0091`) the grammar encodes the **full PCRE2 class-open model** — member *visibility*,
negation-through-invisibles, and the initial-`]`-literal rule — all oracle-verified against
`pcre2test` 10.47.

## The PCRE2 class-open model (what the grammar encodes)

PCRE2 reads a class opening like this, and so does this grammar:

1. **Invisible items never count.** A stray `\E` and the empty `\Q\E` are zero-width — they
   can appear anywhere in a class but never make it non-empty and never affect the opening
   rules. `[\E]` is **rejected** (err 106 in PCRE2): the `\E` is invisible, so the `]` becomes
   a literal member and the class is then unterminated.
2. **The first non-invisible char after `[` is the negation caret** (if it is `^`). So
   `[\E^a]` is a *negated* class of `a` — and a `^` anywhere later is an ordinary member:
   `[^^]` is a negated class containing the literal `^`.
3. **A `]` seen before any visible member is a LITERAL member** — immediately after the
   opening (`[]]`), after the negation (`[^]]`), or after invisibles (`[\E]x]` is the class
   `]x`). Only after at least one visible member (or that initial `]`-literal) does `]` close
   the class. `[]` and `[^]` therefore reject: the `]` is a member and nothing closes.
4. **In-class `\Q` is always the quote-opener**, never a shorthand escape. `[\Q]` quotes the
   `]` and leaves the class unterminated → reject.

| Input | Verdict | Why |
|---|---|---|
| `[\E]` `[\Q\E]` `[^\E]` | REJECT | invisible-only — the `]` became a literal member, unterminated |
| `[\E]x]` `[\Q\E]]` `[^\E]x]` | ACCEPT | invisibles skipped → initial `]` is a literal member |
| `[^^]` `[^\E^]` | ACCEPT | the second caret is a member of the negated class |
| `[\E^a]` | ACCEPT | negation recognized through the invisible `\E` (`negated: true`) |
| `[\E^]` `[\Q\E^]` | REJECT | the caret negates → the class is then empty/unterminated |
| `[\Q^\E]` `[\^]` | ACCEPT | a quoted/escaped caret is a member, never negation |
| `[\Q]` `[\Qa]` `[a\Q]` | REJECT | unterminated in-class quote (the `]` is quoted) |
| `[]` `[^]` | REJECT | the first `]` is a literal member; nothing closes the class |

## `char_class`

```ebnf
char_class     = "[" class_negated_open? class_zero_width* class_initial_close class_body "]"    -> {type: "atom", kind: "char_class", negated: $2, initial_close: $4, body: $5}
               | "[" class_negated_open class_body_nonempty "]"              -> {type: "atom", kind: "char_class", negated: $2, initial_close: [], body: $3}
               | "[" class_body_nonempty_nocaret "]"                         -> {type: "atom", kind: "char_class", negated: [], initial_close: [], body: $2}
```

Three alternatives, one typed shape:

- **Alt 1 — the initial-`]`-literal form**: optional negation opening, optional invisibles,
  then a literal `]` member, then any items (`[]]`, `[^]a]`, `[\E]x]`).
- **Alt 2 — negated, ordinary body**: a negation opening (which itself absorbs surrounding
  invisibles), then a body with ≥1 *visible* member (`[^a]`, `[^^]`, `[\E^a]`).
- **Alt 3 — non-negated, ordinary body**: a body with ≥1 visible member whose *first* visible
  member is structurally caret-free — a leading bare `^` would have been the negation
  (`[abc]`, `[\Ea]`, `[a^]` — the caret is fine after the first member).

### Shape

```json
{
  "type": "atom",
  "kind": "char_class",
  "negated": <true when a negation caret opened the class, [] otherwise>,
  "initial_close": <true when the class opened with a literal-`]` member, [] otherwise>,
  "body": [ <class item values, in source order — see class_item> ]
}
```

The invisible items consumed by the opening slots (`class_negated_open`, the alt-1
`class_zero_width*`) are **dropped** from the typed shape (the same convention as
`class_range`'s zero-width slots). Invisibles *inside* the body (after the first visible
member) still appear as body items.

### Examples (all verified against the running parser)

| Input | Typed value |
|---|---|
| `[ab]` | `{"body": ["a", "b"], "initial_close": [], "negated": [], …}` |
| `[]]` | `{"body": [], "initial_close": true, "negated": [], …}` |
| `[\E]x]` | `{"body": ["x"], "initial_close": true, "negated": [], …}` |
| `[^^]` | `{"body": ["^"], "initial_close": [], "negated": true, …}` |
| `[\E^a]` | `{"body": ["a"], "initial_close": [], "negated": true, …}` |
| `[a-z]` | `{"body": [{"type": "class_range", "start": "a", "end": "z"}], …}` |
| `[[:alpha:]]` | `{"body": [{"type": "posix_class", "name": "alpha", "negated": []}], …}` |
| `[\Qa\E]` | `{"body": [{"type": "class_quoted_literal", "body": ["a"]}], …}` |
| `[a\Q\E]` | `{"body": ["a", {"type": "class_quoted_literal", "body": []}], …}` |
| `[a\E]` | `{"body": ["a", "\\E"], …}` (an in-body stray `\E` stays a body item) |

> **AST correction note (release `1.1.84`).** `[\E^a]`, `[\Q\E^a]` and `[\E^-z]` previously
> parsed as *non-negated* classes carrying `"\\E"` and `"^"` as members. That was
> PCRE2-unfaithful; they are now `negated: true` with the invisibles dropped, as above.

## `class_negated_open`

```ebnf
class_negated_open = class_zero_width* negation class_zero_width*    -> $2
```

The PCRE2 negation opening: the caret with its surrounding invisibles. The `-> $2`
passthrough keeps the classic `negated` slot convention — a matched optional contributes the
bare `true`, an unmatched one contributes `[]`.

## `negation`

```ebnf
@generate: "^" if $1 else ""
@semantic_value: $1 != null
negation = "^"                                                 -> true
```

Emits the typed `true`.

## `class_initial_close`

```ebnf
class_initial_close = "]"                                            -> true
```

The literal-`]`-member marker (PCRE2 quirk: a `]` before any visible member is a member, not
the terminator). Emits the typed `true` into the `initial_close` slot.

## `class_body`

```ebnf
class_body     = class_item*
```

The alt-1 body (after an initial `]`-literal): zero or more items, no visibility requirement
(the `]`-literal already made the class non-empty). Its value is the flat item list.

## `class_body_nonempty` / `class_body_nonempty_nocaret`

```ebnf
class_body_nonempty = class_zero_width* class_item_visible class_item*   -> [$1*, $2, $3*]
class_body_nonempty_nocaret = class_zero_width* class_item_visible_nocaret class_item*   -> [$1*, $2, $3*]
```

The ordinary bodies: optional invisible prefix, then **one visible member**, then any items.
The `-> [$1*, $2, $3*]` flatten keeps the value a single flat item list, byte-identical to
the pre-`1.1.84` shape for every previously-accepted input. The `_nocaret` variant (used by
the no-negation alternative) excludes a bare `^` from the *first visible* position only —
that caret would have been the negation.

## `class_item`

```ebnf
class_item     = !invalid_class_range class_item_core   -> $2
class_item_core = posix_class
               | stray_class_end_quote
               | class_range
               | quoted_class_literal
               | class_member_literal
               | class_escape
```

The unrestricted item (used after the first visible member / after an initial `]`-literal).
The leading `!invalid_class_range` is a zero-width guard that rejects a class range with a
nonliteral endpoint (see [Nonliteral class-range endpoints](#nonliteral-class-range-endpoints-the-invalid_class_range-guard)
below); `-> $2` passes the matched `class_item_core` value through unchanged, so the typed
value is byte-identical to the pre-guard alternation. Branch 4 of `class_item_core` is
`class_member_literal` (a guarded one-char literal — see
[POSIX name validity](#posix-name-validity-the-class_member_literal-guard)), whose value is
the same bare-string char as the underlying `class_literal`.

| Branch | Form | Typed value |
|---|---|---|
| 0 | `[:alpha:]` etc. | `{"type": "posix_class", "name": <str>, "negated": <true \| []>}` |
| 1 | stray `\E` (zero-width) | `"\\E"` (string) |
| 2 | `a-z` etc. | `{"type": "class_range", "start": <atom>, "end": <atom>}` |
| 3 | `\Q…\E` (possibly empty) | `{"type": "class_quoted_literal", "body": [<chars>]}` |
| 4 | one literal char (`class_member_literal`) | `"<char>"` (string) |
| 5 | `\d`, `\x41`, … | the typed escape object — see [Escape Subtree](rules-escape.md) |

## `class_item_visible` / `class_item_visible_nocaret`

```ebnf
class_item_visible = !invalid_class_range class_item_visible_core   -> $2
class_item_visible_core = posix_class
                        | class_range
                        | quoted_class_literal_nonempty
                        | class_member_literal
                        | class_escape
class_item_visible_nocaret = !invalid_class_range class_item_visible_nocaret_core -> $2
class_item_visible_nocaret_core = posix_class
                                | class_range
                                | quoted_class_literal_nonempty
                                | class_member_literal_nocaret
                                | class_escape
```

`class_item` minus the invisible forms (branch 1 stray `\E`; the *empty* `\Q\E`), preserving
the same relative alternative order, each carrying the same `!invalid_class_range` guard. The
`_nocaret` variant swaps in `class_member_literal_nocaret` so a bare `^` cannot fill the
first-visible slot of a non-negated class. Their values are the same typed values as the
matching `class_item` branches — consumers never see a difference.

## Nonliteral class-range endpoints (the `invalid_class_range` guard)

A character-class range whose LEFT or RIGHT endpoint is a **nonliteral** shorthand
(`\d \D \h \H \s \S \v \V \w \W`) or property (`\p…` / `\P…`) escape is malformed in PCRE2
(err 150 "invalid range"): `[\d-x]`, `[a-\d]`, `[\p{Lu}-x]`, `[a-\p{Lu}]`, `[\d-\w]`. A
zero-width negative lookahead at each class-item position rejects them at the grammar layer
(they were previously rejected only by the out-of-band compile-contract validator — this
migrates the reject into the EBNF, `REGEX-PCRE2-FIDELITY.4.5.b`, ledger `REGEX-0107`):

```ebnf
invalid_class_range = class_range_nonliteral_atom class_zero_width* "-" class_zero_width* ( class_range_nonliteral_atom | class_atom )
                    | class_atom class_zero_width* "-" class_zero_width* class_range_nonliteral_atom
                    | descending_class_range
class_range_nonliteral_atom = "\\" class_range_nonliteral_shorthand
                            | "\\" property_escape
class_range_nonliteral_shorthand = 'd' | 'D' | 'h' | 'H' | 's' | 'S' | 'v' | 'V' | 'w' | 'W'
```

The guard is deliberately narrow so the **literal-dash carve-outs** stay valid: `[a-]` and
`[-a]` (dash at an edge), `[\d-]` (a nonliteral followed by a trailing dash and `]` — shape 1
requires a real endpoint atom after the `-`), `[\d\-x]` (an *escaped* dash is a literal
member — the `-` in the guard is a bare terminal), `[\da-z]` and `[-\d]` (a nonliteral member
adjacent to but not part of a range). Valid literal-escape ranges (`[\n-\r]`, `[\x30-\x39]`)
and nonliteral **members** (`[\d]`, `[\p{Lu}]`, `[\s\w]`) are unaffected — only a nonliteral
used as a range *endpoint* is rejected.

`invalid_class_range` and its `class_range_nonliteral_*` helpers are referenced only inside
the `!(…)` lookahead, so they are never positively entered; certificate-coverage certifies
them by **PROOF** (the lookahead-only / positively-unreachable class), keeping regex
`UNKNOWN=0`.

### Descending literal ranges (the `descending_class_range` alternative)

The third `invalid_class_range` alternative rejects a range of two **literal** endpoints whose
left code point exceeds the right — `[z-a]`, `[9-0]`, `[\x39-\x30]`, `[\x{100}-a]` (PCRE2 err
108 "range out of order in character class"). Unlike the nonliteral shapes, this is a **value**
constraint, so it is gated by the general `value_compare_codepoint` `@predicate`
(`RULE-SPAN-VALUE-CONSTRAINT.3`, ledger `REGEX-0108`, `REGEX-PCRE2-FIDELITY.4.5.c`):

```ebnf
@predicate: { name: value_compare_codepoint, args: [$1, gt, $5], phase: post }
descending_class_range = class_range_endpoint class_zero_width* "-" class_zero_width* class_range_endpoint
class_range_endpoint = class_range_decodable_atom                                -> $text
class_range_decodable_atom = class_range_decodable_escape | letter | digit | class_safe_special | unicode_char
class_range_decodable_escape = "\\" hex_escape | "\\" octal_escape | "\\" control_escape | "\\" class_range_simple_escape
```

The predicate DECODES each endpoint's RAW spelling (`class_range_endpoint -> $text` re-emits
the matched text) to its Unicode code point and matches when left > right — so the rule
succeeds exactly for a descending range, and the `!invalid_class_range` guard blocks it. The
braced `[\x{100}-a]` is the code-point discriminator: a textual compare (`\`=92 < `a`=97) would
wrongly ACCEPT; only decoding (256 > 97) rejects. `class_range_decodable_atom` is restricted to
endpoints the decoder can recover: bare **non-whitespace** literals plus the hex/octal/control/
simple escapes. Two endpoint families are deliberately **excluded** because their `$text`
reaches the predicate as `None` or empty — making the gate inapplicable (non-blocking) and
over-blocking a *valid* range: an undecodable escape (`\Q..\E`, `\u{}`) and a **bare whitespace**
literal (a `class_literal → whitespace` match reaches the predicate with an empty raw spelling —
a latent pipeline finding). So ascending whitespace ranges (`[ -!]`, `[ --]`, `[\t-a]`) keep
ACCEPTing, and descending ranges with those endpoints stay validator-owned.

**Scope:** shorthand + property endpoints (`.4.5.b`) and descending literal ranges with a
decodable non-whitespace endpoint (`.4.5.c`) are grammar-owned; the POSIX-left (`[[:alpha:]-z]`),
`-[`-right (`[a-[:digit:]]`, ledger `REGEX-0105`), and quoted/`\u{}`/bare-whitespace-endpoint
descending cases stay validator-owned pending `REGEX-PCRE2-FIDELITY.4.5.d`, which then deletes
the validator range-check entirely.

## Walking a class body

```rust
fn class_items(class_atom: &Value) -> Vec<&Value> {
    class_atom["body"].as_array().unwrap().iter().collect()
}

fn classify_class_item(item: &Value) -> ClassItemKind {
    match item {
        Value::String(s) if s == "\\E" => ClassItemKind::StrayEndQuote, // zero-width
        Value::String(s) => ClassItemKind::Literal(s),                  // one member char
        Value::Object(map) => match map["type"].as_str() {
            Some("posix_class") => ClassItemKind::PosixClass,
            Some("class_range") => ClassItemKind::Range,
            Some("class_quoted_literal") => ClassItemKind::QuotedLiteral,
            Some("escape") => ClassItemKind::Escape,
            _ => ClassItemKind::Unknown,
        },
        _ => ClassItemKind::Unknown,
    }
}
```

For `[abc-z]`: `body` = `["a", "b", {"type": "class_range", "start": "c", "end": "z"}]`.

## `posix_class`

```ebnf
@generate: generate_posix_class_check($3)
@semantic_value: {type: "posix", name: $3, negated: $2 != null}
posix_class    = "[:" posix_negation? posix_name ":]"
-> {type: "posix_class", name: $3, negated: $2}
```

Typed object. `negated` is `true` for `[[:^alpha:]]`, `[]` when no `^` was matched
(consumers map `[]` → `false`, the same convention as `quantifier.greediness`).

| Input | Value |
|---|---|
| `[[:alpha:]]` | `{"type": "posix_class", "name": "alpha", "negated": []}` |
| `[[:^digit:]]` | `{"type": "posix_class", "name": "digit", "negated": true}` |

### POSIX name validity — the `class_member_literal` guard

`posix_name` lists exactly the 14 valid POSIX names. A `[:name:]` token whose name is **not** one
of them is a compile error in PCRE2 (err 130, *"unknown POSIX class name"*), not a set of literal
characters. Since **`REGEX-0101`** (release `1.1.91`, `REGEX-PCRE2-FIDELITY.4.6`) the grammar owns
this rule — previously it lived only in an out-of-band validator, so the grammar alone wrongly
accepted `[[:foo:]]` (as the literals `[`, `:`, `f`, `o`, `o`, `:`).

The mechanism is a guard on the class-member `[` literal:

```ebnf
class_member_literal         = !( "[:" "^"? ( !":]" builtin_any_char )* ":]" ) class_literal         -> $2
class_member_literal_nocaret = !( "[:" "^"? ( !":]" builtin_any_char )* ":]" ) class_literal_nocaret -> $2
```

A `[` that opens a `:]`-terminated `[:…:]` token can no longer be a literal. For a **valid** name
`posix_class` wins the longest-match tournament (so the guard is inert); for an **invalid** name
`posix_class` fails *and* the `[` literal is blocked, so the class cannot close and the pattern is
rejected at the grammar layer (`E_PARSE_FAILURE` — match on the code, never the message). The value
is the same bare-string char as `class_literal`, so accepted-class ASTs are byte-identical.

| Input | Verdict (PGEN = PCRE2 10.47) | Why |
|---|---|---|
| `[[:alpha:]]`, `[[:^alpha:]]`, `[[:alnum:][:digit:]]` | ACCEPT | valid name(s) → `posix_class` |
| `[[:foo:]]`, `[a[:<:]]`, `[[::]]`, `[x[:foo:]y]`, `[^[:foo:]]` | REJECT | unknown / empty name → err 130 |
| `[[:foo]`, `[[:]]`, `([[:]+)`, `[a:foo:]` | ACCEPT | no `:]` terminator → `[:` is literal |
| `[\Q[:foo:]\E]` | ACCEPT | quoted inside `\Q…\E` → literal |
| `[[:<:]]`, `[[:>:]]` | ACCEPT | word-boundary anchor aliases (matched before `char_class`) |

> **Honest bound (pre-existing).** The guard scans to the first `:]` across both escaped and
> unescaped `]`, exactly replicating the validator it replaced. PCRE2's posix-name boundary stops at
> an *unescaped* `]`, so `[x[:foo]bar:]y]` accepts in PCRE2 but rejects in PGEN (both before and after
> this release). A PCRE2-exact `]`-boundary is a tracked follow-up.

### Escape validity inside a class — the escape-in-class guards

Inside a character class PCRE2 forbids the anchor / assertion / special escapes
`\A \B \C \G \K \R \X \Z \z`, and allows `\N` only in its braced named-codepoint form `\N{…}`
(bare `\N` is invalid). Since **`REGEX-0102`** (release `1.1.92`, `REGEX-PCRE2-FIDELITY.4.4`) the
grammar owns this rule — previously it lived only in the out-of-band validator, so the grammar alone
wrongly accepted `[\B]`, `[\C]`, and even `[\A-x]` (via the range path). Three edits close the three
reach paths:

1. Both `class_simple_escape` variants carry the guards
   `!"A" !"B" !"C" !"G" !"K" !"R" !"X" !"Z" !"z" !( "N" !"{" )` — the only conditional one,
   `!( "N" !"{" )`, rejects a bare `\N` while a following `{` keeps `\N{…}` a valid member.
2. `single_byte_escape` (`\C`) is dropped from the class-context `class_escape_unit` (it stays valid
   in the pattern body via `escape_unit`).
3. `A`, `G`, `z` are dropped from `class_range_literal_escape_letter_strict` — they were reachable as
   a range *endpoint* (`[\A-x]`, `[\z-\x{FFFF}]`), which the member-side guards do not cover.
4. `V`, `v` are dropped from `class_range_literal_escape_letter_strict` (`.4.5.a.1`, `REGEX-0106`) — the
   vertical-whitespace shorthands are NONLITERAL (like `\h`/`\H`), so PCRE2 rejects them as a range
   *endpoint* (err 150) even though they are valid class *members*. They stay valid members via
   `class_simple_escape`; removing the range endpoint (paired with the validator's `is_nonliteral_class_escape`
   gaining `v`/`V`) makes `\v`/`\V` behave byte-identically to `\h`/`\H`.

| Input | Verdict (PGEN = PCRE2 10.47) | Why |
|---|---|---|
| `[\B]`, `[\K]`, `[\C]`, `a[\NB]c`, `[\A-x]`, `[a-\B]`, `[\z-\x{FFFF}]` | REJECT | escape invalid as a class member (err 137/108) |
| `[\v-x]`, `[\V-x]`, `[a-\v]` | REJECT | vertical-whitespace shorthand is a NONLITERAL range endpoint (err 150) |
| `[\d]`, `[\w]`, `[\s]`, `[\b]` (backspace), `[\n]`, `[\a]`, `[\e]`, `[\v]`, `[\V]` | ACCEPT | legal class shorthand / c-escape (member) |
| `[\N{U+00E9}]`, `[\pL]`, `[\x41]`, `[\x0b-\x0c]`, `[\g]`, `[\j]`, `[\k]` | ACCEPT | braced `\N{…}` / property / hex range / literal-transport letter |
| `[\I-x]`, `[\a-x]`; body `\B`, `\K`, `\A`, `\z` (outside a class) | ACCEPT | valid literal range / valid pattern-body anchor |

Behavior-neutral: every accept/reject verdict is byte-identical to `1.1.91` (validator rejected these
before, the grammar rejects them now — only the message moved). `\N{…}` and all lowercase / non-reject
letters (`\g`, `\j`, `\k`, `\r`, …) stay literal-transport members, so accepted-class ASTs are
unchanged.

## `class_range`

```ebnf
class_range    = class_atom class_zero_width* "-" class_zero_width* class_atom   -> {type: "class_range", start: $1, end: $5}
```

Typed object; the two zero-width slots (PCRE2 `\E`/`\Q\E` markers around the dash, e.g.
`[a\E-z]`) are dropped from the typed shape. `start`/`end` are the typed `class_atom` values.

| Input | Value |
|---|---|
| `[a-z]` | `{"type": "class_range", "start": "a", "end": "z"}` |
| `[\Qa\E-\Qz\E]` | `{"type": "class_range", "start": {"type": "class_quoted_range_atom", "char": "a"}, "end": {…"z"}}` |
| `[a\E-z]` | `{"type": "class_range", "start": "a", "end": "z"}` (the stray `\E` dropped) |

## `class_atom`

```ebnf
class_atom     = quoted_class_range_atom | class_range_escape | class_literal
```

Range endpoints. Each branch emits its own typed value (see below / the escape subtree).

## `class_literal` / `class_literal_nocaret`

```ebnf
class_literal  = letter | digit | whitespace | class_safe_special | unicode_char
class_literal_nocaret = letter | digit | whitespace | class_safe_special_nocaret | unicode_char
```

One member character; the value is the 1-char string. The `_nocaret` variant excludes `^`
(see `class_item_visible_nocaret`).

## `class_escape`

```ebnf
class_escape   = "\\" class_escape_unit -> $2
```

Transparent passthrough — emits the matched escape-unit's typed object (see
[Escape Subtree](rules-escape.md)). Class context admits `\<digit>` octal forms (`[\8]`) and,
since `1.1.84`, **excludes `\Q` and `\E`** from the shorthand catch-alls in *both* profiles —
those two are always the quote-opener / quote-end marker (`[\Q]` rejects as an unterminated
class; a lone `\E` is the zero-width `stray_class_end_quote`).

## `quoted_class_literal` / `quoted_class_literal_nonempty`

```ebnf
quoted_class_literal = "\\Q" quoted_class_literal_char* "\\E"        -> {type: "class_quoted_literal", body: $2}
quoted_class_literal_nonempty = "\\Q" quoted_class_literal_char+ "\\E"        -> {type: "class_quoted_literal", body: $2}
```

The in-class quoted run. Both emit the same typed shape; the `_nonempty` variant (≥1 char)
fills the visible-member slot, while the possibly-empty form remains available after the
first visible member (`[a\Q\E]` → `body: ["a", {"type": "class_quoted_literal", "body": []}]`).

| Input | Value |
|---|---|
| `[\Qa\E]` | `{"type": "class_quoted_literal", "body": ["a"]}` |
| `[ab\Q^$.\E]` | `["a", "b", {"type": "class_quoted_literal", "body": ["^", "$", "."]}]` (body) |

## `quoted_class_range_atom`

```ebnf
quoted_class_range_atom = "\\Q" quoted_class_literal_char "\\E"      -> {type: "class_quoted_range_atom", char: $2}
```

A single-character `\Q<char>\E` as a range endpoint (`[\Qa\E-\Qz\E]`).

## `stray_class_end_quote`

```ebnf
stray_class_end_quote = "\\E"
```

A bare `\E` inside a class — PCRE2-invisible (zero-width). As a *body* item it emits the
string `"\\E"`; in the opening slots (`class_negated_open`, the alt-1 invisible prefix) it is
consumed and dropped. It never counts toward class non-emptiness.

## `empty_quoted_class_literal`

```ebnf
empty_quoted_class_literal = "\\Q" "\\E"   -> {type: "class_quoted_literal", body: []}
```

The empty `\Q\E` — also PCRE2-invisible. Typed like `quoted_class_literal` with an empty
`body`, so a `\Q\E` routed through `class_zero_width` is byte-identical to one routed through
`class_item`.

## `class_zero_width`

```ebnf
class_zero_width = stray_class_end_quote | empty_quoted_class_literal
```

The two invisible forms, used by the opening slots, the visible-body prefixes, and the
`class_range` dash surroundings.

## `class_safe_special` / `class_safe_special_nocaret`

```ebnf
class_safe_special = '[' | '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' | '-' | '+' | '=' | '{' | '}' | '|' | ':' | ';' | '"' | "'" | '<' | '>' | ',' | '.' | '?' | '/' | '`' | '~' | '_'
class_safe_special_nocaret = '[' | '!' | '@' | '#' | '$' | '%' | '&' | '*' | '(' | ')' | '-' | '+' | '=' | '{' | '}' | '|' | ':' | ';' | '"' | "'" | '<' | '>' | ',' | '.' | '?' | '/' | '`' | '~' | '_'
```

ASCII punctuation that is a literal member inside a class without escaping (note: **no
backslash** — escapes go through `class_escape`). The `_nocaret` variant is the same set
minus `^`, used only for the first-visible slot of a non-negated class.

## `class_range_escape`

```ebnf
class_range_escape = "\\" class_range_escape_unit                     -> $2
```

Transparent passthrough for escapes acting as range endpoints.

## `class_range_escape_unit`

```ebnf
class_range_escape_unit = hex_escape
                        | unicode_escape
                        | octal_escape
                        | control_escape
                        | class_range_simple_escape
```

The restricted escape set valid as a range endpoint; the matched unit's typed object surfaces
through the `class_range_escape` passthrough.

## `class_range_simple_escape`, `class_range_any_char_no_orphan_quote_end`, `class_range_literal_escape_letter`

Inner sub-rules controlling which escaped letters can act as class-range endpoints (the
strict variant already excludes `E`/`Q`, the six `.3.1` PCRE2-unsupported letters, `A`/`G`/`z`
(`.4.4`), and `V`/`v` (`.4.5.a.1` — the nonliteral vertical-whitespace shorthands); the
`relaxed` profile re-admits the six `.3.1` letters only). Consumers rarely walk these — the outer
`class_range_escape` passthrough is the shape that surfaces.

## `posix_negation`, `posix_name`, `letter_no_upper_e`

Inner POSIX sub-rules (`posix_negation -> true`; `posix_name` emits the matched name string).
`letter_no_upper_e` is the quoted-run letter set excluding `E` (so `\E` always terminates a
quoted run).
