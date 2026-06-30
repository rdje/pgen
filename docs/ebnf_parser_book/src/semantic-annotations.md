# Semantic Annotations

Where a [return annotation](return-annotations.md) (`-> …`) shapes the *value a rule returns*, a
**semantic annotation** (`@name: value`) shapes the *parser-generation behaviour* — gating a rule on facts
already seen, emitting facts into a semantic store, restricting a rule to a profile, transforming a
matched value, or steering stimuli generation. They are the normative mechanism for making a generated
parser **context-aware**.

This chapter is the grammar-author overview. The deep reference — the full `@name: value` value language,
the complete directive catalog, and the semantic-store lifecycle — is the dedicated
[semantic_annotation parser book](../../semantic_annotation_parser_book/src/welcome.md),
`docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`, and
`docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`.

## Placement

A standalone `@…` directive binds to the rule that **follows** it (authoritative: `grammars/ebnf.ebnf`,
`grammar_rule := annotation_list? rule_definition`). Place the directive directly above its rule; more than
one may stack:

```ebnf
@emit_fact: type_name
type_declaration := "typedef" data_type identifier ";"
```

```ebnf
@predicate: has_fact(type_name, $head)
known_type_identifier := identifier
```

The text after `@` is *semantic-annotation source* — its own small value language (literals, structured
`{…}` values, function calls, and `$ref` references). Its *meaning* per directive is the steering
contract.

## The directive catalog (overview)

The directives the AST pipeline interprets, grouped by what they do:

| Group | Directives | Purpose |
| --- | --- | --- |
| Store: declare | `@fact_kind` | declare a fact kind and its attributes (exportable, …) |
| Store: emit | `@emit_fact`, `@open_scope` | record a fact / open a lexical scope as the parser matches |
| Store: query (gate) | `@predicate` (+ `has_fact` / `lacks_fact` / `fact_attribute_equals` / `len_bounds` / `numeric_bounds` / `fact_count_at_least` / …) | accept or reject a rule based on the store |
| Store: compose | `@predicate_def` | name a reusable composed predicate |
| Cross-file | `@export_to_library`, `@import_from_library` | publish / consume facts across compilation artifacts |
| Profiles | `@profiles` | restrict a rule to a grammar profile (e.g. `sv_2017` vs `sv_2023`) |
| Value | `@transform`, `@semantic_value` | post-process a matched value |
| Stimuli | `@generate`, `@sample`, `@dispatch_table`, … | steer stimuli generation |
| Pragmas | `@stop_at_rule_boundary` | bound how far a sequence consumes |

The store-backed gating directives are the heart of context-aware parsing — for example, "only treat this
bare identifier as a type name if a `typedef` already emitted a `type_name` fact for it". A grammar rule
that categorizes a bare identifier into a tracked category (type / class / package / …) **should** consult
the store via a `@predicate` rather than matching the identifier unconditionally.

## A worked shape

```ebnf
# declare the fact kind
@fact_kind: { name: type_name, exportable: true }

# a typedef EMITS a type_name fact for the new name
@emit_fact: type_name
type_declaration := "typedef" data_type identifier ";" -> {kind: "typedef", name: $3}

# a use-site is a known type ONLY if that fact exists
@predicate: has_fact(type_name, $1)
known_type := identifier -> {kind: "type_ref", name: $1}
```

## Lexical annotations

The `[> … ]` / `[>! … ]` follow-restriction is a *lexical* annotation in the same family; it is documented
in [Lookaheads](lookaheads.md) and the platform book's
[Lexical Annotations](../../book/src/lexical-annotations.md) chapter.

## Where to go deep

- [semantic_annotation parser book](../../semantic_annotation_parser_book/src/welcome.md) — the value
  language, the full directive catalog, and the seven-stage semantic-store lifecycle.
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` — the normative directive semantics.
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md` — which knob steers which behaviour.
- The platform book's [Annotation System](../../book/src/annotation-system.md) chapter — the
  two-annotation-family overview.

> Reminder: PGEN has no `{? … ?}` semantic-predicate *syntax* (see
> [Rules and Expressions](rules-and-expressions.md)). Context gating is done with a `@predicate`
> annotation, which the codegen genuinely consumes.
