# Quickstart for Downstream Consumers

A minimal "compile this, walk that" recipe for embedding the PGEN rtl_frontend parser. Read [Build Recipe](build-recipe.md) and [Public API Surface](public-api.md) for the long-form versions of each step.

## 1. Cold-clone build

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
# Regenerate the rtl_frontend parser from the EBNF source:
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/rtl_frontend.ebnf \
    --generate-parser --output ../generated/rtl_frontend_parser.rs
```

The generated parser lands at `generated/rtl_frontend_parser.rs`. Regeneration is the canonical proof of `grammars/rtl_frontend.ebnf` → `rtl_frontend_parser.rs` correspondence.

## 2. Wire the generated parser into your downstream Cargo build

```bash
export PGEN_RTL_FRONTEND_PARSER_PATH=/absolute/path/to/pgen/generated/rtl_frontend_parser.rs

cargo build --release --features generated_parsers
```

`rust/build.rs` discovers the parser via that environment variable. Absolute paths are safest.

## 3. Parse via the parser registry

The rtl_frontend family does not yet expose a per-grammar convenience entry point in `pgen::embedding_api`. The stable host surface during this release is the parser-registry handle plus the generic `parse_grammar_profile_named` path.

```rust
use pgen::embedding_api::{parse_grammar_profile_named, ParseStatus};

let outcome = parse_grammar_profile_named(
    "rtl_frontend",
    "default",
    "module m; endmodule\n",
);

match outcome.status {
    ParseStatus::Success => {
        // For the AST dump, call parse_grammar_profile_named_ast_dump
        // (see Public API Surface).
    }
    ParseStatus::Failure => {
        eprintln!("parse failed: {:?}", outcome.diagnostic);
    }
}
```

## 4. Write the AST walker

See [Walking the AST](walking-the-ast.md) for the recommended walker pattern. The short version: dispatch on `kind` (and sometimes `type`) discriminators in JSON objects.

```rust
use serde_json::Value as JsonValue;

fn walk(node: &JsonValue) {
    match node {
        JsonValue::Object(obj) => {
            // rtl_frontend typed-shape node; dispatch on `type` (root)
            // or `kind` (dispatch wrappers like design_item / module_item).
            // E.g. design_item -> {kind: "typedef" | "package" | "module" | "semi", body?}
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

Pin your downstream code to the parser-release version recorded in `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" (currently `1.0.1`). When you bump to a new PGEN version, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the rules you consume.

For the binop_chain shape used across the 10-level expression hierarchy (`expression` → `conditional` → ... → `unary`), see [Walking the AST](walking-the-ast.md) for the consumer-facing left-fold contract.
