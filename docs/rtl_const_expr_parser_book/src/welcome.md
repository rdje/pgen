# Welcome — PGEN rtl_const_expr Parser Integration Reference

This book is the **canonical AST reference** for downstream consumers of PGEN's `rtl_const_expr` parser family. The intended audience is RTLSyn (for deterministic parameter / width / generate evaluation before elaboration) and any other tool needing a focused constant-expression parser.

## What this book covers

- **Build and integration.** See [Build Recipe](build-recipe.md), [Public API Surface](public-api.md).
- **AST envelope and walking conventions.** See [AST Envelope Structure](ast-envelope.md), [Walking the AST](walking-the-ast.md).
- **Per-rule typed shapes.** See [Per-Rule Shape Reference](rules-top-level.md). The grammar (`grammars/rtl_const_expr.ebnf`) carries **24 typed annotations** covering the conditional/binary/unary/primary expression hierarchy plus literal and identifier leaves.
- **The `binop_chain` shape.** All 10 binary-operator chain rules (logical_or through multiplicative) emit a uniform `{type: "binop_chain", level, lhs, rest}` shape so consumers fold left.
- **Worked examples**: see [Literal 42](examples-literal-42.md) and [Binary Addition](examples-binary-addition.md).

## What this book is NOT

- It is **not** a substitute for the integration contract `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` (in progress under `docs/tasks/RTL-CE-CONTRACT-BODY.md`).
- It does **not** cover statements, modules, or any non-expression construct. For those see the rtl_frontend book.

## Status

The PGEN rtl_const_expr parser is **on-demand-only** in the default build. Current grammar covers decimal and sized-based integer literals, identifiers (including dotted and package-qualified names), unary `+ - ! ~`, binary arithmetic / shift / comparison / equality / bitwise / logical operators, and ternary `?:`. The single typing slice (`RTL-CE-Slice-1`) landed on 2026-05-14.
