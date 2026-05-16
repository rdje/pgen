# Walking the AST

This chapter is a recommended walker pattern for downstream consumers traversing the PGEN rtl_frontend AST-dump JSON. It uses real rtl_frontend rule and `kind` names from the live return-annotation inventory (`generated/rtl_frontend_return_annotations.json`, 156 annotations on 74 rules, schema version `1`).

## The dual-shape walker

Because the AST tree mixes typed objects (the RTL-FE-Slice-1..7 surface) and recursive-envelope arrays (terminal/regex leaves, `named_data_type`, and the passthrough expression forms), the walker dispatches on JSON value kind, then on the `type` / `kind` discriminator:

```rust
use serde_json::Value as JsonValue;

fn walk(node: &JsonValue) {
    match node {
        JsonValue::Object(obj) => {
            // Typed-shape node. The root carries `type`; binop_chain /
            // ternary / unary carry `type`; every other rtl_frontend
            // typed shape uses a `kind` discriminator.
            if obj.get("type").and_then(|v| v.as_str()) == Some("rtl_frontend_file") {
                if let Some(items) = obj.get("items") {
                    walk(items); // recursive-envelope array of design_item shapes
                }
                return;
            }
            match obj.get("type").and_then(|v| v.as_str()) {
                Some("binop_chain") => { walk_binop_chain(obj); return; }
                Some("ternary")     => {
                    walk(obj.get("condition").unwrap_or(&JsonValue::Null));
                    walk(obj.get("then_expr").unwrap_or(&JsonValue::Null));
                    walk(obj.get("else_expr").unwrap_or(&JsonValue::Null));
                    return;
                }
                Some("unary") => { walk(obj.get("expr").unwrap_or(&JsonValue::Null)); return; }
                _ => {}
            }
            match obj.get("kind").and_then(|v| v.as_str()) {
                Some("module")  => walk_module(obj.get("body")),
                Some("package") => walk_package(obj.get("body")),
                Some("typedef") => walk_typedef(obj.get("body")),
                // ... other design_item / module_item / generate_item kinds
                Some("semi")    => { /* lone `;` separator — nothing to do */ }
                _ => {
                    // Unknown or field-only typed object (e.g. a named-field
                    // shape like module_declaration, or a primary_expr kind
                    // the walker doesn't special-case). Fall through to a
                    // generic child walk so the walker stays robust to
                    // shapes it doesn't special-case.
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
            // Annotation-produced scalars. The current rtl_frontend surface
            // uses none of these (absent optionals are `[]`, not null), but
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

The rtl_frontend grammar landed its full surface across RTL-FE-Slice-1..7, but subsequent shape-affecting slices can add new `kind` values or restructure a shape. A walker that hard-fails on an unrecognized `kind` will break on every parser-release that extends the grammar. A walker that walks the children of unknown shapes degrades gracefully — it won't extract structured info for shapes it doesn't understand, but it will still reach descendants.

Recommendation: only hard-fail on unknown shapes when you are explicitly pinning to a specific `schema_version` and your test corpus exercises every `kind` value in that schema (see [Schema Versioning](schema-versioning.md)).

## Dispatching the design-item list

Every parse roots at `{ "type": "rtl_frontend_file", "items": [ ... ] }`. The per-item dispatch is the 4-kind `design_item`:

```rust
fn handle_design_item(node: &serde_json::Value) {
    let kind = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let body = node.get("body");
    match kind {
        "module"  => handle_module(body),   // {name, imports_pre, parameters, imports_post, ports, items}
        "package" => handle_package(body),  // {name, items}
        "typedef" => handle_typedef(body),  // {data_type, packed_range, name}
        "semi"    => { /* lone `;` */ }
        _         => { /* unknown — degrade gracefully */ }
    }
}
```

The same `{kind, body?}` pattern recurs for the nested dispatch wrappers: `module_item` (10 kinds), `generate_item` (11 kinds), and `package_item` (3 kinds). All use the bodyless `{kind: "semi"}` for a stray `;`. The leaf keyword rules (`parameter_flavor`, `port_direction`, `port_direction_token`, `event_edge`, `assignment_operator`, `builtin_data_type`, `always_star_event`, the `"wildcard"` branch of `port_connection`, the `"byte"` branch of `struct_union_field_name`) are bare `{kind}` objects with no `body`.

## Folding the `binop_chain` expression hierarchy

The ten expression-precedence rules (`logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr`) all emit the same `binop_chain` shape. A single fold handles the whole hierarchy:

```rust
/// Folds one binop_chain level left-associatively. `lhs` is the leading
/// operand (itself a binop_chain of the next-tighter level, bottoming out
/// at unary_expr -> a typed primary_expr). `rest` is the recursive-envelope
/// iteration of (operator, operand) pairs.
fn walk_binop_chain(node: &serde_json::Map<String, serde_json::Value>) {
    debug_assert_eq!(
        node.get("type").and_then(|v| v.as_str()),
        Some("binop_chain")
    );

    // `level` names the precedence tier without re-deriving from context:
    // "logical_or" | "logical_and" | "bit_or" | "bit_xor" | "bit_and" |
    // "equality" | "relational" | "shift" | "additive" | "multiplicative".
    let _level = node.get("level").and_then(|v| v.as_str());

    let mut acc = lower_operand(node.get("lhs").expect("lhs present"));

    // `rest` is the recursive-envelope array of (op, operand) iterations.
    // rtl_frontend has NO `sign` field — prefix +/-/!/~ live in unary_expr,
    // below this cascade. All ten levels iterate `*`.
    if let Some(rest) = node.get("rest").and_then(|v| v.as_array()) {
        for pair in rest {
            let (op, rhs_node) = split_op_operand(pair);
            acc = Expr::Binary {
                op,
                lhs: Box::new(acc),
                rhs: Box::new(lower_operand(rhs_node)),
            };
        }
    }
    // ... use `acc`
}
```

The leaf operand bottoms out at `unary_expr` then `primary_expr`. Because `conditional_expr` (ternary) and `unary_expr` are **passthrough** when their syntax is absent, `lower_operand` must accept three shapes in any operand slot:

```rust
fn lower_operand(node: &serde_json::Value) -> Expr {
    match node.get("type").and_then(|v| v.as_str()) {
        Some("binop_chain") => fold_binop(node),   // nested precedence level
        Some("ternary")     => lower_ternary(node), // {condition, then_expr, else_expr}
        Some("unary")       => lower_unary(node),   // {op, expr}; op in
                                                    //  plus|minus|logical_not|bit_not
        _ => match node.get("kind").and_then(|v| v.as_str()) {
            // primary_expr passthrough (unary_expr / conditional_expr -> $1):
            Some("repetition")    => lower_repetition(node.get("body")),
            Some("concatenation") => lower_concat(node.get("body")),
            Some("ranged_signal") => lower_ranged_signal(node.get("body")),
            Some("signal")        => lower_signal(node.get("body")),
            Some("literal")       => lower_literal(node.get("body")),
            Some("parens")        => lower_operand(node.get("expr").unwrap()),
            _ => lower_envelope(node), // un-annotated descent
        },
    }
}
```

`level` tells you which precedence tier you are at without re-deriving it from context. The fold is identical at every level precisely because the carrier is identical at every level.

## Identifier extraction

rtl_frontend identifiers (module names, signal names, genvars, labels) are bound to the **un-annotated** `identifier` rule, so they surface as a recursive envelope, **not** a bare JSON string. The identifier text is the terminal string nested inside that envelope. Walk to the terminal rather than indexing a fixed depth:

```rust
fn extract_identifier(node: &serde_json::Value) -> Option<String> {
    match node {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Array(items) => {
            // The identifier envelope nests through positional sequence
            // arrays; the text is the deepest non-empty string leaf.
            // Walk every child and take the first string we reach,
            // skipping empty-array `[]` optional placeholders.
            items.iter().find_map(extract_identifier)
        }
        _ => None,
    }
}
```

For `module_declaration` the name lives at `body["name"]`; for `signal_reference` it is `body["name"]` (a `scoped_identifier` — itself a `{first, rest}` shape whose `first` is an identifier envelope). The per-rule chapters ([Top-Level Rules](rules-top-level.md)) document the field that holds the identifier for each rule that produces one. Always treat these as envelopes — see [The Json Carrier](json-carrier.md#worked-example-a-minimal-module) for the captured shape of `module m; endmodule`, where `name` is a nested array ending in `[ [], "m" ]`, not the string `"m"`.

## Iterating `{first, rest}` lists

rtl_frontend separated lists (`port_list`, `parameter_declaration_sequence`, `genvar_declaration`, `net_declaration`, `scoped_identifier`, `event_control_list`, `module_instantiation`, `concatenation_expr`, `parameter_override_list`, `port_connection_list`, `struct_union_field`, `enum_type`, …) use the `{first, rest}` carrier. `rest` is the recursive-envelope iteration of the `(separator element)*` tail, so each `rest` entry wraps a separator token plus one element:

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

This `{first, rest}` shape is uniform across the rtl_frontend surface (it was **not** flattened to `[$N, $M::2*]` the way the SystemVerilog grammar's lists were in its slice-58 audit). Some list rules are additionally `kind`-tagged (`parameter_override_list`, `port_connection_list` carry `{kind: "named"|"positional", first, rest}`); read `kind` first, then iterate `first` + `rest` the same way.

## Avoiding deep recursion

rtl_frontend can produce deep AST trees — a module with deeply-nested generate regions and the ten-level expression cascade can reach hundreds of levels (every binary expression descends through all ten `binop_chain` levels even when no operator is present, because each level's `lhs` is the next level).

Recommendations:

1. **Use an explicit stack-based walker** (push children, pop work items in a loop) instead of recursive function calls. Stack depth becomes irrelevant.
2. **Or grow the thread stack** (e.g. the `stacker` crate). PGEN itself uses a large-stack worker internally for the parser; consumers can do the same for traversal.
3. **For pure AST drop**, use a non-recursive drop — `serde_json::Value`'s default `Drop` is recursive and can blow the stack on deeply-nested values. PGEN's test path uses a large-stack worker to avoid this; downstream consumers should consider similar patterns.

## Schema-version-aware walking

If your tool needs to support multiple PGEN versions, branch on the payload's schema version (`AstDumpPayload.schema_version`):

```rust
match ast_dump_payload.schema_version {
    1 => walk_schema_v1(&ast_dump_payload.root),
    // (future) 2 => walk_schema_v2(...),
    other => {
        eprintln!("unsupported rtl_frontend schema version: {}", other);
    }
}
```

See [Schema Versioning](schema-versioning.md) for what triggers a schema bump and what stays stable within a single schema version.
