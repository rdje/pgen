# Worked Example: Binary Addition `a + b`

The companion to [Empty Module](examples-empty-module.md): this one shows
the **non-empty `rest`** (operator) shape of the ten-level
`binop_chain` cascade and the consumer left-fold. Every JSON value here
is the **real captured output** of `generated/rtl_frontend_parser.rs`
(parser release `1.0.3`, AST-dump schema version `3`) for the
regression-locked `assignment_expr` sample in
`rust/test_data/ast_shape_contract/rtl_frontend_v1.json`. The
`binop_chain` shape shown here is **unchanged** between schema `2`
(`RTL-FE-0001` fix) and schema `3` (the `1.0.3` POST-SV-AUDIT batch +
`RTL-FE-0002` touched the Category-A list rules and `event_control_list`,
not the expression cascade). Nothing here is idealized.

> Schema `2` note: at `1.0.1` (schema `1`) this exact input emitted
> `"rest": "<invalid_sequence_access>"` plus a malformed nested
> `{type:"binop_chain", level:"additive", lhs:["","+"], rest:<invalid>}`
> object at the `additive` level — a real parser defect (`RTL-FE-0001`)
> fixed in `1.0.2` by lifting the five inline operator alternations into
> the named, un-annotated rules `equality_op` / `relational_op` /
> `shift_op` / `additive_op` / `multiplicative_op` (the proven
> `rtl_const_expr` RTL-CE-Slice-2 / `systemverilog.ebnf`
> `binary_operator` idiom; see [Schema Versioning](schema-versioning.md)).
> The shape below is the corrected, gate-locked output. The annotation
> inventory is **unchanged at 156 / 74** — the five `*_op` rules are
> un-annotated alternations.

## Input

```verilog
module m;
assign y = a + b;
endmodule
```

(With a trailing newline. The whole input is 41 bytes:
`module m;\nassign y = a + b;\nendmodule\n`.)

This is the smallest rtl_frontend source that produces a **non-empty
`binop_chain` `rest`**: a `continuous_assign` (`assign y = …`) whose
right-hand value is the ten-level expression cascade with one `+`
operator at the `additive` level.

## Reproducing the dump

The rtl_frontend parser is on-demand-only (see
[Build Recipe](build-recipe.md)). With
`generated/rtl_frontend_parser.rs` in place, build the probe with the
generated backend and dump the AST:

```bash
cd rust
cargo build --release --features generated_parsers --bin parseability_probe
printf 'module m;\nassign y = a + b;\nendmodule\n' > /tmp/ab.sv
./target/release/parseability_probe \
    --parse-dump-ast-pretty rtl_frontend /tmp/ab.sv /tmp/ab.json
# -> parse_full passed for grammar 'rtl_frontend' on '/tmp/ab.sv'
```

## The relevant node (`additive` level)

`root` is `{type: "rtl_frontend_file", items: [...]}`; `items[0]` is the
`{kind: "module", body: {…}}` design item; the module body's single
`items` element is the `{kind: "continuous_assign", body: {lvalue,
value}}` module item. `body.value` is the ten-level cascade entered at
`logical_or`. The upper eight levels (`logical_or` … `shift`) are
empty-`rest` wrappers — unwrap each to its `lhs`, exactly as in the
[Empty Module](examples-empty-module.md) walkthrough's nested shapes.
The operator appears at the `additive` level:

```json
{
  "type": "binop_chain",
  "level": "additive",
  "lhs": {
    "type": "binop_chain", "level": "multiplicative", "rest": [],
    "lhs": {
      "kind": "signal",
      "body": { "name": { "first": [ /* … */ [ [], "a" ] ], "rest": [] }, "path": [] }
    }
  },
  "rest": [
    [
      [ [], "+" ],
      {
        "type": "binop_chain", "level": "multiplicative", "rest": [],
        "lhs": {
          "kind": "signal",
          "body": { "name": { "first": [ /* … */ [ [], "b" ] ], "rest": [] }, "path": [] }
        }
      }
    ]
  ]
}
```

