# Anchors, Backreferences, and Misc

This chapter covers the remaining rule families that don't fit cleanly into the per-subtree chapters: callouts, conditionals, directive verbs, code blocks, comment groups, extended classes, and the auxiliary lexical helpers.

## `callout`

```ebnf
callout = "(?C" callout_arg? ")"
```

The PCRE2 callout construct `(?C...)`. **Typed** — the atom is:

```json
{ "type": "atom", "kind": "callout", "arg": <arg> }
```

where `arg` is `[]` for the bare `(?C)`, a typed integer for the numeric form, or a
`{"payload", "quote"}` object for the string forms (table below).

### `callout_arg`

```ebnf
callout_arg = callout_number | callout_string
```

2-way Or, folded into the parent's `arg` field. `callout_number` is the typed integer
(annotated). `callout_string` is one of 8 string-delimited forms.

### `callout_number` — the numeric argument, value-bounded to [0, 255]

```ebnf
@transform: str::parse::<usize>().unwrap_or(0)
callout_number      = callout_number_body
callout_number_body = "0"+ callout_number_core? | callout_number_core
callout_number_core = "25" ("0" | "1" | "2" | "3" | "4" | "5")
                    | "2" ("0" | "1" | "2" | "3" | "4") digit
                    | "1" digit digit
                    | nonzero_digit digit
                    | nonzero_digit
```

PCRE2 rejects a numeric callout whose **value** exceeds 255 (compile error 138), while
accepting arbitrary leading zeros — `(?C255)`, `(?C0255)`, and `(?C000000000255)` all
compile; `(?C256)` and `(?C0256)` do not (`pcre2test` 10.47). Since release 1.1.84's
follow-up (REGEX-PCRE2-FIDELITY.3.16) this bound is encoded **structurally in the
grammar**: the digit run must be leading zeros plus an optional nonzero-led core ≤ 255,
so an out-of-range run has no fully-consuming parse (the trailing `")"` fails) and the
stimuli generator can only emit in-range callout numbers by construction. Previously
the bound lived in the out-of-band compile-contract validator
(`find_invalid_numeric_callout`), invisible to generation.

The `@transform` applies to the rule's matched **span**, so the carrier stays the exact
typed integer the former `digits` branch produced: `(?C0255)` → `"arg": 255`. Rejection
for out-of-range forms is now an ordinary parse failure (`Parser did not consume full
input …`) rather than the validator's `numeric callout argument exceeds PCRE2 compile
limit 255` message — match on the error **code** (`E_PARSE_FAILURE`), not the text.

The same bound covers the condition-callout site `(?(?C255)(?=y)x|z)` — both callout
forms share `callout_arg`. String callout-args are not value-bounded (unchanged).

### `callout_string`

```ebnf
callout_string = callout_backtick_string
               | callout_single_string
               | callout_double_string
               | callout_caret_string
               | callout_percent_string
               | callout_hash_string
               | callout_dollar_string
               | callout_brace_string
