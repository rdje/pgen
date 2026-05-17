# Worked Example: Binary Addition `b + c * d`

The companion to [Minimal Entity](examples-minimal-entity.md): that one
shows the top-level dispatch on a declaration; this one shows the
**non-empty `binop_chain.rest`** (operator) shape and the consumer
left-fold across two precedence levels. Every JSON value here is the
**real captured output** of `generated/vhdl_parser.rs` (parser release
`1.0.3`, AST-dump schema version `3`) and is the regression-locked
sample `arithmetic_expr` in
`rust/test_data/ast_shape_contract/vhdl_v1.json`. The `binop_chain`
shape shown here is **unchanged** between schema `2` (the `VHDL-0001`
fix) and schema `3` (the `1.0.3` POST-SV-AUDIT.2.3 batch touched the 17
Category-A list rules, **not** the expression cascade).

> Schema `2` note (`VHDL-0001`): at `1.0.1` / schema `1` the `additive`
> (`simple_expression`) and `multiplicative` (`term`) `binop_chain`
> `rest` emitted `"<invalid_sequence_access>"` (three of them at the
> `additive` level) plus malformed nested objects for multi-operand
> input — a real parser defect (`VHDL-0001`) fixed in `1.0.2` by
> lifting the inline operator alternations into the **named** rules
> `adding_operator` / `multiplying_operator` (see
> [Schema Versioning](schema-versioning.md) and the contract's
> "Resolved Defects — `VHDL-0001`"). The shape below is the corrected,
> gate-locked output. **`vhdl` was the final grammar in the systemic
> inline-alternation-`$N` class; the class is now fully resolved across
> `rtl_const_expr` / `systemverilog_preprocessor` / `rtl_frontend` /
> `vhdl`.**

## Input

```vhdl
entity e is end;
architecture a of e is begin x <= b + c * d; end;
```

(With a trailing newline.) This is the smallest input that exercises a
**multi-operand expression at two precedence levels** — `b + (c * d)`
under VHDL's operator precedence — so both the `additive`
(`simple_expression`) and `multiplicative` (`term`) `binop_chain`
levels carry a non-empty `rest`. It is the exact input the AST shape
contract regression-locks (sample `arithmetic_expr`).

## Reproducing the dump

The VHDL parser is on-demand-only (see [Build Recipe](build-recipe.md)).
With `generated/vhdl_parser.rs` in place:

```bash
cd rust
cargo build --release --features generated_parsers --bin parseability_probe
printf 'entity e is end;\narchitecture a of e is begin x <= b + c * d; end;\n' > /tmp/arith.vhd
./target/release/parseability_probe \
    --parse-dump-ast-pretty vhdl /tmp/arith.vhd /tmp/arith_ast.json
# -> parse_full passed for grammar 'vhdl' on '/tmp/arith.vhd'
```

## The relevant node (`additive` level)

The signal assignment `x <= b + c * d;` parses to a `binop_chain`
cascade. The upper levels (`logical`, `relational`) are empty-`rest`
wrappers (unwrap to `lhs`). The operators appear at the `additive`
(`+`) and, nested inside its right operand, the `multiplicative` (`*`)
levels. This is the **real captured shape** (key-sorted, as the canonical
dump emits it):

```json
{
  "type": "binop_chain",
  "level": "additive",
  "sign": [],
  "lhs": {
    "type": "binop_chain",
    "level": "multiplicative",
    "rest": [],
    "lhs": {
      "type": "binop_chain",
      "level": "power",
      "rest": [],
      "lhs": {
        "kind": "function_call",
        "name": { "first": [ [], "b" ], "rest": [] },
        "params": []
      }
    }
  },
  "rest": [
    [
      { "kind": "plus" },
      {
        "type": "binop_chain",
        "level": "multiplicative",
        "lhs": {
          "type": "binop_chain",
          "level": "power",
          "rest": [],
          "lhs": {
            "kind": "function_call",
            "name": { "first": [ [], "c" ], "rest": [] },
            "params": []
          }
        },
        "rest": [
          [
            { "kind": "mul" },
            {
              "type": "binop_chain",
              "level": "power",
              "rest": [],
              "lhs": {
                "kind": "function_call",
                "name": { "first": [ [], "d" ], "rest": [] },
                "params": []
              }
            }
          ]
        ]
      }
    ]
  ]
}
```

(`b` / `c` / `d` are bare names; VHDL's `primary` resolves a name with
no parameter part to the `{kind: "function_call", name, params}` branch
with `params: []` — the name text is at `name.first[1]`. See
[Top-Level Rules](rules-top-level.md) § "`primary` and aggregates".)

## Reading `rest` (the operator-chain contract)

`rest` is a **clean array of iteration entries** — one per
`(NAMED_op operand)` repetition of `next (NAMED_op next)*`. Each entry
is a two-element array:

