# Schema Versioning

This chapter explains how the PGEN rtl_frontend parser's AST shape is versioned, what guarantees consumers can rely on, and how to pin to a known shape. The authoritative numbers come from `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md`; where this chapter and the contract disagree, **the contract wins**.

## Two versioning axes

The rtl_frontend parser carries **two** version numbers:

1. **Parser release version** — currently `1.0.3`. Tracks the parser library's release identity. Bumped on every functional change to the parser, including bug fixes, perf work, and grammar changes.
2. **AST-dump schema version** — currently `3`. Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

A single parser release can carry the same schema version as the previous release (no shape change) or a bumped schema version (shape changed). The two numbers move independently.

These numbers are taken from the integration contract's "Contract Identity" section, which records:

- Contract version: `1.0.3`
- Parser release version: `1.0.3`
- rtl_frontend AST-dump schema version: `3`
- Annotation count: **156** (RTL-FE-Slice-1..7 — the full grammar typed across seven slices, on 74 distinct rules; the `1.0.2` `RTL-FE-0001` correctness fix and the `1.0.3` POST-SV-AUDIT batch — the 15 Category-A list-shape corrections + the `RTL-FE-0002` `event_control_list` inline-alternation fix — did **not** change this count: the five `1.0.2` `*_op` rules and the new `1.0.3` `event_separator` rule are un-annotated alternations, and the 15 Category-A rules changed annotation form not count)

The contract document `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` is the authoritative source for all of these per release.

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

The `1.0.2` correctness fix **did** bump the schema (`1` → `2`): although it fixed a bug (`RTL-FE-0001`), it changed a user-visible shape — the ten `binop_chain` levels' `rest` (was `"<invalid_sequence_access>"` plus a malformed nested object for multi-operand input, now the clean `[ [op-envelope, operand] … ]` array). It restructured a shape a consumer could have observed, so it is a breaking change under the policy below, not a transparent fix. It did **not** change the annotation inventory: the five inline operator alternations were lifted into the **un-annotated** named rules `equality_op` / `relational_op` / `shift_op` / `additive_op` / `multiplicative_op`, so the count stays **156 annotations / 74 distinct rules**.

