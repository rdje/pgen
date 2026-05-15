# Build Recipe

The PGEN sv_preprocessor parser is **on-demand-only**: it is not in the default `cargo test --features generated_parsers` build. It is produced on-demand by running `ast_pipeline` against `grammars/systemverilog_preprocessor.ebnf` into `generated/systemverilog_preprocessor_parser.rs`.

Downstream consumers receive the parser via one of these mechanisms:

1. **Regenerate locally** when working on the parser, and point `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH` at the result.
2. **Vendor the generated file** under your downstream project's source tree and check it into your VCS, tracked against the parser-release version recorded in the contract.
3. **Build into a CI pipeline** that regenerates and runs the family gates before any downstream consumer build.

## Cold-clone build

From a fresh clone of `github.com/richarddje/pgen`:

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/systemverilog_preprocessor.ebnf \
    --generate-parser --output ../generated/systemverilog_preprocessor_parser.rs
```

This reads `grammars/systemverilog_preprocessor.ebnf` and writes the parser to `generated/systemverilog_preprocessor_parser.rs`.

## Verifying the book gate

PGEN ships a book gate for the sv_preprocessor parser book + tracked HTML:

```bash
make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate
```

This runs `mdbook build` over `docs/systemverilog_preprocessor_parser_book/` and verifies the tracked HTML landing pages are present at `docs/systemverilog_preprocessor_parser_book-html/`.

## Wiring into a downstream Cargo build

PGEN's `rust/build.rs` discovers the generated parser via `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH`. Set it before invoking cargo:

```bash
export PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH=/absolute/path/to/generated/systemverilog_preprocessor_parser.rs
cargo build --release --features generated_parsers
```

The path is resolved relative to `rust/` if not absolute. **Use absolute paths to avoid surprises.** The build.rs sets the `has_generated_systemverilog_preprocessor_parser` cfg when the path resolves to a valid file.

## Cargo features

The PGEN crate exposes these features relevant to sv_preprocessor consumers:

- `generated_parsers` — required to enable any generated parser backend.

The sv_preprocessor grammar does not have profile variants — the `default` profile is the only target.

## Verifying the build

After cargo finishes, verify the parser is wired in:

```bash
printf '`define WIDTH 8\n' | \
    ./target/release/parseability_probe \
        --parse-dump-ast-pretty systemverilog_preprocessor /dev/stdin
```

If the probe reports `parse_full passed for grammar 'systemverilog_preprocessor' on '/dev/stdin'`, the integration is good.

## Build / availability requirements

Per the integration contract:

- Downstream consumers should treat the generated backend as **required** for real host integration.
- Build-time generated-parser discovery is mediated by `rust/build.rs`, not by direct use of internal parser modules.

## Family closure / proof gates

Once the parser is built, the sv_preprocessor family exposes validation/release gates that should be run by anyone publishing a parser-release version bump:

```bash
make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate
cargo test --lib --features generated_parsers systemverilog_preprocessor_ast_shape_contract
```

See the integration contract § "Validation / Release Gates" for the full list.
