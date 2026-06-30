# Public API and Tooling

This chapter collects the entry points a grammar author uses to turn a `.ebnf` into a working parser and to
embed it. It complements [The Build Recipe](build-recipe.md), which has the copy-paste commands.

## The EBNF frontend CLI (`ast_pipeline`)

`ast_pipeline` is the EBNF frontend and code generator. Reading a `.ebnf` directly requires the
`ebnf_dual_run` feature; certificate-coverage additionally needs `generated_parsers`. The flags a grammar
author reaches for:

| Flag | Purpose |
| --- | --- |
| `--emit-raw-ast-json RAW.json` | EBNF text → raw AST JSON (the grammar IR) |
| `--generate-parser` | raw AST JSON → generated recursive-descent parser |
| `--output PATH` / `-o PATH` | where to write the generated parser |
| `--entry-rule NAME` | the grammar's entry/start rule (for cert-coverage and stimuli) |
| `--grammar-profile NAME` | select a profile (e.g. `sv_2017`, `sv_2023`, `relaxed`) |
| `--lint-grammar` | static well-formedness report, then exit |
| `--report-certificate-coverage` | per-rule proof/witness/UNKNOWN trustworthiness report |
| `--dump-gen-ast FILE` (+ `--dump-gen-ast-pretty`) | the normalized generation-input IR |
| `--eliminate-left-recursion` | run the left-recursion normalization pass |
| `--generate-stimuli` | in-memory stimuli generation |
| `--generate-stimuli-module` | emit a generated stimuli module (`generated/<g>_stimuli.rs`) |
| `--bootstrap-mode` | force the [bootstrap](bootstrap.md) annotation path |

```bash
# the canonical two-step generation
./rust/target/debug/ast_pipeline grammars/foolang.ebnf --emit-raw-ast-json generated/foolang.json
./rust/target/debug/ast_pipeline --generate-parser generated/foolang.json -o generated/foolang_parser.rs
```

For a tracked grammar, the per-grammar `make -C rust focus_<grammar>` target wires both steps for you.

## The parser registry

A generated parser is registered under its grammar name so the tooling and the embedding API can find it.
`parseability_probe --supports <grammar>` checks a grammar is registered; `--parse <grammar> file` parses
with it. Registration is what makes `parseability_probe --parse foolang …` resolve to your generated
parser.

## The embedding API

To call a generated parser from Rust, use the stable embedding API (`rust/src/embedding_api.rs`; contract
in `rust/docs/EMBEDDING_API_CONTRACT.md`). It exposes parse entry points, the typed-AST result, and
diagnostics, and — for the annotation grammars — a family selector routing to the Bootstrap or Generated
backend. The annotation-family entry points (`parse_annotation` / `parse_annotation_result` /
`parse_annotation_named`) are documented in the
[return_annotation](../../return_annotation_parser_book/src/public-api.md) and
[semantic_annotation](../../semantic_annotation_parser_book/src/public-api.md) books.

## Debugging tooling

When a grammar misbehaves, `TOOLBOX.md` is the authoritative catalog. The grammar author's most-used
surfaces:

- `parseability_probe --parse … 2>&1 | tail -1` — the `furthest_position` of a rejection;
- `parseability_probe --parse-dump-ast-pretty …` — confirm the produced shape;
- `PGEN_TRACE_VERBOSITY=high … --trace-rules <rule>` — watch a specific rule, including `@predicate`
  verdicts;
- `--lint-grammar` and `--report-certificate-coverage` — the two proofs (see
  [Codegen Mental Model](codegen-model.md)).

## Where to go next

- [The Build Recipe](build-recipe.md) — the full author loop and the gates.
- The platform book's [Parser Families](../../book/src/parser-families.md) chapter — every shipped
  grammar and its per-parser book.
