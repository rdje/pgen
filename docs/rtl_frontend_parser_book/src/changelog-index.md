# Changelog Index

This chapter is an index — pointers into the documents that carry the full changelog detail, plus the short list of releases relevant to this book. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | **The authoritative contract.** Its "Schema Versioning" table and per-release Highlights sections list the AST shape changes consumers care about. Where this book and the contract disagree, the contract wins. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a downstream bug is accepted and fixed in a release, the ledger row records the input/output shape change and the fix proof. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all repository changes, rtl_frontend among them. |
| Git tags + commit log | Commit-by-commit | The most granular source — use for diffs once you know which release to inspect. |

When investigating "what changed and why," start with the contract document, drop down to the bug ledger for specific accepted bugs, and fall back to git for diffs.

## Why this index is short by design

The SystemVerilog parser's changelog index is long because its return-annotation campaign landed rule-by-rule across 115 slices, each bumping the schema version and getting its own row. **The rtl_frontend grammar is different: it was typed across a small number of grouped slices — RTL-FE-Slice-1..7, all landed together on 2026-05-14 — so the rtl_frontend schema timeline has exactly two entries.** This is also unlike the VHDL grammar, which was typed in a single comprehensive batch (`VHDL-Slice-1`); rtl_frontend used seven grouped slices rather than one batch or a long campaign. Either way, the result is the same: this is the intended state, not an incomplete index. Subsequent shape-affecting slices, if any, will each add a contract Highlights section, a [Schema Versioning](schema-versioning.md) row, and an entry below.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. The two entries below mirror the "Schema Versioning" table in `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md`; the contract is authoritative for the live state.

### 1.0.0 / Contract 1.0.1 — RTL-FE-Slice-1..7: full grammar typed (156 annotations / 74 rules)

The initial typing campaign, covering the entire `grammars/rtl_frontend.ebnf` surface across **seven grouped slices** that landed together on 2026-05-14.

- **Schema-version milestone:** `1.0.0` (first parser release: `1.0.1`).
- **AST-dump schema version field value:** `1` — the integer consumers branch on at runtime (`AstDumpPayload.schema_version`).
- **Annotation count:** **156** (was `1` / pre-typing baseline) on **74 distinct rules**. The campaign was structured as: dispatch wrappers (slice 1 — `design_item` / `package_item` / `module_item` / `generate_item`); keyword/operator leaves (slice 2); expression dispatch + procedural blocks (slice 3); the ten-rule `binop_chain` hierarchy (slice 4 — `logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr`); declarations + module structure (slice 5); parameter/port rules (slice 6); and the module-instantiation / ports / statements / signals / datatypes mass batch (slice 7).
- **Accept set:** unchanged — same accepted inputs as the pre-typing baseline; only the AST shape became typed.
- **Contract section:** `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.0.1 / Contract 1.0.1 Highlights — RTL-FE-Slice-1..7". That section's header reads "164 rules / 156 annotations": `164` is the total number of grammar rules in `grammars/rtl_frontend.ebnf`, while the typed surface is **156 annotations on 74 of those rules**. The inventory-accurate figure used throughout this book is **156 annotations / 74 distinct rules**; this is a reconciled wording difference between the contract header and the inventory, not a contradiction (the same way the VHDL contract's analogous header is reconciled in the VHDL book).
- **Machine-checkable inventory:** `generated/rtl_frontend_return_annotations.json` (156 entries) and its content-identical embedded mirror `rust/test_data/ast_shape_contract/rtl_frontend_v1.json`.
- **Per-rule shapes:** [Top-Level Rules](rules-top-level.md).

### 0.1.0 / release 1.0.0 — foundation baseline

The pre-typing baseline.

- **Schema-version milestone:** `0.1.0` (first parser release: `1.0.0`).
- **State:** `grammars/rtl_frontend.ebnf` un-annotated except for the `rtl_frontend_file -> {type, items}` root. The AST dump was the recursive-envelope shape across all other rules (see [AST Envelope Structure](ast-envelope.md)).
- **Contract section:** the `0.1.0` row of the "Schema Versioning" table in `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md`.

## Bug ledger status

`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` is the canonical per-bug tracker. As of this writing it carries no accepted rtl_frontend rows. When a downstream rtl_frontend bug is accepted, it gets a ledger row recording the reproducer bundle, root cause, fix proof, and the parser release it was fixed in; this index will then point at the relevant contract Highlights section for any accompanying shape change. Reports follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## How to follow per-slice changes

Each shape-affecting slice after RTL-FE-Slice-7 gets:

1. A grammar change in `grammars/rtl_frontend.ebnf` (the `-> ...` annotation or restructure).
2. A manifest update in `rust/test_data/ast_shape_contract/rtl_frontend_v1.json` (and the regenerated `generated/rtl_frontend_return_annotations.json`).
3. A parser-release / contract-version bump and a Highlights section in `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the milestone.
5. An entry in this changelog index summarizing the slice.
6. A regression-lock test pinning the new typed shape.

The live-book policy bundles all six in the same commit. Because RTL-FE-Slice-1..7 already typed the full grammar, the realistic future driver of new entries here is bug-ledger-driven shape fixes, annotating the remaining un-annotated rules (terminal/regex leaves like `identifier`, `named_data_type`, and the passthrough forms of `conditional_expr` / `unary_expr`), and any targeted restructure (for example, a `{first, rest}` list-flattening slice), not a long rule-by-rule campaign. rtl_frontend remains an `In Progress` family in the live tracker (`LIVE_ACHIEVEMENT_STATUS.md`); the current grammar covers the synthesizable RTL subset, and the full IEEE 1800 SystemVerilog surface is out of scope — see the `systemverilog` family for that.
