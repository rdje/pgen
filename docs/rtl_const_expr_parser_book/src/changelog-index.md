# Changelog Index

This chapter is an index — pointers into the documents that carry the full changelog detail, plus the short list of releases relevant to this book. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | **The authoritative contract.** Its "Schema Versioning" table and per-release Highlights sections list the AST shape changes consumers care about. Where this book and the contract disagree, the contract wins. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a downstream bug is accepted and fixed in a release, the ledger row records the input/output shape change and the fix proof. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all repository changes, rtl_const_expr among them. |
| Git tags + commit log | Commit-by-commit | The most granular source — use for diffs once you know which release to inspect. |

When investigating "what changed and why," start with the contract document, drop down to the bug ledger for specific accepted bugs, and fall back to git for diffs.

## Why this index is short by design

The SystemVerilog parser's changelog index is long because its return-annotation campaign landed rule-by-rule across 115 slices, each bumping the schema version and getting its own row. **The rtl_const_expr grammar is different: it is a small grammar (constant expressions only) that was typed in a single comprehensive slice — RTL-CE-Slice-1 — followed by one correctness fix — RTL-CE-Slice-2 — so the rtl_const_expr schema timeline has exactly three entries.** This is the intended state, not an incomplete index. Subsequent shape-affecting slices, if any, will each add a contract Highlights section, a [Schema Versioning](schema-versioning.md) row, and an entry below.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. The three entries below mirror the "Schema Versioning" table in `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`; the contract is authoritative for the live state.

### 1.0.2 / Contract 1.0.2 — RTL-CE-Slice-2: correctness fix (3 bugs); schema 1 → 2

A worked-example pass surfaced that the `1.0.1` baseline shipped three return-annotation defects that the (root-keys-only) shape-contract regression lock did not catch. All three are fixed, the parser is regenerated, and the manifest inventory is tightened to the full 26-entry surface so the corrected shapes are machine-locked.

- **Schema-version milestone:** `1.0.2` (first parser release: `1.0.2`).
- **AST-dump schema version field value:** `2` — the integer consumers branch on at runtime (`AstDumpPayload.schema_version`); bumped from `1` because three user-visible shapes changed.
- **The three fixes:**
  - **`binop_chain` `rest` (Issue A).** Was the literal string `"<invalid_sequence_access>"` inside a malformed nested object on any input exercising an operator at any of the ten levels. The five multi-token inner operator alternations were lifted into **named rules** (`equality_op`, `relational_op`, `shift_op`, `additive_op`, `multiplicative_op`), so every level is now `next ( NAMED_op next )* -> {type: "binop_chain", level, lhs: $1, rest: $2}` with bare `$2`. `rest` is now a **clean array** of `[ <op-envelope>, <operand> ]` iteration entries (operator text at `entry[0][1]`), `[]` when no operator at that level.
  - **`identifier.text` (Issue B).** Was `$1` (the empty leading `trivia`), so every identifier `text` was `""`. Now `$2`, the real name.
  - **`literal.text` (Issue C).** `based_integer` / `decimal_integer` were unannotated, so `literal.text` surfaced the envelope `["", "42"]`. Both leaves are now annotated `-> $2`; `literal.text` is a clean string.
