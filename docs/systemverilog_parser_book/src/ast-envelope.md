# AST Envelope Structure

The PGEN SystemVerilog parser produces a JSON AST through `parse_grammar_profile_ast_dump_named`. This chapter documents the top-level structure of that JSON.

## The envelope

The AST-dump host entry points (the generic
`parse_grammar_profile_ast_dump*` family and the named-result form
`parse_grammar_profile_ast_dump_named`) return — on success — an
`AstDumpPayload`, defined in `rust/src/embedding_api.rs` (the
authoritative API contract is `rust/docs/EMBEDDING_API_CONTRACT.md`). It
is a **canonical-JSON payload string plus truncation metadata** — it has
**exactly four fields**:

```rust
pub struct AstDumpPayload {
    pub dump_json: String,    // canonical (key-sorted) JSON encoding of the typed AST
    pub truncated: bool,      // true if max_ast_bytes was exceeded
    pub full_bytes: usize,    // byte length of the full encoded AST (pre-truncation)
    pub emitted_bytes: usize, // byte length actually placed in dump_json
}
```

`dump_json` is a **JSON string you must parse** (e.g.
`serde_json::from_str`) to obtain the `systemverilog_file` **root
object** that this book's per-rule chapters describe. There is no `root`
/ `schema_version` / `grammar` / `profile` field on `AstDumpPayload`
itself: the AST-dump schema version is the `1` tracked in
[Schema Versioning](schema-versioning.md) (pinned from the contract,
**not** a payload field); the grammar/profile are fixed
(`systemverilog` / `sv_2017` | `sv_2023`).

When `truncated` is `true`, `dump_json` is **not** the AST — it is a
deterministic truncation diagnostic envelope carrying
`pgen_dump_contract_version` (currently `1`),
`kind: "pgen_ast_dump_truncation"`, `truncated: true`,
`dump_kind: "parser_return_ast"`, `max_bytes`, `full_bytes`, and
`reason`. Consumers must check `truncated` (or, equivalently, the
presence of `pgen_dump_contract_version` / `kind ==
"pgen_ast_dump_truncation"` in the parsed `dump_json`) before treating
`dump_json` as a SystemVerilog AST. If `max_ast_bytes` is too small to
fit even the diagnostic envelope, the API returns `E_INVALID_LIMITS`.
The downstream integration contract
`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`
("Contract Identity" + "Schema Versioning") is the authoritative
restatement of this for consumers.

The parsed `dump_json` is what this book's per-rule chapters describe.

## The root rule

The SystemVerilog grammar root is the `systemverilog_file` rule (the
parser-generation default and the embedding-API entry). It is
**typed** — it wraps the `source_text` body:

```ebnf
systemverilog_file := trivia source_text trivia
                   -> {type: "systemverilog_file", source_text: $2}
```

```json
{ "type": "systemverilog_file", "source_text": [ /* source_text_item shapes */ ] }
```

Dispatch on `root["type"] == "systemverilog_file"`, then iterate
`root["source_text"]` (a flat array of typed `source_text_item`
shapes, each carrying a `kind` discriminator).

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
