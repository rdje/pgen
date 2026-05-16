# AST Envelope Structure

The PGEN sv_preprocessor parser produces a JSON AST through `parse_grammar_profile_ast_dump_named` (grammar `"systemverilog_preprocessor"`). This chapter documents the top-level structure of that JSON.

## The envelope

The `ast_dump` field of `NamedGrammarAstDumpOutcome` carries an `AstDumpPayload`:

```rust
pub struct AstDumpPayload {
    pub pgen_dump_contract_version: u32,  // currently 1
    pub schema_version: u32,              // 1 — see Schema Versioning
    pub grammar: String,                  // "systemverilog_preprocessor"
    pub profile: String,                  // "default"
    pub root: JsonValue,                  // the actual AST tree
    pub truncated: bool,                  // true if max_ast_bytes was hit
}
```

The `root` field is the sv_preprocessor AST as a `serde_json::Value`. It is what this book's per-rule chapters describe.

## The root rule

The sv_preprocessor grammar root is `systemverilog_preprocessor_file`:

```ebnf
systemverilog_preprocessor_file := pp_item*
```

`systemverilog_preprocessor_file` produces a JSON array of zero or more typed `pp_item` objects.

## pp_item dispatch

`pp_item` is the primary top-level dispatcher. It is a 10-branch `kind`-tagged shape covering all preprocessor directive forms:

```ebnf
pp_item := pp_define              -> {kind: "define",              body: $1}
         | pp_undef               -> {kind: "undef",               body: $1}
         | pp_include             -> {kind: "include",             body: $1}
         | pp_timescale           -> {kind: "timescale",           body: $1}
         | pp_default_nettype     -> {kind: "default_nettype",     body: $1}
         | pp_celldefine          -> {kind: "celldefine"}
         | pp_endcelldefine       -> {kind: "endcelldefine"}
         | pp_conditional         -> {kind: "conditional",         body: $1}
         | pp_non_directive_line  -> {kind: "non_directive_line",  body: $1}
         | pp_blank_line          -> {kind: "blank_line"}
```

Each branch produces an object with:

- `kind` — one of: `"define"`, `"undef"`, `"include"`, `"timescale"`, `"default_nettype"`, `"celldefine"`, `"endcelldefine"`, `"conditional"`, `"non_directive_line"`, `"blank_line"`.
- `body` — the child shape for that branch (absent for `"celldefine"`, `"endcelldefine"`, and `"blank_line"`, which carry only the kind).

A typical consumer dispatch:

```rust
fn handle_pp_item(node: &serde_json::Value) {
    let kind = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let body = node.get("body");
    match kind {
        "define"             => handle_define(body),
        "undef"              => handle_undef(body),
        "include"            => handle_include(body),
        "timescale"          => handle_timescale(body),
        "default_nettype"    => handle_default_nettype(body),
        "celldefine"         => set_celldefine_state(true),
        "endcelldefine"      => set_celldefine_state(false),
        "conditional"        => handle_conditional(body),
        "non_directive_line" => handle_non_directive(body),
        "blank_line"         => {/* nothing */}
        _                    => unreachable!("unknown pp_item kind: {}", kind),
    }
}
```

## Conditional-compilation tree

`pp_conditional` describes the recursive `` `ifdef / `elsif / `else / `endif`` (and `` `ifndef``) chain. The body is a typed shape mirroring the grammar's recursive layout — see [Walking the AST](walking-the-ast.md) for the full tree shape.

## macro_body fragment dispatch

`pp_define` bodies (the macro replacement text) are tokenized into a `macro_body_fragment` list with **9** `kind`-tagged kinds: `token_paste`, `stringize`, `macro_reference` (`{kind, body}`), `text` (`{kind, body}`), `lparen`, `rparen`, `comma`, `question`, `colon`. The separate `macro_default_atom` dispatcher (macro-formal default values) has **8** kinds. The 64-annotation surface (contract 1.0.1) covers all of these; the authoritative per-branch enumeration is in [Top-Level Rules](rules-top-level.md) and the live inventory `generated/systemverilog_preprocessor_return_annotations.json`.

## Two carrier kinds: typed and recursive-envelope

Per-rule, the AST dump produces JSON in one of two shapes:

- **Typed shape** — rules with `-> {...}` annotations (pp_item dispatch, all directive bodies, conditional-compilation tree, macro_body fragments).
- **Recursive-envelope shape** — rules without annotations produce a JSON value derived from grammar shape (sequence → array, alternation → matched-branch shape, etc.).

The 64-annotation surface covers all load-bearing dispatchers. Identifier-leaf and whitespace-leaf rules remain envelope-shaped strings.

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

See [Walking the AST](walking-the-ast.md) for the full walker pattern.

## Determinism

The AST dump is **deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical (no trailing zeros for integers, etc.).
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed) but the underlying JSON value is the same.

Any non-determinism in the dump is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Truncation

If `AstDumpOptions.max_ast_bytes` is set and the encoded JSON exceeds it, the dump is truncated and `truncated: true` is set on the payload. The truncated payload is still valid JSON (the truncation happens at a node boundary). Consumers should check the `truncated` flag and either bail or note the truncation.
