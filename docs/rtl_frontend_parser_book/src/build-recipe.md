# Build Recipe

The PGEN rtl_frontend parser is **on-demand-only**: it is not in the default `cargo test --features generated_parsers` build. It is produced on-demand by running `ast_pipeline` against `grammars/rtl_frontend.ebnf` into `generated/rtl_frontend_parser.rs`.

Downstream consumers receive the parser via one of these mechanisms:

1. **Regenerate locally** when working on the parser, and point `PGEN_RTL_FRONTEND_PARSER_PATH` at the result.
2. **Vendor the generated file** under your downstream project's source tree and check it into your VCS, tracked against the parser-release version recorded in the contract.
3. **Build into a CI pipeline** that regenerates and runs the family gates before any downstream consumer build.

## Cold-clone build

From a fresh clone of `github.com/richarddje/pgen`:

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/rtl_frontend.ebnf \
    --generate-parser --output ../generated/rtl_frontend_parser.rs
```

This reads `grammars/rtl_frontend.ebnf` and writes the parser to `generated/rtl_frontend_parser.rs`.

## Verifying the book gate

PGEN ships a book gate for the rtl_frontend parser book + tracked HTML:

```bash
make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_parser_book_gate
```

This runs `mdbook build` over `docs/rtl_frontend_parser_book/` and verifies the tracked HTML landing pages are present at `docs/rtl_frontend_parser_book-html/`.

## Wiring into a downstream Cargo build

PGEN's `rust/build.rs` discovers the generated parser via `PGEN_RTL_FRONTEND_PARSER_PATH`. Set it before invoking cargo:

```bash
export PGEN_RTL_FRONTEND_PARSER_PATH=/absolute/path/to/generated/rtl_frontend_parser.rs
cargo build --release --features generated_parsers
```

The path is resolved relative to `rust/` if not absolute. **Use absolute paths to avoid surprises.** The build.rs sets the `has_generated_rtl_frontend_parser` cfg when the path resolves to a valid file.

## Cargo features

The PGEN crate exposes these features relevant to rtl_frontend consumers:

- `generated_parsers` — required to enable any generated parser backend.

The rtl_frontend grammar does not have profile variants — the `default` profile is the only target.

## Verifying the build

After cargo finishes, verify the parser is wired in:

```bash
echo 'module m; endmodule' | \
    ./target/release/parseability_probe --parse-dump-ast-pretty rtl_frontend /dev/stdin
```

If the probe reports `parse_full passed for grammar 'rtl_frontend' on '/dev/stdin'`, the integration is good.

## Build / availability requirements

Per the integration contract:

- Downstream consumers should treat the generated backend as **required** for real host integration.
- Build-time generated-parser discovery is mediated by `rust/build.rs`, not by direct use of internal parser modules.

See `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` § "Build / Availability Requirements" for the full list.

## Family closure / proof gates

Once the parser is built, the rtl_frontend family exposes validation/release gates that should be run by anyone publishing a parser-release version bump:

```bash
make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_parser_book_gate
make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_generated_contract_gate
cargo test --lib --features generated_parsers rtl_frontend_ast_shape_contract
```

See the integration contract § "Validation / Release Gates" for the full list.
