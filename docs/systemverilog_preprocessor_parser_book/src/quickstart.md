# Quickstart for Downstream Consumers

A minimal "compile this, walk that" recipe for embedding the PGEN sv_preprocessor parser. This parser models the SystemVerilog preprocessor directive surface (`define / `undef / `include / `timescale / `default_nettype / `celldefine / conditional compilation). Read [Build Recipe](build-recipe.md) and [Public API Surface](public-api.md) for the long-form versions of each step.

## 1. Cold-clone build

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
# Regenerate the sv_preprocessor parser from the EBNF source:
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/systemverilog_preprocessor.ebnf \
    --generate-parser --output ../generated/systemverilog_preprocessor_parser.rs
```

The generated parser lands at `generated/systemverilog_preprocessor_parser.rs`.

## 2. Wire the generated parser into your downstream Cargo build

```bash
export PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH=/absolute/path/to/pgen/generated/systemverilog_preprocessor_parser.rs

cargo build --release --features generated_parsers
```

`rust/build.rs` discovers the parser via that environment variable. Absolute paths are safest.

## 3. Parse via the parser registry

The sv_preprocessor family does not yet expose a per-grammar convenience entry point in `pgen::embedding_api`. The stable host surface during this release is the generic-by-grammar `parse_grammar_profile_named` path.

```rust
use pgen::embedding_api::{parse_grammar_profile_named, ParseStatus};

let outcome = parse_grammar_profile_named(
    "systemverilog_preprocessor",
    "default",
    "`define WIDTH 8\n`define DEPTH 16\n",
);

match outcome.status {
    ParseStatus::Success => {
        // For the AST dump, call parse_grammar_profile_ast_dump_named
        // (see Public API Surface).
    }
    ParseStatus::Failure => {
        eprintln!("parse failed: {:?}", outcome.diagnostic);
    }
}
```

## 4. Walk the pp_item dispatch

The sv_preprocessor AST root is `systemverilog_preprocessor_file`, which produces a JSON array of zero or more typed `pp_item` objects. Each `pp_item` is a `kind`-tagged variant for one of 10 directive forms:

```json
{"kind": "define", "body": {...}}
{"kind": "undef", "body": {...}}
{"kind": "include", "body": {...}}
{"kind": "timescale", "body": {...}}
{"kind": "default_nettype", "body": {...}}
{"kind": "celldefine"}
{"kind": "endcelldefine"}
{"kind": "conditional", "body": {...}}
{"kind": "non_directive_line", "body": {...}}
{"kind": "blank_line"}
```

See [Walking the AST](walking-the-ast.md) for the full walker pattern.

## 5. Track the contract version

Pin your downstream code to the parser-release version recorded in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" (currently `1.0.3`, AST-dump schema `3`). When you bump to a new PGEN version, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the directives you consume (e.g. the `1.0.2` `SVPP-0001` fix changed `pp_if_branch.keyword` to `{kind: "ifdef"|"ifndef"}`; the `1.0.3` POST-SV-AUDIT fix changed `pp_define.formals` from the raw `{first, rest}` envelope to a clean `macro_formal[]` list).
