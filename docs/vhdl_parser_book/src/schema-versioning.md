# Schema Versioning

This chapter explains how the PGEN VHDL parser's AST shape is versioned, what guarantees consumers can rely on, and how to pin to a known shape. The authoritative numbers come from `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`; where this chapter and the contract disagree, **the contract wins**.

## Two versioning axes

The VHDL parser carries **two** version numbers:

1. **Parser release version** — currently `1.0.3`. Tracks the parser library's release identity. Bumped on every functional change to the parser, including bug fixes, perf work, and grammar changes.
2. **AST-dump schema version** — currently `3`. Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

A single parser release can carry the same schema version as the previous release (no shape change) or a bumped schema version (shape changed). The two numbers move independently.

These numbers are taken from the integration contract's "Contract Identity" section, which records:

- Contract version: `1.0.3`
- Parser release version: `1.0.3`
- VHDL AST-dump schema version: `3`
- Annotation count: **256** (VHDL-Slice-1's 249-annotation baseline plus the `1.0.2` `VHDL-0001` correctness fix's 7 new operator-rule branches; the `1.0.3` POST-SV-AUDIT Category-A batch — 17 list-shape corrections — did **not** change this count: the 14 bare-list rules flip `return_object` → `return_array` and the `target`-aggregate + `aggregate` rules change `normalized_text` only. On 112 distinct rules)

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

The VHDL grammar was typed in a single comprehensive batch (VHDL-Slice-1, 249 annotations / 110 rules) rather than the slice-by-slice cadence used by the SystemVerilog campaign, so the VHDL schema timeline is short. A follow-up correctness fix (parser release `1.0.2`, schema `2`, landed 2026-05-17) brought the inventory to **256 annotations / 112 rules** — see the schema-`2` row below and the contract's Release 1.0.2 Highlights. The POST-SV-AUDIT batch (parser release `1.0.3`, schema `3`, landed 2026-05-17) corrected 17 Category-A list shapes with the inventory **unchanged at 256 / 112** — see the schema-`3` row below and the contract's "AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT)". Subsequent shape-affecting slices each get their own contract-version row and a [Changelog Index](changelog-index.md) entry.

The `1.0.2` correctness fix **did** bump the schema (`1` → `2`): although it fixed a bug, it changed a user-visible shape — the `additive` (`simple_expression`) and `multiplicative` (`term`) `binop_chain` `rest` (was `"<invalid_sequence_access>"` on multi-operand input, now a clean `[ <op-envelope>, <operand> ]` array with a typed `{kind}` op-envelope) — and added seven return annotations. It restructured a shape a consumer could have observed, so it is a breaking change under the policy below, not a transparent fix.

The `1.0.3` POST-SV-AUDIT batch **also** bumped the schema (`2` → `3`). The POST-SV-AUDIT.2.3 static classification (`docs/POST_SV_AUDIT_LEDGER.md`, `PGEN-POST-SV-AUDIT-0004`) found **17 static-conclusive Category-A raw-envelope list rules** in `grammars/vhdl.ebnf` (`library_clause`, `use_clause`, `selected_name`, `identifier_list`, `generic_interface_list`, `port_interface_list`, `parameter_list`, `enumeration_type_definition`, `index_constraint`, `association_list`, `sensitivity_list`, `actual_parameter_part`, `choices`, `aggregate_choice_list`, plus the `target` aggregate branch and the two `aggregate` branches) that exposed the raw `{first, rest}` (resp. `{…, first, rest}`) iteration envelope — `rest` was the raw `[[sep, item], …]` single-token-separator envelope a consumer had to walk past. Each was corrected to the canonical extraction-spread `[$F, $R::2*]` so the value (or its element field) is now a **clean flat list**. The 14 bare-list rules now emit a **top-level array**; the `target` aggregate branch and the two `aggregate` branches keep their meaningful discriminator/value fields and carry the cleaned trailing list in `items` / `rest`. These restructured consumer-visible shapes (a `1.0.2` consumer that walked `.first` + `.rest[][1]` must repin), so they are breaking changes. They did **not** change the annotation count: the 14 bare-list rules flip `return_object` → `return_array`, the `target`-aggregate + `aggregate` ones stay `return_object` with a new `normalized_text` — so the count stays **256 annotations / 112 distinct rules**. Every separator is a single token (comma / semi / dot / bar); there is **no** inline alternation and **no** `<invalid_sequence_access>` — this is a clean Category-A shape improvement, **not** a parser bug, and is **not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (that ledger is reserved for the `<invalid_sequence_access>` corruption/crash class — `VHDL-0001`, the systemic inline-alternation defect, lives there). Tracked via `docs/POST_SV_AUDIT_LEDGER.md` and the contract.