- **`entry[0]`** — the **op-envelope**: the typed `{kind: …}` object
  emitted by the named operator rule. At the `additive` level
  (`adding_operator`) it is `{"kind": "plus"}` / `{"kind": "minus"}` /
  `{"kind": "concat"}`; at the `multiplicative` level
  (`multiplying_operator`) it is `{"kind": "mul"}` / `{"kind": "div"}` /
  `{"kind": "mod"}` / `{"kind": "rem"}`. This is **uniform with the
  `logical` / `relational` levels** (whose `logical_operator` /
  `relational_operator` op-envelope is already a `{kind}` object) —
  VHDL's op-envelope is the typed `{kind}` object at **every** level,
  never `<invalid_sequence_access>` (schema `2`).
- **`entry[1]`** — the right-hand operand: the next-tighter level's
  `binop_chain`, itself unwrappable to its `lhs` leaf. Here the `+`
  entry's operand is the `multiplicative` `binop_chain` for `c * d`,
  which itself has a one-entry `rest` (`[{"kind": "mul"}, <d>]`).

`b + c * d` therefore yields **one** entry in the `additive` `rest`
(`[{"kind": "plus"}, <c*d>]`) whose operand carries **one** entry in its
own `multiplicative` `rest` (`[{"kind": "mul"}, <d>]`). The consumer
left-folds each level onto its `lhs`, bottoming out at the typed
`primary` leaf: `(b + (c * d))`.

The `"additive"` level additionally carries a leading **`sign`** field
for the optional unary `+`/`-` (`(plus | minus)?` in
`simple_expression`); it is `[]` here (no leading sign) and was
**deliberately left as an inline optional** by the `VHDL-0001` fix
because it is not an iteration lead and was empirically unaffected.

## Walker code — the left-fold

```rust
use pgen::embedding_api::{parse_vhdl_1076_2019_ast_dump, AstDumpOptions, ParseStatus};

/// Unwrap empty-`rest` binop_chain wrappers down to the first node that
/// either has operators or is a typed leaf (`primary`).
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
        // a typed `primary` leaf: dispatch on its `kind`.
        return match node["kind"].as_str() {
            // bare name -> {kind:"function_call", name:{first:[_,txt],rest:[]}, params:[]}
            Some("function_call") => node["name"]["first"][1]
                .as_str().unwrap_or("?").to_string(),
            Some("name") => node["body"][1].as_str().unwrap_or("?").to_string(),
            _ => "<primary>".to_string(),
        };
    }
    let mut acc = fold(&node["lhs"]);
    for entry in node["rest"].as_array().unwrap() {
        // entry[0] is the typed {kind:...} op-envelope (schema 2);
        // never "<invalid_sequence_access>".
        let op = entry[0]["kind"].as_str().unwrap_or("?");
        let rhs = fold(&entry[1]);
        acc = format!("({acc} {op} {rhs})");
    }
    acc
}

let outcome = parse_vhdl_1076_2019_ast_dump(
    "entity e is end;\narchitecture a of e is begin x <= b + c * d; end;\n",
    &AstDumpOptions { pretty: true, max_ast_bytes: None },
);

// AST-dump schema version you integrated against, pinned from the
// contract (NOT a field of AstDumpPayload):
const VHDL_AST_SCHEMA_VERSION: u32 = 3;

if let ParseStatus::Success = outcome.status {
    let d = outcome.ast_dump.expect("Success carries an AstDumpPayload");
    assert!(!d.truncated, "dump_json would hold the truncation envelope");
    let _ = VHDL_AST_SCHEMA_VERSION; // re-check vs the contract on PGEN bumps

    // AstDumpPayload exposes dump_json/truncated/full_bytes/emitted_bytes;
    // parse dump_json to get the typed root object.
    let root: serde_json::Value =
        serde_json::from_str(&d.dump_json).expect("dump_json is valid JSON");
    assert_eq!(root["type"], "vhdl_file");
    // ... navigate to the architecture body's signal-assignment rhs,
    // then fold the binop_chain: fold(&rhs) == "(b + (c mul d))".
}
```

(The fold renders the operator by its `kind` token — `plus` / `mul` /
… — exactly the typed discriminator the parser emits; map to surface
syntax (`+` / `*`) in your own lowering.)

## Why this is a canonical operator-shape test

- It is the smallest input with a **non-empty `rest` at two precedence
  levels**, so the operator-chain contract (`entry[0]` typed `{kind}`
  op-envelope, `entry[1]` operand, recursive left-fold) is proven where
  precedence actually matters (`b + (c * d)`, not `(b + c) * d`).
- It is the regression-locked sample `arithmetic_expr`
  (`rule_under_test: "vhdl_file"`,
  `expected_json_object_keys_present: ["type", "design_units"]`).
  Combined with the now-256-entry `declared_annotation_inventory`, any
  reversion to the pre-`1.0.2` `<invalid_sequence_access>` shape fails
  `vhdl_ast_shape_contract_holds_against_running_generated_parser`
  immediately.

See [Walking the AST](walking-the-ast.md) for the full multi-level fold
and [Top-Level Rules](rules-top-level.md) for every level's shape.
