# Walking the AST

This chapter is a recommended walker pattern for downstream consumers traversing the PGEN VHDL AST-dump JSON. It uses real VHDL rule and `kind` names from the live return-annotation inventory (`generated/vhdl_return_annotations.json`, 249 annotations, schema version `1`).

## The dual-shape walker

Because the AST tree mixes typed objects (the VHDL-Slice-1 surface) and recursive-envelope arrays (terminal/regex leaves and a few un-annotated rules), the walker dispatches on JSON value kind, then on the `type` / `kind` discriminator:

```rust
use serde_json::Value as JsonValue;

fn walk(node: &JsonValue) {
    match node {
        JsonValue::Object(obj) => {
            // Typed-shape node. The root carries `type`; every other
            // VHDL typed shape uses a `kind` discriminator.
            if obj.get("type").and_then(|v| v.as_str()) == Some("vhdl_file") {
                if let Some(units) = obj.get("design_units") {
                    walk(units); // recursive-envelope array of design_unit shapes
                }
                return;
            }
            match obj.get("kind").and_then(|v| v.as_str()) {
                Some("entity")        => walk_entity(obj.get("body")),
                Some("architecture")  => walk_architecture(obj.get("body")),
                Some("package")       => walk_package(obj.get("body")),
                Some("process")       => walk_process(obj.get("body")),
                // ... other design_unit / statement / declaration kinds
                Some("semi")          => { /* lone `;` separator — nothing to do */ }
                _ => {
                    // Unknown or field-only typed object (e.g. a named-field
                    // shape like entity_declaration, or binop_chain). Fall
                    // through to a generic child walk so the walker stays
                    // robust to shapes it doesn't special-case.
                    for (_key, value) in obj.iter() {
                        walk(value);
                    }
                }
            }
        }
        JsonValue::Array(items) => {
            // Recursive-envelope shape (sequence / quantified / `rest`
            // iteration / un-matched optional `[]`) — walk children.
            for item in items {
                walk(item);
            }
        }
        JsonValue::String(_text) => {
            // Terminal text — identifier / literal value. No children.
        }
        JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::Null => {
            // Annotation-produced scalars. The current VHDL surface uses
            // none of these (absent optionals are `[]`, not null), but
            // handle them so future slices don't break the walker.
        }
    }
}
```

## Why the unknown-shape fallback matters

The fallthrough that walks the children of any typed object the walker doesn't special-case is **important**:

```rust
_ => {
    for (_key, value) in obj.iter() {
        walk(value);
    }
}
```

The VHDL grammar landed its full surface in one batch (VHDL-Slice-1), but subsequent shape-affecting slices can add new `kind` values or restructure a shape. A walker that hard-fails on an unrecognized `kind` will break on every parser-release that extends the grammar. A walker that walks the children of unknown shapes degrades gracefully — it won't extract structured info for shapes it doesn't understand, but it will still reach descendants.

Recommendation: only hard-fail on unknown shapes when you are explicitly pinning to a specific `schema_version` and your test corpus exercises every `kind` value in that schema (see [Schema Versioning](schema-versioning.md)).

## Dispatching the design-unit list

Every parse roots at `{ "type": "vhdl_file", "design_units": [ ... ] }`. The per-unit dispatch is the 10-kind `design_unit`:

```rust
fn handle_design_unit(node: &serde_json::Value) {
    let kind = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let body = node.get("body");
    match kind {
        "entity"            => handle_entity(body),            // {name, items, end_label}
        "architecture"      => handle_architecture(body),      // {name, entity_name, items, statements, end_label}
        "package"           => handle_package(body),           // {name, header, items, end_label}
        "package_body"      => handle_package_body(body),      // {name, items, end_label}
        "configuration"     => handle_configuration(body),     // {name, entity_name, items, end_label}
        "context"           => handle_context(body),           // {name, items, end_label}
        "context_reference" => handle_context_reference(body), // {name}
        "library"           => handle_library_clause(body),    // {first, rest}
        "use"               => handle_use_clause(body),        // {first, rest}
        "semi"              => { /* lone `;` */ }
        _                   => { /* unknown — degrade gracefully */ }
    }
}
```

