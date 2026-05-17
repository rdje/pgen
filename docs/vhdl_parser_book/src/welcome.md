# Welcome — PGEN VHDL Parser Integration Reference

This book is the **canonical AST reference** for downstream consumers of PGEN's `vhdl` parser family. The intended audience is any tool embedding the PGEN-generated VHDL parser through the `pgen::embedding_api` host surface (`parse_vhdl_1076_2019`, `parse_grammar_profile`, etc.).

## What this book covers

- **How to build the generated parser and wire it into your project.** See [Build Recipe](build-recipe.md) and [Public API Surface](public-api.md).
- **What the AST envelope looks like.** See [AST Envelope Structure](ast-envelope.md) and [Walking the AST](walking-the-ast.md).
- **The shape every grammar rule produces in the AST dump.** See [Per-Rule Shape Reference](rules-top-level.md). The current grammar (`grammars/vhdl.ebnf`) has **256 typed annotations** covering design units, declarations, types, statements, expressions, and literals.
- **Worked examples by VHDL feature** — what does the AST look like for a minimal entity, a basic architecture, a package? Each example chapter pins a current production AST so consumers can write their walkers against a concrete, tested reference.
- **Schema versioning policy.** See [Schema Versioning](schema-versioning.md). The schema is currently at version `1`.
- **A release-by-release index of what changed and why.** See [Changelog Index](changelog-index.md).

## What this book is NOT

- It is **not** a substitute for the integration contract `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`. The contract is the authoritative source of truth; this book documents what the contract describes. **Where the book and the contract disagree, the contract wins.**
- It is **not** the IEEE 1076 LRM. The LRM is the authoritative VHDL language reference; this book documents PGEN's parser-side AST shape for the LRM-conformant inputs the parser accepts.
- It is **not** a tutorial on VHDL. It assumes you are already an embedded user of the language and need to walk PGEN's AST output programmatically.

## How to use this book

1. **First-time integrators**: read [Quickstart](quickstart.md), then [Build Recipe](build-recipe.md), then [Public API Surface](public-api.md).
2. **Walking the AST**: read [AST Envelope Structure](ast-envelope.md), then look up the per-rule chapters for the rules you consume.
3. **Pinning to a known shape**: read [Schema Versioning](schema-versioning.md) for the policy and [Changelog Index](changelog-index.md) for the per-release shape changes.
4. **Reporting issues**: follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` for the bug-report protocol.

## Status

The PGEN VHDL parser is **on-demand-only** in the default build (it is not in `cargo test --features generated_parsers` until activated). The current grammar covers design units (library/use/entity/architecture/package/package body/configuration/context), entity generics/ports, architecture declarations and concurrent statements, process + core sequential statements, and the expression/type/literal baseline. See `LIVE_ACHIEVEMENT_STATUS.md` for the live closure status and `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` for the formal trust statement.

The VHDL return-annotation campaign landed `VHDL-Slice-1` (110+ rules / 249 annotations) on 2026-05-14 in one comprehensive batch. A follow-up correctness fix (`VHDL-0001`, parser release `1.0.2`, AST-dump schema `1` → `2`, landed 2026-05-17) lifted the `simple_expression` / `term` inline operator alternations into the named `adding_operator` / `multiplying_operator` rules (256 annotations / 112 rules). The `1.0.3` POST-SV-AUDIT.2.3 Category-A batch (AST-dump schema `2` → `3`, landed 2026-05-17) corrected 17 separated-list rules from the raw `{first, rest}` envelope to clean flat arrays, with the inventory **unchanged** at **256 annotations / 112 rules** — see the [Changelog Index](changelog-index.md) and [Binary Addition](examples-binary-addition.md). Subsequent shape-affecting slices get their own changelog entries here.
