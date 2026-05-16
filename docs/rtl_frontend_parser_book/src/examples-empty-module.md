# Worked Example: Empty Module

A "what does the AST actually look like" walkthrough for the simplest
rtl_frontend input that produces a complete design item. Every JSON
value shown here is the **real, captured output** of
`generated/rtl_frontend_parser.rs` — not a hypothetical shape — and it
is the exact input the AST shape contract regression-locks (sample
`empty_module` in `rust/test_data/ast_shape_contract/rtl_frontend_v1.json`).

## Input

```verilog
module m; endmodule
```

(With a trailing newline. The whole input is 19 bytes:
`module m; endmodule\n`.)

This is the canonical first-test input because it is the smallest
source that exercises `rtl_frontend_file → design_item →
module_declaration` while having no parameters, ports, or module items,
so the tree is shallow enough to reason about by hand — with one
instructive exception (the `name` envelope, below).

## Reproducing the dump

The rtl_frontend parser is on-demand-only (see
[Build Recipe](build-recipe.md)). With `generated/rtl_frontend_parser.rs`
in place, build the probe with the generated backend and dump the AST:

```bash
cd rust
cargo build --release --features generated_parsers --bin parseability_probe
printf 'module m; endmodule\n' > /tmp/empty_module.rtl
./target/release/parseability_probe \
    --parse-dump-ast-pretty rtl_frontend /tmp/empty_module.rtl /tmp/empty_module_ast.json
# -> parse_full passed for grammar 'rtl_frontend' on '/tmp/empty_module.rtl'
```

## The captured AST (raw probe envelope)

`parseability_probe --parse-dump-ast` serialises the parser's
`ParseNode` directly, so the dump is wrapped in the parse-node envelope
(`content` / `rule_name` / `span`) described in
[AST Envelope Structure](ast-envelope.md). Below, the `name` array is
shown with its 40 leading unmatched-alternative `[]` placeholders
elided **for readability only** — see "The `name` envelope" for the
exact, unabridged structure and how to regenerate it byte-for-byte.

```json
{
  "content": {
    "Json": {
      "items": [
        {
          "body": {
            "imports_post": [],
            "imports_pre": [],
            "items": [],
            "name": [
              /* indices 0..39: each element is exactly []  (40 entries) */
              [ [], "m" ]
            ],
            "parameters": [],
            "ports": []
          },
          "kind": "module"
        }
      ],
      "type": "rtl_frontend_file"
    }
  },
  "rule_name": "rtl_frontend_file",
  "span": {
    "end": 19,
    "start": 0
  }
}
```

`content.Json` is the typed AST. `rule_name` is the entry rule
(`rtl_frontend_file`); `span` is the byte range covered (`0..19` — the
whole input).

## The consumer-facing view (parsed `dump_json`)

Downstream code does not call the probe — it calls the public API
([Public API Surface](public-api.md)), which hands back an
`AstDumpPayload` (real fields: `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`
— there is no `root` field). After confirming `truncated == false`,
`serde_json::from_str(&payload.dump_json)` yields the **unwrapped**
typed AST (the same value the probe shows under `content.Json`):

```json
{
  "type": "rtl_frontend_file",
  "items": [
    {
      "kind": "module",
      "body": {
        "name": [ /* 40 × [] */ [ [], "m" ] ],
        "imports_pre": [],
        "parameters": [],
        "imports_post": [],
        "ports": [],
        "items": []
      }
    }
  ]
}
```

## Field-by-field walk

- **`type: "rtl_frontend_file"`** — the typed root object from the
  `rtl_frontend_file` root rule. Dispatch on
  `obj["type"] == "rtl_frontend_file"`.
- **`items`** — the `design_item*` array, one element here.
- **`items[0].kind: "module"`** — the `design_item` dispatcher matched
  the `module_declaration` branch (`-> {kind: "module", body: $1}`).
  Dispatch on `obj["kind"]`.
- **`body`** — the `module_declaration` named-field object:
  - **`name`** — the un-annotated `identifier` envelope (see below).
  - **`imports_pre` / `imports_post`** — `[]` (no `import` clauses
    before/after the header).
  - **`parameters`** — `[]` (no `#(...)` parameter port list).
  - **`ports`** — `[]` (no `(...)` port list).
  - **`items`** — `[]` (`module_item*` matched zero items).

Every absent-optional / empty-iteration field is the empty array `[]`,
**never** `null` — the uniform rtl_frontend convention documented in
[The Json Carrier](json-carrier.md).

## The `name` envelope (the instructive part)

