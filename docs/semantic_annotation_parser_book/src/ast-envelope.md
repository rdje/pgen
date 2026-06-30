# AST Envelope Structure

When you parse semantic-annotation source through the embedding API, you get back a structured
**annotation AST**. This chapter is the canonical reference for its shapes.

## Top-level shape

Every annotation parses to a `semantic_annotation` node carrying the directive name and the value:

```ebnf
semantic_annotation := "@" /\s*/ annotation_name /\s*/ ":" /\s*/ annotation_value
    -> {type: "semantic_annotation", name: $3, value: $6}
```

- `name` is the bare name for a **predefined** name, or `{type: "custom_annotation", name: …}` for a
  custom one.
- `value` is one of the value-language node shapes (see [Annotation Values and
  References](values-and-references.md) for the full table — `{type: "string", …}`,
  `{type: "integer", …}`, `{type: "array", …}`, `{type: "object", …}`, `{type: "function_call", …}`,
  `{type: "rule_reference", …}`, and so on).

## Worked example

```text
@precedence: {level: 5, associativity: "left"}
```

parses to (abridged):

```json
{
  "type": "semantic_annotation",
  "name": "precedence",
  "value": {
    "type": "object",
    "properties": [
      { "type": "property", "key": "level",
        "value": { "type": "integer", "value": "5", "base": 10 } },
      { "type": "property", "key": "associativity",
        "value": { "type": "string", "value": "\"left\"", "quote_style": "double" } }
    ]
  }
}
```

## Bootstrap classification: TransformExpr / Structured / Raw

The **bootstrap** backend (see [Backends](backends.md)) does not parse the full value grammar; instead
it classifies a payload into one of three envelopes, in this strict order:

1. **TransformExpr** — the payload contains both `::parse::<` and `>().unwrap_or(` (a procedural,
   marker-based test, not structural parsing). Steers canonical transform code generation (emits the
   `TransformedTerminal` code path).
2. **Structured** — the trimmed payload is accepted by the bootstrap structured-value parser: scalar
   strings, numbers (int/decimal/scientific), booleans/null-likes, rule references (`$N` / `$name`),
   dotted identifiers, arrays, and objects.
3. **Raw** — anything else (including empty payloads) — kept as arbitrary text, the fallback.

The classification is best-effort and never hard-fails on the bootstrap path: a malformed or unknown
structured payload simply falls back to `Raw`.

## Directive compilation forms

When the AST pipeline compiles the interpreted directives into a generated parser, they become:

- **effect directives** — for `@emit_fact`, `@open_scope`, `@close_scope` (operations over scoped
  semantic runtime state);
- **typed predicate directives** — for `@predicate` (with explicit phase defaults pre/post/branch and
  object/scalar payloads);
- **compiled rule views** — a rule's directives are split across *pre predicates*, *post predicates*,
  *branch predicates*, and *effect directives* as appropriate to each directive's phase.

These compiled forms are internal to the AST pipeline and are not part of the downstream parser
contract; the downstream contract is parser acceptance, the diagnostics, and the family/backend
selection surface (see [Public API](public-api.md)).
