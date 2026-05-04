# Quickstart for Downstream Consumers

A minimal "compile this, walk that" recipe for embedding the PGEN SystemVerilog parser. Read [Build Recipe](build-recipe.md) and [Public API Surface](public-api.md) for the long-form versions of each step.

> See also: [`parseability_probe` CLI Reference](../../reference/PARSEABILITY_PROBE.md) — canonical reference for the verification tool used in CI / bug-repro flows.

## 1. Cold-clone build

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate
```

The gate produces the generated parser at `rust/target/sv_stimuli_quality_gate/work/systemverilog_parser.rs` and runs it against the stimuli corpus to verify behavior.

## 2. Wire the generated parser into your downstream Cargo build

```bash
# In your downstream project, before invoking cargo:
export PGEN_SYSTEMVERILOG_PARSER_PATH=/absolute/path/to/pgen/rust/target/sv_stimuli_quality_gate/work/systemverilog_parser.rs

cargo build --release --features generated_parsers,sv_2017
```

`rust/build.rs` discovers the parser via that environment variable. Both relative-to-`rust/` and absolute paths work; absolute paths are the safest.

## 3. Parse and walk

```rust
use pgen::embedding_api::{parse_grammar_profile_named, ParseStatus};

let outcome = parse_grammar_profile_named(
    "systemverilog",
    "sv_2017",
    "module m; endmodule\n",
);

match outcome.status {
    ParseStatus::Success => {
        // Outcome's `diagnostic` is None on success.
        // For the AST dump, call `parse_grammar_profile_ast_dump_named`
        // (see Public API Surface).
    }
    ParseStatus::Failure => {
        eprintln!("parse failed: {:?}", outcome.diagnostic);
    }
}
```

For typed AST output, use `parse_grammar_profile_ast_dump_named` instead. The AST dump is the JSON shape this book documents per-rule.

## 4. Write the AST walker

See [Walking the AST](walking-the-ast.md) for the recommended walker pattern. The short version:

```rust
use serde_json::Value as JsonValue;

fn walk(node: &JsonValue) {
    match node {
        JsonValue::Object(obj) => {
            // Typed-shape node; dispatch on `type` / `kind` fields.
            // See per-rule chapters for the field set per rule.
        }
        JsonValue::Array(items) => {
            // Sequence / un-typed shape; walk children.
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

Pin your downstream code to the parser-release version recorded in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity". When you bump to a new PGEN version, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the rules you consume.
