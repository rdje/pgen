# docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `systemverilog_preprocessor` frontend/parsing stage.

## Contract Identity
- Contract version:
  - `1.0.1`
- Parser release version:
  - `1.0.1`
- systemverilog_preprocessor AST-dump schema version:
  - `1`
- Last updated:
  - `2026-05-15`
- Current grammar family label:
  - `systemverilog_preprocessor`
- Per-family mdBook:
  - `docs/systemverilog_preprocessor_parser_book/` (tracked HTML at `docs/systemverilog_preprocessor_parser_book-html/`)
- Per-family gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate`
- Per-family ast-shape-contract manifest:
  - `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`

## Schema Versioning

The systemverilog_preprocessor parser carries two version axes:

1. **Parser release version** (`1.0.1`). Tracks the parser library's release identity.
2. **AST-dump schema version** (`1`). Tracks the AST output shape.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 1.0.0 | 1.0.1 | **SVPP-Slice-1** — initial 64-annotation baseline. pp_item dispatch (10 kinds), 7 directive shapes (define/undef/include/timescale/default_nettype/celldefine/endcelldefine), include_path/nettype_value/time_literal, conditional-compilation tree (5 nodes), condition_expr/condition_atom (12 kinds), macro_formals/formal/default_value/default_atom (8 kinds) / body/body_fragment (9 kinds), passthrough lines. |
| 0.1.0 | 1.0.0 | Foundation baseline. Grammar (`grammars/systemverilog_preprocessor.ebnf`) with the `systemverilog_preprocessor_file -> {type, items}` root only. AST dump is the recursive-envelope shape across all other rules. |

## Release 1.0.1 / Contract 1.0.1 Highlights — SVPP-Slice-1: full grammar typed (40+ rules / 63 annotations)

Single comprehensive slice landed on 2026-05-14 covering the entire grammar surface:

```ebnf
# File root (pre-existing)
systemverilog_preprocessor_file  -> {type: "systemverilog_preprocessor_file", items}

# Dispatch wrapper (10 kinds)
pp_item                          -> {kind: "define" | "undef" | "include" | "timescale"
                                          | "default_nettype" | "celldefine" | "endcelldefine"
                                          | "conditional" | "non_directive_line" | "blank_line",
                                     body?}

# Per-directive shapes (7)
pp_define                        -> {name, formals, body}
pp_undef                         -> {name, comment}
pp_include                       -> {path, comment}
pp_timescale                     -> {unit, precision, comment}
pp_default_nettype               -> {nettype, comment}
pp_celldefine                    -> {comment}
pp_endcelldefine                 -> {comment}

# Include path + nettype (2 kinds each)
include_path                     -> {kind: "quoted"|"angle", text}
nettype_value                    -> {kind: "identifier"|"none", body?}
time_literal                     -> {value, unit}

# Conditional compilation (5 nodes)
pp_conditional                   -> {if_branch, elsif_branches, else_branch}
pp_if_branch                     -> {keyword, macro, tail, items}
pp_elsif_branch                  -> {condition, items}
pp_else_branch                   -> {tail, items}
pp_endif                         -> {tail}

# Condition expression (12-kind atom)
condition_expr                   -> {atoms}
condition_atom                   -> {kind: "token_paste"|"stringize"|"macro_reference"|"text"
                                          |"lparen"|"rparen"|"comma"|"question"|"colon"
                                          |"logical_or"|"logical_and"|"bang", body?}

# Macro formals + default values (8-kind atom)
macro_formals                    -> {first, rest}
macro_formal                     -> {name, default}
macro_default_value              -> {atoms}
macro_default_atom               -> {kind: "token_paste"|"stringize"|"macro_reference"|"text"
                                          |"lparen"|"rparen"|"question"|"colon", body?}

# Macro body fragment (9 kinds)
macro_body                       -> {fragments}
macro_body_fragment              -> {kind: "token_paste"|"stringize"|"macro_reference"|"text"
                                          |"lparen"|"rparen"|"comma"|"question"|"colon", body?}

# Passthrough lines
pp_non_directive_line            -> {text}
pp_blank_line                    -> {kind: "blank"}
```

Annotation count: **64** (was 1 / foundation baseline). Same accept set.

## Source Of Truth
- Grammar source:
  - `grammars/systemverilog_preprocessor.ebnf`
- Runtime execution stage:
  - `rust/src/sv_preprocessor.rs`
- Generated-parser build discovery:
  - `rust/build.rs`
  - `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH`
- Current operational guide:
  - `PGEN_USER_GUIDE.md`
- Live status surface:
  - `LIVE_ACHIEVEMENT_STATUS.md`

## Stable Integration Surface
- Current downstream-facing contract is narrower than the main SystemVerilog/VHDL/regex host surface.
- The repository does expose generated-parser registry coverage for `systemverilog_preprocessor`, but it does not currently publish a dedicated general-purpose embedding API profile for it in `pgen::embedding_api`.
- The practical stable surface today is:
  - the Rust preprocessor execution/runtime module in `rust/src/sv_preprocessor.rs`
  - the executable quality and differential gates documented in `PGEN_USER_GUIDE.md`

## Build / Availability Requirements
- Do not treat internal parser-registry exposure as equivalent to a published general-purpose downstream host contract.
- If a downstream project needs a generic public embedding API for `systemverilog_preprocessor`, that should be treated as new product-surface work, not assumed from current internal registry availability.

## Validation / Release Gates
- `make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate`
- `make -C rust SHELL=/bin/bash sv_preprocessor_curated_differential_gate`
- `make -C rust SHELL=/bin/bash sv_preprocessor_template_differential_gate`

## Scope / Non-Goals
- This document is intentionally explicit that `systemverilog_preprocessor` does not yet have the same published host-embedding shape as `systemverilog`, `vhdl`, or `regex`.
- Downstream consumers should not couple themselves to internal generated parser modules as if they were already a stable public API.
- If a downstream integrator still reports a reproducible preprocessor/runtime bug, use `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` and log accepted released-parser issues in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
