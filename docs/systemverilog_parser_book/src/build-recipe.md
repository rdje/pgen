# Build Recipe

The PGEN SystemVerilog parser is **not in the default `cargo test --features generated_parsers` build**. It is produced on-demand by the `sv_stimuli_quality_gate` make target (and similar gates) into `rust/target/<gate>/work/systemverilog_parser.rs` and discarded after the gate run.

This is intentional — the generated SV parser is a large file (~MB scale) and regenerating it on every `cargo build` would be costly. Downstream consumers receive the parser via one of three mechanisms:

1. **Run the gate locally**, copy the generated file to a stable location, and point `PGEN_SYSTEMVERILOG_PARSER_PATH` at it (this is the recommended path for downstream development).
2. **Vendor the generated file** under your downstream project's source tree and check it into your VCS, tracked against the parser-release version recorded in the contract.
3. **Run the gate in CI** as part of your downstream build pipeline.

## Cold-clone build

From a fresh clone of `github.com/richarddje/pgen`:

```bash
git clone https://github.com/richarddje/pgen.git
cd pgen
make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate
```

The gate:

1. Reads `grammars/systemverilog.ebnf` (and the per-profile wrappers).
2. Generates the parser via PGEN's codegen.
3. Writes the parser to `rust/target/sv_stimuli_quality_gate/work/systemverilog_parser.rs`.
4. Compiles it.
5. Runs the SV stimuli corpus against it.
6. Validates AST shape contract (`rust/test_data/ast_shape_contract/systemverilog_v1.json`).

## Pinning the generated parser

Once `sv_stimuli_quality_gate` succeeds, the parser at `rust/target/sv_stimuli_quality_gate/work/systemverilog_parser.rs` is the version that matches the current source tree's grammar + codegen. To pin it:

```bash
cp rust/target/sv_stimuli_quality_gate/work/systemverilog_parser.rs \
   /your/vendoring/location/systemverilog_parser.rs
sha256sum /your/vendoring/location/systemverilog_parser.rs
```

Record the SHA256 in your downstream project's manifest. When you bump the PGEN version, re-run the gate and verify the SHA changed only because of expected grammar/codegen changes (cross-reference [Changelog Index](changelog-index.md) for the parser-release version).

## Wiring into a downstream Cargo build

PGEN's `rust/build.rs` discovers the generated parser via `PGEN_SYSTEMVERILOG_PARSER_PATH`. Set it before invoking cargo:

```bash
export PGEN_SYSTEMVERILOG_PARSER_PATH=/absolute/path/to/systemverilog_parser.rs
cargo build --release --features generated_parsers,sv_2017
```

The path is resolved relative to `rust/` if not absolute. **Use absolute paths to avoid surprises.** The build.rs will complain at compile time if the path doesn't exist or doesn't point at a valid generated parser file.

## Cargo features

The PGEN crate exposes these features relevant to SV consumers:

- `generated_parsers` — required to enable any generated parser backend.
- `sv_2017` — enable the IEEE 1800-2017 profile.
- `sv_2023` — enable the IEEE 1800-2023 profile.

Both profiles share the same `grammars/systemverilog.ebnf` source; the profile distinguishes which top-level entry rule the parser starts at.

## Verifying the build

After cargo finishes, verify the parser is wired in:

```bash
cargo run --release --features generated_parsers,sv_2017 \
    --bin parseability_probe -- \
    --parse systemverilog /path/to/input.sv \
    --profile sv_2017
```

If the probe reports `parse_full passed for grammar 'systemverilog' on '/path/to/input.sv'`, the integration is good.

> See the [`parseability_probe` CLI Reference](../../reference/PARSEABILITY_PROBE.md) for the full flag set (`--parse-dump-ast`, `--parse-dump-ast-pretty`, `--max-bytes`, `--trace`, `--trace-log-file`, etc.) and the supported grammar / profile matrix.

## Build / availability requirements

Per the integration contract:

- Downstream consumers should treat the generated backend as **required** for real host integration.
- Startup should inspect `parser_embedding_api_contract().supports_systemverilog_generated_backend` to verify the backend is available.
- Build-time generated-parser discovery is mediated by `rust/build.rs`, not by direct use of internal parser modules.

See `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` § "Build / Availability Requirements" for the full list.
