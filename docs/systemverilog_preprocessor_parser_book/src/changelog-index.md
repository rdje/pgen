# Changelog Index

This chapter is an index — pointers into the documents that carry the full changelog detail, plus the short list of releases relevant to this book. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | **The authoritative contract.** Its "Schema Versioning" table and per-release Highlights sections list the AST shape changes consumers care about, and its "Known Defects" section records released-parser defects. Where this book and the contract disagree, the contract wins. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | The canonical per-bug tracker. When a defect is accepted (whether downstream-reported or internally surfaced), the ledger row records the reproducer, root cause, fix proof, and the parser release it was fixed in. Carries `SVPP-0001` (status `Released`, fixed in `1.0.2` / schema `2` — see "Resolved defects" below). The `1.0.3` POST-SV-AUDIT `macro_formals` Category-A fix is **not** in this ledger — it is a deliberate audit-driven AST-shape correction (no `<invalid_sequence_access>`, no crash), tracked via `docs/POST_SV_AUDIT_LEDGER.md` and the schema-`3` Highlights instead. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all repository changes, sv_preprocessor among them. |
| Git tags + commit log | Commit-by-commit | The most granular source — use for diffs once you know which release to inspect. |

When investigating "what changed and why," start with the contract document, drop down to the bug ledger for specific accepted bugs, and fall back to git for diffs.

## Why this index is short by design

The main SystemVerilog parser's changelog index is long because its return-annotation campaign landed rule-by-rule across 115 slices, each bumping the schema version and getting its own row. **The sv_preprocessor parser is different: it has a small, line-oriented directive grammar that was typed in a single comprehensive slice — SVPP-Slice-1 — plus a follow-up correctness fix (`SVPP-0001`, release `1.0.2` / schema `2`) and a POST-SV-AUDIT Category-A shape correction (`macro_formals`, release `1.0.3` / schema `3`), so the sv_preprocessor schema timeline has exactly four entries.** This is the intended state, not an incomplete index. Subsequent shape-affecting slices, if any, will each add a contract Highlights section, a [Schema Versioning](schema-versioning.md) row, and an entry below.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. The four entries below mirror the "Schema Versioning" table in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`; the contract is authoritative for the live state.

### Schema 3 / release 1.0.3 — POST-SV-AUDIT `macro_formals` Category-A correction (raw `{first, rest}` → clean list; 66 annotations / 28 rules)

The POST-SV-AUDIT.2.1 Category-A AST-shape correction (landed 2026-05-17, `PGEN-POST-SV-AUDIT-0002`).

