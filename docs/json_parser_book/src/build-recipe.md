# Build Recipe

The `json` parser is generated from `grammars/json.ebnf` like any other PGEN parser, via the EBNF →
raw-AST-JSON → generated-parser pipeline.

## Generate the parser

```bash
# regenerate generated/json_parser.rs from grammars/json.ebnf
make -C rust SHELL=/bin/bash focus_json
```

This runs the two canonical steps (mirrored from the regex flow):

1. `grammars/json.ebnf → generated/json.json` (the EBNF frontend, `--emit-raw-ast-json`).
2. `generated/json.json → generated/json_parser.rs` (the generator,
   `ast_pipeline --generate-parser`).

The generated parser carries the standard transactional coverage instrumentation
(`enable_coverage` / `exercised_rule_names`), so it participates in the certificate-coverage gate.

> `generated/` is regenerated locally and is not committed in this clone; `make -C rust focus_json`
> reproduces `generated/json_parser.rs` deterministically.

## Parse a file

Build `parseability_probe` with the json parser compiled in (the heavy parsers can be skipped for a fast
build), then parse:

```bash
cd rust && PGEN_SYSTEMVERILOG_PARSER_PATH=/nonexistent PGEN_VHDL_PARSER_PATH=/nonexistent \
  PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH=/nonexistent PGEN_RTL_CONST_EXPR_PARSER_PATH=/nonexistent \
  PGEN_RTL_FRONTEND_PARSER_PATH=/nonexistent PGEN_REGEX_PARSER_PATH=/nonexistent \
  cargo build --features generated_parsers,ebnf_dual_run --bin parseability_probe && cd ..

# accept/reject (exit 0 = accepted, nonzero = rejected)
./rust/target/debug/parseability_probe --parse json path/to/input.json

# dump the typed AST envelope
./rust/target/debug/parseability_probe --parse-dump-ast-pretty json path/to/input.json out.json
```

## Certificate-coverage (EBNF-internal quality)

```bash
./rust/target/debug/ast_pipeline grammars/json.ebnf --report-certificate-coverage --entry-rule json
# => total=9 proof=0 witness=9 UNKNOWN=0 fully_certified=true (sample_parse_failures=0)
```

## External-corpus characterization (real-world conformance)

```bash
PGEN_PARSEABILITY_PROBE=./rust/target/debug/parseability_probe \
  json_corpus_bundle/scripts/run_json_corpus.sh
```

See [External-Corpus Characterization](external-corpus-characterization.md) for the interpretation.

## Book gate

```bash
make -C rust SHELL=/bin/bash json_parser_book_gate
```
