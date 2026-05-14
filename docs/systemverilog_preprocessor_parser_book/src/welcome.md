# Welcome — PGEN SystemVerilog Preprocessor Parser Integration Reference

This book is the **canonical AST reference** for downstream consumers of PGEN's `systemverilog_preprocessor` parser family. The preprocessor parser handles directive parsing (`define`, `undef`, `include`, `ifdef`/`ifndef`/`elsif`/`else`/`endif`, `timescale`, `default_nettype`, `celldefine`/`endcelldefine`), conditional-compilation blocks, macro definitions (formals, defaults, bodies with token-paste and stringize), and the non-directive passthrough text shape.

## What this book covers

- **Build and integration.** See [Build Recipe](build-recipe.md), [Public API Surface](public-api.md).
- **AST envelope and walking conventions.** See [AST Envelope Structure](ast-envelope.md), [Walking the AST](walking-the-ast.md).
- **Per-rule typed shapes.** See [Per-Rule Shape Reference](rules-top-level.md). The grammar (`grammars/systemverilog_preprocessor.ebnf`) carries **64 typed annotations** covering the pp_item dispatch (10 kinds), per-directive shapes (7), conditional branches (5), and macro-body / condition fragment shapes (12+8+9 kinds).
- **Worked examples**: [Single Define](examples-single-define.md), [Conditional Compilation](examples-conditional.md).

## What this book is NOT

- It is **not** a substitute for the integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` (in progress under `docs/tasks/SVPP-CONTRACT-BODY.md`).
- It is **not** the main systemverilog parser. The preprocessor is a separate parser family with a line-oriented deterministic grammar.

## Status

The PGEN systemverilog_preprocessor parser is **on-demand-only** in the default build. The single typing slice (`SVPP-Slice-1`) landed on 2026-05-14 covering the full current grammar (40+ rules / 63 new annotations on top of the baseline 1).
