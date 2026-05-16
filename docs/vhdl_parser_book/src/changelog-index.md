# Changelog Index

This chapter is an index — pointers into the documents that carry the full changelog detail, plus the short list of releases relevant to this book. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | **The authoritative contract.** Its "Schema Versioning" table and per-release Highlights sections list the AST shape changes consumers care about. Where this book and the contract disagree, the contract wins. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a downstream bug is accepted and fixed in a release, the ledger row records the input/output shape change and the fix proof. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all repository changes, VHDL among them. |
| Git tags + commit log | Commit-by-commit | The most granular source — use for diffs once you know which release to inspect. |

When investigating "what changed and why," start with the contract document, drop down to the bug ledger for specific accepted bugs, and fall back to git for diffs.

## Why this index is short by design

The SystemVerilog parser's changelog index is long because its return-annotation campaign landed rule-by-rule across 115 slices, each bumping the schema version and getting its own row. **The VHDL grammar is different: it was typed in a single comprehensive batch — VHDL-Slice-1 — so the VHDL schema timeline has exactly two entries.** This is the intended state, not an incomplete index. Subsequent shape-affecting slices, if any, will each add a contract Highlights section, a [Schema Versioning](schema-versioning.md) row, and an entry below.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. The two entries below mirror the "Schema Versioning" table in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`; the contract is authoritative for the live state.

### 1.0.0 / Contract 1.0.1 — VHDL-Slice-1: full grammar typed (110 rules / 249 annotations)

The initial typing campaign, covering the entire `grammars/vhdl.ebnf` surface in **one batch**.

- **Schema-version milestone:** `1.0.0` (first parser release: `1.0.1`).
- **AST-dump schema version:** `1` — the integer consumers **pin** from the contract (it is **not** a field of `AstDumpPayload`, whose real fields are `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`).
- **Annotation count:** **249** (was `1` / pre-typing baseline). Coverage: the `vhdl_file` root; `design_unit` and the declarative-item / statement dispatch rules; design units (entity, architecture, package, package body, configuration, context); generic / port / parameter interfaces; declarations; types and constraints; the five-level `binop_chain` expression hierarchy (`expression` → `relation` → `simple_expression` → `term` → `factor`); and the literal dispatch.
- **Accept set:** unchanged — same accepted inputs as the pre-typing baseline; only the AST shape became typed.
- **Contract section:** `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.0.1 / Contract 1.0.1 Highlights — VHDL-Slice-1".
- **Machine-checkable inventory:** `generated/vhdl_return_annotations.json` (249 entries) and its byte-identical embedded mirror `rust/test_data/ast_shape_contract/vhdl_v1.json`.
- **Per-rule shapes:** [Top-Level Rules](rules-top-level.md).

### 0.1.0 / release 1.0.0 — foundation baseline

The pre-typing baseline.

- **Schema-version milestone:** `0.1.0` (first parser release: `1.0.0`).
- **State:** `grammars/vhdl.ebnf` un-annotated except for the `vhdl_file -> {type, design_units}` root. The AST dump was the recursive-envelope shape across all other rules (see [AST Envelope Structure](ast-envelope.md)).
- **Contract section:** the `0.1.0` row of the "Schema Versioning" table in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`.

## Bug ledger status

`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` is the canonical per-bug tracker. As of this writing it carries **no VHDL rows** — the `VHDL-NNNN` form appears there only as a report-ID naming-convention example. When a downstream VHDL bug is accepted, it gets a `VHDL-NNNN` row recording the reproducer bundle, root cause, fix proof, and the parser release it was fixed in; this index will then point at the relevant contract Highlights section for any accompanying shape change.

## How to follow per-slice changes

Each shape-affecting slice after VHDL-Slice-1 gets:

1. A grammar change in `grammars/vhdl.ebnf` (the `-> ...` annotation or restructure).
2. A manifest update in `rust/test_data/ast_shape_contract/vhdl_v1.json` (and the regenerated `generated/vhdl_return_annotations.json`).
3. A parser-release / contract-version bump and a Highlights section in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the milestone.
5. An entry in this changelog index summarizing the slice.
6. A regression-lock test pinning the new typed shape.

The live-book policy bundles all six in the same commit. Because VHDL-Slice-1 already typed the full grammar, the realistic future driver of new entries here is bug-ledger-driven shape fixes and any targeted restructure (for example, a list-flattening slice), not a long rule-by-rule campaign.