The `1.0.3` POST-SV-AUDIT batch **also** bumped the schema (`2` → `3`). The POST-SV-AUDIT.2.2 static classification (`docs/POST_SV_AUDIT_LEDGER.md`, `PGEN-POST-SV-AUDIT-0003`) found **15 static-conclusive Category-A raw-envelope list rules** in `grammars/rtl_frontend.ebnf` (`parameter_declaration_sequence`, `port_list`, `port_group`, `genvar_declaration`, `module_instantiation`, `parameter_override_list` named+positional, `port_connection_list` named+positional, `net_declaration`, `assignment_target` concat, `repetition_expr`, `concatenation_expr`, `scoped_identifier`, `enum_type`, `struct_union_field`) that exposed the raw `{first, rest}` (resp. `{…, first, rest}`) iteration envelope — `rest` was the raw `[[sep, item], …]` separator envelope a consumer had to walk past. Each was corrected to the canonical extraction-spread `[$N, $M::K*]` so the value (or its element field) is now a **clean flat list**. Separately, `RTL-FE-0002`: `event_control_list` had an **inline alternation** `( comma | kw_or )` as its `( … )*` iteration lead feeding the bare `$4`, the same emit-time corruption class as `RTL-FE-0001` (but **not** a `binop_chain` level), so for multi-entry sensitivity input `procedural_block.event_control` surfaced `"<invalid_sequence_access>"`; the inline alternation was lifted into a new **un-annotated** named rule `event_separator := comma | kw_or` and the separator dropped, so `event_control` is now a clean `[{edge, expr}, …]` list. These restructured consumer-visible shapes (a `1.0.2` consumer that walked `.first` + `.rest[][1]` must repin), so they are breaking changes. They did **not** change the annotation count: the bare-list Category-A rules flip `return_object` → `return_array`, the `{…, items/ports/names: […]}` ones stay `return_object` with a new `normalized_text`, and the new `event_separator` is un-annotated — so the count stays **156 annotations / 74 distinct rules**. The 15 Category-A corrections are a clean shape improvement (no `<invalid_sequence_access>`) and are **not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`; the `RTL-FE-0002` `event_control_list` inline-alternation corruption **is** an `<invalid_sequence_access>` defect and **is** logged there (status `Released`).

The rtl_frontend grammar was typed across seven slices (RTL-FE-Slice-1..7, 156 annotations / 74 rules) landed together on 2026-05-14, rather than the long slice-by-slice cadence used by the SystemVerilog campaign, so the rtl_frontend schema timeline is short. A follow-up correctness fix (parser release `1.0.2`, schema `2`, landed 2026-05-16) corrected the `binop_chain.rest` shape with the inventory unchanged — see the schema-`2` row below and the contract's Release 1.0.2 Highlights. The POST-SV-AUDIT batch (parser release `1.0.3`, schema `3`, landed 2026-05-17) corrected the 15 Category-A list shapes + the `RTL-FE-0002` `event_control_list` inline-alternation defect, again with the inventory unchanged — see the schema-`3` row below and the contract's "AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT)". Subsequent shape-affecting slices each get their own contract-version row and a [Changelog Index](changelog-index.md) entry.

## Byte-equivalence guarantee

For any input the parser accepts, the AST dump is **byte-deterministic** for a given parser-release version: object keys in canonical (alphabetical) order, canonical number formatting, no embedded timestamps or hashes. Re-running the parse on the same input produces an identical JSON value. Whitespace is configurable via `AstDumpOptions.pretty` but the underlying JSON value is the same.

This determinism is a **hard guarantee** of the schema. Any non-determinism is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Schema version timeline

This table mirrors the "Schema Versioning" table in `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md`. The contract is authoritative for the live state.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 1.0.3 | 1.0.3 | **POST-SV-AUDIT Category-A batch + `RTL-FE-0002` (POST-SV-AUDIT.2.2, breaking).** Two changes landed together. (1) **15 Category-A list-shape corrections** (not parser bugs): `parameter_declaration_sequence`, `port_list`, `port_group`, `genvar_declaration`, `module_instantiation`, `parameter_override_list` (named+positional), `port_connection_list` (named+positional), `net_declaration`, `assignment_target` (concat), `repetition_expr`, `concatenation_expr`, `scoped_identifier`, `enum_type`, `struct_union_field` no longer expose the raw `{first, rest}` (resp. `{…, first, rest}`) iteration envelope — the POST-SV-AUDIT.2.2 static classification (`docs/POST_SV_AUDIT_LEDGER.md`, `PGEN-POST-SV-AUDIT-0003`) found these were static-conclusive Category-A raw-envelope misuses (`rest` surfaced the raw `[[sep, item], …]` separator envelope). Each is corrected to the canonical extraction-spread `[$N, $M::K*]` (drop the semantically-irrelevant separator; emit a clean flat list). The five bare-list rules (`parameter_declaration_sequence`, `port_list`, `genvar_declaration`, `concatenation_expr`, `scoped_identifier`) now emit a **top-level array**; the others keep their leading typed fields and carry the element list in a single named field (`ports` / `instances` / `items` / `names`). No `<invalid_sequence_access>` (clean shape improvement, **not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`). (2) **`RTL-FE-0002` correctness fix:** `event_control_list := at lparen event_control_item ( ( comma \| kw_or ) event_control_item )* rparen -> {first: $3, rest: $4}` bound the bare `$4` to an **inline alternation** `( comma \| kw_or )` iteration lead — the same emit-time corruption class as `RTL-FE-0001` (but **not** a `binop_chain` level) — so `procedural_block.event_control` surfaced `"<invalid_sequence_access>"` for multi-entry sensitivity input. Lifted the inline alternation into a new **un-annotated** named rule `event_separator := comma \| kw_or` and rewrote to `at lparen event_control_item ( event_separator event_control_item )* rparen -> [$3, $4::2*]` (separator dropped; clean `event_control_item` list). Tracked (status `Released`) as `RTL-FE-0002` in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`. Annotation inventory **unchanged at 156 / 74** (bare-list Category-A rules flip `return_object` → `return_array`; the `{…, items/ports/names: […]}` ones stay `return_object` with a new `normalized_text`; the new `event_separator` is an un-annotated alternation, not in the inventory). Same accept set as `1.0.2`. AST-dump schema version field value: `3`. |
| 1.0.2 | 1.0.2 | **Correctness fix (`RTL-FE-0001`).** The ten `binop_chain` levels' `rest` no longer emits `"<invalid_sequence_access>"` (plus a malformed nested `{level, lhs:["","<op>"], rest:<invalid>}` object) for multi-operand input: the five inline operator alternations were lifted into five **named, un-annotated** rules (`equality_op := eqeq \| ne`, `relational_op := less_equal \| lt \| ge \| gt`, `shift_op := shl \| shr`, `additive_op := plus \| minus`, `multiplicative_op := star \| slash \| percent`), so each level's `rest: $2` now captures a **clean array** of `[ [op-envelope], operand ]` entries (operator token text preserved at `entry[0][1]`, `[]` when no operator). This mirrors the gate-locked `rtl_const_expr` RTL-CE-Slice-2 fix and the `systemverilog.ebnf` `binary_operator` op-chain idiom. The five `binop_chain` level annotations are **unchanged**; only the inline `( a \| b )` became a named rule. Annotation inventory **unchanged at 156 / 74** (the five `*_op` rules are un-annotated alternations and not in the inventory). Same accept set as `1.0.1`. AST-dump schema version field value: `2`. |
| 1.0.0 | 1.0.1 | **RTL-FE-Slice-1..7** — initial 156-annotation baseline (74 distinct rules). Dispatch wrappers (`design_item` / `package_item` / `module_item` / `generate_item`), keyword/operator leaves, expression dispatch + procedural blocks, the 10-rule `binop_chain` hierarchy (`logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr`), declarations + module structure, parameter/port rules, and the module-instantiation / ports / statements / signals / datatypes mass batch — all typed across seven slices. Same accept set as the pre-typing baseline. Pre-correctness shape: the ten `binop_chain` levels' `rest` could surface `"<invalid_sequence_access>"` for multi-operand input (`RTL-FE-0001`). AST-dump schema version field value: `1`. |
| 0.1.0 | 1.0.0 | **Foundation baseline.** Grammar (`grammars/rtl_frontend.ebnf`) un-annotated except for the `rtl_frontend_file -> {type, items}` root. AST dump is the recursive-envelope shape across all rules. |

> Note on the version columns: the contract's "Schema version" column uses the `1.0.3` / `1.0.2` / `1.0.0` / `0.1.0` labels above for the milestones; the AST-dump schema version consumers pin against is the integer **`3`**. That integer is **not** a runtime field of `AstDumpPayload` (the real struct is `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`) — you pin it from this contract/book at integration time, not by reading the payload (see [Walking the AST](walking-the-ast.md)); use the contract's milestone labels when reading the changelog.

## How to pin to a known shape

1. **Record the parser-release version** your downstream code was written against — `1.0.3` as of this writing. It is recorded in `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity".
2. **Pin the AST-dump schema version you built against** — currently `3`, from `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity". This is a *compile-time constant in your consumer*, **not** a field of `AstDumpPayload` (that struct exposes only `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`). Check `truncated`, parse `dump_json`, then walk against the schema you pinned; re-validate the pin against the contract whenever you bump PGEN:

   ```rust
   // Schema version you integrated against, from the contract:
   const RTL_FRONTEND_AST_SCHEMA_VERSION: u32 = 3;

   let payload = outcome.ast_dump.expect("Success carries an AstDumpPayload");
   if payload.truncated {
       return Err("rtl_frontend AST dump truncated (dump_json holds the diagnostic envelope)".into());
   }
   let root: serde_json::Value = serde_json::from_str(&payload.dump_json)?;
   // RTL_FRONTEND_AST_SCHEMA_VERSION drives which walker you compiled;
   // re-check the contract's Schema Versioning table on every PGEN bump.
   // (schema 3: the 15 Category-A list rules — e.g. port_list,
   //  net_declaration, scoped_identifier — are now clean flat arrays,
   //  and procedural_block.event_control is a clean [{edge,expr},…]
   //  list; both were the raw {first, rest} envelope at schema 2 —
   //  POST-SV-AUDIT / RTL-FE-0002.)
   walk_schema_v3(&root);
   ```

3. **Vendor or pin the generated parser.** The rtl_frontend parser is on-demand-only (see [Build Recipe](build-recipe.md)); vendor `generated/rtl_frontend_parser.rs` against the recorded parser-release version, or build it in CI from the pinned `grammars/rtl_frontend.ebnf`.
4. **When you bump PGEN**, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the rules you consume, and re-run your walker's test corpus.

## Additive vs breaking changes

Within a single integer schema version, shape changes are intended to be **additive** wherever possible:

- **Additive (no integer schema bump expected):** a new optional field on an existing typed object, a new `kind` value on a dispatch rule for a previously-unparseable construct, a new typed shape on a rule that was previously raw envelope. Consumers using the unknown-shape fallthrough from [Walking the AST](walking-the-ast.md) keep working; consumers that hard-match a closed `kind` set must extend it.
- **Breaking (integer schema bump):** renaming or removing a field on an existing typed object, changing a `kind` discriminator value, restructuring a `{first, rest}` list into a flat array (the `1.0.3` POST-SV-AUDIT schema-`2`→`3` change did exactly this for the 15 Category-A list rules + `event_control_list`), or changing the default fall-through of an unannotated rule in a way that moves data consumers were already reading.

The contract's bump-trigger guidance is the binding policy; this section paraphrases it for walker authors. A consumer that (a) branches on the integer `schema_version`, (b) treats absent optionals as `[]`, and (c) uses the unknown-shape fallthrough is resilient to additive changes and fails loudly — not silently — on breaking ones.

## Reporting drift

If you observe an AST shape that disagrees with this book, the contract, or the live inventory `generated/rtl_frontend_return_annotations.json`:

1. Confirm against the machine-checkable inventory (`generated/rtl_frontend_return_annotations.json` / `rust/test_data/ast_shape_contract/rtl_frontend_v1.json`, 156 entries, identical content).
2. If the inventory agrees with what you observe but the contract does not, the contract is authoritative for intended behavior — file via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
3. Accepted released-parser bugs are tracked in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Future major version

A future schema milestone will land if and when the rtl_frontend grammar's remaining un-annotated rules (terminal/regex leaves like `identifier`, `named_data_type`, and the passthrough forms of `conditional_expr` / `unary_expr`) are either annotated or given a deliberate decision to remain raw envelope, and the shape definitions move to a locked tier. The rtl_frontend family is still an `In Progress` family in the live tracker (`LIVE_ACHIEVEMENT_STATUS.md`); the current grammar covers the synthesizable RTL subset, and the full IEEE 1800 SystemVerilog surface is out of scope (see the `systemverilog` family for that). Downstream integrators should treat the embedding surface as real but keep an eye on the live blocker list and the [Changelog Index](changelog-index.md).
