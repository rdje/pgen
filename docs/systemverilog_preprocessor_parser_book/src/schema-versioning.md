# Schema Versioning

This chapter explains how the PGEN sv_preprocessor parser's AST shape is versioned, what guarantees consumers can rely on, and how to pin to a known shape. The authoritative numbers come from `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`; where this chapter and the contract disagree, **the contract wins**.

## Two versioning axes

The sv_preprocessor parser carries **two** version numbers:

1. **Parser release version** — currently `1.0.3`. Tracks the parser library's release identity. Bumped on every functional change to the parser, including bug fixes, perf work, and grammar changes.
2. **AST-dump schema version** — currently `3`. Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

A single parser release can carry the same schema version as the previous release (no shape change) or a bumped schema version (shape changed). The two numbers move independently.

These numbers are taken from the integration contract's "Contract Identity" section, which records:

- Contract version: `1.0.3`
- Parser release version: `1.0.3`
- systemverilog_preprocessor AST-dump schema version: `3`
- Annotation count: **66** (SVPP-Slice-1's full-grammar batch plus the `1.0.2` `SVPP-0001` correctness fix; the `1.0.3` POST-SV-AUDIT `macro_formals` Category-A fix changed annotation form but not the count; across 28 distinct rules — now 65 `return_object` + 1 `return_array`)

The contract document `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` is the authoritative source for all of these per release.

## What "shape change" means

Any of these triggers a schema version bump:

- A new return annotation lands on a previously-unannotated rule.
- An existing return annotation is restructured.
- A grammar rule changes shape in a way that's user-visible (new branch added, branch removed, a sub-rule renamed in a way that affects shape, the `macro_formals` `{first, rest}` list flattened — exactly what the `1.0.3` POST-SV-AUDIT fix did, see the schema-`3` row below, etc.).
- The default fall-through behavior of unannotated rules changes.

These do **not** trigger a schema bump:

- Pure performance optimizations that produce the same AST.
- Internal codegen reorganization that doesn't reach the output.
- Parser-side bug fixes that produce the same shape consumers were already relying on.

The `1.0.2` `SVPP-0001` correctness fix **did** bump the schema (`1` → `2`): although it fixed a bug, it changed the user-visible `pp_if_branch.keyword` shape (was the malformed `"<invalid_sequence_access>"` object, now the typed `{kind: "ifdef"|"ifndef"}` `pp_if_keyword` polarity discriminator) and added two return annotations. It restructured a shape a consumer could have observed, so it is a breaking change under the policy below, not a transparent fix.

The `1.0.3` POST-SV-AUDIT `macro_formals` Category-A correction **also** bumped the schema (`2` → `3`): the static-conclusive audit (`docs/POST_SV_AUDIT_LEDGER.md`, `PGEN-POST-SV-AUDIT-0002`) found `macro_formals` exposed the raw `{first, rest}` iteration envelope — `rest` was the raw `[[comma, macro_formal], …]` separator envelope a consumer had to walk past. It was corrected to the canonical extraction-spread `[$2, $3::2*]` so `pp_define.formals` is now a **clean flat `macro_formal[]` list**. This restructured `pp_define.formals` in a consumer-visible way (a `1.0.2` consumer that walked `formals.first` + `formals.rest[][1]` must repin), so it is a breaking change. It did **not** change the annotation count: `macro_formals` is still one rule / one annotation — only its `annotation_type` changed `return_object` → `return_array` and its `normalized_text` `{first: $2, rest: $3}` → `[$2, $3::2*]`. This is **not** a released-parser bug (no `<invalid_sequence_access>`, no crash — it is a clean Category-A shape improvement); it is a deliberate audit-driven shape correction and is **not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

The sv_preprocessor grammar was typed in a single comprehensive batch (SVPP-Slice-1, 64 annotations / 27 rules) rather than the long slice-by-slice cadence used by the main SystemVerilog campaign, so the sv_preprocessor schema timeline is short. A follow-up correctness fix (parser release `1.0.2`, schema `2`, landed 2026-05-16) brought the inventory to 66 annotations / 28 rules; the POST-SV-AUDIT `macro_formals` Category-A correction (parser release `1.0.3`, schema `3`, landed 2026-05-17) kept the inventory at 66 annotations / 28 rules but changed `macro_formals` from `return_object` to `return_array` — see the schema-`3` and schema-`2` rows below and the contract's Highlights. Subsequent shape-affecting slices each get their own contract-version row and a [Changelog Index](changelog-index.md) entry.

## Byte-equivalence guarantee

For any input the parser accepts, the AST dump is **byte-deterministic** for a given parser-release version: object keys in canonical (alphabetical) order, canonical number formatting, no embedded timestamps or hashes. Re-running the parse on the same input produces an identical JSON value. Whitespace is configurable via `AstDumpOptions.pretty` but the underlying JSON value is the same.

This determinism is a **hard guarantee** of the schema. Any non-determinism is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Schema version timeline

This table mirrors the "Schema Versioning" table in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`. The contract is authoritative for the live state.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 3 | 1.0.3 | **`macro_formals` Category-A AST-shape correction (POST-SV-AUDIT, breaking, not a parser bug).** `macro_formals` no longer exposes the raw `{first, rest}` iteration envelope. The POST-SV-AUDIT.2.1 static classification (`docs/POST_SV_AUDIT_LEDGER.md`, `PGEN-POST-SV-AUDIT-0002`) found `macro_formals := lparen macro_formal (comma macro_formal)* rparen -> {first: $2, rest: $3}` was a static-conclusive Category-A raw-envelope misuse — `rest` surfaced the raw `[[comma, macro_formal], …]` separator envelope, forcing consumers to index past the `comma` separator. Corrected to the canonical extraction-spread `macro_formals := lparen macro_formal (comma macro_formal)* rparen -> [$2, $3::2*]` (drop the semantically-irrelevant `comma`; emit a clean flat `macro_formal` list — the `object_properties` reference idiom). For input `` `define M(a, b, c) a+b+c `` `pp_define.formals` was `{"first": {"default": [], "name": [[], "a"]}, "rest": [[[[], ","], {"default": [], "name": [[" "], "b"]}], [[[], ","], {"default": [], "name": [[" "], "c"]}]]}`; it is now `[{"default": [], "name": [[], "a"]}, {"default": [], "name": [[" "], "b"]}, {"default": [], "name": [[" "], "c"]}]` — a clean flat list of `macro_formal` `{name, default}` objects. No `<invalid_sequence_access>` (a clean Category-A shape improvement, **not** the inline-alternation corruption class of `SVPP-0001`). Annotation count **unchanged 66 / 28 distinct rules** — `macro_formals` is still one rule / one annotation; only its `annotation_type` changed `return_object` → `return_array` and `normalized_text` `{first: $2, rest: $3}` → `[$2, $3::2*]`, so the surface is now **65 `return_object` + 1 `return_array`** (was all 66 `return_object`). Same accept set. This is **not** a released-parser bug (not logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`) — a deliberate audit-driven correction. AST-dump schema version field value: `3`. |
| 2 | 1.0.2 | **`SVPP-0001` correctness fix (breaking).** `pp_if_branch.keyword` no longer emits the malformed `"<invalid_sequence_access>"` object for `` `ifdef`` / `` `ifndef`` conditional input. The inline alternation `(kw_ifdef \| kw_ifndef)` that was the lead element of `pp_if_branch` (corrupting the positional model so the bare `keyword: $1` mis-recursed) is lifted into a **named** rule `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} \| kw_ifndef -> {kind: "ifndef"}` — the proven `rtl_const_expr` RTL-CE-Slice-2 / `systemverilog.ebnf` op-chain idiom. `pp_if_branch`'s annotation is **unchanged** (`{keyword: $1, macro: $2, tail: $3, items: $5}`); only `$1` now binds the clean named rule, so `if_branch.keyword` is now the typed polarity object `{kind: "ifdef"}` (or `{kind: "ifndef"}`). Annotation inventory **64 → 66** (the 2 new `pp_if_keyword` `return_object` branches); distinct rules **27 → 28** (the new `pp_if_keyword`). Same accept set (no grammar acceptance change — purely the alternation lift + its 2 branch annotations). AST-dump schema version field value: `2`. |
| 1.0.0 | 1.0.1 | **SVPP-Slice-1** — initial 64-annotation baseline (27 distinct rules). `pp_item` dispatch (10 kinds), 7 directive shapes (`define` / `undef` / `include` / `timescale` / `default_nettype` / `celldefine` / `endcelldefine`), `include_path` / `nettype_value` / `time_literal`, conditional-compilation tree (5 nodes), `condition_expr` / `condition_atom` (12 kinds), `macro_formals` / `macro_formal` / `macro_default_value` / `macro_default_atom` (8 kinds) / `macro_body` / `macro_body_fragment` (9 kinds), passthrough lines — all typed in one comprehensive batch. Same accept set as the pre-typing baseline. **NOTE:** the `pp_if_branch.keyword` shape in this baseline was defective (`SVPP-0001`, the inline-alternation-`$N` `"<invalid_sequence_access>"` malformation) — see schema `2` for the correction. AST-dump schema version field value: `1`. |
| 0.1.0 | 1.0.0 | **Foundation baseline.** Grammar (`grammars/systemverilog_preprocessor.ebnf`) un-annotated except for the `systemverilog_preprocessor_file -> {type, items}` root. AST dump is the recursive-envelope shape across all other rules. |

> Note on the version columns: the contract's "Schema version" column uses the integer `3` / `2` rows plus the `1.0.0` / `0.1.0` milestone labels above; the AST-dump schema version consumers pin against is the integer **`3`**. That integer is **not** a runtime field of `AstDumpPayload` (the real struct is `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`) — you pin it from this contract/book at integration time, not by reading the payload (see [Walking the AST](walking-the-ast.md)); use the contract's milestone labels when reading the changelog.

## How to pin to a known shape

1. **Record the parser-release version** your downstream code was written against — `1.0.3` as of this writing. It is recorded in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity".
2. **Pin the AST-dump schema version you built against** — currently `3`, from `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity". This is a *compile-time constant in your consumer*, **not** a field of `AstDumpPayload` (that struct exposes only `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`). Check `truncated`, parse `dump_json`, then walk against the schema you pinned; re-validate the pin against the contract whenever you bump PGEN:

   ```rust
   // Schema version you integrated against, from the contract:
   const SVPP_AST_SCHEMA_VERSION: u32 = 3;

   let payload = outcome.ast_dump.expect("Success carries an AstDumpPayload");
   if payload.truncated {
       return Err("sv_preprocessor AST dump truncated (dump_json holds the diagnostic envelope)".into());
   }
   let root: serde_json::Value = serde_json::from_str(&payload.dump_json)?;
   // SVPP_AST_SCHEMA_VERSION drives which walker you compiled;
   // re-check the contract's Schema Versioning table on every PGEN bump.
   // (schema 2 added pp_if_keyword: if_branch.keyword is now
   // {kind: "ifdef"|"ifndef"}, was the SVPP-0001 malformed object at 1.
   //  schema 3: pp_define.formals is now a clean macro_formal[] list,
   //  was the raw {first, rest} envelope at schema 2 — POST-SV-AUDIT.)
   match SVPP_AST_SCHEMA_VERSION {
       3 => walk_schema_v3(&root),
       other => return Err(format!("unsupported sv_preprocessor schema version: {other}")),
   }
   ```

3. **Vendor or pin the generated parser.** The sv_preprocessor parser is on-demand-only (see [Build Recipe](build-recipe.md)); vendor `generated/systemverilog_preprocessor_parser.rs` against the recorded parser-release version, or build it in CI from the pinned `grammars/systemverilog_preprocessor.ebnf`.
4. **When you bump PGEN**, scan the [Changelog Index](changelog-index.md) for shape-change rows that affect the directives you consume, and re-run your walker's test corpus.

## Additive vs breaking changes

Within a single integer schema version, shape changes are intended to be **additive** wherever possible:

- **Additive (no integer schema bump expected):** a new optional field on an existing typed object, a new `kind` value on a dispatch rule for a previously-unparseable construct, a new typed shape on a rule that was previously raw envelope. Consumers using the unknown-shape fallthrough from [Walking the AST](walking-the-ast.md) keep working; consumers that hard-match a closed `kind` set must extend it.
- **Breaking (integer schema bump):** renaming or removing a field on an existing typed object, changing a `kind` discriminator value, restructuring the `macro_formals` `{first, rest}` list into a flat array (the `1.0.3` POST-SV-AUDIT schema-`2`→`3` change did exactly this), or changing the default fall-through of an unannotated rule in a way that moves data consumers were already reading.

The contract's bump-trigger guidance is the binding policy; this section paraphrases it for walker authors. A consumer that (a) branches on its pinned `SVPP_AST_SCHEMA_VERSION` constant, (b) treats absent optionals as `[]`, and (c) uses the unknown-shape fallthrough is resilient to additive changes and fails loudly — not silently — on breaking ones.

## Reporting drift

If you observe an AST shape that disagrees with this book, the contract, or the live inventory `generated/systemverilog_preprocessor_return_annotations.json`:

1. Confirm against the machine-checkable inventory (`generated/systemverilog_preprocessor_return_annotations.json` / `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`, 66 entries; identical tuples — the contract-embedded copy omits only the cosmetic `raw_text` field).
2. If the inventory agrees with what you observe but the contract does not, the contract is authoritative for intended behavior — file via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
3. Accepted released-parser bugs are tracked in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Future major version

A future schema milestone will land if and when the sv_preprocessor grammar's remaining un-annotated rules (terminal/regex leaves like `identifier` / `macro_name`, and the literal-text runs `macro_body_text` / `condition_text` / `macro_default_text` / `non_directive_text`) are either annotated or given a deliberate decision to remain raw envelope, and the shape definitions move to a locked tier. Downstream integrators should treat the embedding surface as real but keep an eye on the live blocker list and the [Changelog Index](changelog-index.md). Note that, per the integration contract, sv_preprocessor does **not** yet publish a dedicated general-purpose embedding API profile the way `systemverilog`, `vhdl`, or `regex` do — the stable surface is the generic `parse_grammar_profile*` family (see [Public API Surface](public-api.md)).
