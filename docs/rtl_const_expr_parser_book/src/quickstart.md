# Quickstart for Downstream Consumers

A minimal "compile this, walk that" recipe for embedding the PGEN rtl_const_expr parser. This parser is RTLSyn's deterministic constant-expression evaluator (parameter / width / generate evaluation before elaboration). Read [Build Recipe](build-recipe.md) and [Public API Surface](public-api.md) for the long-form versions of each step.

## 1. Cold-clone build

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
# Regenerate the rtl_const_expr parser from the EBNF source:
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/rtl_const_expr.ebnf \
    --generate-parser --output ../generated/rtl_const_expr_parser.rs
```

The generated parser lands at `generated/rtl_const_expr_parser.rs`. There is also a standalone bootstrap crate at `rtl_const_expr/` for early-bootstrap consumers; check the contract for which artifact your downstream pipeline should target.

## 2. Wire the generated parser into your downstream Cargo build

```bash
export PGEN_RTL_CONST_EXPR_PARSER_PATH=/absolute/path/to/pgen/generated/rtl_const_expr_parser.rs

cargo build --release --features generated_parsers
```

`rust/build.rs` discovers the parser via that environment variable. Absolute paths are safest.

## 3. Parse via the parser registry

The rtl_const_expr family does not yet expose a per-grammar convenience entry point in `pgen::embedding_api`. The stable host surface during this release is the generic-by-grammar `parse_grammar_profile_named` path.

```rust
use pgen::embedding_api::{parse_grammar_profile_named, ParseStatus};

let outcome = parse_grammar_profile_named(
    "rtl_const_expr",
    "default",
    "(WIDTH + 1) * 8",
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

## 4. Walk the binop_chain AST

The rtl_const_expr grammar is essentially a precedence-climbing chain of left-associative binary operators (conditional → logical_or → ... → multiplicative) plus prefix-unary plus primary. Each binary level is a `binop_chain` shape:

```json
{
  "type": "binop_chain",
  "level": "multiplicative_expr",
  "lhs": <left-shape>,
  "rest": [
    {"op": "*", "rhs": <right-shape>},
    {"op": "*", "rhs": <right-shape>}
  ]
}
```

Consumers fold `rest` left-associatively against `lhs` to recover the evaluation tree. See [Walking the AST](walking-the-ast.md) for the full fold pattern.

## 5. Track the contract version

Pin your downstream code to the parser-release version recorded in `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" (currently `1.0.1`). When you bump to a new PGEN version, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the rules you consume.