The same `{kind, body}` pattern recurs for the declarative-item dispatch rules (`architecture_declarative_item`, `package_declarative_item`, `process_declarative_item`, …), `sequential_statement` (13 kinds), and `concurrent_statement` (7 kinds). All use the bodyless `{kind: "semi"}` for a stray `;`, and `sequential_statement` additionally uses bodyless `{kind: "null"}` for the VHDL `null;` statement.

## Folding the `binop_chain` expression hierarchy

The five expression-precedence rules (`expression` → `relation` → `simple_expression` → `term` → `factor`) all emit the same `binop_chain` shape. A single fold handles the whole hierarchy. As of the `1.0.2` `VHDL-0001` fix (schema `2`), `rest` is a **clean** `[op-envelope, operand]` array where the op-envelope is the typed `{kind: …}` object at every level (uniform — `logical_operator` / `relational_operator` / `adding_operator` / `multiplying_operator` all emit `{kind}`); it is **never** `"<invalid_sequence_access>"` (the pre-fix schema-`1` malformation):

```rust
/// Folds one binop_chain level left-associatively. `lhs` is the leading
/// operand (itself a binop_chain of the next-tighter level, bottoming out
/// at a typed `primary`). `rest` is the clean iteration array of
/// [op-envelope, operand] entries; the op-envelope is the typed
/// {kind: ...} object (schema 2).
fn fold_binop_chain(node: &serde_json::Value) -> Expr {
    debug_assert_eq!(
        node.get("type").and_then(|v| v.as_str()),
        Some("binop_chain")
    );

    // `simple_expression` (level == "additive") carries an extra leading
    // `sign` field for the optional unary +/- (`[]` when absent).
    let unary_sign = node.get("sign"); // None for non-additive levels

    let mut acc = lower_operand(node.get("lhs").expect("lhs present"));
    if let Some(sign) = unary_sign {
        acc = apply_unary_sign(sign, acc); // no-op if sign == []
    }

    // `rest` is an array of [op-envelope, operand] entries. For `relation`
    // (level == "relational") and `factor` (level == "power") it holds at
    // most one entry (grammar uses `?`, not `*`); the other three levels
    // iterate `*`.
    if let Some(rest) = node.get("rest").and_then(|v| v.as_array()) {
        for entry in rest {
            // entry[0] is the typed {kind:...} op-envelope (schema 2 —
            // never "<invalid_sequence_access>"); entry[1] is the operand.
            let op = entry[0]["kind"].as_str().expect("typed op-envelope kind");
            let rhs_node = &entry[1];
            acc = Expr::Binary {
                op: op.to_string(),
                lhs: Box::new(acc),
                rhs: Box::new(lower_operand(rhs_node)),
            };
        }
    }
    acc
}
```

`level` tells you which precedence tier you are at (`"logical"`, `"relational"`, `"additive"`, `"multiplicative"`, `"power"`) without re-deriving it from context. The op-envelope `kind` is the typed operator discriminator (`"plus"` / `"minus"` / `"concat"` at the `additive` level via `adding_operator`; `"mul"` / `"div"` / `"mod"` / `"rem"` at the `multiplicative` level via `multiplying_operator`; `"and"` … at the `logical` level; `"eq"` … at the `relational` level). The leaf operand is a typed `primary` (`{kind: "literal" | "aggregate" | "attribute_name" | "function_call" | "name" | "parens" | "not", ...}`); dispatch on its `kind` to bottom out the recursion. See the [Binary Addition](examples-binary-addition.md) worked example for the real captured `b + c * d` shape.

## Identifier extraction

VHDL identifiers (entity names, signal names, labels) surface as JSON strings once you reach the relevant named field. For a typed parent the field is direct:

```rust
fn entity_name(entity_body: &serde_json::Value) -> Option<&str> {
    // entity_declaration -> {name, items, end_label}
    entity_body.get("name").and_then(|v| v.as_str())
}
```

When the identifier is reached through an envelope (e.g. inside an un-annotated leaf), walk to the terminal text:

