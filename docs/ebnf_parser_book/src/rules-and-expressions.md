# Rules and Expressions

A rule body is an **expression**: terminals and rule references combined with ordered choice, sequencing,
grouping, and optionals. PGEN compiles a grammar to a **recursive-descent PEG parser** — so the semantics
here are PEG semantics, and ordering matters.

## Rule references (non-terminals)

Naming another rule in a body calls that rule:

```ebnf
statement      := assignment | if_statement | loop_statement
assignment     := lvalue "=" expression ";"
```

A reference to an *undefined* name resolves to a built-in matcher when the name is one of the
`any_char` family ([Terminals](terminals.md)); otherwise an undefined reference is a grammar defect that
`--lint-grammar` reports.

## Ordered choice (`|`)

`|` is **ordered choice**, not symmetric alternation. The parser tries each alternative left-to-right and
commits to the **first** that matches:

```ebnf
keyword := "module" | "macromodule" | "module_extern"
```

Because choice is ordered, **put the longer / more specific alternative first** when alternatives share a
prefix — otherwise the shorter one wins and strands the rest. This "ordered-choice shadowing" is exactly
what `--lint-grammar` warns about (see [Codegen Mental Model](codegen-model.md) and the platform book's
[Grammar Well-Formedness](../../book/src/grammar-wellformedness.md) chapter). Each non-last branch may also
carry its **own** inline `-> …` return annotation before the `|`:

```ebnf
sign := "+" -> {kind: "plus"}
      | "-" -> {kind: "minus"}
```

## Sequences

Writing elements one after another matches them in order:

```ebnf
port := direction data_type identifier ";"
```

Each element's result is captured positionally (`$1`, `$2`, …) for the return annotation. By default a
sequence with two or more elements does **not** get an implicit `-> $1` — see
[The Implicit / Passthrough Return Policy](return-policy.md).

## Grouping `( … )`

Parentheses group a sub-expression so a quantifier or choice applies to the whole group:

```ebnf
arg_list := expression ( "," expression )*
term     := ( "+" | "-" ) factor
```

## Optional `[ … ]`

`[ expr ]` is **optional** — equivalent to `( expr )?`. (This is the element-level `[ … ]`; see the
[Terminals](terminals.md) footgun — it is *not* a character class.)

```ebnf
declaration := data_type identifier [ "=" expression ] ";"
# identical to:
declaration := data_type identifier ( "=" expression )? ";"
```

## Quantifiers and lookaheads

Repetition (`?` `*` `+` `{n,m}`) and assertions (`&` `!`) are covered in their own chapters:
[Quantifiers](quantifiers.md) and [Lookaheads](lookaheads.md).

## Precedence and how to override it

From loosest to tightest binding:

1. ordered choice `|`
2. sequence (juxtaposition)
3. quantifier postfix (`?` `*` `+` `{…}`) and lookahead prefix (`&` `!`) on a single element
4. primary element (terminal, rule reference, group, optional)

Use `( … )` to override — e.g. `( a | b )+` repeats the *choice*, whereas `a | b+` is "`a`, or one-or-more
`b`".

## Constructs the codegen does NOT implement

The self-hosting meta-grammar `grammars/ebnf.ebnf` *describes* a number of extension constructs so that it
can parse grammars that use them, but the PGEN code generators **do not act on them**. Do **not** rely on
these — they are silently inert (or a defect) in a real grammar:

| Construct | Surface in `ebnf.ebnf` | Status |
| --- | --- | --- |
| Exception rules | `expr except (…)` | not implemented |
| Case control | `~"X"`, `~i"X"` | not implemented |
| Named captures (as syntax) | `name:pattern` | not implemented as an element |
| Parametric rules | `rule[param, …]` | not implemented |
| Template instantiation | `rule<Type, …>` | not implemented |
| Lexer modes | `mode name { … }` | not implemented |
| Grammar inheritance | `grammar X extends Y` | not implemented — use [includes](includes.md) |
| Import statements | `import "…" as ns` | not implemented — use [includes](includes.md) |
| Error productions | `error : sync/skip/panic` | not implemented |
| Semantic predicates | `{? … ?}` | not implemented — use a `@predicate` annotation instead |
| Action blocks | `{ … }` | not implemented — use `-> …` / `@…` instead |
| Epsilon as a construct | `ε` / `epsilon` / `empty` | recognized, but not a codegen primitive — express "optional" with `?`/`[ … ]` |

Two near-misses worth calling out so you reach for the supported mechanism:

- to gate a rule on context, do **not** write a `{? … ?}` semantic predicate — write a `@predicate`
  [semantic annotation](semantic-annotations.md), which the codegen genuinely consumes;
- to compose grammars, do **not** use `extends` / `import` — use the [include system](includes.md).
