# AST Envelope Structure

When you parse semantic-annotation source through the embedding API, you get back a structured
**annotation AST**. This chapter is the canonical reference for its shapes.

## Top-level shape

Every annotation parses to a `semantic_annotation` node carrying the directive name and the value:

```ebnf
semantic_annotation := "@" /\s*/ annotation_name /\s*/ ":" /\s*/ annotation_value
    -> {type: "semantic_annotation", name: $3, value: $7}
```

- `name` is the bare name for a **predefined** name, or `{type: "custom_annotation", name: …}` for a
  custom one.
- `value` is one of the value-language node shapes (see [Annotation Values and
  References](values-and-references.md) for the full table — `{type: "string", …}`,
  `{type: "integer", …}`, `{type: "array", …}`, `{type: "object", …}`, `{type: "function_call", …}`,
  `{type: "rule_reference", …}`, and so on).

> ⛔ **`$7`, not `$6` — and until 2026-08-23 this grammar said `$6`.** A positional `$N` counts
> **every** top-level element of the rule body, the `/\s*/` layout regexes included, so `$6` is the
> third separator and `annotation_value` is `$7`. The separator always matches the empty string, so
> the shipped parser published `value: ""` for **every** annotation — measured, 12 of 12 value shapes
> — while this chapter documented the populated shape below. Fixed in
> `GRAMMAR-WELLFORMED.H.16.7`; the example that follows is now a verbatim parser dump rather than a
> hand-written one, for exactly that reason.

## Worked example

```text
@precedence: {level: 5, associativity: "left"}
```

parses to **exactly** this (`parseability_probe --parse-dump-ast-pretty semantic_annotation`, the
`content.Json` payload — nothing abridged):

```json
{
  "name": "precedence",
  "type": "semantic_annotation",
  "value": {
    "type": "object",
    "properties": [
      [
        { "type": "property",
          "key": { "type": "identifier", "name": "level" },
          "value": { "type": "integer", "value": "5", "base": 10 } },
        [
          [ "", ",", "",
            { "type": "property",
              "key": { "type": "identifier", "name": "associativity" },
              "value": { "type": "string", "value": "\"left\"", "quote_style": "double" } } ]
        ]
      ],
      ""
    ]
  }
}
```

Three things about that shape are worth stating plainly, because an idealised version of this example
stood here previously and a consumer written against it would not have worked:

- **A property `key` is a node, not a bare string** — `{"type": "identifier", "name": "level"}`.
- **A collection is a `[first, [repetitions…]]` PAIR, not a flat list.** Every collection rule is
  spelled `head (sep item)*`, and the typed AST mirrors that spelling: element 0 is the first item,
  element 1 is the list of repetitions, and each repetition carries its own separator slots
  (`"", ",", ""` — the two `/\s*/` and the comma). Walk it as a pair; do not index it as a list.
- ⚠️ **The trailing `""` is a known defect, not part of the contract.** Every collection rule closes
  its annotation with `[$3, $4*]`, where `$4` is the trailing `/\s*/` — the same off-by-one class as
  the `$6` above, but cosmetic rather than lossy. It affects `array_value`, `object_value`,
  `map_value`, `set_value`, `tuple_value`, `generic_type`, `function_call`, `exception_spec` and
  `platform_spec`. Tracked as `GRAMMAR-WELLFORMED.H.16.7a`; **do not depend on that element**, and
  expect it to disappear.

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
