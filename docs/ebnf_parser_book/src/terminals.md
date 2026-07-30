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

## The complete built-in list

The `builtin_any_char` pair are the *whole* list
(`AstBasedGenerator::NATIVE_UNRESOLVED_REFERENCE_BUILTINS`) — exactly two names:

| Name | What it is |
| --- | --- |
| `builtin_any_char` | the single-UTF-8-character matcher above |
| `builtin_ascii_char` | the single-ASCII-character matcher above |

> **This list used to be longer, and every name that left it was hiding a bug.** Until
> `LANG-CAPABILITY-AUDIT.10.4` it also carried `true` and `false`, which compiled to matchers that
> *always succeeded and consumed nothing* — `probe := "T" true "T"` accepted `TT`. Until
> `LANG-CAPABILITY-AUDIT.10.3` it carried `semantic_annotation`, an `@`-to-end-of-line matcher that
> existed only because the EBNF meta-grammar referenced that name without defining it; the entry
> stood in for a broken `include` and, being line-bounded, could not match a multi-line
> `@dispatch: { … }` payload at all. `grammars/ebnf.ebnf` now defines the rule itself, and all
> three names are ordinary undefined references — the linter reports them and codegen emits a
> never-matching stub. If your grammar needs to *parse* annotation lines, define a rule for them,
> the way the meta-grammar does; if it merely needs to *carry* annotations, see
> [Semantic annotations](semantic-annotations.md) — those are read by PGEN, not by your parser.
>
> The reason this matters to *you*, not just to PGEN's maintainers, is the coupling: the linter's
> allowlist **is** this list, so anything on it is invisible to `undefined_references`. A short
> list is a check you can trust.

**Every other undefined name is an error.** A reference to a rule you never defined is reported by
`--lint-grammar` as `undefined_references=N (error)` naming the exact rule and reference, and
codegen emits a never-matching stub plus an unconditional warning — so the production dies loudly
rather than silently.

To match literal words like `true`, quote them. This is what every shipped grammar does:

```ebnf
# ✅ quoted terminals — ordinary, unambiguous, and unrelated to the list above
boolean_literal := ("true" | "false")
```

> **History (`LANG-CAPABILITY-AUDIT.10.4`).** `true` and `false` used to be on that list, and they
> compiled to matchers that **always succeeded and consumed nothing** — `probe := "T" true "T"`
> accepted `TT`, and `--lint-grammar` still reported `undefined_references=0`, because the linter
> consumes the same list as codegen. No tracked grammar referenced either name. Both were removed,
> so those names are now ordinary rule names you may use freely.

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
