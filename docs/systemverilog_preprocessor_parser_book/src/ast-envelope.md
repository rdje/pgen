# AST Envelope Structure

The PGEN sv_preprocessor parser produces a JSON AST through `parse_grammar_profile_ast_dump_named` (grammar `"systemverilog_preprocessor"`). This chapter documents the top-level structure of that JSON.

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
`serde_json::from_str`) to obtain the `systemverilog_preprocessor_file`
**root object** that this book's per-rule chapters describe. There is no
`root` / `schema_version` / `grammar` / `profile` field on
`AstDumpPayload` itself: the AST-dump schema version is the `2` tracked
in [Schema Versioning](schema-versioning.md) (pinned from the contract,
**not** a payload field); the grammar/profile are fixed
(`systemverilog_preprocessor` / `default`).

When `truncated` is `true`, `dump_json` is **not** the AST — it is a
deterministic truncation diagnostic envelope carrying
`pgen_dump_contract_version` (currently `1`),
`kind: "pgen_ast_dump_truncation"`, `truncated: true`,
`dump_kind: "parser_return_ast"`, `max_bytes`, `full_bytes`, and
`reason`. Consumers must check `truncated` (or, equivalently, the
presence of `pgen_dump_contract_version` / `kind ==
"pgen_ast_dump_truncation"` in the parsed `dump_json`) before treating
`dump_json` as a systemverilog_preprocessor AST. If `max_ast_bytes` is
too small to fit even the diagnostic envelope, the API returns
`E_INVALID_LIMITS`. The downstream integration contract
`docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`
("AST Envelope and pp_item Dispatch") is the authoritative restatement
of this for consumers.

The parsed `dump_json` is what this book's per-rule chapters describe.

## The root rule

The sv_preprocessor grammar root is `systemverilog_preprocessor_file`:

```ebnf
systemverilog_preprocessor_file := pp_item*
```

`systemverilog_preprocessor_file` produces a JSON array of zero or more typed `pp_item` objects.

## pp_item dispatch

`pp_item` is the primary top-level dispatcher. It is an 18-branch `kind`-tagged shape covering every preprocessor line form (10 branches through `1.0.7`; release `1.0.8` added the eight kinds of the seven clause-22 directives the grammar had not derived):

```ebnf
pp_item := pp_define              -> {kind: "define",              body: $1}
         | pp_undef               -> {kind: "undef",               body: $1}
         | pp_include             -> {kind: "include",             body: $1}
         | pp_timescale           -> {kind: "timescale",           body: $1}
         | pp_default_nettype     -> {kind: "default_nettype",     body: $1}
         | pp_celldefine          -> {kind: "celldefine"}
         | pp_endcelldefine       -> {kind: "endcelldefine"}
         | pp_resetall            -> {kind: "resetall",            body: $1}
         | pp_undefineall         -> {kind: "undefineall",         body: $1}
         | pp_unconnected_drive   -> {kind: "unconnected_drive",   body: $1}
         | pp_nounconnected_drive -> {kind: "nounconnected_drive", body: $1}
         | pp_pragma              -> {kind: "pragma",              body: $1}
         | pp_line                -> {kind: "line",                body: $1}
         | pp_begin_keywords      -> {kind: "begin_keywords",      body: $1}
         | pp_end_keywords        -> {kind: "end_keywords",        body: $1}
         | pp_conditional         -> {kind: "conditional",         body: $1}
         | pp_non_directive_line  -> {kind: "non_directive_line",  body: $1}
         | pp_blank_line          -> {kind: "blank_line"}
```

Each branch produces an object with:

- `kind` — one of: `"define"`, `"undef"`, `"include"`, `"timescale"`, `"default_nettype"`, `"celldefine"`, `"endcelldefine"`, `"resetall"`, `"undefineall"`, `"unconnected_drive"`, `"nounconnected_drive"`, `"pragma"`, `"line"`, `"begin_keywords"`, `"end_keywords"`, `"conditional"`, `"non_directive_line"`, `"blank_line"`.
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
        "resetall"           => reset_directive_state(),            // §22.3
        "undefineall"        => clear_macro_table(),                // §22.5.3
        "unconnected_drive"  => handle_unconnected_drive(body),     // body.drive.kind: "pull0" | "pull1"
        "nounconnected_drive" => clear_unconnected_drive(),
        "pragma"             => handle_pragma(body),                // name + expressions (Syntax 22-8)
        "line"               => handle_line(body),                  // number, filename, level
        "begin_keywords"     => push_keyword_version(body),         // body.version: "\"1800-2017\"" …
        "end_keywords"       => pop_keyword_version(),
        "conditional"        => handle_conditional(body),
        "non_directive_line" => handle_non_directive(body),
        "blank_line"         => {/* nothing */}
        _                    => unreachable!("unknown pp_item kind: {}", kind),
    }
}
```

The catch-all arm is exactly where a walker written against schema `5` meets release `1.0.8`: the eight clause-22
kinds are new values of `kind`, so a walker that treats an unknown kind as unreachable must be updated before it
reads schema-`6` output — one reason `1.0.8` bumped the schema (see [Schema Versioning](schema-versioning.md)).

## Conditional-compilation tree

`pp_conditional` describes the recursive `` `ifdef / `elsif / `else / `endif`` (and `` `ifndef``) chain. The body is a typed shape mirroring the grammar's recursive layout. The `` `ifdef``/`` `ifndef`` polarity is a typed `pp_if_keyword` object — `pp_if_branch.keyword` is `{kind: "ifdef"}` or `{kind: "ifndef"}` as of the `1.0.2` `SVPP-0001` correctness fix (was the malformed `<invalid_sequence_access>` object at `1.0.1` / schema `1`; see [Schema Versioning](schema-versioning.md) and the [Conditional Compilation](examples-conditional.md) worked example). See [Walking the AST](walking-the-ast.md) for the full tree shape.

## macro_body fragment dispatch

`pp_define` bodies (the macro replacement text) are tokenized into a `macro_body_fragment` list with **9** `kind`-tagged kinds: `token_paste`, `stringize`, `macro_reference` (`{kind, body}`), `text` (`{kind, body}`), `lparen`, `rparen`, `comma`, `question`, `colon`. The separate `macro_default_atom` dispatcher (macro-formal default values) has **8** kinds. The 66-annotation surface (contract 1.0.3; 65 `return_object` + 1 `return_array`) covers all of these; the authoritative per-branch enumeration is in [Top-Level Rules](rules-top-level.md) and the live inventory `generated/systemverilog_preprocessor_return_annotations.json`.

## Two carrier kinds: typed and recursive-envelope

Per-rule, the AST dump produces JSON in one of two shapes:

- **Typed shape** — rules with `-> {...}` annotations (pp_item dispatch, all directive bodies, conditional-compilation tree, macro_body fragments).
- **Recursive-envelope shape** — rules without annotations produce a JSON value derived from grammar shape (sequence → array, alternation → matched-branch shape, etc.).

The 66-annotation surface covers all load-bearing dispatchers. Identifier-leaf and whitespace-leaf rules remain envelope-shaped strings.

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