- **Schema-version milestone:** integer `3` (first parser release: `1.0.3`).
- **AST-dump schema version:** `3` — the integer consumers **pin** from `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" as a compile-time constant (still **not** a field of `AstDumpPayload`; re-validate the pin against the contract's "Schema Versioning" table when bumping PGEN).
- **What changed:** the POST-SV-AUDIT static classification found `macro_formals := lparen macro_formal (comma macro_formal)* rparen -> {first: $2, rest: $3}` was a static-conclusive Category-A raw-envelope misuse — `rest` surfaced the raw `[[comma, macro_formal], …]` separator envelope, forcing consumers to index past the `comma`. Corrected to the canonical extraction-spread `macro_formals := lparen macro_formal (comma macro_formal)* rparen -> [$2, $3::2*]` (drop the separator; emit a clean flat `macro_formal` list — the `object_properties` reference idiom). For input `` `define M(a, b, c) a+b+c `` `pp_define.formals` was `{"first": {"default": [], "name": [[], "a"]}, "rest": [[[[], ","], {"default": [], "name": [[" "], "b"]}], [[[], ","], {"default": [], "name": [[" "], "c"]}]]}`; it is now `[{"default": [], "name": [[], "a"]}, {"default": [], "name": [[" "], "b"]}, {"default": [], "name": [[" "], "c"]}]` — a clean flat list of `macro_formal` `{name, default}` objects. No `<invalid_sequence_access>` (a clean Category-A shape improvement, **not** the `SVPP-0001` inline-alternation corruption class).
- **Annotation count:** **66 across 28 distinct rules — unchanged.** `macro_formals` is still one rule / one annotation; only its `annotation_type` changed `return_object` → `return_array` and `normalized_text` `{first: $2, rest: $3}` → `[$2, $3::2*]`. The surface is now **65 `return_object` + 1 `return_array`** (was all 66 `return_object`).
- **Accept set:** unchanged — same accepted inputs; only the `macro_formals` annotation form changed.
- **Breaking:** yes — `pp_define.formals` changed shape in a consumer-visible way (schema `2 → 3`). A consumer that walked `formals.first` + `formals.rest[][1]` must repin to schema `3` and treat `formals` as a flat `macro_formal[]`.
- **Not a released-parser bug:** this is a deliberate audit-driven shape correction tracked via `docs/POST_SV_AUDIT_LEDGER.md` (`PGEN-POST-SV-AUDIT-0002`), **not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (that ledger is reserved for the `<invalid_sequence_access>` corruption/crash class).
- **Contract section:** `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` → "AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT) — `macro_formals` Category-A raw-envelope → clean list".
- **Machine-checkable inventory:** `generated/systemverilog_preprocessor_return_annotations.json` (66 entries) and its content-identical embedded mirror `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` (new `macro_with_formals` sample; `macro_formals` now `return_array` / `[$2, $3::2*]`).
- **Per-rule shapes:** the macro-formals section of [Top-Level Rules](rules-top-level.md); the `macro_formals` iteration in [Walking the AST](walking-the-ast.md#iterating-the-macro_formals-list); the schema-`3` row in [Schema Versioning](schema-versioning.md).

### Schema 2 / release 1.0.2 — SVPP-0001 correctness fix (pp_if_branch.keyword; 66 annotations / 28 rules)

The single follow-up correctness fix after SVPP-Slice-1 (landed 2026-05-16).

- **Schema-version milestone:** integer `2` (first parser release: `1.0.2`).
- **AST-dump schema version:** `2` — the integer consumers **pin** from `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" as a compile-time constant (still **not** a field of `AstDumpPayload`; re-validate the pin against the contract's "Schema Versioning" table when bumping PGEN).
- **What changed:** `SVPP-0001` fixed. `pp_if_branch.keyword` no longer emits the malformed `<invalid_sequence_access>` object for `` `ifdef`` / `` `ifndef`` conditional input. The inline alternation `(kw_ifdef | kw_ifndef)` that was the lead element of `pp_if_branch` is lifted into a **named** rule `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} | kw_ifndef -> {kind: "ifndef"}` (the proven `rtl_const_expr` RTL-CE-Slice-2 / `systemverilog.ebnf` op-chain idiom). `pp_if_branch`'s annotation is unchanged (`{keyword: $1, …}`); only `$1` now binds the clean named rule, so `if_branch.keyword` is now the typed polarity object `{kind: "ifdef"}` (or `{kind: "ifndef"}`). Read the conditional polarity from `if_branch.keyword.kind`.
- **Annotation count:** **64 → 66** across **27 → 28** distinct rules (the +2 = the new `pp_if_keyword` `{kind: "ifdef"}` / `{kind: "ifndef"}` `return_object` branches; +1 distinct rule = `pp_if_keyword`). All 66 remain `annotation_type: "return_object"`.
- **Accept set:** unchanged — same accepted inputs; only the `pp_if_branch.keyword` shape changed (purely the alternation lift + its 2 branch annotations).
- **Breaking:** yes — `pp_if_branch.keyword` changed shape in a consumer-visible way (schema `1 → 2`). Consumers written against the pre-fix schema-`1` shape must repin to schema `2` and switch to the `keyword.kind` discriminator.
- **Contract section:** `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.0.2 / Contract 1.0.2 Highlights — SVPP-0001 correctness fix".
- **Machine-checkable inventory:** `generated/systemverilog_preprocessor_return_annotations.json` (66 entries) and its content-identical embedded mirror `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`.
- **Worked example / per-rule shapes:** the schema-`2` transition in [Conditional Compilation](examples-conditional.md); [Top-Level Rules](rules-top-level.md); the schema-`2` row in [Schema Versioning](schema-versioning.md).

