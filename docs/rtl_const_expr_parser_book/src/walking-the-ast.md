# Walking the AST

This chapter is a recommended walker pattern for downstream consumers traversing the PGEN rtl_const_expr AST-dump JSON. It uses real rtl_const_expr rule, `type`, `level`, and `op` names from the live return-annotation inventory (`generated/rtl_const_expr_return_annotations.json`, 24 annotations on 16 distinct rules, schema version `1`).

## The dual-shape walker

Because the AST tree mixes typed objects (the RTL-CE-Slice-1 surface) and recursive-envelope arrays (the `binop_chain` `rest` iteration and the un-annotated leaves), the walker dispatches on JSON value kind, then on the `type` discriminator. rtl_const_expr uses a `type` tag on every typed object — there is no bare `kind` dispatcher in this grammar:

```rust
use serde_json::Value as JsonValue;

fn walk(node: &JsonValue) {
    match node {
        JsonValue::Object(obj) => {
            // Every rtl_const_expr typed object carries `type`.
            match obj.get("type").and_then(|v| v.as_str()) {
                Some("rtl_const_expr") => {
                    walk(obj.get("expr").unwrap_or(&JsonValue::Null));
                }
                Some("binop_chain") => { walk_binop_chain(obj); }
                Some("ternary") => {
                    walk(obj.get("condition").unwrap_or(&JsonValue::Null));
                    walk(obj.get("then_expr").unwrap_or(&JsonValue::Null));
                    walk(obj.get("else_expr").unwrap_or(&JsonValue::Null));
                }
                Some("unary") => {
                    // op in: "plus" | "minus" | "logical_not" | "bit_not"
                    walk(obj.get("expr").unwrap_or(&JsonValue::Null));
                }
                Some("literal") => {
                    // kind in: "based" | "decimal"; `text` is the source string.
                }
                Some("identifier") => {
                    // `text` is the identifier source string.
                }
                _ => {
                    // Unknown / future typed object. Fall through to a generic
                    // child walk so the walker stays robust to shapes it does
                    // not special-case.
                    for (_key, value) in obj.iter() {
                        walk(value);
                    }
                }
            }
        }
        JsonValue::Array(items) => {
            // Recursive-envelope shape — the binop_chain `rest` iteration
            // (or `[]` when no operator at that level). Walk children.
            for item in items {
                walk(item);
            }
        }
        JsonValue::String(_text) => {
            // Terminal text — a literal / identifier `text` value. No children.
        }
        JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::Null => {
            // Annotation-produced scalars. The current rtl_const_expr surface
            // uses none of these (empty iterations are `[]`, not null), but
            // handle them so future slices do not break the walker.
        }
    }
}
```

## Why the unknown-shape fallback matters

The fallthrough that walks the children of any typed object the walker does not special-case is **important**:

```rust
_ => {
    for (_key, value) in obj.iter() {
        walk(value);
    }
}
```

The rtl_const_expr grammar landed its full expression surface in RTL-CE-Slice-1, but subsequent shape-affecting slices can add new `type` values or restructure a shape. A walker that hard-fails on an unrecognized `type` will break on every parser-release that extends the grammar. A walker that walks the children of unknown shapes degrades gracefully — it will not extract structured info for shapes it does not understand, but it will still reach descendants.

Recommendation: only hard-fail on unknown shapes when you are explicitly pinning to a specific `schema_version` and your test corpus exercises every `type` value in that schema (see [Schema Versioning](schema-versioning.md)).

## Dispatching from the root

Every parse roots at `{ "type": "rtl_const_expr", "expr": <conditional_expr-shape> }`. There is exactly one root object; descend into `expr`:

```rust
fn handle_root(node: &serde_json::Value) -> &serde_json::Value {
    debug_assert_eq!(
        node.get("type").and_then(|v| v.as_str()),
        Some("rtl_const_expr")
    );
    node.get("expr").expect("rtl_const_expr always carries `expr`")
}
```

