# Build Recipe

This chapter is the end-to-end loop for a grammar author: write a `.ebnf`, generate its parser, parse a
sample, and prove the grammar with PGEN's gates. The two binaries you will use are introduced in
`TOOLBOX.md`; everything here uses the canonical flow from [Welcome](welcome.md).

## The two binaries

| Binary | Build (from `rust/`) | Used for |
| --- | --- | --- |
| `ast_pipeline` | `cargo build --features "generated_parsers ebnf_dual_run"` | EBNF frontend, `--generate-parser`, lint, certificate-coverage, `--dump-gen-ast`, stimuli |
| `parseability_probe` | `cargo build --release --features generated_parsers` | parse a file, dump the typed AST, trace |

`ast_pipeline` needs `--features ebnf_dual_run` to read a `.ebnf` **directly**, and
`--features generated_parsers` for certificate-coverage (it verifies witnesses through the real generated
parser).

## Generate a parser from a grammar

The fastest path for a tracked grammar is its per-grammar `make` target, which runs the whole pipeline:

```bash
# regenerate generated/<grammar>_parser.rs (+ generated/<grammar>.json) for a tracked grammar
make -C rust focus_regex
make -C rust focus_json
make -C rust focus_systemverilog
```

> `generated/` is **not** tracked in git — it is regenerated locally. The `make focus_<grammar>` targets
> reproduce `generated/<grammar>_parser.rs` deterministically.

The two underlying steps the target wires together are the canonical flow:

```bash
# 1) EBNF text → raw AST JSON (the grammar IR)
./rust/target/debug/ast_pipeline grammars/foolang.ebnf --emit-raw-ast-json generated/foolang.json

# 2) raw AST JSON → generated recursive-descent parser
./rust/target/debug/ast_pipeline --generate-parser generated/foolang.json -o generated/foolang_parser.rs
```

The annotation grammars (`return_annotation`, `semantic_annotation`) are generated on the **bootstrap**
path instead; see [The Bootstrap Path](bootstrap.md).

## Parse a sample with the generated parser

Once the grammar is registered and built into `parseability_probe`, parse a file and inspect the AST:

```bash
# does it parse? (exit 0 = fully consumed)
./rust/target/release/parseability_probe --supports foolang
./rust/target/release/parseability_probe --parse foolang sample.foo

# what shape did it produce?
./rust/target/release/parseability_probe --parse-dump-ast-pretty foolang sample.foo out.json
```

On a rejection, the error line carries `furthest_position` — the deepest byte any branch reached — which
is where the real defect lives (see `TOOLBOX.md` §3.2).

## Prove the grammar

Two read-only proofs are the grammar author's first stop after any edit:

```bash
# static well-formedness: left-recursion info, non-terminating ERRORS, ordered-choice shadowing WARNINGS
./rust/target/debug/ast_pipeline grammars/foolang.ebnf --lint-grammar

# trustworthiness: every rule covered by an unreachability PROOF or a reachability WITNESS?
./rust/target/debug/ast_pipeline grammars/foolang.ebnf --report-certificate-coverage \
    --entry-rule foolang_file --count 40 --seed 0
```

`UNKNOWN=0` with no failures, deterministic across seeds `0/7/42`, is the objective "trustworthy on this
grammar" number. See [Codegen Mental Model](codegen-model.md) and the platform book's
[Grammar Well-Formedness](../../book/src/grammar-wellformedness.md) chapter.

## See what the generator actually consumes

When a parse or shape surprises you, dump the **normalized generation-input AST** — the IR the generators
consume after left-recursion elimination and other normalization:

```bash
./rust/target/debug/ast_pipeline grammars/foolang.ebnf --generate-parser \
    --dump-gen-ast gen_ast.json --dump-gen-ast-pretty --eliminate-left-recursion --output /tmp/p.rs
```

## Book gate

This book is gated like every other per-parser book — it must build cleanly and ship its tracked HTML:

```bash
make -C rust SHELL=/bin/bash ebnf_parser_book_gate
```

The gate requires `mdbook` on `PATH`, checks the chapter set is present, runs
`mdbook build docs/ebnf_parser_book`, and verifies the rendered HTML landing pages exist under
`docs/ebnf_parser_book-html/` (the rendered HTML is tracked in git so the book is browsable directly on
GitHub).
