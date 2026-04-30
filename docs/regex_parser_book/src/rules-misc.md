# Anchors, Backreferences, and Misc

This chapter covers the remaining rule families that don't fit cleanly into the per-subtree chapters: callouts, conditionals, directive verbs, code blocks, comment groups, extended classes, and the auxiliary lexical helpers.

## `callout`

```ebnf
callout = "(?C" callout_arg? ")"
```

The PCRE2 callout construct `(?C...)`. 4-element Sequence: `["(?C", <callout_arg?>, ")"]`.

### `callout_arg`

```ebnf
callout_arg = digits | callout_string
```

2-way Or. `digits` is the typed integer (annotated). `callout_string` is one of 8 string-delimited forms.

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
directive_verb = "(*" directive_body ")"
```

3-element Sequence: `["(*", <body>, ")"]`.

### `directive_body`

```ebnf
directive_body = directive_named | directive_mark_shorthand
```

2-way Or.

### `directive_named`

```ebnf
directive_named = directive_name directive_payload_suffix?
```

2-element Sequence.

### `directive_name`

```ebnf
directive_name = directive_name_start directive_name_continue*
```

2-element Sequence: `[<first-char>, <Quantified of remaining chars>]`.

### `directive_payload_suffix`

```ebnf
directive_payload_suffix = ":" directive_payload_simple?
                         | "=" directive_payload_simple?
```

2-way Or.

### `directive_payload_simple`

```ebnf
directive_payload_simple = directive_payload_char*
```

Quantified-`*` of payload chars.

### `directive_payload_char`

```ebnf
directive_payload_char = /([^)])/
```

Single-character regex matching anything except `)`. Emits `Terminal(<char>)`.

For `(*UTF8)`:

```json
"atom": [
  "(*",
  [
    [<directive_name for "UTF8">],
    []   // no payload suffix
  ],
  ")"
]
```

For `(*MARK:label)`:

```json
"atom": [
  "(*",
  [
    [<directive_name for "MARK">],
    [":", [<chars: l,a,b,e,l>]]
  ],
  ")"
]
```

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

- All `atom` alternatives (`literal`, `escape`, `anchor`, `dot`, `backreference`, `quoted_literal`, `posix_word_boundary_alias`, `char_class`, the group family) — eventually each gets a typed `{type: "...", ...}` shape.
- The `quantifier` rule itself — eventually `{type: "quantifier", min, max, greediness}`.
- `pattern`, `alternation`, `alternative` — eventually a clean `{type: "alternation", alternatives: [...]}` flat shape replaces the current 4-deep raw nesting.
- The full character-class subtree — eventually `{type: "char_class", negated: <bool>, items: [...]}` with each item itself a typed shape.

Until those land, the per-rule shapes documented above are the operative reference.