`expr` is the `conditional_expr` shape. Because `conditional_expr` branch 1 is **passthrough** (`-> $1`, a `return_scalar`), a non-ternary input surfaces here directly as a `logical_or_expr` `binop_chain` — there is no `conditional_expr` wrapper object. Dispatch on `obj["type"]`:

```rust
fn lower_expr(node: &serde_json::Value) -> Expr {
    match node.get("type").and_then(|v| v.as_str()) {
        Some("ternary") => Expr::Ternary {
            cond:      Box::new(lower_expr(node.get("condition").unwrap())),
            then_arm:  Box::new(lower_expr(node.get("then_expr").unwrap())),
            else_arm:  Box::new(lower_expr(node.get("else_expr").unwrap())),
        },
        Some("binop_chain") => fold_binop(node),
        Some("unary") => Expr::Unary {
            // op in: "plus" | "minus" | "logical_not" | "bit_not"
            op:   node.get("op").and_then(|v| v.as_str()).unwrap().to_string(),
            expr: Box::new(lower_expr(node.get("expr").unwrap())),
        },
        Some("literal") => Expr::Literal {
            // kind in: "based" | "decimal"
            kind: node.get("kind").and_then(|v| v.as_str()).unwrap().to_string(),
            text: node.get("text").and_then(|v| v.as_str()).unwrap().to_string(),
        },
        Some("identifier") => Expr::Ident(
            node.get("text").and_then(|v| v.as_str()).unwrap().to_string(),
        ),
        _ => lower_envelope(node), // unknown / future shape — degrade gracefully
    }
}
```

There is no separate `primary_expr` shape to handle: all three `primary_expr` branches are `return_scalar` passthroughs, so a `primary_expr` always surfaces as a `literal`, an `identifier`, or (for the parenthesized form) the inner `conditional_expr`'s shape directly.

## Folding the `binop_chain` expression hierarchy

The ten precedence levels (`logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr`) all emit the same `binop_chain` shape. A single fold handles the whole hierarchy:

```rust
/// Folds one binop_chain level left-associatively. `lhs` is the leading
/// operand (itself a binop_chain of the next-tighter level, bottoming out
/// at unary_expr -> a passthrough primary_expr -> literal/identifier).
/// `rest` is the recursive-envelope iteration of the `( op operand )*` tail.
fn fold_binop(node: &serde_json::Value) -> Expr {
    debug_assert_eq!(
        node.get("type").and_then(|v| v.as_str()),
        Some("binop_chain")
    );

    // `level` names the precedence tier without re-deriving from context:
    // "logical_or" | "logical_and" | "bit_or" | "bit_xor" | "bit_and" |
    // "equality" | "relational" | "shift" | "additive" | "multiplicative".
    let _level = node.get("level").and_then(|v| v.as_str());

    let mut acc = lower_expr(node.get("lhs").expect("lhs present"));

    // `rest` is the recursive-envelope array of `( op operand )` iterations
    // (NOT a typed {op, rhs} object). It is `[]` when there was no operator
    // at this level — in that case the node is a pure wrapper around `lhs`.
    // rtl_const_expr has NO `sign` field; prefix +/-/!/~ live in unary_expr,
    // below this cascade. All ten levels iterate `*`.
    if let Some(rest) = node.get("rest").and_then(|v| v.as_array()) {
        for pair in rest {
            // Each `pair` is the envelope of one `( operator operand )`
            // iteration: the operator-token sub-shape followed by the
            // next-level operand. Walk to the operand rather than assuming a
            // fixed index, so a grammar tweak to the operator token does not
            // break extraction.
            let (op, rhs_node) = split_op_operand(pair);
            acc = Expr::Binary {
                op,
                lhs: Box::new(acc),
                rhs: Box::new(lower_expr(rhs_node)),
            };
        }
    }
    acc
}

/// `pair` is the recursive envelope of one `( operator operand )` iteration.
/// The operand is the last child; the operator token is what precedes it.
fn split_op_operand(pair: &serde_json::Value) -> (String, &serde_json::Value) {
    let arr = pair.as_array().expect("rest entry is a sequence envelope");
    let operand = arr.last().expect("rest entry has an operand");
    // The operator token is the leading child; walk it to its terminal text.
    let op = arr
        .first()
        .and_then(extract_terminal_text)
        .unwrap_or_default();
    (op, operand)
}
```

