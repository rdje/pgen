# Schema Versioning

This chapter explains how the PGEN rtl_const_expr parser's AST shape is versioned, what guarantees consumers can rely on, and how to pin to a known shape. The authoritative numbers come from `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`; where this chapter and the contract disagree, **the contract wins**.

## Two versioning axes

The rtl_const_expr parser carries **two** version numbers:

1. **Parser release version** — currently `1.0.1`. Tracks the parser library's release identity. Bumped on every functional change to the parser, including bug fixes, perf work, and grammar changes.
2. **AST-dump schema version** — currently `1`. Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

A single parser release can carry the same schema version as the previous release (no shape change) or a bumped schema version (shape changed). The two numbers move independently.

These numbers are taken from the integration contract's "Contract Identity" section, which records:

- Contract version: `1.0.1`
- Parser release version: `1.0.1`
- rtl_const_expr AST-dump schema version: `1`
- Annotation count: **24** (RTL-CE-Slice-1 — the full expression hierarchy typed in one slice, on 16 distinct rules)

The contract document `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` is the authoritative source for all of these per release.

## What "shape change" means

Any of these triggers a schema version bump:

- A new return annotation lands on a previously-unannotated rule.
- An existing return annotation is restructured.
- A grammar rule changes shape in a way that's user-visible (new branch added, branch removed, a sub-rule renamed in a way that affects shape, a `binop_chain` `rest` iteration flattened, etc.).
- The default fall-through behavior of unannotated rules changes.

These do **not** trigger a schema bump:

- Pure performance optimizations that produce the same AST.
- Internal codegen reorganization that doesn't reach the output.
- Parser-side bug fixes that produce the same shape consumers were already relying on.

The rtl_const_expr grammar was typed in a single slice (RTL-CE-Slice-1, 24 annotations / 16 rules), landed on 2026-05-14, rather than the slice-by-slice cadence used by the SystemVerilog campaign, so the rtl_const_expr schema timeline is short. (That slice required the codegen outer-branch remap fix `PGEN-PIP-001` to make the Pattern-A `digit ( sep | digit )*` `binop_chain` shape work; see the contract's Release 1.0.1 Highlights.) Subsequent shape-affecting slices each get their own contract-version row and a [Changelog Index](changelog-index.md) entry.

## Byte-equivalence guarantee

For any input the parser accepts, the AST dump is **byte-deterministic** for a given parser-release version: object keys in canonical (alphabetical) order, canonical number formatting, no embedded timestamps or hashes. Re-running the parse on the same input produces an identical JSON value. Whitespace is configurable via `AstDumpOptions.pretty` but the underlying JSON value is the same.

This determinism is a **hard guarantee** of the schema. Any non-determinism is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Schema version timeline

This table mirrors the "Schema Versioning" table in `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`. The contract is authoritative for the live state.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 1.0.0 | 1.0.1 | **RTL-CE-Slice-1** — initial 24-annotation baseline (16 distinct rules). Expression hierarchy (conditional + the 10-rule `binop_chain` hierarchy `logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr` + unary + primary), `literal` (2 kinds: `"based"` / `"decimal"`), and `identifier` all typed in one slice. Same accept set as the pre-typing baseline. AST-dump schema version field value: `1`. |
| 0.1.0 | 1.0.0 | **Foundation baseline.** Grammar (`grammars/rtl_const_expr.ebnf`) with the `rtl_const_expr -> {type, expr}` root, `unary_expr` per-branch typed shapes, `primary_expr` / `literal` typed shapes, and `identifier -> {type, text}` already in place; the 10 binop-chain rules were the unannotated tail. AST dump is the recursive-envelope shape across the binop-chain rules. |

> Note on the version columns: the contract's "Schema version" column uses the `1.0.0` / `0.1.0` labels above for the typing-campaign milestones, while the `AstDumpPayload.schema_version` integer field consumers branch on is currently `1`. Pin against the integer schema-version field for runtime dispatch (see [Walking the AST](walking-the-ast.md)); use the contract's milestone labels when reading the changelog.

## How to pin to a known shape

1. **Record the parser-release version** your downstream code was written against — `1.0.1` as of this writing. It is recorded in `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity".
2. **Branch on the integer schema version at runtime.** `AstDumpPayload.schema_version` is `1`. Reject or warn on an unexpected value rather than mis-parsing a newer shape:

   ```rust
   match ast_dump_payload.schema_version {
       1 => walk_schema_v1(&ast_dump_payload.root),
       other => return Err(format!("unsupported rtl_const_expr schema version: {other}")),
   }
   ```

3. **Vendor or pin the generated parser.** The rtl_const_expr parser is on-demand-only (see [Build Recipe](build-recipe.md)); vendor `generated/rtl_const_expr_parser.rs` against the recorded parser-release version, or build it in CI from the pinned `grammars/rtl_const_expr.ebnf`.
4. **When you bump PGEN**, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the rules you consume, and re-run your walker's test corpus.

## Additive vs breaking changes

Within a single integer schema version, shape changes are intended to be **additive** wherever possible:

- **Additive (no integer schema bump expected):** a new optional field on an existing typed object, a new `op` / `kind` / `level` value on a discriminated rule for a previously-unparseable construct, a new typed shape on a rule that was previously raw envelope. Consumers using the unknown-shape fallthrough from [Walking the AST](walking-the-ast.md) keep working; consumers that hard-match a closed discriminator set must extend it.
- **Breaking (integer schema bump):** renaming or removing a field on an existing typed object, changing a `type` / `level` / `op` / `kind` discriminator value, restructuring the `binop_chain` `rest` iteration into a flat array, or changing the default fall-through of an unannotated rule in a way that moves data consumers were already reading.

The contract's bump-trigger guidance is the binding policy; this section paraphrases it for walker authors. A consumer that (a) branches on the integer `schema_version`, (b) treats an empty `binop_chain` `rest` as `[]` (unwrap `lhs`), and (c) uses the unknown-shape fallthrough is resilient to additive changes and fails loudly — not silently — on breaking ones.

## Reporting drift

If you observe an AST shape that disagrees with this book, the contract, or the live inventory `generated/rtl_const_expr_return_annotations.json`:

1. Confirm against the machine-checkable inventory (`generated/rtl_const_expr_return_annotations.json` / `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`, 24 entries, content-identical — the `(rule, branch_index, annotation_type, normalized_text)` tuples match exactly; the live inventory additionally carries a per-entry `raw_text` field).
2. If the inventory agrees with what you observe but the contract does not, the contract is authoritative for intended behavior — file via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
3. Accepted released-parser bugs are tracked in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Future major version

A future schema milestone will land if and when the rtl_const_expr grammar's remaining un-annotated rules (the keyword/operator tokens, the `based_integer` / `decimal_integer` regex leaves, and `trivia`) are either annotated or given a deliberate decision to remain raw envelope, and the shape definitions move to a locked tier. The rtl_const_expr family covers only **constant expressions** (decimal and sized-based integer literals, identifiers, unary `+ - ! ~`, binary arithmetic / shift / comparison / equality / bitwise / logical operators, ternary `?:`); for statements, modules, and control flow see the `rtl_frontend` family. Downstream integrators should treat the embedding surface as real but keep an eye on the [Changelog Index](changelog-index.md).
