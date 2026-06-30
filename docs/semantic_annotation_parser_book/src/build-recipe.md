# Build Recipe

The `semantic_annotation` parser is generated from `grammars/semantic_annotation.ebnf` through PGEN's
EBNF → raw-AST-JSON → generated-parser pipeline, on the **bootstrap** path (like its sibling
`return_annotation`; see [Backends](backends.md) for why annotation parsers are special).

## Generate the parser

```bash
# regenerate generated/semantic_annotation_parser.rs (+ generated/semantic_annotation.json)
make -C rust SHELL=/bin/bash semantic_annotation_parser
```

This runs the two canonical steps:

1. `grammars/semantic_annotation.ebnf → generated/semantic_annotation.json` — the EBNF frontend
   (`--emit-raw-ast-json`).
2. `generated/semantic_annotation.json → generated/semantic_annotation_parser.rs` — the **bootstrap**
   generator. Annotation parsers are generated in bootstrap mode only, so the annotation parser is
   never asked to parse the annotations that define itself.

`semantic_parser` is an alias for this target, and `return_semantic_parsers` regenerates both
annotation parsers (semantic + return) together.

> `generated/` is regenerated locally and is **not** committed in this clone;
> `make -C rust semantic_annotation_parser` reproduces `generated/semantic_annotation_parser.rs`
> deterministically.

## Parse annotation source

Grammar authors do not normally invoke this parser directly — the AST pipeline reads semantic
annotations while generating a parser from a `.ebnf` and interprets the steering directives (see
[Steering Directives](steering-directives.md)). To parse semantic-annotation source programmatically,
use the stable embedding API (`parse_annotation` / `parse_annotation_result` / `parse_annotation_named`
with `AnnotationFamily::Semantic`); see [Public API](public-api.md). The host can route a parse to
either the **Bootstrap** or the **Generated** backend (see [Backends](backends.md)).

## Validation / release gates

The semantic-annotation family's proof surface is exercised by these maintained gates:

```bash
# the aggregate annotation contract spine (validator coverage + built-in/shared suites + SC slices + …)
make -C rust SHELL=/bin/bash annotation_contract_gate

# the semantic-annotation leverage contract (does the steering actually move parser/stimuli behavior?)
make -C rust SHELL=/bin/bash semantic_usage_gate

# the semantic runtime / typed-AST contract checks
make -C rust SHELL=/bin/bash semantic_runtime_contract_gate

# the focused aggregate semantic proof surface (runtime + round-trip + differential regression)
make -C rust SHELL=/bin/bash semantic_full_contract_gate
```

## Book gate

This book is gated like every other per-parser book — it must build cleanly and ship its tracked HTML:

```bash
make -C rust SHELL=/bin/bash semantic_annotation_parser_book_gate
```

The gate requires `mdbook` on `PATH`, checks the chapter set is present, runs
`mdbook build docs/semantic_annotation_parser_book`, and verifies the rendered HTML landing pages exist
under `docs/semantic_annotation_parser_book-html/` (the rendered HTML is tracked in git so the book is
browsable directly on GitHub).
