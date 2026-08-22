# Grammar and Scope

This chapter is the precise statement of the surface the `semantic_annotation` parser accepts, derived
from `grammars/semantic_annotation.ebnf` (version `2.0`).

## Entry: `@name: value`

Every semantic annotation is a `@`, a name, a colon, and a value:

```ebnf
semantic_annotation := "@" /\s*/ annotation_name /\s*/ ":" /\s*/ annotation_value
    -> {type: "semantic_annotation", name: $3, value: $7}
```

(`annotation` is an accepted compact alias for the same shape.) A directive is written on its own line
**above** the rule it applies to — the directive binds to the *following* rule:

```ebnf
@predicate: has_fact(type_name, $head)
known_type_identifier := identifier
```

## Annotation names: predefined vs. custom

```ebnf
annotation_name := (predefined_annotation | custom_annotation) -> $1
```

- **Predefined names** are a recognized set of standard metadata names the grammar lists explicitly —
  e.g. `type`, `category`, `kind`, `effect`, `precedence`, `associativity`, `scope`, `visibility`,
  `generate`, `transform`, `deprecated`, `description`, `version`, and many more (see the grammar for
  the full list). A predefined name resolves to the bare name string.
- **Custom names** are any other identifier (`/[a-zA-Z_][a-zA-Z0-9_]*/`), resolving to
  `{type: "custom_annotation", name: …}`.

> **Important:** the value grammar is essentially *name-agnostic* — it parses `@<name>: <value>` for
> any name. The **steering** directives (`@predicate`, `@emit_fact`, `@fact_kind`, `@open_scope`,
> `@profiles`, `@predicate_def`, `@sample`, `@probe_sample`, …) are parsed through this same surface
> (most of them as `custom_annotation` names, with their payload parsed by the value language), and it
> is PGEN's **AST pipeline** that interprets each name's *meaning*. So a directive's *syntax* is
> documented here; its *behavior* is documented in [Steering Directives](steering-directives.md) and
> governed by the normative spec + steering control matrix.

## The value language

The value after the colon is one of four families (ordered choice):

```ebnf
annotation_value := (primitive_value | structured_value | expression_value | reference_value) -> $1
```

| Family | Includes | Example values |
| --- | --- | --- |
| **primitive** | string / numeric / boolean / null / identifier | `"pure"`, `42`, `0xFF`, `true`, `null`, `expression` |
| **structured** | array / object / tuple / set / map | `[A, B]`, `{level: 5}`, `(a, b)`, `#{x, y}`, `{k => v}` |
| **expression** | arithmetic / logical / comparison / conditional / function call / lambda | `base * 2 + offset`, `a && b`, `x > 0`, `c ? t : f`, `f(x)`, `x => x` |
| **reference** | type / rule (`$…`) / symbol (`%…`) / path / URL | `Map<String,Int>`, `$head`, `%TOKEN`, `./p.sv`, `https://…` |

Each family is detailed, with the produced node shapes, in
[Annotation Values and References](values-and-references.md).

## Scope and non-goals

- This grammar parses the `@name: value` **syntax** and classifies the value. It does **not**
  itself enforce the steering *semantics* — that is the AST pipeline's job (see
  [Steering Directives](steering-directives.md)).
- Whitespace between tokens is handled implicitly (`/\s*/` interleaves the productions); line/block/doc
  comments (`//`, `/* */`, `///`) are recognized.
- The bootstrap backend is intentionally permissive — an unrecognized or malformed value falls back to
  a `Raw` classification rather than hard-failing (see [Backends](backends.md) and
  [AST Envelope](ast-envelope.md)).
