# Build Recipe

The PGEN rtl_const_expr parser is **on-demand-only**: it is not in the default `cargo test --features generated_parsers` build. It is produced on-demand by running `ast_pipeline` against `grammars/rtl_const_expr.ebnf` into `generated/rtl_const_expr_parser.rs`.

There is also a standalone bootstrap crate at `rtl_const_expr/` used to seed the PGEN bootstrap pipeline. Most downstream consumers should target the generated `rtl_const_expr_parser.rs` file rather than the bootstrap crate — see the integration contract for guidance.

## Cold-clone build

From a fresh clone of `github.com/richarddje/pgen`:

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/rtl_const_expr.ebnf \
    --generate-parser --output ../generated/rtl_const_expr_parser.rs
```

This reads `grammars/rtl_const_expr.ebnf` and writes the parser to `generated/rtl_const_expr_parser.rs`.

## Verifying the book gate

PGEN ships a book gate for the rtl_const_expr parser book + tracked HTML:

```bash
make -C rust SHELL=/opt/homebrew/bin/bash rtl_const_expr_parser_book_gate
```

This runs `mdbook build` over `docs/rtl_const_expr_parser_book/` and verifies the tracked HTML landing pages are present at `docs/rtl_const_expr_parser_book-html/`.

## Wiring into a downstream Cargo build

PGEN's `rust/build.rs` discovers the generated parser via `PGEN_RTL_CONST_EXPR_PARSER_PATH`. Set it before invoking cargo:

```bash
export PGEN_RTL_CONST_EXPR_PARSER_PATH=/absolute/path/to/generated/rtl_const_expr_parser.rs
cargo build --release --features generated_parsers
```

The path is resolved relative to `rust/` if not absolute. **Use absolute paths to avoid surprises.** The build.rs sets the `has_generated_rtl_const_expr_parser` cfg when the path resolves to a valid file.

## Cargo features

The PGEN crate exposes these features relevant to rtl_const_expr consumers:

- `generated_parsers` — required to enable any generated parser backend.

The rtl_const_expr grammar does not have profile variants — the `default` profile is the only target.

## Verifying the build

After cargo finishes, verify the parser is wired in:

```bash
echo '(WIDTH + 1) * 8' | \
    ./target/release/parseability_probe --parse-dump-ast-pretty rtl_const_expr /dev/stdin
```

If the probe reports `parse_full passed for grammar 'rtl_const_expr' on '/dev/stdin'`, the integration is good.

## Build / availability requirements

Per the integration contract:

- Downstream consumers should treat the generated backend as **required** for real host integration.
- Build-time generated-parser discovery is mediated by `rust/build.rs`, not by direct use of internal parser modules.

## Family closure / proof gates

Once the parser is built, the rtl_const_expr family exposes validation/release gates that should be run by anyone publishing a parser-release version bump:

```bash
make -C rust SHELL=/opt/homebrew/bin/bash rtl_const_expr_parser_book_gate
cargo test --lib --features generated_parsers rtl_const_expr_ast_shape_contract
```

See the integration contract § "Validation / Release Gates" for the full list.