```rust
fn extract_identifier(node: &serde_json::Value) -> Option<&str> {
    match node {
        serde_json::Value::String(text) => Some(text.as_str()),
        serde_json::Value::Array(items) if items.len() == 1 => {
            extract_identifier(&items[0])
        }
        _ => None,
    }
}
```

The per-rule chapters ([Top-Level Rules](rules-top-level.md)) document the field that holds the identifier for each rule that produces one.

## Iterating `{first, rest}` lists

VHDL separated lists (`identifier_list`, `selected_name`, `association_list`, `library_clause`, `use_clause`, `parameter_list`, `choices`, `enumeration_type_definition`, …) use the `{first, rest}` carrier. `rest` is the recursive-envelope iteration of the `(separator element)*` tail, so each `rest` entry wraps a separator token plus one element:

```rust
fn iter_list<'a>(list: &'a serde_json::Value) -> Vec<&'a serde_json::Value> {
    let mut out = Vec::new();
    if let Some(first) = list.get("first") {
        out.push(first);
    }
    if let Some(rest) = list.get("rest").and_then(|v| v.as_array()) {
        for entry in rest {
            // `entry` is the envelope of one `(sep element)` iteration;
            // the element is the last child. Walk to it rather than
            // assuming a fixed index, so a grammar tweak to the
            // separator doesn't break extraction.
            if let Some(elem) = entry.as_array().and_then(|a| a.last()) {
                out.push(elem);
            } else {
                out.push(entry);
            }
        }
    }
    out
}
```

This `{first, rest}` shape is uniform across the VHDL surface (it was **not** flattened to `[$N, $M::2*]` the way the SystemVerilog grammar's lists were in its slice-58 audit). If a future VHDL slice flattens these, it will get a [Schema Versioning](schema-versioning.md) row and a [Changelog Index](changelog-index.md) entry.

## Avoiding deep recursion

VHDL can produce deep AST trees — a large architecture with nested blocks, generate statements, and deeply-nested expressions can reach hundreds of levels.

Recommendations:

1. **Use an explicit stack-based walker** (push children, pop work items in a loop) instead of recursive function calls. Stack depth becomes irrelevant.
2. **Or grow the thread stack** (e.g. the `stacker` crate). PGEN itself uses a large-stack worker internally for the parser; consumers can do the same for traversal.
3. **For pure AST drop**, use a non-recursive drop — `serde_json::Value`'s default `Drop` is recursive and can blow the stack on deeply-nested values. PGEN's test path uses a large-stack worker to avoid this; downstream consumers should consider similar patterns.

## Schema-version-aware walking

`AstDumpPayload` has **no** `schema_version` (or `root`) field — the real struct is `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`. The AST-dump schema version is not discoverable at runtime from the payload; you **pin** it from `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" as a constant in your consumer and re-validate that pin against the contract's "Schema Versioning" table whenever you bump PGEN. The shape-contract manifest (`rust/test_data/ast_shape_contract/vhdl_v1.json`) is the machine-checkable lock that fails CI if the parser drifts from the pinned schema.

```rust
// The AST-dump schema version you integrated against (from the contract):
const VHDL_AST_SCHEMA_VERSION: u32 = 2;

let payload = outcome.ast_dump.expect("Success carries an AstDumpPayload");
if payload.truncated {
    // dump_json holds the truncation diagnostic envelope, not the AST.
    return Err("VHDL AST dump truncated".into());
}
let root: serde_json::Value = serde_json::from_str(&payload.dump_json)?;

// VHDL_AST_SCHEMA_VERSION selects which walker your code was built for.
// When you bump PGEN, diff the contract's Schema Versioning table; if the
// integer schema version moved, update the constant and the walker together.
match VHDL_AST_SCHEMA_VERSION {
    2 => walk_schema_v2(&root),
    // schema 1 (pre-`VHDL-0001`) emitted "<invalid_sequence_access>" in
    // simple_expression / term binop_chain.rest — do not target it.
    other => eprintln!("no walker compiled for VHDL AST schema version {other}"),
}
```

See [Schema Versioning](schema-versioning.md) for what triggers a schema bump and what stays stable within a single schema version.
