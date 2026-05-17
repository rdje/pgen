# Walking the AST

This chapter is a recommended walker pattern for downstream consumers traversing the SystemVerilog AST dump JSON.

## The dual-shape walker

Because the annotation campaign types rules incrementally, the AST tree carries a mix of typed objects and recursive-envelope arrays. The walker dispatches on JSON value kind:

```rust
use serde_json::Value as JsonValue;

fn walk(node: &JsonValue) {
    match node {
        JsonValue::Object(obj) => {
            // Typed-shape node — dispatch on `type` / `kind` fields.
            let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match ty {
                "module"      => walk_module(obj),
                "interface"   => walk_interface(obj),
                "class"       => walk_class(obj),
                // ... (other typed shapes; see per-rule chapters)
                _ => {
                    // Unknown typed shape — fall through to generic
                    // child walk. This keeps the walker robust to new
                    // typed shapes added in subsequent slices.
                    for (_key, value) in obj.iter() {
                        walk(value);
                    }
                }
            }
        }
        JsonValue::Array(items) => {
            // Recursive-envelope shape — walk children.
            for item in items {
                walk(item);
            }
        }
        JsonValue::String(text) => {
            // Terminal text — no children. Use as identifier / literal
            // value if your traversal needs it.
        }
        JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::Null => {
            // Annotation-produced scalars (booleans for negation flags,
            // numbers for typed integer transforms, null for absent
            // optional fields). No children.
        }
    }
}
```

## Why the unknown-typed-shape fallback matters

The walker pattern above includes a fallthrough that walks the children of any typed object the walker doesn't know about:

```rust
_ => {
    for (_key, value) in obj.iter() {
        walk(value);
    }
}
```

This is **important** because the annotation campaign adds new typed shapes over time. A walker that hard-fails on unknown `type` values will break on every parser-release that adds a new typed rule. A walker that walks children of unknown typed shapes degrades gracefully — it won't extract structured info for shapes it doesn't understand, but it'll still find descendants.

Recommendation: only hard-fail on unknown shapes when you're explicitly pinning to a specific AST-dump schema version (the constant you pin from the contract — see [Schema-version-aware walking](#schema-version-aware-walking) below) and your test corpus covers every `type` value in that schema.

## Identifier extraction

For SystemVerilog identifiers (module names, signal names, etc.), look for the rule that produces the identifier and walk to its terminal text:

```rust
fn extract_identifier(node: &JsonValue) -> Option<&str> {
    // Once `simple_identifier` carries a typed annotation, this
    // becomes obj.get("name").as_str() directly. Until then, walk
    // to the matched terminal text.
    match node {
        JsonValue::String(text) => Some(text.as_str()),
        JsonValue::Array(items) if items.len() == 1 => {
            extract_identifier(&items[0])
        }
        _ => None,
    }
}
```

The per-rule chapters (e.g. [Top-Level Rules](rules-top-level.md)) document the path for each rule that produces an identifier in its envelope.

## Avoiding deep recursion

SystemVerilog can produce very deep AST trees — a real-world module with N hierarchical instances + class hierarchies + always blocks easily reaches 200-500 levels of nesting. Stack-recursive walkers can blow the default Rust thread stack on large inputs.

Recommendations:

1. **Use an explicit stack-based walker** (push children, pop work items in a loop) instead of recursive function calls. The walker becomes iterative; stack depth is irrelevant.
2. **Or use the stacker crate** to grow the thread stack as needed. PGEN itself uses this approach internally for the parser; consumers can do the same for AST traversal.
3. **For pure AST drop** (releasing all the JSON values once parsing is done), use a non-recursive drop — `serde_json::Value`'s default `Drop` is recursive and can blow the stack on deeply-nested values. PGEN's test path uses `run_with_regex_worker_stack` (a 64MB-stack worker) to avoid this; downstream consumers should consider similar patterns.

## Schema-version-aware walking

`AstDumpPayload` has **no** `schema_version` (or `root`) field — the real struct is `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`. The AST-dump schema version is not discoverable at runtime from the payload; you **pin** it from `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" as a constant in your consumer and re-validate that pin against the contract's "Schema Versioning" table whenever you bump PGEN. The shape-contract manifest (`rust/test_data/ast_shape_contract/systemverilog_v1.json`) is the machine-checkable lock that fails CI if the parser drifts from the pinned schema.

```rust
// The AST-dump schema version you integrated against (from the contract):
const SV_AST_SCHEMA_VERSION: u32 = 3;

let payload = outcome.ast_dump.expect("Success carries an AstDumpPayload");
if payload.truncated {
    // dump_json holds the truncation diagnostic envelope, not the AST.
    return Err("systemverilog AST dump truncated".into());
}
let root: serde_json::Value = serde_json::from_str(&payload.dump_json)?;

// SV_AST_SCHEMA_VERSION selects which walker your code was built for.
// When you bump PGEN, diff the contract's Schema Versioning table; if the
// integer schema version moved, update the constant and the walker together.
match SV_AST_SCHEMA_VERSION {
    3 => walk_schema_v3(&root), // current: list_of_*_identifiers / *_list_of_arguments / parameter_port_list type_only / assignment_pattern named are clean factored record lists (was structured {first:{…},rest} / {first_name,first_value,rest} at schema 2)
    2 => walk_schema_v2(&root), // legacy: net_alias is {lvalues: […]}; the above rules still {first:{…},rest}
    1 => walk_schema_v1(&root), // legacy: net_alias was {first, second, rest}
    other => eprintln!("no walker compiled for systemverilog AST schema version {other}"),
}
```

See [Schema Versioning](schema-versioning.md) for what triggers a schema bump and what stays stable within a single schema version.
