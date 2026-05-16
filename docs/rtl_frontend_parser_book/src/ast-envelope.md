# AST Envelope Structure

The PGEN rtl_frontend parser produces a JSON AST through `parse_grammar_profile_ast_dump_named` (grammar `"rtl_frontend"`). This chapter documents the top-level structure of that JSON.

## The envelope

The AST-dump host entry points (the generic
`parse_grammar_profile_ast_dump*` family and the named-result form
`parse_grammar_profile_ast_dump_named`, used with grammar family
`"rtl_frontend"` / profile `"default"`) return — on success — an
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
`serde_json::from_str`) to obtain the `rtl_frontend_file` **root object**
that this book's per-rule chapters describe. There is no `root` /
`schema_version` / `grammar` / `profile` field on `AstDumpPayload`
itself: the AST-dump schema version is the `1` tracked in
[Schema Versioning](schema-versioning.md); the grammar/profile are fixed
(`rtl_frontend` / `default`).

When `truncated` is `true`, `dump_json` is **not** the AST — it is a
deterministic truncation diagnostic envelope carrying
`pgen_dump_contract_version` (currently `1`),
`kind: "pgen_ast_dump_truncation"`, `truncated: true`,
`dump_kind: "parser_return_ast"`, `max_bytes`, `full_bytes`, and
`reason`. Consumers must check `truncated` (or, equivalently, the
presence of `pgen_dump_contract_version` / `kind ==
"pgen_ast_dump_truncation"` in the parsed `dump_json`) before treating
`dump_json` as an rtl_frontend AST. If `max_ast_bytes` is too small to
fit even the diagnostic envelope, the API returns `E_INVALID_LIMITS`. The
downstream integration contract
`docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md`
("AST Envelope and Dispatch") is the authoritative restatement of this
for consumers.

## The root rule

The rtl_frontend grammar root is `rtl_frontend_file`, defined with a typed `{type, items}` envelope:

```ebnf
rtl_frontend_file := trivia design_item* trivia
                  -> {type: "rtl_frontend_file", items: $2}
```

`rtl_frontend_file` produces a JSON object with:

- `type: "rtl_frontend_file"`
- `items`: a JSON array of zero or more typed `design_item` objects.

## design_item dispatch

`design_item` is the primary top-level dispatcher. It is a 4-branch `kind`-tagged shape covering all rtl_frontend library-unit forms:

```ebnf
design_item := typedef_declaration -> {kind: "typedef", body: $1}
             | package_declaration -> {kind: "package", body: $1}
             | module_declaration  -> {kind: "module",  body: $1}
             | semi                -> {kind: "semi"}
```

Each branch produces an object with:

- `kind` — one of: `"typedef"`, `"package"`, `"module"`, `"semi"`.
- `body` — the child shape for that branch (absent for the `"semi"` branch).

## Nested dispatchers

The rtl_frontend grammar uses three additional dispatchers below `design_item`:

- `module_item` — 10 `kind` variants (parameter, port, signal, datatype, instantiation, always, initial, assign, generate, semi).
- `generate_item` — 11 variants (parameter, port, signal, datatype, instantiation, always, initial, assign, generate, semi, if).
- `package_item` — 3 variants (parameter, typedef, semi).

Each follows the same `{kind, body?}` shape pattern.

A typical consumer dispatch:

```rust
fn handle_design_item(node: &serde_json::Value) {
    let kind = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let body = node.get("body");
    match kind {
        "typedef" => handle_typedef(body),
        "package" => handle_package_decl(body),
        "module"  => handle_module_decl(body),
        "semi"    => {/* lone `;` separator */}
        _         => unreachable!("unknown design_item kind: {}", kind),
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

The 156-annotation surface (as of contract 1.0.1) covers all the load-bearing rules including the 10-level binop_chain expression hierarchy.

### Recursive envelope shape (rules without annotations)

When a grammar rule has no return annotation, the rule produces a JSON value derived from its grammar shape:

- A **terminal literal** produces a JSON string of the matched text.
- A **regex literal** produces a JSON string of the matched text.
- A **rule reference** produces whatever shape that rule produces.
- A **sequence** produces a JSON array of the per-element shapes.
- An **alternation** produces the matched branch's shape directly.
- A **quantified rule** (e.g. `expr*`, `expr+`) produces a JSON array of the per-iteration shapes.
- An **optional rule** (`expr?`) produces either the matched shape (if matched) or `[]` (if un-matched).

## Mixing typed and envelope shapes

Consumers need to handle both shapes:

```rust
fn walk(node: &serde_json::Value) {
    match node {
        serde_json::Value::Object(obj) => {
            // Typed shape — dispatch on `kind` (or `type` at envelope root).
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

See [Walking the AST](walking-the-ast.md) for the full walker pattern, including the binop_chain consumer-fold across the 10-level expression hierarchy.

## Determinism

The AST dump is **deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical (no trailing zeros for integers, etc.).
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed) but the underlying JSON value is the same.

Any non-determinism in the dump is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Truncation

If `AstDumpOptions.max_ast_bytes` is set and the encoded JSON exceeds it, the dump is truncated and `truncated: true` is set on the payload. The truncated payload is still valid JSON (the truncation happens at a node boundary). Consumers should check the `truncated` flag and either bail or note the truncation.