```

8-way Or. Each variant uses a different delimiter pair: backtick, single quote, double quote, caret, percent, hash, dollar, brace.

Each string form emits `{"payload": <string>, "quote": <delimiter-name>}` — the
payload is the clean joined string, the quote names the delimiter. Live-verified:

| Source | Emitted atom |
|---|---|
| `(?C)` | `{"type":"atom","kind":"callout","arg":[]}` |
| `(?C12)` | `{"type":"atom","kind":"callout","arg":12}` |
| `(?C0255)` | `{"type":"atom","kind":"callout","arg":255}` (leading zeros fold into the typed value) |
| `(?C"str")` | `{"type":"atom","kind":"callout","arg":{"payload":"str","quote":"double"}}` |
| `(?C'x')` | `{"type":"atom","kind":"callout","arg":{"payload":"x","quote":"single"}}` |
| `` (?C`bt`) `` | `{"type":"atom","kind":"callout","arg":{"payload":"bt","quote":"backtick"}}` |
| `(?C^c^)` | `{"quote":"caret"}` · `(?C%p%)` → `"percent"` · `(?C#h#)` → `"hash"` · `(?C$d$)` → `"dollar"` · `(?C{br})` → `"brace"` |

## `conditional`

```ebnf
conditional = "(?(" condition ")" yes_branch ("|" no_branch)? ")"
```

PCRE2 conditional pattern. **Typed** — the atom is:

```json
{ "type": "atom", "kind": "conditional",
  "condition": <condition>,
  "yes_branch": [<pieces>],
  "no_branch": ["|", [<pieces>]] }
```

`yes_branch` is the yes-concatenation's piece array; `no_branch` is the optional
`("|" no_branch)?` slot — the 2-element `["|", [<pieces>]]` pair when present, `[]`
when absent. The `condition` value is typed per branch (table under each sub-rule
below; the one-look summary lives in [Group Family](rules-groups.md#conditional)).

### `condition`

```ebnf
condition = define_condition
          | version_condition
          | condition_callout_assertion
          | condition_assertion
          | name_ref
          | recursion_condition
          | name
          | signed_digits
          | digits
```

9-way Or covering the PCRE2 condition forms. Every branch folds to a typed value in
the parent's `condition` field — consumers dispatch on the value's SHAPE: a
signed-number object, a plain name string, or an object with its own `kind`. The
complete live-verified table:

| Source | `condition` value |
|---|---|
| `(?(1)…)` / `(?(+1)…)` / `(?(-1)…)` | `{"sign":[],"value":1}` / `{"sign":"+","value":1}` / `{"sign":"-","value":1}` |
| `(?(<n>)…)` / `(?('n')…)` / `(?(n)…)` | `"n"` (clean name string — all three spellings) |
| `(?(R)…)` | `{"kind":"recursion","group":[]}` |
| `(?(R2)…)` | `{"kind":"recursion","group":2}` |
| `(?(R&n)…)` | `{"kind":"recursion_named","name":"n"}` |
| `(?(DEFINE)…)` | `{"kind":"define"}` |
| `(?(VERSION>=10.4)…)` | `{"kind":"version","operator":">=","number":{"major":10,"minor":4}}` |
| `(?(?=x)…)` etc. | the assertion's own typed lookaround object (`{"kind":"lookahead","positive":true,"body":<pattern>}`, …) |
| `(?(?C1)(?=x)…)` | `{"kind":"callout_assertion","callout":{"kind":"callout","arg":1},"assertion":<lookaround object>}` |

The sub-rules behind those branches (grammar reference):

### `define_condition`

```ebnf
define_condition = "DEFINE"
```

Emits `{"kind": "define"}`.

### `version_condition`

```ebnf
version_condition = "VERSION" version_operator version_number
version_operator  = ">=" | "="
version_number    = digits ("." digits)?
```

Emits `{"kind": "version", "operator": <op>, "number": {"major": <int>, "minor": <int>}}`
(a version without a `.minor` part carries only the parsed components the source had).

### `condition_callout_assertion`

```ebnf
condition_callout_assertion = condition_callout "(" condition_assertion
condition_callout           = "?C" callout_arg? ")"
```

Emits `{"kind": "callout_assertion", "callout": {"kind":"callout","arg":<arg>},
"assertion": <lookaround object>}` — the callout's `arg` uses the same typed forms as
the standalone [`callout`](#callout) (the value bound to [0, 255] applies here too).

### `condition_assertion`

```ebnf
condition_assertion = "?=" pattern
                    | "?!" pattern
                    | "?<=" pattern
                    | "?<!" pattern
                    | alpha_condition_assertion
alpha_condition_assertion = "*" atomic_alpha_lookaround_name ":" pattern?
```

Emits the same typed lookaround objects as atom-position lookarounds
(`kind: "lookahead"` / `"lookbehind"` with `positive`, or `kind: "alpha_lookaround"`
with `name`) — one shape family for both positions.

### `recursion_condition`

```ebnf
recursion_condition = "R" digits?
                    | "R&" name
```

Emits `{"kind": "recursion", "group": <int or []>}` or
`{"kind": "recursion_named", "name": <string>}`.

## `directive_verb`

```ebnf
directive_verb          = "(*" directive_body_quantifiable ")"
directive_verb_nonquant = "(*" directive_body_nonquantifiable ")"
```

Both emit `{type:"atom", kind:"directive_verb", body:<...>}` — the split (release `1.1.87`,
`REGEX-PCRE2-FIDELITY.3.20`) is purely about **quantifiability**. In PCRE2 only `(*ACCEPT)` may take
a quantifier; every other `(*...)` directive quantified is err 109. So `directive_verb` (the
quantifiable rule, an `atom` alternative) matches only the ACCEPT verb and the relaxed unknown-name
catch-all, while `directive_verb_nonquant` (matched only by the non-quantifiable
[`piece` branch](rules-piece.md#piece) `directive_verb_nonquant !quantifier`) matches every other
directive. Because `atom` no longer matches a non-ACCEPT directive, a quantified one rejects at the
grammar (`(*PRUNE)+`, `(*:x)+`, `(*UTF)+`, `(*LIMIT_HEAP=5)+` all err 109; ledger **REGEX-0096**).

### `directive_body_quantifiable` / `directive_body_nonquantifiable`

```ebnf
directive_body_quantifiable    = directive_accept_named | directive_relaxed_named
directive_body_nonquantifiable = directive_mark_named
                               | directive_verb_named
                               | directive_mark_shorthand
```

The quantifiable body is the ACCEPT verb + the `@profiles:["relaxed"]` unknown-name catch-all (so
relaxed `(*foo)+` stays accepted). The non-quantifiable body is the 6 non-ACCEPT verbs, MARK, and the
`(*:x)` shorthand. Both bodies pass their branch's object through unchanged.

**REGEX-PCRE2-FIDELITY.4.8 — the bare start options were PEELED OUT of `directive_body_nonquantifiable`.**
`directive_limit_named` / `directive_option_named` (the `(*LIMIT_HEAP=…)` / `(*UTF)`-class start
options) are no longer members of `directive_verb_nonquant`; they moved into a dedicated
`start_option_body` (`= directive_limit_named | directive_option_named`) under the
[`start_option_piece`](rules-piece.md#piece) rule, which is reachable ONLY from the distinguished
[`entry_concatenation`](rules-top-level.md). That structural move is what encodes the PCRE2
start-option **position** rule — a start option is valid only as a contiguous run at the very start of
the whole pattern (`(*CRLF)abc` ACCEPT; `a(*CR)b`, `(a)(*CRLF)`, `((*CRLF)a)`, `(*CRLF)a|(*LF)b` all
err 160) — with no semantic fact/predicate and no validator walk (`find_invalid_verb_construct` +
`is_start_option_position` DELETED). The verbs / MARK / shorthand that remain in
`directive_verb_nonquant` stay valid **anywhere** (including nested — `((*ACCEPT))`, `a(*PRUNE)b`).

### the named-directive classes — name-class-conditional argument shapes

```ebnf
directive_accept_named = "ACCEPT" directive_payload_colon?
directive_mark_named   = "MARK" directive_payload_colon_required
directive_verb_named   = directive_verb_name_nonaccept directive_payload_colon?
directive_limit_named  = directive_limit_name directive_payload_equals
directive_option_named = directive_option_name
directive_relaxed_named = ... directive_name_relaxed directive_payload_suffix?
```

**REGEX-PCRE2-FIDELITY.3.14 (release 1.1.83) — the EBNF now owns PCRE2's per-name-class ARGUMENT
shapes** (they previously lived in the host compile-contract; `.3.2` had already moved NAME
acceptance in). Every branch emits the same released carrier `{kind:"named", name:<string>,
payload:<payload>}`, so previously-accepted patterns are byte-identical. The classes, each pinned
against `pcre2test` 10.47:

| Branch | Names | Argument shape | Oracle boundary |
|---|---|---|---|
| `directive_accept_named` (quantifiable) | `ACCEPT` | optional `:`-payload, empty allowed | `(*ACCEPT)` / `(*ACCEPT:x)` ACCEPT; **quantifiable** — `(*ACCEPT)+` / `(*ACCEPT:x)+` ACCEPT |
| `directive_mark_named` | `MARK` | `:`-payload **required non-empty** | `(*MARK:x)` ACCEPT; `(*MARK)` / `(*MARK:)` REJECT (err 166); `(*MARK=x)` REJECT (err 160) |
| `directive_verb_named` | `FAIL F COMMIT PRUNE SKIP THEN` (the 6 non-ACCEPT verbs) | optional `:`-payload, empty allowed | `(*PRUNE)` / `(*PRUNE:)` / `(*PRUNE:x)` ACCEPT; `(*PRUNE=x)` / `(*SKIP=)` REJECT (err 160); **not quantifiable** — `(*PRUNE)+` REJECT (err 109) |
| `directive_limit_named` | `LIMIT_HEAP LIMIT_MATCH LIMIT_DEPTH LIMIT_RECURSION` | `=digits` **required**, value **≤ 4294967289** | `(*LIMIT_HEAP=500)` / `(*LIMIT_HEAP=4294967289)` ACCEPT; bare `(*LIMIT_HEAP)`, `(*LIMIT_HEAP=)`, `(*LIMIT_HEAP=abc)`, `(*LIMIT_HEAP:5)`, **`(*LIMIT_HEAP=4294967290)`** REJECT (err 160) |
| `directive_option_named` | the other 21 start options (`UTF UTF8 UTF16 UTF32 UCP NOTEMPTY NOTEMPTY_ATSTART NO_AUTO_POSSESS NO_DOTSTAR_ANCHOR NO_JIT NO_START_OPT CASELESS_RESTRICT TURKISH_CASING CR LF CRLF ANY NUL ANYCRLF BSR_ANYCRLF BSR_UNICODE`) | **bare only** | `(*UTF)` ACCEPT; `(*UTF:x)` / `(*UTF=5)` / `(*CR=5)` / `(*TURKISH_CASING=5)` REJECT (err 160) |
| `directive_relaxed_named` (`@profiles: ["relaxed"]`) | any **unrecognized** `[A-Za-z][A-Za-z0-9_-]*` name | any suffix (`:`/`=`/bare) | relaxed-profile catch-all; see below |

Name matching is **case-sensitive** (PCRE2 rejects `(*accept)`/`(*Skip)`); the name lists stay
ordered longest-first for prefix overlaps (`FAIL`>`F`, `UTF32/16/8`>`UTF`, …). This release fixed
two real accepts-invalid divergences the old validator missed (ledger **REGEX-0089**): PGEN
accepted `=digits` on non-LIMIT options (`(*UTF=5)`) and accepted bare `(*LIMIT_HEAP)` — PCRE2
rejects both (err 160).

The relaxed catch-all carries an inline negative lookahead excluding every RECOGNIZED name at a
name boundary (`:`/`=`/`)`), so the strict shapes above stay authoritative in **both** profiles
(`(*MARK)` rejects under `relaxed` too), while extended spellings (`(*LIMIT_HEAPX=5)`, `(*SKIPX)`)
still reach the catch-all under `relaxed`.

| Pattern | default (`pcre2`) | `relaxed` |
|---|---|---|
| `(*ACCEPT)` `(*FAIL:x)` `(*MARK:x)` `(*:x)` `(*UTF)` `(*LIMIT_MATCH=100)` | ACCEPT | ACCEPT |
| `(*FOO)` `(*BAR:x)` `(*FOO=x)` `(*MARKX)` `(*accept)` (unrecognized / wrong case) | REJECT | ACCEPT |
| `(*:)` `(*MARK)` `(*MARK:)` `(*SKIP=)` `(*UTF=5)` `(*LIMIT_HEAP)` (invalid shapes) | REJECT | REJECT |

**Quantifiability (release `1.1.87`, `REGEX-PCRE2-FIDELITY.3.20`, ledger REGEX-0096):** only
`(*ACCEPT)` may take a quantifier — grammar-encoded via the `directive_verb` /
`directive_verb_nonquant` split above. A quantified KNOWN non-ACCEPT directive rejects in **both**
profiles (`(*PRUNE)+`, `(*:x)+`, `(*UTF)+`, `(*LIMIT_HEAP=5)+` — err 109); a quantified UNKNOWN-name
verb (`(*foo)+`) stays `relaxed`-accepted (the catch-all is quantifiable) and default-rejected.

**LIMIT value range (release `1.1.88`, `REGEX-PCRE2-FIDELITY.3.21`, ledger REGEX-0097):** a LIMIT
`=value` is VALUE-bounded to **[0, 4294967289]** in **both** profiles — grammar-encoded on
`directive_payload_digits` (structurally, the `.3.16`/`.3.18` idiom). PCRE2's Horner overflow guard
caps the value at `429496728*10 + 9 = 4294967289` (`0xFFFFFFF9`), **not** u32 max `4294967295`
(which itself rejects, err 160). The bound is purely value-based: arbitrary leading zeros are fine
(`(*LIMIT_HEAP=00000000004294967289)` ACCEPT, `…4294967290` REJECT; all-zeros = value 0). `value`
stays the raw digit string (leading zeros preserved). This closed a latent accepts-invalid hole —
no validator ever bounded the LIMIT value.

Ownership status: the start-option **position** rule is GRAMMAR-owned since release
`1.1.102` (`REGEX-PCRE2-FIDELITY.4.8` — the `entry_concatenation` structural encoding
described above; the standalone validator walk was DELETED). The quantified-verb rule
(release 1.1.87) and the LIMIT value range (release 1.1.88) are grammar-owned too.
(Historically the position rule lived in the out-of-band validator; the `=`-form
position hole was fixed in release 1.1.83, ledger **REGEX-0090**.)

### `directive_mark_shorthand`

```ebnf
directive_mark_shorthand = ":" directive_payload_required
```

The `(*:name)` MARK shorthand — emits `{kind:"mark_shorthand", payload:<string>}`. Since `.3.14`
the payload is **required non-empty** (`(*:)` rejects, PCRE2 err 166 — this closed the
STIMULI-SIGNOFF duality-break class where the generator emitted `(*:)`).

### Payload rules

```ebnf
directive_payload_colon_required = ":" directive_payload_required   -> {separator: ":", value: $2}
directive_payload_colon  = ":" directive_payload_simple?            -> {separator: ":", value: $2}
directive_payload_equals = "=" directive_payload_digits             -> {separator: "=", value: $2}
directive_payload_digits = digit+ -> $text
directive_payload_required = directive_payload_core
                           | ( !")" builtin_any_char )+ -> $text
directive_payload_core = ( letter | digit | '_' )+ -> $text
directive_payload_simple = ( !")" builtin_any_char )* -> $text
```

All payload values are clean strings; the suffix objects are `{separator:":"|"=",
value:<string>}` — unchanged from the released carrier. `directive_payload_required` pairs a
positively-enumerated generatable core with PCRE2's full any-char-but-`)` superset; parsing is
identical to the plain superset (the longest-match tournament keeps the union exact) — the split
exists purely so stimuli generation, which has no native `builtin_any_char` emitter (tracked as
`STIMULI-SIGNOFF.14`), can emit required payloads.

`directive_payload_suffix` (`":"|"=" directive_payload_simple?` per branch) is now the
relaxed-only any-shape suffix reachable solely through `directive_relaxed_named`, and carries the
same `@profiles: ["relaxed"]` gate.

For `(*UTF8)`:

```json
"atom": {
  "type": "atom",
  "kind": "directive_verb",
  "body": { "kind": "named", "name": "UTF8", "payload": [] }
}
```

For `(*LIMIT_HEAP=500)` the payload is `{"separator": "=", "value": "500"}`; for `(*MARK:x)` it is
`{"separator": ":", "value": "x"}`; for `(*:x)` the body is `{"kind": "mark_shorthand",
"payload": "x"}`.

## `extended_class`

```ebnf
extended_class = "(?[" extended_class_content "])"
```

PCRE2 `(?[ ... ])` extended class. **Typed** at the atom level:

```json
{ "type": "atom", "kind": "extended_class", "body": [<elements>] }
```

`body` is the element array. Each element is the matched `extended_class_element`
shape: a typed escape object where the element is an escape (`(?[\p{L}])` →
`body: [{"type":"escape","kind":"property","name":"L","negated":false}]`), a bare
char string for regular chars, or — for a NESTED `[...]` — the nested bracket triple
in its raw form (`(?[[a][b]])` → `body: [["[",["a"],"]"], ["[",["b"],"]"]]`; the
nested content list holds the inner elements). The inner element rules are otherwise
un-annotated (their raw shapes appear inside the nested triples).

### `extended_class_content`

```ebnf
extended_class_content = extended_class_element*
```

Quantified-`*`.

### `extended_class_element`

```ebnf
extended_class_element = extended_class_nested
                       | escape
                       | extended_class_backspace_escape
                       | extended_class_regular
```

4-way Or. The matched alternative's shape appears.

### `extended_class_backspace_escape`

```ebnf
extended_class_backspace_escape = "\\b" -> {type: "escape", kind: "shorthand", char: "b"}
```

The class-context meaning of the dual-meaning escape `\b` (release `1.1.106`,
`PGEN-RGX-0089` / ledger `REGEX-0115`): in the pattern BODY `\b` is a
word-boundary **anchor** (and `\b*` rejects, PCRE2 err 109), but inside a
character class it means **backspace** (U+0008). Ordinary classes get that
meaning from the class escape family; the extended class routes other escapes
through the generic `escape` rule (whose shorthand set deliberately excludes
the anchor letters), so this dedicated branch carries the backspace meaning
for `(?[...])` — bare or nested:

```text
(?[\b])    → body: [{"char":"b","kind":"shorthand","type":"escape"}]
(?[[\b]])  → body: [["[",[{"char":"b","kind":"shorthand","type":"escape"}],"]"]]
```

The node shape is byte-identical to ordinary-class `[\b]`'s member and to the
pre-`1.1.82` extended-class shape. `(?[\B])` / `(?[\A])` stay **rejected**
(PCRE2 err 107 — not admitted by this branch). Oracle: `pcre2test` 10.47.

### `extended_class_nested`

```ebnf
extended_class_nested = "[" extended_class_content "]"
```

3-element Sequence.

### `extended_class_regular` and `extended_class_special`

```ebnf
extended_class_regular = letter | digit | whitespace | extended_class_special | unicode_char
extended_class_special = '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+'
                       | ',' | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@'
                       | '^' | '_' | '`' | '{' | '|' | '}' | '~'
```

5-way Or and 29-way Or respectively. Each emits the matched-char Terminal.

## `code_block`

```ebnf
code_block = code_block_lang | code_block_plain
```

2-way Or; both branches emit the same **typed** carrier:

```json
{ "type": "atom", "kind": "code_block", "lang": <lang>, "content": [<chars>] }
```

- `lang` is `null` for the plain Perl-style `(?{ ... })` form, or the language name
  string (`"lua"`, `"js"`, `"javascript"`, `"rhai"`, `"native"`, `"wasm"`) for the
  `(?{lang: ...})` form.
- `content` is the per-char array of the code body (concatenate to recover the code
  string). `(?{ ab })` → `{"lang": null, "content": [" ","a","b"," "]}`;
  `(?{lua: print(1)})` → `{"lang": "lua", "content": [" ","p","r","i","n","t","(","1",")"]}`.

See [Examples: Groups and Alternations](examples-groups-alt.md#code-block--lua-print1-typed)
for the worked extraction (including the PGEN-RGX-0082 `content` fix note).

### `code_block_plain` / `code_block_lang` / `code_lang`

```ebnf
code_block_plain = "(?{" code_content "})"
code_block_lang  = "(?{" code_lang ":" ws? code_content "})"
code_lang        = "lua" | "js" | "javascript" | "rhai" | "native" | "wasm"
```

The two syntactic forms behind the one carrier above.

### `code_content`, `code_element`, `code_string_*`, `code_balanced_braces`, `code_escaped_char`, `code_regular_char`, `code_safe_special`, `code_not_quote_or_backslash`, `code_not_squote_or_backslash`

The internal grammar for parsing balanced-brace code-block bodies. All un-annotated. Consumers extracting code-block payloads typically just want the raw text between `(?{` and `})`, which can be obtained from the `span` field of the code_block atom (the original input slice).

## `comment_group`

```ebnf
comment_group = "(?#" comment_text? ")"
```

**Typed** — the comment text arrives as one clean string:

```json
{ "type": "atom", "kind": "comment", "text": "note" }
```

`(?#note)` → `text: "note"`; the empty comment `(?#)` → `text: ""`. (The
`comment_text`/`comment_char` sub-rules are folded into the joined `text` string —
no per-char walking.)

## Auxiliary lexical helpers

| Rule | Form | Shape |
|---|---|---|
| `letter` | `/([A-Za-z])/` | `Terminal(<char>)` |
| `digit` | `/([0-9])/` | `Terminal(<char>)` |
| `nonzero_digit` | `/([1-9])/` | `Terminal(<char>)` |
| `hex_digit` | `/([0-9A-Fa-f])/` | `Terminal(<char>)` |
| `octal_digit` | `/([0-7])/` | `Terminal(<char>)` |
| `whitespace` | `/([ \t\n\r\f\v])/` | `Terminal(<char>)` |
| `unicode_char` | `/([^\x00-\x7F])/` | `Terminal(<char>)` |
| `any_char` | `/(...big char-class.../` | `Terminal(<char>)` |
| `special_char` | `/(...subset of any_char...)/` | `Terminal(<char>)` |
| `digits` | `/([0-9]+)/` with `@transform: str::parse::<usize>` | typed integer |
| `hex_digits` | `hex_digit+` | Quantified of `Terminal(<char>)` |
| `octal_digits` | `octal_digit+` | Quantified of `Terminal(<char>)` |
| `ws` | `whitespace+` | Quantified of `Terminal(<char>)` |
| `brace_ws` | `(' ' \| '\t')+` | Quantified of `Terminal(<char>)` |

`digits` is the annotated leaf (typed integer). The other lexical helpers emit raw
Terminal/Quantified shapes — but consumers rarely meet them: every typed parent above
folds its helpers into clean string/integer fields.

## Typing status — the campaign is complete

Every consumer-facing construct family is typed: the atom alternatives (literals as
bare strings; everything else as `{type, kind, ...}` objects — see the
[atom identification table](rules-atom.md#identification-table--what-kind-of-atom-is-this)),
the quantifier subtree (slice 6, post-1.1.34), anchors (slice 7), the escape subtree
(slices 14–17), the group/lookaround family (slice 23), the character class (slice 26)
and its items, and the callout/conditional/directive/code-block/comment/extended-class
constructs documented in this chapter.

The `pattern` / `alternation` / `alternative` OUTER carrier keeps its raw
`[<head>, <tail>]` nesting by design (it is the one structural shape every consumer
already walks — see [Walking the AST](walking-the-ast.md)); the shapes documented in
this book are the operative reference for it.
