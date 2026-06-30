# Grammar and Scope

This chapter is the precise statement of what the return-annotation language accepts, derived directly
from `grammars/return_annotation.ebnf` (version `2.0.0`).

## Entry and normalization

An annotation is, in full form, an optional arrow followed by an expression:

```ebnf
return_annotation := arrow expression
                   | arrow
                   | expression
-> $2

arrow := '->'
```

Before parsing, the host normalizes the input (this is the behavior captured in
`grammars/builtin_return_annotation.ebnf` for the bootstrap backend):

1. trim leading whitespace;
2. if the input starts with `-> ` or `->`, strip that prefix;
3. trim leading/trailing whitespace;
4. if the result is empty, return **passthrough**.

So all three of these are equivalent and mean *"return the rule's value unchanged"* (i.e. `$1`):

```text
->            # bare arrow
              # empty (no annotation body)
-> $1         # explicit passthrough
```

> **Passthrough** is the implicit default a rule gets when it has **no** explicit `-> …` and its body
> is a single element. The platform book's grammar-author guidance covers the full implicit-return
> policy (single-element body → synthetic `$1`; multi-element sequences and quantified bodies are
> *not* given an implicit `$1`). This book documents the language you write *inside* `-> …`.

## The expression hierarchy

Everything after the arrow is one **expression**. Expressions are an ordered choice (PEG-style — the
first matching alternative wins), from most specific to most general:

```ebnf
expression := flat_spread_expression      # $x**
            | spread_expression           # $x*
            | extraction_expression       # $x::N , $x::first , $x::last (with optional *)
            | property_access_expression  # $x.field
            | array_access_expression     # $x[i]
            | primary_expression

primary_expression := object_literal          # { … }
                    | array_literal           # [ … ]
                    | matched_text_reference  # $text
                    | positional_reference    # $1, $2, $42, $0
                    | string_literal          # "…"  or  '…'
                    | number_literal          # 42, -2.5
                    | boolean_literal         # true, false
                    | null_literal            # null
                    | identifier              # bare name (used as a target/key)
                    | '(' expression ')'      # grouping
```

Each family has its own chapter:

- references and literals → [References and Literals](references-and-literals.md);
- objects and arrays → [Objects and Arrays](objects-and-arrays.md);
- the `::` / `*` / `**` / `.` / `[…]` operators → [Operators](operators.md).

## Ordering matters

Because alternation is ordered, the grammar lists `flat_spread_expression` (`**`) **before**
`spread_expression` (`*`): `$2**` must be tried before `$2*` would otherwise consume only the first
star and strand the second. Likewise the extraction/access forms are listed before bare
`positional_reference`, so `$2::1` and `$1.value` bind as a whole rather than degrading to a plain
`$2` / `$1`.

## What the language is — and is not

| Accepted | Example |
| --- | --- |
| Passthrough | `->`, *(empty)*, `-> $1` |
| Positional reference | `$1`, `$2`, `$42` |
| Whole-match text | `$text`, `$0` |
| String / number / boolean / null literal | `"node"`, `42`, `-2.5`, `true`, `null` |
| Object / array literal | `{type: "node"}`, `[$1, $2*]`, `{}`, `[]` |
| Extraction | `$2::2`, `$2::first`, `$2::last`, `$2::2*` |
| Spread / flatten-spread | `$2*`, `$2**` |
| Property / index access | `$1.value`, `$1[0]`, `$1.a[0].b` |
| Grouping | `($2)` |

**Not** part of this language (common misconceptions):

- **No arithmetic or binary operators.** Parentheses are *grouping around a single expression* only.
  The `( $1 + $2 )` form that appears in an illustrative comment in the grammar source is **not**
  implemented — `+` is not a token, and `($1 + $2)` does **not** parse. Use grouping only to make
  precedence explicit (e.g. `($2)*`).
- **No function calls, no conditionals, no string concatenation.** Reshaping is structural
  (objects/arrays/references/extraction/spread), not computational. Per-element value transforms live
  in the *grammar* via `@transform`, not in the return annotation.
- **No escape decoding in string literals.** A string literal keeps its surrounding quotes and its
  raw contents (see [References and Literals](references-and-literals.md)).
