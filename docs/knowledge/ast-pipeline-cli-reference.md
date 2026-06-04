---
id: ast-pipeline-cli-reference
title: ast_pipeline CLI — modes, key flags, env vars, and the gotchas
answers:
  - "how do I run the ast_pipeline binary and what are its modes"
  - "how do I generate grammar stimuli from the command line"
  - "how do I dump the grammar IR / gen_ast.json used by stimuli generation"
  - "why does ast_pipeline say it requires --features ebnf_dual_run"
  - "how do I run the witness pass (target-report-input) by hand"
  - "what env vars tune stimuli/witness generation"
  - "how do I write generated stimuli to a file (--output vs positional path)"
  - "how do I lint a grammar for well-formedness from the CLI"
  - "how do I dump the parsed AST of a grammar rule"
tags: [cli, ast_pipeline, stimuli, tooling, reference]
date: 2026-06-04
status: current
evidence: rust/src/main.rs (arg parsing); `rust/target/release/ast_pipeline --help`
reverify: run `rust/target/release/ast_pipeline --help` (flags can change); for .ebnf input the binary must be built `--features ebnf_dual_run`
---

The `ast_pipeline` binary (build: `cargo build --release --bin ast_pipeline`, optionally
`--features ebnf_dual_run` to read `.ebnf` directly) is the one-stop CLI for the EBNF→parser /
stimuli pipeline. `<INPUT>` is a grammar source: a `.json` raw/transformed/gen-ast AST, **or** a
`.ebnf` (only when built `--features ebnf_dual_run` — otherwise you get
`Error: EBNF input '…' requires building with --features ebnf_dual_run`).

## Modes (the `--generate-*` / dump verbs)
- `INPUT --generate-parser --output PARSER.rs` — generate the Rust parser.
- `INPUT --generate-stimuli [--count N] [--seed S] --output FILE` — generate grammar-valid stimuli.
- `INPUT --generate-stimuli-module --output generated/<g>_stimuli.rs` — emit a Rust stimuli module.
- `INPUT --preprocess-systemverilog --output OUT.sv` — run the SV preprocessor stage.
- `INPUT.ebnf --emit-raw-ast-json RAW.json` — export the Rust-frontend raw AST envelope.
- `INPUT --generate-* --dump-gen-ast GEN_AST.json` — dump the **normalized generation-input AST**
  (the grammar IR the stimuli/parser generators actually consume). Regenerate it after a grammar
  edit with: `ast_pipeline grammar.ebnf --generate-parser --emit-raw-ast-json raw.json
  --dump-gen-ast gen_ast.json --eliminate-left-recursion --output parser.rs` (this is exactly
  what `sv_stimuli_quality_gate.sh` does). `--eliminate-left-recursion` is the default-on LR
  pass — see [[pgen-parsing-model]].

## ⚠️ Gotchas (each cost real time)
- **`--output FILE` writes the artifact; a trailing POSITIONAL path does NOT.** A positional
  second arg is the *transformed-AST JSON* output (JSON mode only) — in generation modes it is
  ignored and stimuli print to stdout. `cmp` on a non-existent positional path silently reports
  "differ"/"no such file" → a bogus determinism result. Always use `-o/--output` + verify the file exists.
- **`.ebnf` input needs `--features ebnf_dual_run`.** For the plain release binary, feed a
  `gen_ast.json` (from `--dump-gen-ast`) or a grammar JSON instead.
- Generation under a profile: `--grammar-profile sv_2017|sv_2023` (filters profile-tagged rules —
  see [[project_semantic_annotation_composition_doctrine]]).

## Witness / target-driven coverage (the closed-loop residual surface)
- Witness pass by hand: `INPUT --generate-stimuli --grammar-profile sv_2017 --target-report-input
  SAMPLE.json --target-max-attempts 0 --target-generation-timeout-ms MS --seed S --output OUT.json`.
  Prints `Witness pass: resolved A -> B of N …` + (since PGEN-SV-EXH-PROOF-0150/0151) the
  `target_timeout`-failure samples + still-UNRESOLVED samples. See [[sv-literal-0-residual-decomposition]].
- `--entry-rule R` roots generation at rule R (used to probe a single rule, e.g. orphan checks).
- `--report-k-path-coverage K`, `--gap-report-json/-text` — coverage reports.

## Env vars (A/B tuning, default-off / default-floor)
- `PGEN_WITNESS_TIMEOUT_FLOOR_MS` — witness-pass per-target budget floor (default 200 ms; the gate
  reads it). `PGEN_WITNESS_NO_PURDOM=1` — disable Purdom witness ordering (A/B). See [[sv-witness-purdom-ordering]].

## Other useful flags
`--lint-grammar` (well-formedness checks), `--max-depth N`, `--max-repeat N`,
`--emit-return-annotations-json`, `--trace` / `--trace-log-file` / `--verbosity N` (⚠️ high
verbosity floods — a debug run produced a multi-GB log; prefer the bounded in-summary samples),
`--lib-in DIR` / library import, `--sv-include-dir`, `--coverage-input/-output`.
