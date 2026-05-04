# ParseContent Variants

This chapter documents the `ParseContent` variants that the generated parser produces internally, and how each maps to the public AST-dump JSON shape consumers receive.

> **Note:** `ParseContent` is the internal AST-pipeline representation used by PGEN's parser hooks and codegen. Downstream consumers do not call `ParseContent` directly — they call `parse_grammar_profile_ast_dump_named` and walk the resulting JSON. This chapter is included as reference for tooling that introspects PGEN's behaviour or for downstream maintainers debugging shape issues.

## The variants

PGEN's `ast_pipeline::ParseContent` enum has these variants:

```rust
pub enum ParseContent {
    Terminal(String),                      // matched literal text
    Sequence(Vec<UnifiedReturnAST>),       // ordered children (e.g. from `a b c`)
    Quantified(Vec<UnifiedReturnAST>),     // iterations of `expr*` / `expr+`
    Or(Box<UnifiedReturnAST>),             // matched branch of an alternation
    Alternative(Box<UnifiedReturnAST>),    // boxed inner shape (per-branch wrap)
    Json(serde_json::Value),               // typed JSON value (from annotations)
    // (other variants for internal codegen use)
}
```

## How each variant lands in the AST dump JSON

| `ParseContent` variant | JSON shape consumers see |
|---|---|
| `Terminal(text)`        | `"<text>"` (string) |
| `Sequence(children)`    | `[<child0_json>, <child1_json>, ...]` |
| `Quantified(iters)`     | `[<iter0_json>, <iter1_json>, ...]` |
| `Or(inner)`             | `<inner_json>` (transparent) |
| `Alternative(inner)`    | `<inner_json>` (transparent) |
| `Json(value)`           | `<value>` directly (typed object/string/number/bool/null) |

So at the JSON layer, consumers see either:

- A **JSON string** (from `Terminal`), or
- A **JSON array** (from `Sequence` / `Quantified`), or
- A **JSON object / number / bool / null** (from typed `Json` annotations).

`Or` and `Alternative` are transparent — they pass their inner shape through.

## When you'll see each shape

In a fully-annotated grammar (the goal of the campaign), most rules produce typed `Json(object)` shapes and consumers walk a tree of typed objects with occasional arrays for ordered children.

In a partially-annotated grammar (the current state of the SV parser), un-annotated rules produce recursive `Sequence` / `Quantified` arrays — JSON arrays of strings and other arrays, walked by structural recursion. Annotated rules produce typed objects in the middle of those arrays.

The annotation-system is designed so you can walk either shape with the same dispatch:

```rust
match node {
    JsonValue::Object(obj) => { /* typed shape */ }
    JsonValue::Array(items) => { /* recursive-envelope shape */ }
    JsonValue::String(text) => { /* terminal */ }
    JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::Null => {
        /* annotation-produced scalar */
    }
}
```

See [Walking the AST](walking-the-ast.md) for the recommended walker pattern.

## Why two shapes coexist

The annotation campaign types rules incrementally. Each slice annotates one rule (or a small family of related rules) and bumps the schema version. At any point in time the grammar has a mix of annotated rules (typed shape) and un-annotated rules (envelope shape). The downstream AST is the natural composition of both.

The schema version 1.0.0 milestone (see [Schema Versioning](schema-versioning.md)) lands when every rule is either annotated or has a deliberate "stay un-annotated" decision — at which point the AST shape is fully locked.