- **Annotation count:** **26** (was `24` / RTL-CE-Slice-1 baseline). The `+2` is the two new `based_integer` / `decimal_integer` `return_scalar` leaf captures; the five new `*_op` rules are un-annotated alternations and not in the inventory. The 26 are **19 `return_object` + 7 `return_scalar`** across **18 distinct rules**.
- **Accept set:** unchanged — same accepted inputs as the `1.0.1` baseline; this was purely annotation shaping (no grammar acceptance change).
- **Contract section:** `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.0.2 / Contract 1.0.2 Highlights — RTL-CE correctness fix".
- **Machine-checkable inventory:** `generated/rtl_const_expr_return_annotations.json` (26 entries) and its content-identical embedded mirror `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`.
- **Per-rule shapes:** [Top-Level Rules](rules-top-level.md); the schema-`2` row in [Schema Versioning](schema-versioning.md).

### 1.0.0 / Contract 1.0.1 — RTL-CE-Slice-1: full expression hierarchy typed (24 annotations)

The initial typing slice, covering the entire `grammars/rtl_const_expr.ebnf` expression surface in **one batch** (landed 2026-05-14).

- **Schema-version milestone:** `1.0.0` (first parser release: `1.0.1`).
- **AST-dump schema version field value:** `1` — the integer consumers branched on at runtime for this release.
- **Annotation count:** **24** (was `1` / pre-typing baseline). Coverage: the `rtl_const_expr` root; `conditional_expr` (ternary + passthrough); the ten-rule `binop_chain` hierarchy (`logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr`); `unary_expr` (four prefix forms + passthrough); `primary_expr` (three passthrough branches); `literal` (two kinds, `"based"` / `"decimal"`); and `identifier`.
- **NOTE — defective shapes:** the `binop_chain` `rest` and the `identifier` / `literal` `text` shapes in this baseline were defective (`rest` could surface `"<invalid_sequence_access>"`; `identifier.text` / `literal.text` carried the empty-`trivia` envelope). They were corrected in schema `2` / release `1.0.2` above — consumers must not target the schema-`1` shapes.
- **Accept set:** unchanged — same accepted inputs as the pre-typing baseline; only the AST shape became typed.
- **Contract section:** `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.0.1 / Contract 1.0.1 Highlights — RTL-CE-Slice-1".

### 0.1.0 / release 1.0.0 — foundation baseline

The pre-typing baseline.

- **Schema-version milestone:** `0.1.0` (first parser release: `1.0.0`).
- **State:** `grammars/rtl_const_expr.ebnf` with the `rtl_const_expr -> {type, expr}` root, `unary_expr` per-branch typed shapes, `primary_expr` / `literal` typed shapes, and `identifier -> {type, text}` already in place; the ten binop-chain rules were the unannotated tail, so the AST dump was the recursive-envelope shape across them (see [AST Envelope Structure](ast-envelope.md)).
- **Contract section:** the `0.1.0` row of the "Schema Versioning" table in `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`.

## Bug ledger status

`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` is the canonical per-bug tracker. As of this writing it carries no accepted rtl_const_expr rows — the three defects corrected in release `1.0.2` were caught internally by a worked-example pass before any downstream report, so they are tracked as the RTL-CE-Slice-2 contract Highlights section rather than as ledger rows. When a downstream rtl_const_expr bug is accepted, it gets a ledger row recording the reproducer bundle, root cause, fix proof, and the parser release it was fixed in; this index will then point at the relevant contract Highlights section for any accompanying shape change. Reports follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## How to follow per-slice changes

Each shape-affecting slice after RTL-CE-Slice-2 gets:

1. A grammar change in `grammars/rtl_const_expr.ebnf` (the `-> ...` annotation or restructure).
2. A manifest update in `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` (and the regenerated `generated/rtl_const_expr_return_annotations.json`).
3. A parser-release / contract-version bump and a Highlights section in `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the milestone.
5. An entry in this changelog index summarizing the slice.
6. A regression-lock test pinning the new typed shape (`cargo test --lib --features generated_parsers rtl_const_expr_ast_shape_contract`).

The live-book policy bundles all six in the same commit. Because RTL-CE-Slice-1 already typed the full expression surface and RTL-CE-Slice-2 corrected it, the realistic future driver of new entries here is bug-ledger-driven shape fixes and any targeted restructure (for example, annotating the remaining un-annotated keyword/operator tokens or the named operator rules, or flattening the `binop_chain` `rest` iteration), not a long rule-by-rule campaign. rtl_const_expr covers only **constant expressions** (decimal and sized-based integer literals, identifiers, unary `+ - ! ~`, binary arithmetic / shift / comparison / equality / bitwise / logical operators, ternary `?:`); for statements, modules, and control flow see the `rtl_frontend` family.
