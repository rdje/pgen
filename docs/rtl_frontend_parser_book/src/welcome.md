# Welcome — PGEN rtl_frontend Parser Integration Reference

This book is the **canonical AST reference** for downstream consumers of PGEN's `rtl_frontend` parser family — primarily RTLSyn and any other tool embedding the PGEN-generated rtl_frontend parser through the `pgen::embedding_api` host surface.

## What this book covers

- **Build and integration.** See [Build Recipe](build-recipe.md), [Public API Surface](public-api.md).
- **AST envelope and walking conventions.** See [AST Envelope Structure](ast-envelope.md), [Walking the AST](walking-the-ast.md).
- **Per-rule typed shapes.** See [Per-Rule Shape Reference](rules-top-level.md). The current grammar (`grammars/rtl_frontend.ebnf`) carries **156 typed annotations** covering design items, declarations, types, ports, statements, expressions (including the `binop_chain` shape for the 10 binary-operator chain rules), and literals.
- **Worked examples** — what does the AST look like for an empty module, a single continuous assign, a generate-for? See [Empty Module](examples-empty-module.md).
- **Schema versioning policy.** See [Schema Versioning](schema-versioning.md).
- **Release-by-release shape changes.** See [Changelog Index](changelog-index.md).

## What this book is NOT

- It is **not** a substitute for the integration contract `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` (in progress under `docs/tasks/RTL-FE-CONTRACT-BODY.md`).
- It is **not** the SystemVerilog LRM. The rtl_frontend grammar covers a synthesizable RTL subset, not the full SV language.

## Status

The PGEN rtl_frontend parser is **on-demand-only** in the default build (it is not in `cargo test --features generated_parsers` until activated). The current grammar covers file-scope typedefs/packages/modules, module headers + imports + parameters + ANSI ports, parameter/localparam declarations, typedefs, imports, genvars, net declarations, continuous assigns, the `always*` family, generate regions (if/for), module instantiations with named/ordered overrides and named/ordered/wildcard port connections, enum/struct/union types, builtin and package-qualified types, and the unary/binary/ternary expression core shared with `rtl_const_expr`.

The rtl_frontend return-annotation campaign landed `RTL-FE-Slice-1..7` on 2026-05-14 across seven slices, growing from 1 → 156 annotations. Subsequent shape-affecting slices get their own changelog entries here.