## Byte-equivalence guarantee

For any input the parser accepts, the AST dump is **byte-deterministic** for a given parser-release version: object keys in canonical (alphabetical) order, canonical number formatting, no embedded timestamps or hashes. Re-running the parse on the same input produces an identical JSON value. Whitespace is configurable via `AstDumpOptions.pretty` but the underlying JSON value is the same.

This determinism is a **hard guarantee** of the schema. Any non-determinism is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Schema version timeline

This table mirrors the "Schema Versioning" table in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`. The contract is authoritative for the live state.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 1.0.3 | 1.0.3 | **POST-SV-AUDIT Category-A list-shape batch (POST-SV-AUDIT.2.3, breaking, not a parser bug).** 17 static-conclusive Category-A raw-envelope list rules — `library_clause`, `use_clause`, `selected_name`, `identifier_list`, `generic_interface_list`, `port_interface_list`, `parameter_list`, `enumeration_type_definition`, `index_constraint`, `association_list`, `sensitivity_list`, `actual_parameter_part`, `choices`, `aggregate_choice_list`, plus the `target` aggregate branch and both `aggregate` branches — no longer expose the raw `{first, rest}` (resp. `{…, first, rest}`) iteration envelope. The POST-SV-AUDIT.2.3 static classification (`docs/POST_SV_AUDIT_LEDGER.md`, `PGEN-POST-SV-AUDIT-0004`) found these were static-conclusive Category-A raw-envelope misuses (`rest` surfaced the raw `[[sep, item], …]` single-token-separator envelope). Each is corrected to the canonical extraction-spread `[$F, $R::2*]` (drop the semantically-irrelevant separator — comma / semi / dot / bar; emit a clean flat list). The 14 bare-list rules now emit a **top-level array** (`return_object` → `return_array`); the `target` aggregate branch and the two `aggregate` branches keep their meaningful discriminator/value fields and carry the cleaned trailing list in `items` / `rest` (`return_object`, new `normalized_text`). No `<invalid_sequence_access>`, no inline alternation (clean shape improvement, **not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`). Annotation inventory **unchanged at 256 / 112**. Same accept set as `1.0.2`. AST-dump schema version field value: `3`. |
| 1.0.2 | 1.0.2 | **`VHDL-0001` correctness fix (breaking).** The `additive` (`simple_expression`) and `multiplicative` (`term`) `binop_chain` `rest` no longer emits `"<invalid_sequence_access>"` for multi-operand input. The two iteration-lead inline operator alternations — `(plus \| minus \| ampersand)` in `simple_expression` and `(star \| slash \| kw_mod \| kw_rem)` in `term` — were lifted into the **named** rules `adding_operator := plus -> {kind: "plus"} \| minus -> {kind: "minus"} \| ampersand -> {kind: "concat"}` and `multiplying_operator := star -> {kind: "mul"} \| slash -> {kind: "div"} \| kw_mod -> {kind: "mod"} \| kw_rem -> {kind: "rem"}`, matching vhdl's own `logical_operator` / `relational_operator` `{kind}` idiom. The `simple_expression` / `term` `binop_chain` annotations are **unchanged** (only the inline group became a named rule), so each level's `rest` is now a clean `[ <op-envelope>, <operand> ]` array where the op-envelope is the typed `{kind: …}` object (uniform with the `logical` / `relational` levels). The leading `(plus \| minus)?` `sign` is **not** an iteration lead and was empirically unaffected — left as-is. Annotation inventory **249 → 256** (the 3 new `adding_operator` + 4 new `multiplying_operator` `return_object` branches); distinct rules **110 → 112** (the new `adding_operator` / `multiplying_operator`). Same accept set as `1.0.1`. AST-dump schema version field value: `2`. |
| 1.0.0 | 1.0.1 | **VHDL-Slice-1** — initial 249-annotation baseline (110 distinct rules). Design units, declarations, types, statements, expressions (the `binop_chain` shape across the 5-level operator hierarchy `expression` → `relation` → `simple_expression` → `term` → `factor`), and literals all typed in one comprehensive batch. Same accept set as the pre-typing baseline. Pre-correctness shapes: the `additive` (`simple_expression`) and `multiplicative` (`term`) `binop_chain` `rest` could surface `"<invalid_sequence_access>"` on multi-operand input (`VHDL-0001`). AST-dump schema version field value: `1`. |
| 0.1.0 | 1.0.0 | **Foundation baseline.** Grammar (`grammars/vhdl.ebnf`) un-annotated except for the `vhdl_file -> {type, design_units}` root. AST dump is the recursive-envelope shape across all rules. |

