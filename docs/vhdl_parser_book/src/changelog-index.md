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

The SystemVerilog parser's changelog index is long because its return-annotation campaign landed rule-by-rule across 115 slices, each bumping the schema version and getting its own row. **The VHDL grammar is different: it was typed in a single comprehensive batch — VHDL-Slice-1 — so the VHDL schema timeline is short.** A follow-up correctness fix (`1.0.2`, schema `2`, `VHDL-0001`) added the third entry, the `1.0.3` POST-SV-AUDIT Category-A list-shape batch (schema `3`) added the fourth, and the `1.0.4` `VHDL-0002` based-literal acceptance fix (schema stays `3`) added the fifth. This is the intended state, not an incomplete index. Subsequent shape-affecting slices, if any, will each add a contract Highlights section, a [Schema Versioning](schema-versioning.md) row, and an entry below.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. The entries below mirror the "Schema Versioning" table and the Resolved-Defects sections in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`; the contract is authoritative for the live state.

### 1.0.5 / Contract 1.0.5 — ENGINE-UNIVERSAL-SERVICES.43 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0081`, 2026-08-21), ledger `VHDL-0004` — the recursion ceiling now bounds TIME as well as stack (ENGINE; ZERO grammar bytes; schema stays 3)

⛔ **This release changes no language and no AST shape. It changes whether the parser RETURNS.**
Measured on the shipped release probe: **575** nested parens in `s <= (((…)))1(((…)));` were
accepted in 0.18 s, and **590 returned nothing in 30 s**. On the embedding path that is an
unbounded hang holding the caller's thread — worse than the process abort that embedding contract
`1.3.1` replaced, because a crash fails fast and loud and a hang does not.

⭐ **VHDL is where the defect was proven ENGINE-level rather than SystemVerilog-specific**, and that
is the reason this family carries a release for a fix it did not cause. The generated VHDL parser
was byte-identical across the concurrent SystemVerilog grammar campaign
(`generated_reproducibility_gate` tier 2, 11/11), and it showed the identical cliff — so a defect
present in an unchanged parser cannot have been introduced by a change to a different one. VHDL's
cliff sits later than SystemVerilog's (~575→590 rather than ~317→318) only because its expression
cascade is shallower per parenthesis: the same event at a different density.

**ROOT CAUSE (shared with `SV-0067`).** Every recursion-guard verdict travelled one field, a
parse-stack **frame index**, and a failure whose block came from a frame outside the memoized rule
is deliberately not cached. `Infinite` and `LeftRecursive` name a frame; the whole-stack **depth
ceiling** names none, so it passed `0` — outside every rule — which disabled FAILURE memoisation
for the entire remaining parse after a single trip. The fix routes the ceiling onto its own channel
and caches its failures under a **depth stamp**, sound because the ceiling is monotone in depth.

**MEASURED, after.** 575 parens unchanged at 0.16 s accepted; **590 → 0.28 s ACCEPTED**; 600 and
620 accepted; 700 and beyond rejected in bounded, input-length-independent time. ⭐ The accept set
did not narrow — 590–660 were never rejections, the runaway search simply never reached a verdict.
`parser_embedding_vhdl_deep_nesting_yields_clean_diagnostic_not_process_abort`, which previously
could not terminate, now passes and carries a wall-clock budget so a recurrence fails rather than
hangs. **No consumer migration**; AST-dump schema stays `3`. Full detail in ledger row `VHDL-0004`,
with the SystemVerilog twin at `SV-0067`.

### 1.0.4 / Contract 1.0.4 — VHDL-0002 based-literal acceptance fix (schema stays 3)

A targeted, bug-ledger-driven correctness fix landed 2026-06-10. The vhdl certificate-coverage
drive (`GRAMMAR-WELLFORMED.H.11`) surfaced that the parser **rejected every VHDL based literal**
— `2#1010#`, `16#FF#`, and all other `base#value#` forms — at every release since `1.0.0`, even
though the grammar models them (`based_literal := unsigned_number hash based_value hash`).

- **Root cause (engine, not grammar):** the generated parser's internal layout skipper hard-coded
  `#`-to-end-of-line comment skipping (an EBNF meta-grammar convention; VHDL comments are `--`),
  so the `hash := trivia /#/` token's own `#` was consumed as a "comment" before the regex could
  match it. The string-terminal side of the skipper already guarded against swallowing
  comment-introducer tokens; the regex side lacked the symmetric guard.
- **Fix (parser-agnostic):** the skipper's comment arms now stand down when the active token's own
  pattern matches at the comment introducer. (Mechanism note, 2026-06-11: a follow-up engine wave —
  the `SV-0001` fix in the bug ledger — superseded this dynamic guard with a stronger *static*
  rule: a comment arm is no longer emitted at all for any introducer the grammar assigns a
  non-comment meaning, so the regenerated vhdl parser simply carries no `#` arm. The vhdl accept
  set and AST output are unchanged by that supersession; release stays `1.0.4`.)
