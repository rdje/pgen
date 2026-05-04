# AST Envelope Structure

The PGEN SystemVerilog parser produces a JSON AST through `parse_grammar_profile_ast_dump_named`. This chapter documents the top-level structure of that JSON.

## The envelope

The `ast_dump` field of `NamedGrammarAstDumpOutcome` carries an `AstDumpPayload`:

```rust
pub struct AstDumpPayload {
    pub pgen_dump_contract_version: u32,  // currently 1
    pub schema_version: u32,              // 1 — see Schema Versioning
    pub grammar: String,                  // "systemverilog"
    pub profile: String,                  // "sv_2017" | "sv_2023"
    pub root: JsonValue,                  // the actual AST tree
    pub truncated: bool,                  // true if max_ast_bytes was hit
}
```

The `root` field is the SystemVerilog AST as a `serde_json::Value`. It is what this book's per-rule chapters describe.

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

Examples (from the regex parser, for shape illustration; SV typed shapes will land per-slice):

```json
{"type": "atom", "kind": "literal", "value": "a"}
{"type": "anchor", "kind": "start_of_line"}
{"type": "escape", "kind": "shorthand", "char": "d"}
```

The annotation campaign (see [Changelog Index](changelog-index.md)) lands these typed shapes one rule at a time.

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

Until the annotation campaign covers every rule, the AST tree carries a mix of typed objects and recursive arrays. Consumers need to handle both:

```rust
fn walk(node: &serde_json::Value) {
    match node {
        serde_json::Value::Object(obj) => {
            // Typed shape — dispatch on `type` / `kind`.
            // See per-rule chapters.
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

See [Walking the AST](walking-the-ast.md) for the full walker pattern.

## Determinism

The AST dump is **deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical (no trailing zeros for integers, etc.).
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed) but the underlying JSON value is the same.

Any non-determinism in the dump is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Truncation

If `AstDumpOptions.max_ast_bytes` is set and the encoded JSON exceeds it, the dump is truncated and `truncated: true` is set on the payload. The truncated payload is still valid JSON (the truncation happens at a node boundary). Consumers should check the `truncated` flag and either bail or note the truncation.