> Note on the version columns: the contract's "Schema version" column uses the `1.0.3` / `1.0.2` / `1.0.0` / `0.1.0` labels above for the milestones; the AST-dump schema version consumers pin against is the integer **`3`**. That integer is **not** a runtime field of `AstDumpPayload` (the real struct is `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`) — you pin it from this contract/book at integration time, not by reading the payload (see [Walking the AST](walking-the-ast.md)); use the contract's milestone labels when reading the changelog.

## How to pin to a known shape

1. **Record the parser-release version** your downstream code was written against — `1.0.3` as of this writing. It is recorded in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity".
2. **Pin the AST-dump schema version you built against** — currently `3`, from `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity". This is a *compile-time constant in your consumer*, **not** a field of `AstDumpPayload` (that struct exposes only `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`). Check `truncated`, parse `dump_json`, then walk against the schema you pinned; re-validate the pin against the contract whenever you bump PGEN:

   ```rust
   // Schema version you integrated against, from the contract:
   const VHDL_AST_SCHEMA_VERSION: u32 = 3;

   let payload = outcome.ast_dump.expect("Success carries an AstDumpPayload");
   if payload.truncated {
       return Err("VHDL AST dump truncated (dump_json holds the diagnostic envelope)".into());
   }
   let root: serde_json::Value = serde_json::from_str(&payload.dump_json)?;
   // VHDL_AST_SCHEMA_VERSION drives which walker you compiled; re-check the
   // contract's Schema Versioning table on every PGEN bump.
   // (schema 3: the 17 Category-A list rules — e.g. library_clause,
   //  identifier_list, association_list, choices — are clean flat
   //  arrays, not the ≤ 1.0.2 {first, rest} envelope; POST-SV-AUDIT.2.3.)
   walk_schema_v3(&root);
   ```

3. **Vendor or pin the generated parser.** The VHDL parser is on-demand-only (see [Build Recipe](build-recipe.md)); vendor `generated/vhdl_parser.rs` against the recorded parser-release version, or build it in CI from the pinned `grammars/vhdl.ebnf`.
4. **When you bump PGEN**, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the rules you consume, and re-run your walker's test corpus.

## Additive vs breaking changes

Within a single integer schema version, shape changes are intended to be **additive** wherever possible:

- **Additive (no integer schema bump expected):** a new optional field on an existing typed object, a new `kind` value on a dispatch rule for a previously-unparseable construct, a new typed shape on a rule that was previously raw envelope. Consumers using the unknown-shape fallthrough from [Walking the AST](walking-the-ast.md) keep working; consumers that hard-match a closed `kind` set must extend it.
- **Breaking (integer schema bump):** renaming or removing a field on an existing typed object, changing a `kind` discriminator value, restructuring a `{first, rest}` list into a flat array (the `1.0.3` POST-SV-AUDIT schema-`2`→`3` change did exactly this for the 17 Category-A list rules), or changing the default fall-through of an unannotated rule in a way that moves data consumers were already reading.

The contract's bump-trigger guidance is the binding policy; this section paraphrases it for walker authors. A consumer that (a) branches on the integer `schema_version`, (b) treats absent optionals as `[]`, and (c) uses the unknown-shape fallthrough is resilient to additive changes and fails loudly — not silently — on breaking ones.

## Reporting drift

If you observe an AST shape that disagrees with this book, the contract, or the live inventory `generated/vhdl_return_annotations.json`:

1. Confirm against the machine-checkable inventory (`generated/vhdl_return_annotations.json` / `rust/test_data/ast_shape_contract/vhdl_v1.json`, 256 entries, identical content on the `(rule, branch_index, annotation_type, normalized_text)` tuples — the live inventory additionally carries a per-entry `raw_text` field).
2. If the inventory agrees with what you observe but the contract does not, the contract is authoritative for intended behavior — file via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
3. Accepted released-parser bugs are tracked in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Future major version

A future schema milestone will land if and when the VHDL grammar's remaining un-annotated rules (terminal/regex leaves and a few utility rules) are either annotated or given a deliberate decision to remain raw envelope, and the shape definitions move to a locked tier. The VHDL family is still an `In Progress` family in the live tracker (`LIVE_ACHIEVEMENT_STATUS.md`); downstream integrators should treat the embedding surface as real but keep an eye on the live blocker list and the [Changelog Index](changelog-index.md).
