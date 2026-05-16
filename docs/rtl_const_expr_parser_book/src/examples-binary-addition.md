# Worked Example: Binary Addition `a + b`

The companion to [Literal `42`](examples-literal-42.md): this one shows
the **non-empty `rest`** (operator) shape and the consumer left-fold.
Every JSON value here is the **real captured output** of
`generated/rtl_const_expr_parser.rs` (parser release `1.0.2`, schema
version `2`) and is the regression-locked sample `binary_addition` in
`rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`.

> Schema `2` note: at `1.0.1` this exact input emitted
> `"rest": "<invalid_sequence_access>"` — a real parser defect fixed in
> `1.0.2` (named operator rules + bare `rest:$2`; see
> [Schema Versioning](schema-versioning.md)). The shape below is the
> corrected, gate-locked output.

## Input

```
a + b
```

## Reproducing the dump

```bash
cd rust
cargo build --release --features generated_parsers --bin parseability_probe
printf 'a + b' > /tmp/ab.rce
./target/release/parseability_probe \
    --parse-dump-ast-pretty rtl_const_expr /tmp/ab.rce /tmp/ab.json
```

## The relevant node (`additive` level)

`root.expr` is again the ten-level cascade; the upper levels
(`logical_or` … `shift`) are empty-`rest` wrappers (unwrap to `lhs`,
exactly as in the literal example). The operator appears at the
`additive` level:

```json
{
  "type": "binop_chain",
  "level": "additive",
  "lhs": {
    "type": "binop_chain", "level": "multiplicative", "rest": [],
    "lhs": { "type": "identifier", "text": "a" }
  },
  "rest": [
    [
      [ "", "+" ],
      {
        "type": "binop_chain", "level": "multiplicative", "rest": [],
        "lhs": { "type": "identifier", "text": "b" }
      }
    ]
  ]
}
```

## Reading `rest` (the operator chain contract)

`rest` is a **clean array of iteration entries** — one per `(op
operand)` repetition of `next (NAMED_op next)*`. Each entry is a
two-element array:

- **`entry[0]`** — the operator envelope of the named op rule
  (`additive_op := plus | minus`). Here `[ "", "+" ]`: the leading `""`
  is the empty `trivia`, the operator **token text is at
  `entry[0][1]`** (`"+"`). For `-`, it would be `[ "", "-" ]`.
- **`entry[1]`** — the right-hand operand: the next-tighter level
  (`multiplicative` `binop_chain`), itself unwrappable to its `lhs`
  leaf (here `{type:"identifier", text:"b"}` — clean name, schema `2`).

`a + b - c` produces **two** entries in `additive.rest`
(`[["","+"],<b>]`, `[["","-"],<c>]`); the consumer left-folds them onto
`lhs`: `((a + b) - c)`.

## Walker code — the left-fold

```rust
use pgen::embedding_api::{
    parse_grammar_profile_ast_dump_named, AstDumpOptions, ParseStatus,
};

fn unwrap_chain(mut n: &serde_json::Value) -> &serde_json::Value {
    while n["type"] == "binop_chain"
        && n["rest"].as_array().map(|a| a.is_empty()).unwrap_or(false)
    {
        n = &n["lhs"];
    }
    n
}

/// Left-fold a binop_chain node into (op, operand) steps over `lhs`.
fn fold(node: &serde_json::Value) -> String {
    let node = unwrap_chain(node);
    if node["type"] != "binop_chain" {
        // a leaf: identifier / literal / ternary / unary
        return match node["type"].as_str() {
            Some("identifier") => node["text"].as_str().unwrap_or("?").to_string(),
            Some("literal")    => node["text"].as_str().unwrap_or("?").to_string(),
            _ => "<expr>".to_string(),
        };
    }
    let mut acc = fold(&node["lhs"]);
    for entry in node["rest"].as_array().unwrap() {
        let op = entry[0][1].as_str().unwrap_or("?"); // op text at [0][1]
        let rhs = fold(&entry[1]);
        acc = format!("({acc} {op} {rhs})");
    }
    acc
}

let outcome = parse_grammar_profile_ast_dump_named(
    "rtl_const_expr", "default", "a + b",
    &AstDumpOptions { pretty: true, max_ast_bytes: None },
);
if let ParseStatus::Success = outcome.status {
    let d = outcome.ast_dump.expect("AST dump");
    assert_eq!(d.schema_version, 2);
    assert_eq!(d.root["type"], "rtl_const_expr");
    assert_eq!(fold(&d.root["expr"]), "(a + b)");
}
```

## Why this is a canonical first-test

- It is the smallest input with a **non-empty `rest`**, so the
  operator-chain contract (`entry[0]` op-envelope, `entry[1]` operand,
  left-fold) is proven on the simplest possible operator expression.
- It is the regression-locked sample `binary_addition`
  (`rule_under_test: "rtl_const_expr"`,
  `expected_json_object_keys_present: ["type","expr"]`). Combined with
  the now-26-entry `declared_annotation_inventory`, any reversion to the
  pre-`1.0.2` `<invalid_sequence_access>` shape fails
  `rtl_const_expr_ast_shape_contract` immediately.

See [Walking the AST](walking-the-ast.md) for the full multi-level fold
and [Top-Level Rules](rules-top-level.md) for every level's shape.
