# Lookaheads

A lookahead **asserts** what comes next without consuming any input. PGEN supports both PEG lookahead
operators, and a separate **lexical follow-restriction** annotation for token-boundary rules.

## Positive `&` and negative `!`

| Operator | Name | Asserts |
| --- | --- | --- |
| `&X` | positive lookahead | the next input **does** match `X` (then backtracks, consuming nothing) |
| `!X` | negative lookahead | the next input does **not** match `X` |

Both bind as a prefix to the single primary element on their right (a terminal, a rule reference, or a
group). They are heavily used in `grammars/regex.ebnf` — 12 positive and 39 negative lookaheads — to
disambiguate ordered-choice branches.

```ebnf
# consume characters until (but not including) a closing ')'
comment_text := ( !")" builtin_any_char )* -> $text

# only take the 'else' branch when an 'else' keyword actually follows
if_tail := &"else" else_clause
```

### The "any char except …" idiom

Negative lookahead plus a [`builtin_any_char` built-in](terminals.md) is how a self-hosting grammar
expresses a negated character set without a regex class:

```ebnf
# a shorthand escape that is NOT one of the reserved letters/digits
simple_escape := !"o{" !"x{" !"p{" !"P{" builtin_any_char -> {type: "escape", char: $5}
```

Each leading `!"…"` is a zero-width assertion, so the matched character is captured by the *first
consuming* element — note the positional reference `$5` skips the four non-consuming lookaheads. Count the
lookaheads when you write the `$N`.

## Lexical follow-restriction: `[> … ]` and `[>! … ]`

PGEN has a dedicated **lexical annotation** for the common "this token must (not) be followed by a word
character" requirement — the kind of thing keyword/identifier boundaries need. Written as a standalone
annotation line that binds to the **following** rule (like the `@…` directives):

```ebnf
# the macro name must NOT be immediately followed by another word char,
# so `define_CV` does not split into the keyword `define` + `_CV`
[>! /\w/]
define_directive := "`define" macro_name macro_body
```

| Form | Meaning |
| --- | --- |
| `[> X ]` | the rule's match **must** be followed by `X` |
| `[>! X ]` | the rule's match must **not** be followed by `X` |

This is the "4th pillar" lexical-faithfulness construct (built for the stimuli generator) and is used in
`grammars/systemverilog_preprocessor.ebnf` for word-boundary (`\b`) enforcement on directive keywords. It
is genuinely codegen-consumed. The deep reference is the platform book's
[Lexical Annotations](../../book/src/lexical-annotations.md) chapter.

> Distinguish the three `[ … ]` shapes again: `[ X ]` (element-level **optional**), `[> X ]` / `[>! X ]`
> (lexical **follow-restriction** — note the `>`), and a regex class `/[X]/` (inside a `/…/` literal). The
> follow-restriction is the only one whose bracket opens with `>`.