- **Accept set:** **widened only** — every input that parsed at `1.0.3` yields a byte-identical
  AST at `1.0.4`; previously-rejected based literals now parse, surfacing as
  `{"kind": "based", "body": {"base": …, "value": …}}` inside `literal` — a shape declared since
  schema `1`, reachable for the first time. Note the grammar accepts any `unsigned_number` as the
  base (e.g. `3374#BFCd#` parses); base-range (2..16) checking is a consumer/semantic concern.
- **AST-dump schema:** **unchanged at `3`** (no previously-observable output shape changed).
- **Annotation count:** **256 / 112 rules** (UNCHANGED — engine fix only; no grammar edit).
- **Bug ledger:** `VHDL-0002` in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
- **Lock:** `vhdl_based_literal_hash_token_is_not_swallowed_as_comment`
  (`rust/src/ast_shape_contract.rs`).

### 1.0.3 / Contract 1.0.3 — POST-SV-AUDIT Category-A list-shape batch (schema 2 → 3)

The POST-SV-AUDIT.2.3 static classification pass (`docs/POST_SV_AUDIT_LEDGER.md`, tracked `PGEN-POST-SV-AUDIT-0004`) found **17 static-conclusive Category-A raw-envelope list rules** in `grammars/vhdl.ebnf`. Landed 2026-05-17. Follows POST-SV-AUDIT.2.1 (`systemverilog_preprocessor` `macro_formals`, `PGEN-POST-SV-AUDIT-0002`, sv_preprocessor 1.0.3 / schema 3) and POST-SV-AUDIT.2.2 (`rtl_frontend` 15 Category-A + `RTL-FE-0002`, `PGEN-POST-SV-AUDIT-0003`, rtl_frontend 1.0.3 / schema 3).

- **Schema-version milestone:** `1.0.3` (first parser release: `1.0.3`).
- **AST-dump schema version:** `3` — the integer consumers **pin** from the contract (it is **not** a field of `AstDumpPayload`, whose real fields are `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`).
- **What changed:** 17 Category-A raw-envelope list rules — `library_clause`, `use_clause`, `selected_name`, `identifier_list`, `generic_interface_list`, `port_interface_list`, `parameter_list`, `enumeration_type_definition`, `index_constraint`, `association_list`, `sensitivity_list`, `actual_parameter_part`, `choices`, `aggregate_choice_list`, plus the `target` aggregate branch and both `aggregate` branches — no longer expose the raw `{first, rest}` (resp. `{…, first, rest}`) iteration envelope (`rest` was the raw `[[sep, item], …]` single-token-separator envelope a consumer had to walk past). Each is corrected to the canonical extraction-spread `[$F, $R::2*]` (drop the semantically-irrelevant separator — comma / semi / dot / bar; emit a clean flat list). The 14 bare-list rules now emit a **top-level array**; the `target` aggregate branch and the two `aggregate` branches keep their meaningful discriminator/value fields and carry the cleaned trailing list in `items` / `rest`. Every separator is a single token; **no** inline alternation and **no** `<invalid_sequence_access>` — this is a clean Category-A shape improvement, **not** a parser bug.
- **Annotation count:** **256** (UNCHANGED — the 14 bare-list rules flip `return_object` → `return_array`; the `target`-aggregate + `aggregate` rules stay `return_object` with a new `normalized_text`; no count delta). **112** distinct rules (UNCHANGED).
- **Accept set:** unchanged — no grammar acceptance change, purely the 17 annotation-form changes.
- **Contract section:** `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` → "AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT) — 17 Category-A raw-envelope list rules → clean lists; schema 2 → 3".
- **Bug ledger:** none. The 17 Category-A corrections are a clean shape improvement (single-token separators, no `<invalid_sequence_access>`) and are deliberately **not** bug-ledger rows — tracked via `docs/POST_SV_AUDIT_LEDGER.md` and the contract's "AST-Shape Corrections — 1.0.3" section. (`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` is reserved for the `<invalid_sequence_access>` corruption/crash class — `VHDL-0001` lives there; this batch adds nothing.)
- **Machine-checkable inventory:** `generated/vhdl_return_annotations.json` (256 entries) and its embedded mirror `rust/test_data/ast_shape_contract/vhdl_v1.json` (new `cat_a_shapes` regression sample).
- **Per-rule shapes:** [Top-Level Rules](rules-top-level.md).

### 1.0.2 / Contract 1.0.2 — VHDL-0001 correctness fix: simple_expression / term binop_chain.rest (schema 1 → 2)

A targeted, bug-ledger-driven correctness fix landed 2026-05-17. A worked-example pass surfaced that the `1.0.1` baseline shipped one systemic return-annotation defect (`VHDL-0001`) the root-keys-only shape-contract lock did not catch.

