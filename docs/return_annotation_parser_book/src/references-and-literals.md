# References and Literals

These are the atoms of the return-annotation language: the things you reference from the match, and
the constant values you write inline.

## Positional references — `$N`

A `$` followed by an integer refers to the **Nth captured element** of the rule body (1-based):

```ebnf
positional_reference := '$' integer
-> {type: "positional", index: $2}
```

| You write | Means | Parsed node |
| --- | --- | --- |
| `$1` | the first captured element | `{type: "positional", index: 1}` |
| `$3` | the third captured element | `{type: "positional", index: 3}` |
| `$42` | the 42nd captured element | `{type: "positional", index: 42}` |

The index is a true number (the `integer` rule carries a `@transform` that parses the digits to an
`i64`), not a string. Indices count the **top-level elements of the rule body** — a sequence
`a b c` exposes `$1 $2 $3`; what a group or quantifier exposes is covered in
[Operators](operators.md).

## Whole-match text — `$text` (and `$0`)

`$text` references the rule's **entire matched source text** as a single string — the native
equivalent of writing a `/…/` capture just to recover the raw span:

```ebnf
matched_text_reference := '$' 'text'
-> {type: "matched_text"}
```

`$0` is an accepted **alias** for `$text`: it parses as `positional_reference` with index `0`, and the
host maps index `0` to the same whole-match-text node. Both lower to `UnifiedReturnAST::MatchedText` in
`rust/src/ast_pipeline/unified_return_ast.rs`.

```text
-> $text      # the whole matched text as one string
-> $0         # identical
```

This is the idiomatic way to return the matched text of a quantified or multi-element body without
keeping the structural tree (introduced by REGEX-SELF-HOSTING so a grammar need not depend on Rust's
regex engine just to recover a span).

## String literals — `"…"` and `'…'`

Either quote style is accepted; the parsed value is the **inner content**, with the surrounding quotes
removed:

```ebnf
string_literal := ('"' string_content_double '"' | "'" string_content_single "'")
-> {type: "string", value: $2}

string_content_double := /[^"]*/
string_content_single := /[^']*/
```

| You write | Parsed value |
| --- | --- |
| `"node"` | `{type: "string", value: "node"}` |
| `'x'` | `{type: "string", value: "x"}` |
| `""` | `{type: "string", value: ""}` |

Two important constraints:

- **No escape decoding.** The content is matched by `[^"]*` / `[^']*` — every character except the
  closing quote is taken literally. A backslash is a literal backslash; there is **no** `\n`, `\t`, or
  `\"` decoding.
- **No embedded matching quote, and `\"` is not an escape.** Because the content stops at the first
  closing quote, you cannot place a `"` inside a `"…"` string. The annotation language does **not**
  support a `\"` escape (the bootstrap parser silently fails on it). If you need a sentinel that would
  contain a quote, use a plain identifier/text label form instead of an embedded-quote string.

Both quote forms produce the *same* typed `{type: "string", value: …}` node. (This was not always
true — see the `string_literal` broadcast fix in [Schema and Versioning](schema-and-versioning.md);
single-quoted strings briefly produced a raw sequence shape. Consumers should expect the typed shape.)

## Number literals — `42`, `-2.5`

```ebnf
number_literal := float | integer

@transform: str::parse::<f64>().unwrap_or(0.0)
float := /[-+]?[0-9]+\.[0-9]+(?:[eE][-+]?[0-9]+)?/

@transform: str::parse::<i64>().unwrap_or(0)
integer := /[-+]?[0-9]+/
```

- An **integer** (`42`, `-7`, `+3`) parses to an `i64`.
- A **float** (`-2.5`, `1.0e9`) parses to an `f64`. A float requires a fractional part — `2.` and `.5`
  are not floats; `2` is an integer. The optional exponent (`e`/`E` with optional sign) is supported.

Number literals lower to native JSON numbers in the annotation AST (see
[AST Envelope](ast-envelope.md)). Use them for constant fields, e.g. `{min: 0, max: 7}`.

## Boolean and null literals

```ebnf
boolean_literal := 'true' | 'false'
null_literal    := 'null'
-> {type: "null"}
```

- `true` / `false` are boolean literals and lower to native JSON booleans.
- `null` is the language's sixth value type and lowers to a `{type: "null"}` node (JSON `null`). It is
  used by typed-AST shape annotations to mark an absent/unbounded field explicitly — e.g.
  `{min: $1, max: null}` for an open-ended `{n,}` quantifier in `regex.ebnf`.

## Identifiers

```ebnf
identifier := /[a-zA-Z_][a-zA-Z0-9_]*/
```

A bare identifier is an alphanumeric/underscore name beginning with a letter or underscore.
Identifiers appear in two roles you will actually use:

- as **object property keys** — `{key: $1}` (the `key` is an identifier); see
  [Objects and Arrays](objects-and-arrays.md);
- as the **property name** in property access — `$1.value` (the `value` is an identifier); see
  [Operators](operators.md).
