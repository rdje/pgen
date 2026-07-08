# Anchors, Backreferences, and Misc

This chapter covers the remaining rule families that don't fit cleanly into the per-subtree chapters: callouts, conditionals, directive verbs, code blocks, comment groups, extended classes, and the auxiliary lexical helpers.

## `callout`

```ebnf
callout = "(?C" callout_arg? ")"
```

The PCRE2 callout construct `(?C...)`. 4-element Sequence: `["(?C", <callout_arg?>, ")"]`.

### `callout_arg`

```ebnf
callout_arg = callout_number | callout_string
```

2-way Or. `callout_number` is the typed integer (annotated). `callout_string` is one of 8 string-delimited forms.

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

Each callout_*_string rule is `[<delim>, <payload-Quantified>, <delim>]` (or `<open>, <payload>, <close>` for the brace form).

For `(?C12)`:

```json
"atom": [
  "(?C",
  12,             // typed integer from digits
  ")"
]
```

For `(?C"comment")`:

```json
"atom": [
  "(?C",
  ["\"", [<chars>], "\""],
  ")"
]
```

## `conditional`

```ebnf
conditional = "(?(" condition ")" yes_branch ("|" no_branch)? ")"
```

PCRE2 conditional pattern. 5-or-6-element Sequence:

```json
[
  "(?(",
  <condition shape>,
  ")",
  <yes_branch shape>,
  <optional ["|", <no_branch>] pair>,
  ")"
]
```

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

9-way Or. The branches cover the various PCRE2 condition forms. Most consumers identify the kind by inspecting the first element of the matched alternative's content.

### `define_condition`

```ebnf
define_condition = "DEFINE"
```

`Terminal("DEFINE")`.

### `version_condition`

```ebnf
version_condition = "VERSION" version_operator version_number
```

3-element Sequence: `["VERSION", <op>, <version-num>]`.

### `version_operator`

```ebnf
version_operator = ">=" | "="
```

`Terminal(">=")` or `Terminal("=")`.

### `version_number`

```ebnf
version_number = digits ("." digits)?
```

2-element Sequence: `[<digits>, <optional [".", <digits>] pair>]`. Both digits are typed integers.

### `condition_callout_assertion`

```ebnf
condition_callout_assertion = condition_callout "(" condition_assertion
```

3-element Sequence with a callout-prefixed assertion.

### `condition_callout`

```ebnf
condition_callout = "?C" callout_arg? ")"
```

4-element Sequence (note: includes the trailing `)` because this rule appears inside the larger `(?(?C...)...)` pattern).

### `condition_assertion`

```ebnf
condition_assertion = "?=" pattern
                    | "?!" pattern
                    | "?<=" pattern
                    | "?<!" pattern
                    | alpha_condition_assertion
```

5-way Or for the various assertion forms.

### `alpha_condition_assertion`

```ebnf
alpha_condition_assertion = "*" atomic_alpha_lookaround_name ":" pattern?
```

The PCRE2 `(*pla:...)`-style assertion in condition position.

### `recursion_condition`

```ebnf
recursion_condition = "R" digits?
                    | "R&" name
```

2-way Or — recursion-by-number or recursion-by-name.

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
                               | directive_limit_named
                               | directive_option_named
                               | directive_mark_shorthand
```

The quantifiable body is the ACCEPT verb + the `@profiles:["relaxed"]` unknown-name catch-all (so
relaxed `(*foo)+` stays accepted). The non-quantifiable body is the 6 non-ACCEPT verbs, MARK, the
`(*:x)` shorthand, LIMIT, and the bare start options. Both bodies pass their branch's object through
unchanged.

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

Still validator-owned (both profiles) until the capstone deletes the compile-contract: the
start-option **position** rule (`a(*UTF)` and `a(*LIMIT_HEAP=500)` reject — the `=`-form position
hole was fixed in release 1.1.83, ledger **REGEX-0090**). The quantified-verb rule (release 1.1.87,
above) and the LIMIT value range (release 1.1.88, above) are now grammar-owned, no longer
validator-owned.

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

PCRE2 `(?[ ... ])` extended class. 3-element Sequence: `["(?[", <content>, "])"]`.

### `extended_class_content`

```ebnf
extended_class_content = extended_class_element*
```

Quantified-`*`.

### `extended_class_element`

```ebnf
extended_class_element = extended_class_nested
                       | escape
                       | extended_class_regular
```

3-way Or. The matched alternative's shape appears.

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

2-way Or.

### `code_block_plain`

```ebnf
code_block_plain = "(?{" code_content "})"
```

3-element Sequence: `["(?{", <content>, "})"]`.

### `code_block_lang`

```ebnf
code_block_lang = "(?{" code_lang ":" ws? code_content "})"
```

5-element Sequence: `["(?{", <lang>, ":", <ws?>, <content>, "})"]`.

### `code_lang`

```ebnf
code_lang = "lua" | "js" | "javascript" | "rhai" | "native" | "wasm"
```

6-way Or. `Terminal(<lang-name>)`.

### `code_content`, `code_element`, `code_string_*`, `code_balanced_braces`, `code_escaped_char`, `code_regular_char`, `code_safe_special`, `code_not_quote_or_backslash`, `code_not_squote_or_backslash`

The internal grammar for parsing balanced-brace code-block bodies. All un-annotated. Consumers extracting code-block payloads typically just want the raw text between `(?{` and `})`, which can be obtained from the `span` field of the code_block atom (the original input slice).

## `comment_group`

```ebnf
comment_group = "(?#" comment_text? ")"
```

3-element Sequence: `["(?#", <text?>, ")"]`.

### `comment_text` and `comment_char`

```ebnf
comment_text = comment_char*
comment_char = letter | digit | whitespace | comment_special | unicode_char
```

Quantified-`*` of chars. Concatenate to recover the comment text.

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

`digits` is the lone annotated leaf. The rest are un-annotated and emit raw Terminal/Quantified shapes.

## What's missing — TBD slices

The following constructs are syntactically supported by the grammar but their AST shape will be cleaned up in future task #40 slices:

- Most `atom` alternatives (`literal`, `escape`, `dot`, `backreference`, `quoted_literal`, `posix_word_boundary_alias`, `char_class`, the group family) — eventually each gets a typed `{type: "...", ...}` shape. The `anchor` alternative landed typed in slice 7 (post-1.1.35) — see [Examples: Anchors and Boundaries](examples-anchors.md).
- `pattern`, `alternation`, `alternative` — eventually a clean `{type: "alternation", alternatives: [...]}` flat shape replaces the current 4-deep raw nesting.
- The full character-class subtree — eventually `{type: "char_class", negated: <bool>, items: [...]}` with each item itself a typed shape.

The `quantifier` subtree (`digits`, `quant_suffix`, `counted_quantifier_body`, `counted_quantifier`, `quant_base`, `quantifier`) is fully typed as of slice 6 (post-1.1.34). See [Quantifier Subtree](rules-quantifier.md).

Until the remaining slices land, the per-rule shapes documented above are the operative reference.