(The `signal` leaf's `name.first` is the un-annotated `scoped_identifier`
→ `identifier` envelope — a long positional array of empty-`[]`
placeholders ending in `[ [], "<name>" ]`. It is abbreviated `/* … */`
above; the operator chain — the point of this example — is byte-exact.
Walk that envelope to its terminal string rather than indexing a fixed
depth, as in [Walking the AST](walking-the-ast.md#identifier-extraction).)

## Reading `rest` (the operator chain contract)

`rest` is a **clean array of iteration entries** — one per `(op
operand)` repetition of `<next> ( <NAMED_op> <next> )*`. Each entry is a
two-element array:

- **`entry[0]`** — the operator envelope of the named op-rule
  (`additive_op := plus | minus`). Here `[ [], "+" ]`: the leading `[]`
  is the empty `trivia`, the operator **token text is at `entry[0][1]`**
  (`"+"`). For `-`, it would be `[ [], "-" ]`.
- **`entry[1]`** — the right-hand operand: the next-tighter level
  (`multiplicative` `binop_chain`), itself unwrappable to its `lhs`
  leaf (here the `{kind: "signal", …}` for `b`).

`a + b - c` produces **two** entries in `additive.rest`
(`[[[],"+"],<b>]`, `[[[],"-"],<c>]`); the consumer left-folds them onto
`lhs`: `((a + b) - c)`. This is the **identical** consumer-fold
contract as `rtl_const_expr`'s `binop_chain` (operator text at
`entry[0][1]`, operand at `entry[1]`).

## Walker code — the left-fold

The rtl_frontend family exposes the **generic-by-grammar** host surface
(there is no `parse_rtl_frontend` convenience function — see
[Public API Surface](public-api.md)):

```rust
use pgen::embedding_api::{
    parse_grammar_profile_ast_dump_named, AstDumpOptions, ParseStatus,
};

/// Walk to the terminal identifier text inside an un-annotated
/// signal/identifier envelope, skipping empty-array `[]` placeholders.
fn signal_name(node: &serde_json::Value) -> Option<String> {
    match node {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(items) => items.iter().find_map(signal_name),
        serde_json::Value::Object(map) => map.values().find_map(signal_name),
        _ => None,
    }
}

/// Unwrap empty-`rest` binop_chain wrappers down to the first node that
/// is not an empty-`rest` binop_chain.
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
        // a leaf: a primary_expr {kind, …} (here {kind: "signal", …})
        return signal_name(node).unwrap_or_else(|| "<expr>".to_string());
    }
    let mut acc = fold(&node["lhs"]);
    for entry in node["rest"].as_array().unwrap() {
        // op token text is at entry[0][1]; the operand is entry[1].
        let op = entry[0][1].as_str().unwrap_or("?");
        let rhs = fold(&entry[1]);
        acc = format!("({acc} {op} {rhs})");
    }
    acc
}

let outcome = parse_grammar_profile_ast_dump_named(
    "rtl_frontend", "default", "module m;\nassign y = a + b;\nendmodule\n",
    &AstDumpOptions { pretty: true, max_ast_bytes: None },
);

// AST-dump schema version you integrated against, pinned from the
// contract (NOT a field of AstDumpPayload):
const RTL_FRONTEND_AST_SCHEMA_VERSION: u32 = 3;

if let ParseStatus::Success = outcome.status {
    let d = outcome.ast_dump.expect("Success carries an AstDumpPayload");
    assert!(!d.truncated, "dump_json would hold the truncation envelope");
    let _ = RTL_FRONTEND_AST_SCHEMA_VERSION; // re-check vs the contract on PGEN bumps

    // AstDumpPayload exposes dump_json/truncated/full_bytes/emitted_bytes;
    // parse dump_json to get the typed root object.
    let root: serde_json::Value =
        serde_json::from_str(&d.dump_json).expect("dump_json is valid JSON");
    assert_eq!(root["type"], "rtl_frontend_file");

    let module = &root["items"][0];
    assert_eq!(module["kind"], "module");
    let assign = &module["body"]["items"][0];
    assert_eq!(assign["kind"], "continuous_assign");
    assert_eq!(fold(&assign["body"]["value"]), "(a + b)");
}
```

## Why this is a canonical operator test

- It is the smallest rtl_frontend input with a **non-empty `rest`**, so
  the operator-chain contract (`entry[0]` op-envelope, `entry[1]`
  operand, left-fold) is proven on the simplest possible operator
  expression.
- It is the regression-locked `assignment_expr` sample
  (`rule_under_test: "rtl_frontend_file"`). Combined with the
  unchanged-156-entry `declared_annotation_inventory`, any reversion to
  the pre-`1.0.2` `<invalid_sequence_access>` shape fails
  `rtl_frontend_ast_shape_contract` immediately.
- **It honestly transitions a real released-parser defect** — the
  ten `binop_chain` levels' `rest` was the malformed
  `<invalid_sequence_access>` + nested object at `1.0.1` / schema `1`
  (`RTL-FE-0001`); the schema-`2` corrected shape is shown, with the
  pre-fix history kept (not whitewashed) in the schema-`2` transition
  note and [Schema Versioning](schema-versioning.md). A book that hid
  the defect history would mislead every downstream expression-folding
  integration repinning across the schema bump.

See [Walking the AST](walking-the-ast.md#folding-the-binop_chain-expression-hierarchy)
for the full multi-level fold and
[Top-Level Rules](rules-top-level.md#family-expressions--the-binop_chain-contract)
for every level's shape and the `RTL-FE-0001` note.
