# User-Facing Surfaces

PGEN exposes a few user-facing surfaces that matter much more than the rest:

## 1. Grammar inputs

- `grammars/*.ebnf`
- bootstrap-safe annotation grammars
- tracked parser-family grammars such as regex, SystemVerilog, VHDL, and ongoing Phase S grammars

## 2. Generated artifacts

- `generated/*.json`
- `generated/*_parser.rs`
- optional `generated/*_stimuli.rs`

## 3. `ast_pipeline`

The main CLI is the central user surface for:

- raw AST export,
- parser generation,
- stimuli generation,
- stimuli module generation,
- parseability-aware generation,
- target-driven coverage and replay,
- newer stimuli controls such as:
  - mutation mode,
  - constrained-random steering,
  - near-valid negative generation,
  - machine-readable corpus bundle export.

## 4. Make-based gate surface

Most serious workflows are exposed through `make -C rust SHELL=/bin/bash ...`, especially for:

- aggregate policy checks,
- contract gates,
- workflow-parity checks,
- family-specific quality gates,
- cross-family stimuli validation.

## 5. Embedding and registry surfaces

PGEN also ships downstream integration seams through:

- parser registry paths,
- embedding API paths,
- parser-family integration contracts.

## Primary Source Docs

- `PGEN_USER_GUIDE.md`
- `rust/docs/EMBEDDING_API_CONTRACT.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
