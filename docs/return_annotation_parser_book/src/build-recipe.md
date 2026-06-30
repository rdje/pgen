# Build Recipe

The `return_annotation` parser is generated from `grammars/return_annotation.ebnf` through PGEN's
EBNF → raw-AST-JSON → generated-parser pipeline, but on the **bootstrap** path (see
[Backends](backends.md) for why annotation parsers are special).

## Generate the parser

```bash
# regenerate generated/return_annotation_parser.rs (+ generated/return_annotation.json)
make -C rust SHELL=/bin/bash return_annotation_parser
```

This runs the two canonical steps:

1. `grammars/return_annotation.ebnf → generated/return_annotation.json` — the EBNF frontend
   (`--emit-raw-ast-json`).
2. `generated/return_annotation.json → generated/return_annotation_parser.rs` — the **bootstrap**
   generator (`ast_pipeline_bootstrap`). Annotation parsers are generated in bootstrap mode only, so
   the annotation parser is never asked to parse the annotations that define itself.

`return_parser` is an alias for this target, and `return_semantic_parsers` regenerates both annotation
parsers (semantic + return) together.

> `generated/` is regenerated locally and is **not** committed in this clone;
> `make -C rust return_annotation_parser` reproduces `generated/return_annotation_parser.rs`
> deterministically.

## Parse annotation source

Grammar authors do not normally invoke this parser directly — the AST pipeline calls it while
generating a parser from a `.ebnf`. To parse return-annotation source programmatically, use the stable
embedding API (`parse_annotation` / `parse_annotation_result` / `parse_annotation_named` with
`AnnotationFamily::Return`); see [Public API](public-api.md). The host can route a parse to either the
**Bootstrap** or the **Generated** backend (see [Backends](backends.md)).

## Validation / release gates

The return-annotation family's proof surface is exercised by these maintained gates:

```bash
# the focused aggregate Done-gate for the return-annotation claim
# (now includes the auto-derived return_annotation_exhaustiveness_gate:
#  grammar-driven coverage closure + stimuli-module parity + parse-tree→typed-AST audit)
make -C rust SHELL=/bin/bash return_annotation_support_gate

# the runtime/round-trip semantics gate
make -C rust SHELL=/bin/bash return_runtime_semantics_gate

# the aggregate annotation contract spine (validator coverage + built-in/shared suites + …)
make -C rust SHELL=/bin/bash annotation_contract_gate

# the closed-loop stimuli-quality proof (includes the return-annotation generator/parser loop)
make -C rust SHELL=/bin/bash annotation_stimuli_quality_gate
```

## Book gate

This book itself is gated like every other per-parser book — it must build cleanly and ship its
tracked HTML:

```bash
make -C rust SHELL=/bin/bash return_annotation_parser_book_gate
```

The gate requires `mdbook` on `PATH`, checks the chapter set is present, runs
`mdbook build docs/return_annotation_parser_book`, and verifies the rendered HTML landing pages exist
under `docs/return_annotation_parser_book-html/` (the rendered HTML is tracked in git so the book is
browsable directly on GitHub).
