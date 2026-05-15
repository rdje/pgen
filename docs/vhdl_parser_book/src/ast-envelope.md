# AST Envelope Structure

The PGEN VHDL parser produces a JSON AST through `parse_vhdl_1076_2019_ast_dump`. This chapter documents the top-level structure of that JSON.

## The envelope

The `ast_dump` field of `GrammarAstDumpOutcome` carries an `AstDumpPayload`:

```rust
pub struct AstDumpPayload {
    pub pgen_dump_contract_version: u32,  // currently 1
    pub schema_version: u32,              // 1 — see Schema Versioning
    pub grammar: String,                  // "vhdl"
    pub profile: String,                  // "vhdl_1076_2019"
    pub root: JsonValue,                  // the actual AST tree
    pub truncated: bool,                  // true if max_ast_bytes was hit
}
```

The `root` field is the VHDL AST as a `serde_json::Value`. It is what this book's per-rule chapters describe.

## The root rule

The VHDL grammar root is `vhdl_file`, defined as:

```ebnf
vhdl_file := design_unit*
```

`vhdl_file` produces a JSON array of zero or more typed `design_unit` objects.

## design_unit dispatch

`design_unit` is the primary top-level dispatcher. It is a 10-branch `kind`-tagged shape covering all VHDL library-unit forms:

```ebnf
design_unit := library_clause              -> {kind: "library",            body: $1}
             | use_clause                  -> {kind: "use",                body: $1}
             | context_reference_clause    -> {kind: "context_reference",  body: $1}
             | entity_declaration          -> {kind: "entity",             body: $1}
             | architecture_body           -> {kind: "architecture",       body: $1}
             | package_declaration         -> {kind: "package",            body: $1}
             | package_body                -> {kind: "package_body",       body: $1}
             | configuration_declaration   -> {kind: "configuration",      body: $1}
             | context_declaration         -> {kind: "context",            body: $1}
             | semi                        -> {kind: "semi"}
```

Each branch produces an object with:

- `kind` — one of: `"library"`, `"use"`, `"context_reference"`, `"entity"`, `"architecture"`, `"package"`, `"package_body"`, `"configuration"`, `"context"`, `"semi"`.
- `body` — the child shape for that branch (absent for the `"semi"` branch, which carries only the kind).

A typical consumer dispatch:

```rust
fn handle_design_unit(node: &serde_json::Value) {
    let kind = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let body = node.get("body");
    match kind {
        "entity"            => handle_entity(body),
        "architecture"      => handle_architecture(body),
        "package"           => handle_package(body),
        "package_body"      => handle_package_body(body),
        "configuration"    => handle_configuration(body),
        "context"           => handle_context(body),
        "context_reference" => handle_context_reference(body),
        "library"           => handle_library_clause(body),
        "use"               => handle_use_clause(body),
        "semi"              => {/* lone `;` separator */}
        _                   => unreachable!("unknown design_unit kind: {}", kind),
    }
}
```

## Two carrier kinds: typed and recursive-envelope

Per-rule, the AST dump produces JSON in one of two shapes:

### Typed shape (rules with return annotations)

When a grammar rule carries a `-> {...}` return annotation, the rule produces a typed JSON object:

```json
{
  "type": "<rule-family>",
  "kind": "<specific-shape>",
  "...": "..."
}
```

Many VHDL rules currently use only a `kind` discriminator (no `type`), reflecting the design choice for tightly-scoped dispatch.

### Recursive envelope shape (rules without annotations)

When a grammar rule has no return annotation, the rule produces a JSON value derived from its grammar shape:

- A **terminal literal** produces a JSON string of the matched text.
- A **regex literal** produces a JSON string of the matched text.
- A **rule reference** produces whatever shape that rule produces.
- A **sequence** produces a JSON array of the per-element shapes.
- An **alternation** produces the matched branch's shape directly.
- A **quantified rule** (e.g. `expr*`, `expr+`) produces a JSON array of the per-iteration shapes.
- An **optional rule** (`expr?`) produces either the matched shape (if matched) or `[]` (if un-matched).

For a 3-element sequence rule like `a b c`, the envelope shape is `[<a-shape>, <b-shape>, <c-shape>]`.

## Mixing typed and envelope shapes

The 249-annotation surface (as of contract 1.0.1) covers the most-load-bearing rules — design_unit dispatch, the 5-level expression hierarchy (`expression` → `relation` → `simple_expression` → `term` → `factor`), entity/architecture/package internals, statement family, and the data-type dispatch. The remaining rules still produce recursive-envelope arrays.

Consumers need to handle both:

```rust
fn walk(node: &serde_json::Value) {
    match node {
        serde_json::Value::Object(obj) => {
            // Typed shape — dispatch on `kind` (VHDL) or `type` (envelope-root).
        }
        serde_json::Value::Array(items) => {
            // Recursive-envelope shape — walk children.
            for item in items {
                walk(item);
            }
        }
        serde_json::Value::String(text) => {
            // Terminal text — no children.
        }
        _ => {
            // Bool / Number / Null — produced by typed annotations using
            // `BooleanLiteral` / `NumberLiteral` / `NullLiteral`.
        }
    }
}
```

See [Walking the AST](walking-the-ast.md) for the full walker pattern, including the binop_chain consumer-fold across the 5-level expression hierarchy.

## Determinism

The AST dump is **deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical (no trailing zeros for integers, etc.).
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed) but the underlying JSON value is the same.

Any non-determinism in the dump is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Truncation

If `AstDumpOptions.max_ast_bytes` is set and the encoded JSON exceeds it, the dump is truncated and `truncated: true` is set on the payload. The truncated payload is still valid JSON (the truncation happens at a node boundary). Consumers should check the `truncated` flag and either bail or note the truncation.
