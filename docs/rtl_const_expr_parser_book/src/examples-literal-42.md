# Worked Example: Literal `42`

A "what does the AST actually look like" walkthrough for the smallest
rtl_const_expr input — a single decimal literal. Every JSON value here
is the **real captured output** of `generated/rtl_const_expr_parser.rs`
(parser release `1.0.2`, schema version `2`) and is the exact input the
shape contract regression-locks (sample `literal_42` in
`rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`).

## Input

```
42
```

(2 bytes, no trailing newline required.)

## Reproducing the dump

```bash
cd rust
cargo build --release --features generated_parsers --bin parseability_probe
printf '42' > /tmp/l42.rce
./target/release/parseability_probe \
    --parse-dump-ast-pretty rtl_const_expr /tmp/l42.rce /tmp/l42.json
```

## The consumer-facing AST (`AstDumpPayload.root`)

The root is `{type: "rtl_const_expr", expr: <conditional_expr>}`. Because
the expression grammar is a ten-level precedence cascade and `42` uses
no operators, `expr` is a **ten-deep stack of empty `binop_chain`
wrappers** — each `{type:"binop_chain", level, lhs:<next>, rest:[]}` —
bottoming out at the typed `literal`. The repetitive middle levels are
elided here **for readability only**; regenerate with the command above
for the byte-exact tree.

```json
{
  "type": "rtl_const_expr",
  "expr": {
    "type": "binop_chain", "level": "logical_or", "rest": [],
    "lhs": {
      "type": "binop_chain", "level": "logical_and", "rest": [],
      "lhs": { "...": "binop_chain levels: bit_or, bit_xor, bit_and, equality, relational, shift, additive (each {rest:[], lhs:<next>})" ,
        "lhs": {
          "type": "binop_chain", "level": "multiplicative", "rest": [],
          "lhs": {
            "type": "literal",
            "kind": "decimal",
            "text": "42"
          }
        }
      }
    }
  }
}
```

Exact, unabridged structure: `expr` is exactly **10** nested
`binop_chain` objects in this order — `logical_or` → `logical_and` →
`bit_or` → `bit_xor` → `bit_and` → `equality` → `relational` → `shift`
→ `additive` → `multiplicative` — each with `rest: []` and `lhs`
pointing at the next; the innermost `multiplicative`'s `lhs` is the
`literal`. (As of schema `2`, `literal.text` is the clean string
`"42"` — pre-`1.0.2` it was the envelope `["", "42"]`; see
[Schema Versioning](schema-versioning.md).)

## Field-by-field walk

- **`type: "rtl_const_expr"`** — the typed root. Dispatch here, then
  descend `obj["expr"]`.
- **`expr`** — the `conditional_expr` result. With no `?:`,
  `conditional_expr` is the **passthrough** branch (`-> $1`), so `expr`
  is directly the `logical_or` `binop_chain` (no `ternary` wrapper).
- **The 10 `binop_chain` levels** — each `{type:"binop_chain", level,
  lhs, rest}`. `rest: []` means "no operator at this level": the
  consumer simply unwraps `lhs`. This is the dominant shape — every
  operator-free sub-expression is a straight `lhs` chain to a leaf.
- **`literal`** — `{type:"literal", kind:"decimal", text:"42"}`.
  `kind` is `"decimal"` or `"based"` (e.g. `8'hFF` →
  `{kind:"based", text:"8'hFF"}`); `text` is the clean matched digits.

## Walker code

rtl_const_expr exposes the **generic** host surface (no
`parse_rtl_const_expr` convenience fn — see
[Public API Surface](public-api.md)):

```rust
use pgen::embedding_api::{
    parse_grammar_profile_ast_dump_named, AstDumpOptions, ParseStatus,
};

/// Unwrap empty binop_chain levels (rest == []) down to the operand.
fn unwrap_chain(mut n: &serde_json::Value) -> &serde_json::Value {
    while n["type"] == "binop_chain"
        && n["rest"].as_array().map(|a| a.is_empty()).unwrap_or(false)
    {
        n = &n["lhs"];
    }
    n
}

let outcome = parse_grammar_profile_ast_dump_named(
    "rtl_const_expr", "default", "42",
    &AstDumpOptions { pretty: true, max_ast_bytes: None },
);
if let ParseStatus::Success = outcome.status {
    let d = outcome.ast_dump.expect("AST dump");
    assert_eq!(d.schema_version, 2);
    let root = &d.root;
    assert_eq!(root["type"], "rtl_const_expr");
    let leaf = unwrap_chain(&root["expr"]);
    assert_eq!(leaf["type"], "literal");
    assert_eq!(leaf["kind"], "decimal");
    assert_eq!(leaf["text"], "42");
}
```

## Why this is a canonical first-test

- It exercises the **full ten-level cascade** with zero operators, so
  the empty-`rest` unwrap path — the single most common shape a
  consumer walks — is proven on the simplest possible input.
- It is the regression-locked sample `literal_42`
  (`rule_under_test: "rtl_const_expr"`,
  `expected_json_object_keys_present: ["type","expr"]`). Drift fails
  `rtl_const_expr_ast_shape_contract` immediately.

See [Binary Addition](examples-binary-addition.md) for the non-empty
`rest` (operator) shape and the left-fold.
