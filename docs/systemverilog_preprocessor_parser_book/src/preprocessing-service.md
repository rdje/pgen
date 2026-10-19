# Preprocessing as a Service — the Expander over the svpp AST

> **Status (2026-10-19): DRAFT contract, `SVPP-EXPANSION.9.1`.** This chapter describes a service whose contract is
> drafted (`docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md`) and whose surfaces land in three
> measured steps (`.9.2` – `.9.4`). Every paragraph below says which of its statements is **live today** and which is
> **contracted for** a named step. Nothing here is a promise without a step.

## Why this chapter exists

The `systemverilog_preprocessor` parser (svpp) reads IEEE 1800 clause-22 **directives** into an AST. It does not execute
them: it decides nothing about which conditional groups are live, substitutes no macro, resolves no `` `include ``. A
downstream project whose goal is *"give me the preprocessed SystemVerilog, so I can work on that"* therefore needs a
second thing, the **expander**, and until 2026-10-19 PGEN offered it only as the SystemVerilog parser's own front end,
reachable through one CLI and never emitting the text. The director's direction of 2026-10-19 (recorded verbatim in
`docs/decisions/project_svpp_expansion_is_a_service_over_the_svpp_ast.md`) makes the expander a **service**: usable
in-process and in-memory, usable as a tool, consuming svpp's AST, and used by PGEN's own SystemVerilog parser — one
engine, so a downstream project receives exactly what PGEN's parser consumed.

## Terms

- **svpp** — the `systemverilog_preprocessor` parser family: text → the directive AST (`pp_item`s). Release `1.0.9`,
  AST-dump schema `7`. Layer 1.
- **The expander** — the service that takes the svpp AST plus a **context** and produces the **preprocessed output**.
  Layer 2. Today its substantive logic lives in `rust/src/families/systemverilog/preprocessor.rs` as a hand-written
  directive scanner (the *expansion stage*); `.9.3` re-points that logic at the AST.
- **The context** — a value, never an implicit filesystem: the edition profile, the predefined macros, the include
  search directories, the compilation unit's earlier files, the declared policy options, and (from `.9.2`) a **file
  provider** that answers *path → bytes*, whose default reads disk and whose in-memory implementation is the consumer's.
- **The preprocessed output** — five things: the expanded `text`; a `source_map` (every expanded byte range → the
  file, line and columns it came from); the `events` (one per directive outcome, each saying whether it fired in a
  live group); the `diagnostics` (coded, with severity); and the `included_files`.
- **A consumption mode** — one of the three ways a project uses the service (next section). Each is first-class: none is
  the "real" one with the others as fallbacks.
- **Schema axis** — the preprocessed output has its own schema version, independent of svpp's AST-dump schema, so a
  change to one never silently changes the other.

## The three consumption modes

| Mode | What the consumer gets | What the consumer does | Live today? |
|---|---|---|---|
| **AST only** | svpp's directive AST | expands itself, or never expands (a linter that must see both arms of a conditional; an editor) | **Yes** — the embedding API's generic parse / AST-dump entry points with grammar `systemverilog_preprocessor` and an edition profile (see [Public API Surface](public-api.md)). |
| **PGEN expands** | the preprocessed output | compiles against the text and the source map | **Yes, in-process and as a CLI tool** (below); the typed Rust API with a published output type and an emit flag on the probe are `.9.2`. |
| **Intercept** | the AST, then the output | walks or edits the AST (injects a define, serves an include from its own store), then hands it to the expander | **Contracted for `.9.3`** (the expander over the AST). Today's nearest: predefined macros (`--symbol`) and include directories are context, not AST edits. |

## PGEN expands — what works today

As a tool, the expansion alone, with every output as a file:

```bash
ast_pipeline INPUT.sv --preprocess systemverilog \
  [--profile sv_2017|sv_2023|verilog_2005] \
  [--include-dir DIR]... [--symbol NAME[=VALUE]]... [--unit-file FILE]... [--stage-opt KEY=VALUE]... \
  --output OUT.sv --source-map-json MAP.json --event-log-json EVENTS.json --diagnostics-json DIAG.json
```

Expand-then-parse in one process (the SystemVerilog parser's front end; exit `3` = the expander refused, `1` = the parser
rejected the expanded text, `0` = accepted). It does **not** emit the text — that flag is `.9.2`'s `--emit-preprocessed`:

```bash
parseability_probe --parse systemverilog INPUT.sv --profile sv_2017 --preprocess [the same context flags]
```

In-process, through the engine's neutral stage socket (one output per file of the unit, in order; `Err` is a refusal):

```rust
use pgen::preprocessing_stage::{stage_for, StageRequest};

