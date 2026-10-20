# Preprocessing as a Service — the Expander over the svpp AST

> **Status (2026-10-20): contract `1.0`, `SVPP-EXPANSION.9.2`.** This chapter describes a service whose contract
> (`docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md`) is at `1.0`: the typed Rust API, the file
> provider, the typed refusal and the probe's emit flags are live, and a currency gate
> (`scripts/check_svpp_expansion_contract_currency.sh`) holds the contract's every vocabulary and version equal to the
> code at every commit. The remaining surfaces land in two measured steps (`.9.3`, `.9.4`). Every paragraph below says
> which of its statements is **live today** and which is **contracted for** a named step. Nothing here is a promise
> without a step.

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
- **The context** — a value, never an implicit filesystem (`PreprocessContext`): the edition profile, the predefined
  macros, the include search directories, the compilation unit's earlier files, the declared policy options, and a
  **file provider** (`FileProvider`, the engine's neutral trait) that answers *path → bytes* — plus *does this path
  exist* for the include search and *what is this path's canonical spelling* for every output that names a file. The
  default (`DiskFileProvider`) reads the disk; `MemoryFileProvider` serves memory over an optional fallback; a
  consumer's own implementation serves anything (a virtual filesystem, a build cache, an editor's unsaved buffers).
- **A refusal** — the expansion did not complete and the parser never sees the text. Typed (`PreprocessRefusal`): a
  **kind** (`RefusalKind`, eighteen of them: an include not found, nesting too deep, an undefined macro, a recursive
  macro, a redefinition under `error`, an unterminated conditional, an unreadable source, an invalid context, …), a
  **code** in the same `E_SVPP_…` vocabulary as the diagnostics, the **file** and **line** the refusing site knew, and
  the **message** — byte-for-byte the text the probe prints after `preprocess_failed`. The kind is attached at the
  refusing site, never parsed back out of the message.
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
| **PGEN expands** | the preprocessed output | compiles against the text and the source map | **Yes** — in-process through the typed API (`preprocess` over paths, `preprocess_str` over text in memory, both over a `FileProvider`) or the neutral stage socket, and as a tool (`ast_pipeline`, or `parseability_probe … --preprocess --emit-preprocessed`). Below. |
| **Intercept** | the AST, then the output | walks or edits the AST (injects a define, serves an include from its own store), then hands it to the expander | **Contracted for `.9.3`** (the expander over the AST). Today's nearest: the context — a consumer's own `FileProvider` serves any include from its own store, and predefined macros (`defines` / `--symbol`) inject definitions — not AST edits. |

## PGEN expands — what works today

As a tool, the expansion alone, with every output as a file:

```bash
ast_pipeline INPUT.sv --preprocess systemverilog \
  [--profile sv_2017|sv_2023|verilog_2005] \
  [--include-dir DIR]... [--symbol NAME[=VALUE]]... [--unit-file FILE]... [--stage-opt KEY=VALUE]... \
  --output OUT.sv --source-map-json MAP.json --event-log-json EVENTS.json --diagnostics-json DIAG.json
```

Expand-then-parse in one process (the SystemVerilog parser's front end; exit `3` = the expander refused, `1` = the parser
rejected the expanded text, `0` = accepted), and — since `.9.2` — emit what was expanded: `--emit-preprocessed` writes the
expanded text, `--emit-source-map` writes the versioned JSON carrier of the other four outputs (`preprocessed_output_schema`,
`service`, `service_version`, then `source_map`, `events`, `diagnostics`, `included_files`). Both are written **before**
the parser runs, so a parse rejection still leaves them; a refusal writes nothing; either flag without `--preprocess` is
refused. The text is byte-identical to `ast_pipeline --output`'s.

```bash
parseability_probe --parse systemverilog INPUT.sv --profile sv_2017 --preprocess [the same context flags] \
    --emit-preprocessed OUT.sv --emit-source-map MAP.json
```