`level` tells you which precedence tier you are at without re-deriving it from context, and lets you validate operator-vs-level conformance. The fold is identical at every level precisely because the carrier is identical at every level. When `rest == []` (no operator at this level), the loop runs zero times and `acc` is just the lowered `lhs` — the consumer transparently unwraps the empty wrapper.

A non-empty `rest` example: see [Binary Addition](examples-binary-addition.md) for the captured shape of an `additive_expr` with one `(plus, operand)` iteration.

## Identifier and literal extraction

rtl_const_expr identifiers and literals **are** annotated, so unlike rtl_frontend they surface as typed `{type, text}` objects, not bare envelopes. The source text is the `text` field of the object — it is the one place rtl_const_expr surfaces a bare string:

```rust
fn extract_identifier(node: &serde_json::Value) -> Option<String> {
    if node.get("type").and_then(|v| v.as_str()) == Some("identifier") {
        return node.get("text").and_then(|v| v.as_str()).map(str::to_owned);
    }
    None
}

fn extract_literal(node: &serde_json::Value) -> Option<(String, String)> {
    if node.get("type").and_then(|v| v.as_str()) == Some("literal") {
        let kind = node.get("kind").and_then(|v| v.as_str())?; // "based" | "decimal"
        let text = node.get("text").and_then(|v| v.as_str())?;
        return Some((kind.to_owned(), text.to_owned()));
    }
    None
}
```

For walking *into* the `binop_chain` `rest` iteration — where operator tokens are un-annotated and surface as recursive envelopes — walk to the terminal string rather than indexing a fixed depth:

```rust
/// Walk an un-annotated envelope to its first terminal string, skipping
/// empty-array `[]` placeholders. Used for the operator token inside a
/// `binop_chain` rest entry, which is NOT annotated.
fn extract_terminal_text(node: &serde_json::Value) -> Option<String> {
    match node {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Array(items) => items.iter().find_map(extract_terminal_text),
        _ => None,
    }
}
```

The per-rule chapter ([Top-Level Rules](rules-top-level.md)) documents which field holds the source text for each typed rule. Treat the `binop_chain` `rest` entries as envelopes — see [The Json Carrier](json-carrier.md#worked-example-a-decimal-literal) for the captured shape of `42`, a ten-deep stack of empty `binop_chain` wrappers bottoming out at `{type: "literal", kind: "decimal", text: "42"}`.

## Avoiding deep recursion

rtl_const_expr produces deep AST trees by construction: every expression descends through **all ten** `binop_chain` levels even when no operator is present, because each level's `lhs` is the next-tighter level. A bare `42` is already a ten-deep wrapper stack; nested parentheses and ternary arms multiply that.

Recommendations:

1. **Use an explicit stack-based walker** (push children, pop work items in a loop) instead of recursive function calls. Stack depth becomes irrelevant.
2. **Or grow the thread stack** (e.g. the `stacker` crate). PGEN itself uses a large-stack worker internally for the parser; consumers can do the same for traversal.
3. **For pure AST drop**, use a non-recursive drop — `serde_json::Value`'s default `Drop` is recursive and can blow the stack on deeply-nested values. PGEN's test path uses a large-stack worker to avoid this; downstream consumers should consider similar patterns.

A practical shortcut: the empty-`rest` `binop_chain` wrappers carry no information, so a consumer can collapse them — when `rest == []`, replace the node with its lowered `lhs` (the `fold_binop` above already does this implicitly).

## Schema-version-aware walking

If your tool needs to support multiple PGEN versions, branch on the payload's schema version (`AstDumpPayload.schema_version`):

```rust
match ast_dump_payload.schema_version {
    1 => walk_schema_v1(&ast_dump_payload.root),
    // (future) 2 => walk_schema_v2(...),
    other => {
        eprintln!("unsupported rtl_const_expr schema version: {}", other);
    }
}
```

See [Schema Versioning](schema-versioning.md) for what triggers a schema bump and what stays stable within a single schema version.