let stage = stage_for("systemverilog").expect("the SystemVerilog family declares a preprocessing stage");
let request = StageRequest {
    unit: vec!["pkg.sv".into(), "top.sv".into()],   // earlier files first; macro tables carry forward (§22.5.1)
    include_dirs: vec!["include".into()],
    symbols: vec!["SYNTHESIS".into(), "WIDTH=8".into()],
    parse_profile: Some("sv_2017".into()),
    options: vec![("undefined_macro_policy".into(), "error".into())],
};
let outputs = stage.execute(&request)?;           // Vec<StageOutput>
let top = &outputs[1];
let origin = top.origin_of_position(1234);        // Option<&SourceLocation> — the file/line/columns of expanded byte 1234
```

The declared option keys (`ast_pipeline --preprocess systemverilog --list-stage-options <file>` prints them with their
defaults) are the contract's: `include_max_depth`, `include_path_policy`, `macro_redefine_policy`,
`conditional_symbol_policy`, `conditional_expr_policy`, `plain_string_substitution`, `strict_warning_codes`,
`undefined_macro_policy`. An undeclared key is refused by name.

## What the output means

- **`text`** — the expanded compilation-unit file. Under the default `undefined_macro_policy=error`, no backtick survives
  outside a comment or a string: an undefined macro, a function-like macro without its actuals, a stray macro-string token
  or an unknown directive is a refusal with its clause, so the text handed to a parser never carries a directive it would
  silently accept. `keep` is the passthrough for a consumer that resolves the rest itself.
- **`source_map`** — entries `{output_start, output_end, source: {file, line, column_start, column_end}}`, sorted by
  `output_start`; `origin_of_position` is the binary search a diagnostic needs. Inside a `` `begin_keywords `` region a
  token the profile reserves and the region does not is emitted as an *escaped identifier*, and the map points back to
  the unescaped bytes the author wrote. **Contracted for `.9.3` (schema `2`)**: each entry also carries the
  **macro-expansion chain** (`M at f.sv:12` → `N at g.svh:3`), so a downstream message can read *"in the expansion of
  `M` at f.sv:12"*.
- **`events`** — one per directive outcome: `define`, `undef`, `include`, `ifdef`, `ifndef`, `elsif`, `else`, `endif`,
  `passthrough_line`, `skipped_line`, `macro_expand`, `line_directive`, `protected_envelope`, `keywords_region`; each
  with `file`, `line`, `active` (did it fire in a live group) and a `detail`. Coverage and lint tools ask this stream
  first: *which groups were live for this compilation.*
- **`diagnostics`** — `{code, severity, file, line, message, detail}`; `severity` is `warning` or `error`. The codes are
  listed in the contract; `strict_warning_codes=all|none|CODE[,CODE…]` promotes warnings to errors.
- **`included_files`** — every file the expansion read, in order of first inclusion.

## One engine, and what it buys

PGEN's SystemVerilog parser does not use a different preprocessor behind the scenes: its `--preprocess` front end and
the service are the same code. Today that code is the hand-written stage; after `.9.3` it is the expander over svpp's
AST, and the SystemVerilog front end runs on it (`.9.4`). The consequence a downstream project relies on: the text and
the verdict it receives are the ones PGEN's own corpus lanes measure
(`docs/systemverilog_parser_book/src/preprocess-parse-pipeline.md`), and a disagreement between the directive parser and
the expander — the class `SVPP-EXPANSION.5`(w) – (ac) names today — closes structurally rather than ticket by ticket.

## What is not yet true, said plainly

- The probe parses the expanded text and does not emit it; `--emit-preprocessed` is `.9.2`.
- The in-process surface is the neutral stage socket above; a typed `preprocess(…)` / `expand(…)` pair with a published
  output type and a file provider is `.9.2`; the by-name/JSON form is a director question (whether it ships with `.9.2`).
- The expander does not yet read svpp's AST; the divergences between the two readers are the ledgered rows
  `SVPP-EXPANSION.5`(w) – (ac), each adjudicated with its clause and expected to move under `.9.3`'s reading.
- The source map has no expansion chain yet (schema `1`); schema `2` adds it.

## Where the authority is

The contract `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md` is authoritative for the service's
surfaces, context, outputs and stability axes; the SystemVerilog integration contract's *"Front End — the
preprocess→parse pipeline"* section stays authoritative for the pipeline command's verdict semantics and the corpus lane
that derives its context; this family's contract stays authoritative for the directive AST. Report a disagreement as a
documentation bug rather than code to the lower-precedence surface.
