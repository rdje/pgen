# Build Recipe

The PGEN VHDL parser is **on-demand-only**: it is not in the default `cargo test --features generated_parsers` build. It is produced on-demand by running `ast_pipeline` against `grammars/vhdl.ebnf` into `generated/vhdl_parser.rs`.

This is intentional — the generated VHDL parser is a large file (~MB scale) and regenerating it on every `cargo build` would be costly. Downstream consumers receive the parser via one of these mechanisms:

1. **Regenerate locally** when working on the parser, and point `PGEN_VHDL_PARSER_PATH` at the result.
2. **Vendor the generated file** under your downstream project's source tree and check it into your VCS, tracked against the parser-release version recorded in the contract.
3. **Build into a CI pipeline** that regenerates and runs the family gates before any downstream consumer build.

## Cold-clone build

From a fresh clone of `github.com/richarddje/pgen`:

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/vhdl.ebnf \
    --generate-parser --output ../generated/vhdl_parser.rs
```

This reads `grammars/vhdl.ebnf` and writes the parser to `generated/vhdl_parser.rs`.

## Verifying the book gate

PGEN ships a book gate for the VHDL parser book + tracked HTML:

```bash
make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_book_gate
```

This runs `mdbook build` over `docs/vhdl_parser_book/` and verifies the tracked HTML landing pages are present at `docs/vhdl_parser_book-html/`.

## Wiring into a downstream Cargo build

PGEN's `rust/build.rs` discovers the generated parser via `PGEN_VHDL_PARSER_PATH`. Set it before invoking cargo:

```bash
export PGEN_VHDL_PARSER_PATH=/absolute/path/to/generated/vhdl_parser.rs
cargo build --release --features generated_parsers
```

The path is resolved relative to `rust/` if not absolute. **Use absolute paths to avoid surprises.** The build.rs sets the `has_generated_vhdl_parser` cfg when the path resolves to a valid file.

## Cargo features

The PGEN crate exposes these features relevant to VHDL consumers:

- `generated_parsers` — required to enable any generated parser backend.

No profile-feature flag is needed for VHDL — the `vhdl_1076_2019` profile is the only stable host profile and is selected via the embedding API entry point (`parse_vhdl_1076_2019` / `parse_grammar_profile`), not via cargo features.

## Verifying the build

After cargo finishes, verify the parser is wired in:

```bash
echo 'entity e is end e;' | \
    ./target/release/parseability_probe --parse-dump-ast-pretty vhdl /dev/stdin
```

If the probe reports `parse_full passed for grammar 'vhdl' on '/dev/stdin'`, the integration is good.

## Build / availability requirements

Per the integration contract:

- Downstream consumers should treat the generated backend as **required** for real host integration.
- Startup should inspect `parser_embedding_api_contract().supports_vhdl_generated_backend` to verify the backend is available.
- Build-time generated-parser discovery is mediated by `rust/build.rs`, not by direct use of internal parser modules.

See `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` § "Build / Availability Requirements" for the full list.

## Family closure / proof gates

Once the parser is built, the VHDL family exposes several validation/release gates that should be run by anyone publishing a parser-release version bump:

```bash
make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_family_status_gate
make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_family_status_contract_gate
make -C rust SHELL=/opt/homebrew/bin/bash vhdl_combined_telemetry_contract_gate
```

See the integration contract § "Validation / Release Gates" for the full list.
