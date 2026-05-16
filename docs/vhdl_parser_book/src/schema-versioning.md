# Schema Versioning

This chapter explains how the PGEN VHDL parser's AST shape is versioned, what guarantees consumers can rely on, and how to pin to a known shape. The authoritative numbers come from `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`; where this chapter and the contract disagree, **the contract wins**.

## Two versioning axes

The VHDL parser carries **two** version numbers:

1. **Parser release version** — currently `1.0.1`. Tracks the parser library's release identity. Bumped on every functional change to the parser, including bug fixes, perf work, and grammar changes.
2. **AST-dump schema version** — currently `1`. Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

A single parser release can carry the same schema version as the previous release (no shape change) or a bumped schema version (shape changed). The two numbers move independently.

These numbers are taken from the integration contract's "Contract Identity" section, which records:

- Contract version: `1.0.1`
- Parser release version: `1.0.1`
- VHDL AST-dump schema version: `1`
- Annotation count: **249** (VHDL-Slice-1 — the full grammar typed in one batch)

The contract document `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` is the authoritative source for all of these per release.

## What "shape change" means

Any of these triggers a schema version bump:

- A new return annotation lands on a previously-unannotated rule.
- An existing return annotation is restructured.
- A grammar rule changes shape in a way that's user-visible (new branch added, branch removed, a sub-rule renamed in a way that affects shape, a `{first, rest}` list flattened, etc.).
- The default fall-through behavior of unannotated rules changes.

These do **not** trigger a schema bump:

- Pure performance optimizations that produce the same AST.
- Internal codegen reorganization that doesn't reach the output.
- Parser-side bug fixes that produce the same shape consumers were already relying on.

The VHDL grammar was typed in a single comprehensive batch (VHDL-Slice-1, 249 annotations / 110 rules) rather than the slice-by-slice cadence used by the SystemVerilog campaign, so the VHDL schema timeline is short. Subsequent shape-affecting slices each get their own contract-version row and a [Changelog Index](changelog-index.md) entry.

## Byte-equivalence guarantee

For any input the parser accepts, the AST dump is **byte-deterministic** for a given parser-release version: object keys in canonical (alphabetical) order, canonical number formatting, no embedded timestamps or hashes. Re-running the parse on the same input produces an identical JSON value. Whitespace is configurable via `AstDumpOptions.pretty` but the underlying JSON value is the same.

This determinism is a **hard guarantee** of the schema. Any non-determinism is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Schema version timeline

This table mirrors the "Schema Versioning" table in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`. The contract is authoritative for the live state.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 1.0.0 | 1.0.1 | **VHDL-Slice-1** — initial 249-annotation baseline (110 distinct rules). Design units, declarations, types, statements, expressions (the `binop_chain` shape across the 5-level operator hierarchy `expression` → `relation` → `simple_expression` → `term` → `factor`), and literals all typed in one comprehensive batch. Same accept set as the pre-typing baseline. AST-dump schema version field value: `1`. |
| 0.1.0 | 1.0.0 | **Foundation baseline.** Grammar (`grammars/vhdl.ebnf`) un-annotated except for the `vhdl_file -> {type, design_units}` root. AST dump is the recursive-envelope shape across all rules. |

> Note on the version columns: the contract's "Schema version" column uses the `1.0.0` / `0.1.0` labels above for the typing-campaign milestones, while the `AstDumpPayload.schema_version` integer field consumers branch on is currently `1`. Pin against the integer schema-version field for runtime dispatch (see [Walking the AST](walking-the-ast.md)); use the contract's milestone labels when reading the changelog.

## How to pin to a known shape

1. **Record the parser-release version** your downstream code was written against — `1.0.1` as of this writing. It is recorded in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity".
2. **Branch on the integer schema version at runtime.** `AstDumpPayload.schema_version` is `1`. Reject or warn on an unexpected value rather than mis-parsing a newer shape:

   ```rust
   match ast_dump_payload.schema_version {
       1 => walk_schema_v1(&ast_dump_payload.root),
       other => return Err(format!("unsupported VHDL schema version: {other}")),
   }
   ```

3. **Vendor or pin the generated parser.** The VHDL parser is on-demand-only (see [Build Recipe](build-recipe.md)); vendor `generated/vhdl_parser.rs` against the recorded parser-release version, or build it in CI from the pinned `grammars/vhdl.ebnf`.
4. **When you bump PGEN**, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the rules you consume, and re-run your walker's test corpus.

## Additive vs breaking changes

Within a single integer schema version, shape changes are intended to be **additive** wherever possible:

- **Additive (no integer schema bump expected):** a new optional field on an existing typed object, a new `kind` value on a dispatch rule for a previously-unparseable construct, a new typed shape on a rule that was previously raw envelope. Consumers using the unknown-shape fallthrough from [Walking the AST](walking-the-ast.md) keep working; consumers that hard-match a closed `kind` set must extend it.
- **Breaking (integer schema bump):** renaming or removing a field on an existing typed object, changing a `kind` discriminator value, restructuring a `{first, rest}` list into a flat array, or changing the default fall-through of an unannotated rule in a way that moves data consumers were already reading.

The contract's bump-trigger guidance is the binding policy; this section paraphrases it for walker authors. A consumer that (a) branches on the integer `schema_version`, (b) treats absent optionals as `[]`, and (c) uses the unknown-shape fallthrough is resilient to additive changes and fails loudly — not silently — on breaking ones.

## Reporting drift

If you observe an AST shape that disagrees with this book, the contract, or the live inventory `generated/vhdl_return_annotations.json`:

1. Confirm against the machine-checkable inventory (`generated/vhdl_return_annotations.json` / `rust/test_data/ast_shape_contract/vhdl_v1.json`, 249 entries, identical content).
2. If the inventory agrees with what you observe but the contract does not, the contract is authoritative for intended behavior — file via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
3. Accepted released-parser bugs are tracked in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Future major version

A future schema milestone will land if and when the VHDL grammar's remaining un-annotated rules (terminal/regex leaves and a few utility rules) are either annotated or given a deliberate decision to remain raw envelope, and the shape definitions move to a locked tier. The VHDL family is still an `In Progress` family in the live tracker (`LIVE_ACHIEVEMENT_STATUS.md`); downstream integrators should treat the embedding surface as real but keep an eye on the live blocker list and the [Changelog Index](changelog-index.md).
