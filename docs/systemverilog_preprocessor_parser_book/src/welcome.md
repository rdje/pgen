# Welcome — PGEN SystemVerilog Preprocessor Parser Integration Reference

This book is the **canonical AST reference** for downstream consumers of PGEN's `systemverilog_preprocessor` parser family. The preprocessor parser handles directive parsing — every IEEE 1800 clause-22 directive since release `1.0.8` (`define`, `undef`, `undefineall`, `include` with a quoted, angle-bracketed or macro path, `ifdef`/`ifndef`/`elsif`/`else`/`endif`, `timescale`, `default_nettype`, `celldefine`/`endcelldefine`, `resetall`, `unconnected_drive`/`nounconnected_drive`, `pragma`, `line`, `begin_keywords`/`end_keywords`) — conditional-compilation blocks, macro definitions (formals, defaults, bodies with token-paste, stringize, the `` `\`" `` escape and backslash-newline continuation), and the passthrough text shape, including a line that opens with a macro usage.

## What this book covers

- **Build and integration.** See [Build Recipe](build-recipe.md), [Public API Surface](public-api.md), and — for a project that wants the PREPROCESSED text rather than the directive AST — [Preprocessing as a Service](preprocessing-service.md) (the expander over this family's AST, in-process and as a tool; draft contract `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md`).
- **AST envelope and walking conventions.** See [AST Envelope Structure](ast-envelope.md), [Walking the AST](walking-the-ast.md).
- **Per-rule typed shapes.** See [Per-Rule Shape Reference](rules-top-level.md). The grammar (`grammars/systemverilog_preprocessor.ebnf`) carries **112 typed annotations** over 60 rules (release `1.0.9`) covering the pp_item dispatch (18 kinds), per-directive shapes (15), conditional branches (5), and the condition / macro-default / macro-body fragment shapes (12, 7 plus the balanced group, 11 kinds).
- **Worked examples**: [Single Define](examples-single-define.md), [Conditional Compilation](examples-conditional.md), [Clause-22 Directives, a Macro-Usage Line, a Continued Define](examples-clause22.md), [Groups, Comments and Lines That Do Not Parse](examples-groups.md). Since release `1.0.9` the parser reads each IEEE edition through a PROFILE — see [Editions and Profiles](profiles.md).

## What this book is NOT

- It is **not** a substitute for the integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` (in progress under `docs/tasks/SVPP-CONTRACT-BODY.md`).
- It is **not** the main systemverilog parser. The preprocessor is a separate parser family with a line-oriented deterministic grammar.

## Status

The PGEN systemverilog_preprocessor parser is **on-demand-only** in the default build. The single typing slice (`SVPP-Slice-1`) landed on 2026-05-14 covering the full current grammar (40+ rules / 63 new annotations on top of the baseline 1).