### Schema 1.0.0 / release 1.0.1 — SVPP-Slice-1: full grammar typed (64 annotations / 27 rules)

The initial typing slice, covering the entire `grammars/systemverilog_preprocessor.ebnf` directive surface in **one batch** (landed 2026-05-14).

- **Schema-version milestone:** `1.0.0` (first parser release: `1.0.1`).
- **AST-dump schema version:** `1` — the integer consumers **pin** from `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" as a compile-time constant. It is **not** a field of `AstDumpPayload` (that struct exposes only `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`); re-validate the pin against the contract's "Schema Versioning" table when bumping PGEN.
- **Annotation count:** **64** across **27 distinct rules** (on top of the pre-typing baseline of one root annotation). All 64 are `annotation_type: "return_object"`. Coverage: the `systemverilog_preprocessor_file` root; the 10-kind `pp_item` dispatch; the 7 per-directive shapes (`define` / `undef` / `include` / `timescale` / `default_nettype` / `celldefine` / `endcelldefine`); `include_path` / `nettype_value` / `time_literal`; the 5-node conditional-compilation tree (`pp_conditional` / `pp_if_branch` / `pp_elsif_branch` / `pp_else_branch` / `pp_endif`); `condition_expr` / `condition_atom` (12 kinds); `macro_formals` / `macro_formal` / `macro_default_value` / `macro_default_atom` (8 kinds); `macro_body` / `macro_body_fragment` (9 kinds); and the passthrough lines (`pp_non_directive_line` / `pp_blank_line`).
- **Accept set:** unchanged — same accepted inputs as the pre-typing baseline; only the AST shape became typed.
- **Known defect shipped (since fixed):** `SVPP-0001` — at this `1.0.1` baseline `pp_if_branch.keyword` emitted `<invalid_sequence_access>` for `` `ifdef`` / `` `ifndef`` conditional input. **Fixed in release `1.0.2` / schema `2`** (see the schema-`2` entry above and "Resolved defects" below). The `` `define`` / non-conditional surface was always unaffected.
- **Contract section:** `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.0.1 / Contract 1.0.1 Highlights — SVPP-Slice-1".
- **Machine-checkable inventory:** `generated/systemverilog_preprocessor_return_annotations.json` (64 entries) and its content-identical embedded mirror `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` (the contract-embedded copy omits only the cosmetic `raw_text` field).
- **Per-rule shapes:** [Top-Level Rules](rules-top-level.md); the schema-`1` row in [Schema Versioning](schema-versioning.md).

### Schema 0.1.0 / release 1.0.0 — foundation baseline

The pre-typing baseline.

- **Schema-version milestone:** `0.1.0` (first parser release: `1.0.0`).
- **State:** `grammars/systemverilog_preprocessor.ebnf` un-annotated except for the `systemverilog_preprocessor_file -> {type, items}` root. The AST dump was the recursive-envelope shape across all other rules (see [AST Envelope Structure](ast-envelope.md)).
- **Contract section:** the `0.1.0` row of the "Schema Versioning" table in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`.

## Resolved defects

`SVPP-0001` — `pp_if_branch.keyword` `<invalid_sequence_access>` (status `Released`; **fixed in parser release `1.0.2` / schema `2`**). *Historical (release `1.0.1`, schema `1`):* for `` `ifdef`` / `` `ifndef`` conditional input the `items[].body.if_branch.keyword` field was a malformed nested object containing three `"<invalid_sequence_access>"` strings instead of the keyword token — a bare positional `$1` bound to the inline `(kw_ifdef | kw_ifndef)` alternation group, the same emit-time defect class fixed for `rtl_const_expr` in RTL-CE-Slice-2. **Fix:** the inline alternation is lifted into the named rule `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} | kw_ifndef -> {kind: "ifndef"}`; `pp_if_branch`'s bare `keyword: $1` now binds the clean named rule, so `if_branch.keyword` is the typed polarity object `{kind: "ifdef"}` / `{kind: "ifndef"}` with no `<invalid_sequence_access>` anywhere. The guard macro was always correct at the sibling `if_branch.macro`; the `` `define`` / non-conditional surface was always unaffected. Recorded (status `Released`) in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0001`) and `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Resolved Defects — `SVPP-0001`"; the schema-`1`→`2` transition is shown with the pre-fix history kept honestly in the [Conditional Compilation](examples-conditional.md) worked example and the schema-`2` row of [Schema Versioning](schema-versioning.md).

