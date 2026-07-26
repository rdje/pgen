# Terminals

A **terminal** matches literal input text. PGEN's EBNF supports string and character literals, regex
literals, and the native `builtin_any_char` built-ins. This chapter is the authoritative list of what the codegen
actually matches — and one important footgun about character classes.

## String literals

Double-quoted and single-quoted string literals match their contents exactly
(`rust/src/ebnf_frontend.rs`):

```ebnf
kw_module  := "module"
semicolon  := ";"
quote_char := '"'        # a single-quoted literal can hold a double-quote
apostrophe := "'"        # …and vice-versa
```

Both quote styles are first-class and used throughout the shipped grammars. Choose whichever avoids
escaping the delimiter (e.g. `'"'` for a double-quote, `"'"` for an apostrophe — both appear in
`grammars/regex.ebnf`).

### Raw strings

A raw string `r"…"` matches its contents with **no escape processing** — backslashes are literal:

```ebnf
literal_backslash_d := r"\d"     # matches the two characters backslash, d
```

### Escapes in ordinary strings

Inside ordinary (non-raw) string literals, standard escapes are decoded by the frontend
(`rust/src/ebnf_frontend.rs`): `\n`, `\t`, `\r`, `\\`, `\"`, `\'`, and the Unicode escapes `\uXXXX`
(4 hex digits) and `\UXXXXXXXX` (8 hex digits).

```ebnf
newline := "\n"
tab     := "\t"
```

## Regex literals

A regex literal `/pattern/` matches against a regular expression, with optional trailing flags
(`/pattern/flags`):

```ebnf
identifier := /[a-zA-Z_][a-zA-Z0-9_]*/
number     := /\d+(\.\d+)?/
ws         := /\s+/
```

Regex literals are where **character classes** (`[abc]`, `[^abc]`), **ranges** (`[a-z]`), and the regex
shorthands (`\d`, `\w`, `\s`, …) live in a PGEN grammar. They are the most concise way to express a
lexical token.

> **Self-hosting guidance.** The regex grammar (`grammars/regex.ebnf`) deliberately avoids `/…/` literals
> so the generated regex parser does not call Rust's own regex engine (the `REGEX-SELF-HOSTING` campaign).
> For a *general* grammar that is not trying to be regex-engine-independent, `/…/` literals are perfectly
> idiomatic and widely used (e.g. `grammars/systemverilog.ebnf`). If you specifically need to avoid the
> embedded regex engine, prefer string literals and the `builtin_any_char` built-ins below.

## The `builtin_any_char` built-ins

PGEN provides two native single-character matchers that need no regex engine
(`rust/src/ast_pipeline/ast_based_generator.rs`). You reference them by name like a rule, but they
are built in — you do **not** define them:

| Built-in | Matches |
| --- | --- |
| `builtin_any_char` | any single UTF-8 character |
| `builtin_ascii_char` | any single ASCII character |

> ⚠️ **The `builtin_` prefix is part of the name, not decoration.** There is no un-prefixed
> `any_char` / `ascii_char` built-in. `grammars/regex.ebnf` defines its *own* ordinary rule named
> `any_char` (`any_char = letter | digit | whitespace | special_char | unicode_char`, line 2239),
> which is why the un-prefixed spelling appears throughout that grammar. Copy it into a grammar that
> does not define it and you get a hard error — measured:
> `[error] … rule 'start' references UNDEFINED rule 'any_char' — codegen emits a never-matching stub`.
> The prefix exists (director 2026-06-07) precisely so a primitive can never be shadowed by a
> same-named grammar rule like that one.

The canonical idiom — used all over `grammars/regex.ebnf` — is "match a run of characters until a
delimiter", combined with a [negative lookahead](lookaheads.md) and the `$text`
[whole-match capture](return-annotations.md):

```ebnf
# match every char up to (but not including) a closing ')'
comment_text := ( !")" builtin_any_char )* -> $text

# a single shorthand escape: backslash already consumed, take the next char
control_escape := "c" builtin_any_char -> {type: "escape", kind: "control", char: $2}
```

This pair plus lookaheads is how a self-hosting grammar expresses "any character except …" without a
character class.

## The reserved-name footgun: `true` and `false` match EMPTY

The `builtin_any_char` pair are not the only names PGEN treats as built in. The full list codegen
recognizes is `builtin_any_char`, `builtin_ascii_char`, `semantic_annotation`, `true` and `false`
(`AstBasedGenerator::NATIVE_UNRESOLVED_REFERENCE_BUILTINS`). Only the first two behave the way the
table above describes.

> ⚠️ **Do not name a rule `true` or `false`.** If your grammar *references* either name without
> defining it, codegen does not report an undefined reference — it synthesizes a matcher that
> **always succeeds and consumes nothing**.

```ebnf
# ⛔ DON'T: `true` here is not your rule and not the text "true"
flag := "flag=" true
```

That grammar accepts `flag=` — with nothing after the `=`. It also *rejects* `flag=true`, because
the built-in never consumes the four characters; the node it returns carries the payload `"true"`
regardless of what the input actually said. And `--lint-grammar` reports
`undefined_references=0`, because the linter consumes the same built-in list as codegen: a name on
that list is invisible to the undefined-reference check by construction.

The contrast with a genuine built-in is exact — same list, opposite behaviour:

| grammar | input | verdict |
| --- | --- | --- |
| `probe := "T" true "T"` | `TT` | ⛔ **accepted** — `true` matched empty |
| `probe := "T" true "T"` | `TtrueT` | rejected — `true` is zero-width, not a literal |
| `probe := "C" builtin_any_char "C"` | `CzC` | accepted — the built-in consumed one character |
| `probe := "C" builtin_any_char "C"` | `CC` | rejected — the built-in refused to match empty |

To match the literal words, quote them — which is what every shipped grammar already does, and it is
completely unaffected by any of the above:

```ebnf
# ✅ DO: quoted terminals, not rule references
boolean_literal := ("true" | "false")
```

This is a known defect, not a designed capability: no tracked grammar references either name, and
removing the trap is tracked by task-tree leaf `LANG-CAPABILITY-AUDIT.10.4`. Until it lands, treat
both names as reserved.

## The character-class footgun: `[ … ]` is OPTIONAL, not a class

This is the single most important terminal rule to internalize:

> At the **grammar-element level**, `[ … ]` means **optional** — it is exactly equivalent to `( … )?`.
> It is **not** a character class.

The EBNF frontend lowers a `[` to a group-open `(` and a `]` to a group-close `)` followed by the `?`
quantifier (`rust/src/ebnf_frontend.rs`). So:

```ebnf
# WRONG if you meant "one lowercase letter": this matches an OPTIONAL run of the
# rule references a, -, and z — almost never what you want.
maybe := [ a-z ]

# RIGHT — a character class belongs inside a regex literal:
lower := /[a-z]/

# RIGHT — [ expr ] as optional:
signed := [ "-" ] digits        # same as ( "-" )? digits
```

Keep the two `[ … ]` meanings straight:

| Where it appears | Meaning |
| --- | --- |
| grammar-element level (`rule := [ X ] Y`) | **optional** — `( X )?` |
| inside a regex literal (`/[a-z]/`) | a **character class** |
| inside a `-> …` return annotation (`-> [ $1, $2 ]`) | an **array** literal (see [Return Annotations](return-annotations.md)) |

When you see `[]` in `grammars/regex.ebnf`'s return annotations (e.g. `quantifier: []`), that is the
return-annotation **empty array**, not a character class and not an optional — context disambiguates all
three.
