# Changelog Index

This chapter is an index — pointers into the documents that carry the full changelog detail, plus the short list of releases relevant to this book. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | **The authoritative contract.** Its "Schema Versioning" table and per-release Highlights sections list the AST shape changes consumers care about, and its "Known Defects" section records released-parser defects. Where this book and the contract disagree, the contract wins. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | The canonical per-bug tracker. When a defect is accepted (whether downstream-reported or internally surfaced), the ledger row records the reproducer, root cause, fix proof, and the parser release it was fixed in. Carries `SVPP-0001` (see "Known defects" below). |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all repository changes, sv_preprocessor among them. |
| Git tags + commit log | Commit-by-commit | The most granular source — use for diffs once you know which release to inspect. |

When investigating "what changed and why," start with the contract document, drop down to the bug ledger for specific accepted bugs, and fall back to git for diffs.

## Why this index is short by design

The main SystemVerilog parser's changelog index is long because its return-annotation campaign landed rule-by-rule across 115 slices, each bumping the schema version and getting its own row. **The sv_preprocessor parser is different: it has a small, line-oriented directive grammar that was typed in a single comprehensive slice — SVPP-Slice-1 — so the sv_preprocessor schema timeline has exactly two entries.** This is the intended state, not an incomplete index. Subsequent shape-affecting slices, if any, will each add a contract Highlights section, a [Schema Versioning](schema-versioning.md) row, and an entry below.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. The two entries below mirror the "Schema Versioning" table in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`; the contract is authoritative for the live state.

### Schema 1.0.0 / release 1.0.1 — SVPP-Slice-1: full grammar typed (64 annotations / 27 rules)

The initial typing slice, covering the entire `grammars/systemverilog_preprocessor.ebnf` directive surface in **one batch** (landed 2026-05-14).

- **Schema-version milestone:** `1.0.0` (first parser release: `1.0.1`).
- **AST-dump schema version:** `1` — the integer consumers **pin** from `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" as a compile-time constant. It is **not** a field of `AstDumpPayload` (that struct exposes only `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`); re-validate the pin against the contract's "Schema Versioning" table when bumping PGEN.
- **Annotation count:** **64** across **27 distinct rules** (on top of the pre-typing baseline of one root annotation). All 64 are `annotation_type: "return_object"`. Coverage: the `systemverilog_preprocessor_file` root; the 10-kind `pp_item` dispatch; the 7 per-directive shapes (`define` / `undef` / `include` / `timescale` / `default_nettype` / `celldefine` / `endcelldefine`); `include_path` / `nettype_value` / `time_literal`; the 5-node conditional-compilation tree (`pp_conditional` / `pp_if_branch` / `pp_elsif_branch` / `pp_else_branch` / `pp_endif`); `condition_expr` / `condition_atom` (12 kinds); `macro_formals` / `macro_formal` / `macro_default_value` / `macro_default_atom` (8 kinds); `macro_body` / `macro_body_fragment` (9 kinds); and the passthrough lines (`pp_non_directive_line` / `pp_blank_line`).
- **Accept set:** unchanged — same accepted inputs as the pre-typing baseline; only the AST shape became typed.
- **Known defect shipped:** `SVPP-0001` — `pp_if_branch.keyword` emits `<invalid_sequence_access>` for `` `ifdef`` / `` `ifndef`` conditional input (see "Known defects" below). The `` `define`` / non-conditional surface is unaffected.
- **Contract section:** `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.0.1 / Contract 1.0.1 Highlights — SVPP-Slice-1".
- **Machine-checkable inventory:** `generated/systemverilog_preprocessor_return_annotations.json` (64 entries) and its content-identical embedded mirror `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` (the contract-embedded copy omits only the cosmetic `raw_text` field).
- **Per-rule shapes:** [Top-Level Rules](rules-top-level.md); the schema-`1` row in [Schema Versioning](schema-versioning.md).

### Schema 0.1.0 / release 1.0.0 — foundation baseline

The pre-typing baseline.

- **Schema-version milestone:** `0.1.0` (first parser release: `1.0.0`).
- **State:** `grammars/systemverilog_preprocessor.ebnf` un-annotated except for the `systemverilog_preprocessor_file -> {type, items}` root. The AST dump was the recursive-envelope shape across all other rules (see [AST Envelope Structure](ast-envelope.md)).
- **Contract section:** the `0.1.0` row of the "Schema Versioning" table in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`.

## Known defects

`SVPP-0001` — `pp_if_branch.keyword` `<invalid_sequence_access>` (status `Root Caused`; fix not yet landed). For `` `ifdef`` / `` `ifndef`` conditional input the `items[].body.if_branch.keyword` field is a malformed nested object containing three `"<invalid_sequence_access>"` strings instead of the keyword token — a bare positional `$1` bound to the inline `(kw_ifdef | kw_ifndef)` alternation group, the same emit-time defect class fixed for `rtl_const_expr` in RTL-CE-Slice-2. The guard macro is correct at the sibling `if_branch.macro`; the `` `define`` / non-conditional surface is unaffected. Recorded in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0001`) and `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Known Defects (release 1.0.1)"; shown honestly with a safe consumer workaround in the [Conditional Compilation](examples-conditional.md) worked example. The scheduled fix (lift `(kw_ifdef | kw_ifndef)` into a named rule per the RTL-CE-Slice-2 playbook) is a shape-affecting change: when it lands it will get its own contract Highlights section, a [Schema Versioning](schema-versioning.md) row, and a new entry above.

## Bug ledger status

`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` is the canonical per-bug tracker. It currently carries one sv_preprocessor row — `SVPP-0001` — surfaced internally during SVPP-MDBOOK worked-example authoring (no external downstream report) and tracked at status `Root Caused`. When a downstream sv_preprocessor bug is accepted, it gets its own ledger row recording the reproducer bundle, root cause, fix proof, and the parser release it was fixed in; this index will then point at the relevant contract Highlights section for any accompanying shape change. Reports follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## How to follow per-slice changes

Each shape-affecting slice after SVPP-Slice-1 (including the scheduled `SVPP-0001` fix) gets:

1. A grammar change in `grammars/systemverilog_preprocessor.ebnf` (the `-> ...` annotation or restructure).
2. A manifest update in `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` (and the regenerated `generated/systemverilog_preprocessor_return_annotations.json`).
3. A parser-release / contract-version bump and a Highlights section in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the milestone.
5. An entry in this changelog index summarizing the slice.
6. A regression-lock test pinning the new typed shape (`cargo test --lib --features generated_parsers systemverilog_preprocessor_ast_shape_contract`).

The live-book policy bundles all six in the same commit. Because SVPP-Slice-1 already typed the full directive grammar, the realistic future driver of new entries here is bug-ledger-driven shape fixes (most immediately the `SVPP-0001` correctness fix) and any targeted restructure (for example, flattening the `macro_formals` `{first, rest}` list or annotating the remaining un-annotated leaf/text rules), not a long rule-by-rule campaign. The sv_preprocessor parser covers only the preprocessor directive surface (`` `define`` / `` `undef`` / `` `include`` / `` `timescale`` / `` `default_nettype`` / `` `celldefine`` / `` `endcelldefine``, conditional compilation, macro formals/defaults/bodies, and passthrough lines); for the full SystemVerilog language grammar see the `systemverilog` family.