- **Schema-version milestone:** `1.0.2` (first parser release: `1.0.2`).
- **AST-dump schema version:** `2` — bumped `1 → 2` because `simple_expression` / `term` `binop_chain.rest` changed shape in a consumer-visible way. The integer consumers **pin** from the contract (it is **not** a field of `AstDumpPayload`, whose real fields are `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`).
- **What changed:** for any multi-operand additive (`simple_expression`) or multiplicative (`term`) expression, `binop_chain.rest` previously emitted the literal sentinel `"<invalid_sequence_access>"` (3× at the `additive` level) plus malformed nested objects. Root cause: the iteration-lead inline operator alternations `(plus | minus | ampersand)` / `(star | slash | kw_mod | kw_rem)` corrupted the positional model so the bare `rest: $N` mis-recursed (the systemic inline-alternation-`$N` defect class — same as `RTL-CE-0001` / `SVPP-0001` / `RTL-FE-0001`). **Fix:** the two inline alternations were lifted into the **named** rules `adding_operator := plus -> {kind: "plus"} | minus -> {kind: "minus"} | ampersand -> {kind: "concat"}` and `multiplying_operator := star -> {kind: "mul"} | slash -> {kind: "div"} | kw_mod -> {kind: "mod"} | kw_rem -> {kind: "rem"}` (matching vhdl's own `logical_operator` / `relational_operator` `{kind}` idiom). The `simple_expression` / `term` `binop_chain` annotations are unchanged; the leading `(plus | minus)?` `sign` (not an iteration lead) was empirically unaffected and left as-is. `rest` is now the clean `[ <op-envelope>, <operand> ]` array with a typed `{kind}` op-envelope at every level — no `<invalid_sequence_access>` anywhere. **`vhdl` was the final grammar in the systemic inline-alternation-`$N` class; the class is now fully resolved across `rtl_const_expr` / `systemverilog_preprocessor` / `rtl_frontend` / `vhdl`.**
- **Annotation count:** **256** (was `249`; +7 = the 3 new `adding_operator` + 4 new `multiplying_operator` `return_object` branches). **112** distinct rules (was `110`; +2).
- **Accept set:** unchanged — no grammar acceptance change, purely the two alternation lifts + their 7 branch annotations.
- **Contract section:** `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` → "Release 1.0.2 / Contract 1.0.2 Highlights — VHDL-0001 correctness fix" and "Resolved Defects — `VHDL-0001`".
- **Bug ledger:** `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` → `VHDL-0001` (status `Released`, fixed in parser release `1.0.2` / schema `2`).
- **Machine-checkable inventory:** `generated/vhdl_return_annotations.json` (256 entries) and its embedded mirror `rust/test_data/ast_shape_contract/vhdl_v1.json` (new `arithmetic_expr` regression sample).
- **Worked example:** [Binary Addition](examples-binary-addition.md) (the schema-`1`→`2` transition with the real captured `{kind}` op-envelope shape).

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

`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` is the canonical per-bug tracker. As of this writing it carries **one VHDL row — `VHDL-0001`** (status `Released`, fixed in parser release `1.0.2` / schema `2`): the systemic inline-alternation-`$N` `simple_expression` / `term` `binop_chain.rest` `<invalid_sequence_access>` defect, with its before/after `parseability_probe` reproducer, root cause, and fix proof. It points at the contract's "Release 1.0.2 / Contract 1.0.2 Highlights" section for the accompanying schema-`1`→`2` shape change. The 17 POST-SV-AUDIT Category-A list-shape corrections that landed in the `1.0.3` release are a clean shape improvement (single-token separators, no `<invalid_sequence_access>`) and are deliberately **not** bug-ledger rows — they are tracked via `docs/POST_SV_AUDIT_LEDGER.md` and the contract's "AST-Shape Corrections — 1.0.3" section, not the bug ledger. Future accepted VHDL bugs get their own `VHDL-NNNN` rows the same way.

## How to follow per-slice changes

Each shape-affecting slice after VHDL-Slice-1 gets:

1. A grammar change in `grammars/vhdl.ebnf` (the `-> ...` annotation or restructure).
2. A manifest update in `rust/test_data/ast_shape_contract/vhdl_v1.json` (and the regenerated `generated/vhdl_return_annotations.json`).
3. A parser-release / contract-version bump and a Highlights section in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the milestone.
5. An entry in this changelog index summarizing the slice.
6. A regression-lock test pinning the new typed shape.

The live-book policy bundles all six in the same commit. Because VHDL-Slice-1 already typed the full grammar, the realistic future driver of new entries here is bug-ledger-driven shape fixes and any targeted restructure (for example, a list-flattening slice), not a long rule-by-rule campaign.