In-process, through the **typed API** (`pgen::families::systemverilog::preprocessing_service`; the contract's `1.0`
surface — its names are the contract, its module path may gain a second spelling with the director's placement answer):

```rust
use pgen::families::systemverilog::preprocessing_service::{preprocess, preprocess_str, MemoryFileProvider, PreprocessContext};
use std::sync::Arc;

// over paths: the unit's files and their `include`s come from the disk (the default provider)
let ctx = PreprocessContext { profile: Some("sv_2017".into()), defines: vec!["SYNTHESIS".into(), "WIDTH=8".into()], ..Default::default() };
let outputs = preprocess(&[Path::new("pkg.sv"), Path::new("top.sv")], &ctx)?;   // Vec<PreprocessedOutput>, index-aligned
let top = &outputs[1];

// in memory: the text is the one file of the unit, its `include`s come from the consumer's own store
let store = MemoryFileProvider::new().with_file("inc.svh", b"`define FROM_STORE 1\n".to_vec());
let ctx = PreprocessContext::default().with_file_provider(Arc::new(store));
let output = preprocess_str("top.sv", "`include \"inc.svh\"\nlogic [`FROM_STORE:0] a;\n", &ctx)?;
assert_eq!(output.text, "logic [1:0] a;\n");

// a refusal is typed: match on it, or print it — the message is what the probe prints after `preprocess_failed`
match preprocess_str("u.sv", "initial `NOT_DEFINED;\n", &ctx) {
    Err(refusal) => eprintln!("{} {} at {:?}:{:?}: {}", refusal.code, format!("{:?}", refusal.kind), refusal.file, refusal.line, refusal.message),
    Ok(_) => unreachable!(),
}
```

`preprocess` runs exactly the code path the socket and PGEN's own SystemVerilog front end run — the same option table,
the same configuration derivation, the same per-file / per-unit entry — so its five outputs are byte-identical to the
socket's for the same unit and context (a test pins it). Two constants carry the service's stability axes:
`PREPROCESSING_SERVICE_VERSION` (`1.0.0`) and `PREPROCESSED_OUTPUT_SCHEMA_VERSION` (`1`).

In-process, through the engine's neutral stage socket (one output per file of the unit, in order; `Err` is a refusal —
the same code path, for a host that names families by string):

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
- **`included_files`** — every file the expansion read: the unit file first, then each include in order of first
  inclusion, each spelled as the file provider canonicalizes it.
- **A refusal** is not one of the five: it is the `Err` — typed in the API (`PreprocessRefusal`: kind, code, file, line,
  message), a chain in the socket, exit `3` and `preprocess_failed …` on the probe — with one message in all three. The
  eighteen refusal codes are listed in the contract beside the diagnostic codes; a refusal that *is* a coded diagnostic
  (a redefinition under `error`, a missing macro argument, a recursive macro) reuses that diagnostic's code.

## One engine, and what it buys

PGEN's SystemVerilog parser does not use a different preprocessor behind the scenes: its `--preprocess` front end and
the service are the same code. Today that code is the hand-written stage; after `.9.3` it is the expander over svpp's
AST, and the SystemVerilog front end runs on it (`.9.4`). The consequence a downstream project relies on: the text and
the verdict it receives are the ones PGEN's own corpus lanes measure
(`docs/systemverilog_parser_book/src/preprocess-parse-pipeline.md`), and a disagreement between the directive parser and
the expander — the class `SVPP-EXPANSION.5`(w) – (ac) names today — closes structurally rather than ticket by ticket.

## What is not yet true, said plainly

- The expander does not yet read svpp's AST, so there is no `expand(ast, ctx)` and no AST-level intercept (`.9.3`); the
  divergences between the two readers are the ledgered rows `SVPP-EXPANSION.5`(w) – (ac), each adjudicated with its
  clause and expected to move under `.9.3`'s reading.
- The source map has no expansion chain yet (schema `1`); schema `2` adds it (`.9.3`).
- The by-name / JSON in-process form and the module path of the typed API (a service of this family, or a third family
  name) are director questions; the names are the contract and will not move.

## Where the authority is

The contract `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md` is authoritative for the service's
surfaces, context, outputs and stability axes; the SystemVerilog integration contract's *"Front End — the
preprocess→parse pipeline"* section stays authoritative for the pipeline command's verdict semantics and the corpus lane
that derives its context; this family's contract stays authoritative for the directive AST. Report a disagreement as a
documentation bug rather than code to the lower-precedence surface.
