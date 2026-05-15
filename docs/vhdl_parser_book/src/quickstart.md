# Quickstart for Downstream Consumers

A minimal "compile this, walk that" recipe for embedding the PGEN VHDL parser. Read [Build Recipe](build-recipe.md) and [Public API Surface](public-api.md) for the long-form versions of each step.

## 1. Cold-clone build

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
# Regenerate the VHDL parser from the EBNF source:
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/vhdl.ebnf \
    --generate-parser --output ../generated/vhdl_parser.rs
```

The generated parser lands at `generated/vhdl_parser.rs`. Unlike SystemVerilog, the VHDL parser does not have a stimuli quality gate yet — regeneration is the canonical proof of `grammars/vhdl.ebnf` → `vhdl_parser.rs` correspondence.

## 2. Wire the generated parser into your downstream Cargo build

```bash
export PGEN_VHDL_PARSER_PATH=/absolute/path/to/pgen/generated/vhdl_parser.rs

cargo build --release --features generated_parsers
```

`rust/build.rs` discovers the parser via that environment variable. Absolute paths are safest.

## 3. Parse and walk

```rust
use pgen::embedding_api::{parse_vhdl_1076_2019, ParseStatus};

let outcome = parse_vhdl_1076_2019("entity e is end e;\n");

match outcome.status {
    ParseStatus::Success => {
        // Outcome's `diagnostic` is None on success.
        // For the AST dump, call `parse_vhdl_1076_2019_ast_dump`
        // (see Public API Surface).
    }
    ParseStatus::Failure => {
        eprintln!("parse failed: {:?}", outcome.diagnostic);
    }
}
```

For typed AST output, use `parse_vhdl_1076_2019_ast_dump` instead. The AST dump is the JSON shape this book documents per-rule.

## 4. Write the AST walker

See [Walking the AST](walking-the-ast.md) for the recommended walker pattern. The short version: dispatch on `type` / `kind` discriminators in JSON objects.

```rust
use serde_json::Value as JsonValue;

fn walk(node: &JsonValue) {
    match node {
        JsonValue::Object(obj) => {
            // VHDL typed-shape node; dispatch on `type` (root) or `kind` (variants).
            // E.g. design_unit -> {kind: "library" | "use" | "entity" | ..., body}
        }
        JsonValue::Array(items) => {
            // Sequence / iteration array — walk children.
            for item in items {
                walk(item);
            }
        }
        JsonValue::String(_) => {
            // Terminal text — no children.
        }
        _ => {}
    }
}
```

## 5. Track the contract version

Pin your downstream code to the parser-release version recorded in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity". When you bump to a new PGEN version, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the rules you consume.

For the binop_chain shape used across the 5-level expression hierarchy (`expression` → `relation` → `simple_expression` → `term` → `factor`), see [Walking the AST](walking-the-ast.md) for the consumer-facing left-fold contract.