`name` is **not** the bare string `"m"`. It is the recursive envelope
of the **un-annotated `identifier` rule** at the
`kw_module identifier` position. The grammar's `identifier` rule is a
large alternation (keyword-exclusion plus many alternative forms), so
its envelope carries **one `[]` slot per unmatched alternative** and
the matched text near the end.

The exact, unabridged shape (regenerate it with the command above and
inspect `/tmp/empty_module_ast.json`):

- `name` is an array of **exactly 41 elements**.
- Indices **0 through 39** are each exactly `[]` (the 40 unmatched
  alternatives).
- Index **40** (the last) is `[ [], "m" ]` — a two-element pair whose
  index `[1]` is the identifier text `"m"`.

**Do not hardcode index 40.** That offset is an artifact of the current
grammar's alternative ordering and is exactly the kind of thing a
schema bump may change. The robust consumer rule — identical to the one
in [Walking the AST](walking-the-ast.md#identifier-extraction) — is to
walk to the terminal string: find the single non-`[]` element, then
read its trailing string. This is the uniform behaviour of every
un-annotated `identifier` position.

## Walker code for this input

The rtl_frontend family exposes the **generic-by-grammar** host surface
(there is no `parse_rtl_frontend` convenience function — see
[Public API Surface](public-api.md)):

```rust
use pgen::embedding_api::{
    parse_grammar_profile_ast_dump_named, AstDumpOptions, ParseStatus,
};

/// Walk to the identifier text inside an un-annotated `identifier`
/// envelope, without assuming a fixed array depth or offset.
fn identifier_text(node: &serde_json::Value) -> Option<&str> {
    match node {
        serde_json::Value::String(s) => Some(s.as_str()),
        serde_json::Value::Array(items) => items
            .iter()
            .find(|v| !matches!(v, serde_json::Value::Array(a) if a.is_empty()))
            .and_then(identifier_text),
        _ => None,
    }
}

let outcome = parse_grammar_profile_ast_dump_named(
    "rtl_frontend",
    "default",
    "module m; endmodule\n",
    &AstDumpOptions { pretty: true, max_ast_bytes: None },
);

// AST-dump schema version you integrated against, pinned from the
// contract (NOT a field of AstDumpPayload):
const RTL_FRONTEND_AST_SCHEMA_VERSION: u32 = 2;

match outcome.status {
    ParseStatus::Success => {
        let dump = outcome.ast_dump.expect("Success carries an AstDumpPayload");
        assert!(!dump.truncated, "dump_json would hold the truncation envelope");
        let _ = RTL_FRONTEND_AST_SCHEMA_VERSION; // re-check vs the contract on PGEN bumps

        // AstDumpPayload exposes dump_json/truncated/full_bytes/emitted_bytes;
        // parse dump_json to get the typed root object.
        let root: serde_json::Value =
            serde_json::from_str(&dump.dump_json).expect("dump_json is valid JSON");
        assert_eq!(root["type"], "rtl_frontend_file");

        let unit = &root["items"][0];
        assert_eq!(unit["kind"], "module");

        // `name` is the un-annotated identifier envelope: walk to the
        // terminal rather than indexing a fixed depth/offset.
        let module_name = identifier_text(&unit["body"]["name"]);
        assert_eq!(module_name, Some("m"));

        for empty in ["imports_pre", "imports_post", "parameters", "ports", "items"] {
            assert!(unit["body"][empty].as_array().unwrap().is_empty());
        }
    }
    ParseStatus::Failure => {
        eprintln!("parse failed: {:?}", outcome.diagnostic);
    }
}
```

## Why this example is the canonical first-test

- **It exercises `rtl_frontend_file → design_item →
  module_declaration`** in 19 bytes.
- **It has no parameters, ports, imports, or module items** — so the
  tree is shallow and every field but `name` is the empty array.
- **It teaches the identifier-envelope rule of thumb early.** The
  verbose `name` array is the single most common source of downstream
  surprise; meeting it on the smallest possible input is the point.
- **It is the regression-locked sample** `empty_module` in
  `rust/test_data/ast_shape_contract/rtl_frontend_v1.json`
  (`rule_under_test: "rtl_frontend_file"`,
  `expected_content_kind: "json_object"`,
  `expected_json_object_keys_present: ["type", "items"]`). Drift in
  this shape fails the
  `rtl_frontend_ast_shape_contract_holds_against_running_generated_parser`
  test immediately.

If your downstream integration cannot parse this input and reach
`type == "rtl_frontend_file"` with one `module` design item, the
integration is broken upstream of any rtl_frontend-specific concern.
Start triage here, then move on to the
[Top-Level Rules](rules-top-level.md) shape reference.
