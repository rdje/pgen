# Welcome — PGEN SystemVerilog Parser Integration Reference

This book is the **canonical AST reference** for downstream consumers of PGEN's `systemverilog` parser family. The intended primary audience is Nexsim and any other tool embedding the PGEN-generated SystemVerilog parser through the `pgen::embedding_api` host surface.

## What this book covers

- **How to build the generated parser and wire it into your project.** See [Build Recipe](build-recipe.md) and [Public API Surface](public-api.md).
- **What the AST envelope looks like.** See [AST Envelope Structure](ast-envelope.md), [ParseContent Variants](parse-content-variants.md), and [Walking the AST](walking-the-ast.md).
- **The shape every grammar rule produces in the AST dump.** See [Per-Rule Shape Reference](rules-top-level.md). As the return-annotation campaign progresses, this section grows to cover every rule that has a stable typed shape.
- **Worked examples by SystemVerilog feature** — what does the AST look like for a minimal module? An interface? A class? Each example chapter pins a current production AST so consumers can write their walkers against a concrete, tested reference.
- **Schema versioning policy.** See [Schema Versioning](schema-versioning.md). The schema is currently at version `1`.
- **A release-by-release index of what changed and why.** See [Changelog Index](changelog-index.md).

## What this book is NOT

- It is **not** a substitute for the integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`. The contract is the authoritative source of truth; this book documents what the contract describes. **Where the book and the contract disagree, the contract wins.** Please report any disagreement as a documentation bug.
- It is **not** the IEEE 1800 LRM. The LRM is the authoritative SystemVerilog language reference; this book documents PGEN's parser-side AST shape for the LRM-conformant inputs the parser accepts.
- It is **not** a tutorial on SystemVerilog. It assumes you are already an embedded user of the language and need to walk PGEN's AST output programmatically.

## How to use this book

1. **First-time integrators**: read [Quickstart](quickstart.md), then [Build Recipe](build-recipe.md), then [Public API Surface](public-api.md).
2. **Walking the AST**: read [AST Envelope Structure](ast-envelope.md), then look up the per-rule chapters for the rules you consume.
3. **Pinning to a known shape**: read [Schema Versioning](schema-versioning.md) for the policy and [Changelog Index](changelog-index.md) for the per-release shape changes.
4. **Reporting issues**: follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` for the bug-report protocol; accepted released-parser bugs land in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` under the `SV-NNNN` ID family.

## Status

The PGEN SystemVerilog parser is **closure-grade for the current Nexsim-facing scope** when consumed through the stable `pgen::embedding_api` host surface. See `LIVE_ACHIEVEMENT_STATUS.md` for the live closure status and `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` for the formal trust statement.

The systemverilog return-annotation campaign is **in early phase**. Most rules currently fall through to the recursive default envelope shape; subsequent slices will type rules one-by-one, mirroring the regex-parser campaign that produced typed shapes for 42+ rules. Each slice gets its own changelog entry here.