## AST-shape corrections (audit-driven, not parser bugs)

`macro_formals` raw `{first, rest}` envelope (POST-SV-AUDIT.2.1, `PGEN-POST-SV-AUDIT-0002`; **corrected in parser release `1.0.3` / schema `3`**). *Historical (≤ release `1.0.2`, schema `2`):* `macro_formals := lparen macro_formal (comma macro_formal)* rparen -> {first: $2, rest: $3}` exposed the raw `[[comma, macro_formal], …]` iteration envelope at `pp_define.formals.rest`, a static-conclusive Category-A raw-envelope misuse (the `comma` carries no payload a consumer needs). **Correction:** the annotation was changed to the canonical extraction-spread `[$2, $3::2*]`, so `pp_define.formals` is now a clean flat `macro_formal[]` list (the count stayed 66 / 28; only `macro_formals`'s annotation form changed `return_object` → `return_array`). This is **not** a released-parser bug — there was no `<invalid_sequence_access>` and no crash; it is a deliberate audit-driven AST-shape correction, so it is tracked in `docs/POST_SV_AUDIT_LEDGER.md` and the schema-`3` Highlights, **not** in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`. The schema-`2`→`3` transition is shown with the pre-fix history kept honestly in [Walking the AST](walking-the-ast.md#iterating-the-macro_formals-list) and the schema-`3` row of [Schema Versioning](schema-versioning.md).

## Bug ledger status

`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` is the canonical per-bug tracker. It currently carries one sv_preprocessor row — `SVPP-0001` — surfaced internally during SVPP-MDBOOK worked-example authoring (no external downstream report), now at status `Released` (fixed in parser release `1.0.2` / schema `2`). The `1.0.3` POST-SV-AUDIT `macro_formals` Category-A correction did **not** add a ledger row — it is an audit-driven shape correction, not a released-parser bug (tracked via `docs/POST_SV_AUDIT_LEDGER.md`). When a downstream sv_preprocessor bug is accepted, it gets its own ledger row recording the reproducer bundle, root cause, fix proof, and the parser release it was fixed in; this index will then point at the relevant contract Highlights section for any accompanying shape change. Reports follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## How to follow per-slice changes

Each shape-affecting slice after SVPP-Slice-1 (the `1.0.2` `SVPP-0001` fix was the first such slice; the `1.0.3` POST-SV-AUDIT `macro_formals` Category-A correction the second) gets:

1. A grammar change in `grammars/systemverilog_preprocessor.ebnf` (the `-> ...` annotation or restructure).
2. A manifest update in `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` (and the regenerated `generated/systemverilog_preprocessor_return_annotations.json`).
3. A parser-release / contract-version bump and a Highlights section in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the milestone.
5. An entry in this changelog index summarizing the slice.
6. A regression-lock test pinning the new typed shape (`cargo test --lib --features generated_parsers systemverilog_preprocessor_ast_shape_contract`).

The live-book policy bundles all six in the same commit (the `1.0.2` `SVPP-0001` fix and the `1.0.3` POST-SV-AUDIT `macro_formals` correction both landed exactly that way). Because SVPP-Slice-1 already typed the full directive grammar, the `1.0.2` fix closed `SVPP-0001`, and the `1.0.3` correction flattened the `macro_formals` `{first, rest}` list to the clean `[$2, $3::2*]` array, the realistic future driver of new entries here is further bug-ledger-driven shape fixes, any remaining POST-SV-AUDIT dispositions, and annotating the remaining un-annotated leaf/text rules — not a long rule-by-rule campaign. The sv_preprocessor parser covers only the preprocessor directive surface (`` `define`` / `` `undef`` / `` `include`` / `` `timescale`` / `` `default_nettype`` / `` `celldefine`` / `` `endcelldefine``, conditional compilation, macro formals/defaults/bodies, and passthrough lines); for the full SystemVerilog language grammar see the `systemverilog` family.
