# docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `systemverilog_preprocessor` frontend/parsing stage.

## Contract Identity
- Contract version:
  - `1.0.4`
- Parser release version:
  - `1.0.4`
- systemverilog_preprocessor AST-dump schema version:
  - `3` (unchanged across `1.0.3`–`1.0.4`; the `1.0.4` `SVPP-0002` fix is a strictly-more-permissive correctness fix with no observable output-shape change — see Release 1.0.4 Highlights)
- Annotation count:
  - `66` (65 `return_object` + 1 `return_array`; 28 distinct rules) — **unchanged** by `1.0.4` (`SVPP-0002` touched only the un-annotated `macro_body_text`/`macro_default_text` regex rules)
- Last updated:
  - `2026-05-18`
- Current grammar family label:
  - `systemverilog_preprocessor`
- Per-family mdBook:
  - `docs/systemverilog_preprocessor_parser_book/` (tracked HTML at `docs/systemverilog_preprocessor_parser_book-html/`)
- Per-family gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate`
- Per-family ast-shape-contract manifest:
  - `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`

## Schema Versioning

The systemverilog_preprocessor parser carries two version axes:

1. **Parser release version** (`1.0.4`). Tracks the parser library's release identity.
2. **AST-dump schema version** (`3`). Tracks the AST output shape. Schema `3` spans releases `1.0.3`–`1.0.4`: the `1.0.4` `SVPP-0002` correctness fix changed no observable output shape (a strictly-more-permissive fix — see the schema-`3` row's `1.0.4` addendum and Release 1.0.4 Highlights).

| Schema version | First parser release | Notable changes |
|---|---|---|
| 3 | 1.0.3 (also spans 1.0.4) | **`1.0.4` addendum — `SVPP-0002` macro-comment correctness fix (schema-neutral, non-breaking).** `macro_default_text` / `macro_body_text` were `:= inline_trivia /[^`(),?:\r\n]+/` — the content regex was **not comment-aware**, greedily eating a `/*` then halting at a backtick *inside* the `block_comment`, so valid SystemVerilog with a backtick inside a comment in a macro body / function-macro default (e.g. `` `define X a /*`*/ ``) was wrongly rejected at `1.0.3`. Fixed in `1.0.4` by making both rules comment-aware (`/(?:\/\*([^*]\|\*+[^*\/])*\*+\/\|[^`(),?:\r\n])+/` — the proven `systemverilog.ebnf` `timeunit_separator_trivia`/`block_comment` idiom): a `/* … */` comment is matched atomically (its internal backtick no longer splits the run); the unchanged `[^`(),?:\r\n]` branch still excludes a bare backtick. **No schema bump**: the rules are un-annotated, the annotation inventory is unchanged (66/28), and every input that parsed at `1.0.3` yields a byte-identical AST at `1.0.4` — only previously-*erroring* inputs now succeed with the standard `{kind:"text", body:$1}` shape (strictly more permissive). Tracked `SVPP-0002` (`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`); locked by the `macro_body_comment_backtick` sample in `systemverilog_preprocessor_v1.json`. **`1.0.3` (original schema-`3` change — `macro_formals` Category-A AST-shape correction, POST-SV-AUDIT, consumer-visible):** `macro_formals` no longer exposes the raw `{first, rest}` iteration envelope — the POST-SV-AUDIT.2.1 audit (`PGEN-POST-SV-AUDIT-0002`) found `macro_formals := lparen macro_formal (comma macro_formal)* rparen -> {first: $2, rest: $3}` was a static-conclusive Category-A raw-envelope misuse: `rest` surfaced the raw `[[comma, macro_formal], …]` separator envelope, forcing every consumer to walk past the `comma` separator. Corrected to the canonical extraction-spread `macro_formals := lparen macro_formal (comma macro_formal)* rparen -> [$2, $3::2*]` (drop the semantically-irrelevant `comma`; emit a clean flat `macro_formal` list — the `object_properties` reference idiom). For input `` `define M(a, b, c) a+b+c `` `pp_define.formals` was the raw `{"first": {"default": [], "name": [[], "a"]}, "rest": [[[[], ","], {"default": [], "name": [[" "], "b"]}], [[[], ","], {"default": [], "name": [[" "], "c"]}]]}` envelope; it is now the clean list `[{"default": [], "name": [[], "a"]}, {"default": [], "name": [[" "], "b"]}, {"default": [], "name": [[" "], "c"]}]` of `macro_formal` `{name, default}` objects. No `<invalid_sequence_access>` (this is a clean Category-A shape improvement, **not** the inline-alternation-`$N` corruption class of `SVPP-0001`). Surface counts **unchanged** (66 annotations / 28 distinct rules): `macro_formals` is still one rule / one annotation — only its `annotation_type` changed `return_object` → `return_array` and `normalized_text` `{first: $2, rest: $3}` → `[$2, $3::2*]`, so the surface is now **65 `return_object` + 1 `return_array`** (was all 66 `return_object`). Same accept set (no grammar acceptance change — only the annotation form). Gate-locked. |
| 2 | 1.0.2 | **SVPP-0001 correctness fix (breaking).** `pp_if_branch.keyword` no longer emits the malformed `"<invalid_sequence_access>"` object for `` `ifdef`` / `` `ifndef`` conditional input. The inline alternation `(kw_ifdef \| kw_ifndef)` that was the lead element of `pp_if_branch` (and corrupted the positional model so the bare `keyword: $1` mis-recursed) is lifted into a **named** rule `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} \| kw_ifndef -> {kind: "ifndef"}`, mirroring the proven `systemverilog.ebnf` op-chain / `rtl_const_expr` RTL-CE-Slice-2 idiom. `pp_if_branch`'s annotation is **unchanged** (`{keyword: $1, macro: $2, tail: $3, items: $5}`); only `$1` now binds the clean named rule, so `if_branch.keyword` is now `{kind: "ifdef"}` (or `{kind: "ifndef"}`) — a real typed polarity discriminator. Annotation count `64 → 66` (the 2 new `pp_if_keyword` `return_object` branches); distinct rules `27 → 28` (the new `pp_if_keyword`). All annotations remain `return_object`. Same accept set (no grammar acceptance change — purely the alternation lift + its 2 branch annotations). Gate-locked. |
| 1.0.0 | 1.0.1 | **SVPP-Slice-1** — initial 64-annotation baseline. pp_item dispatch (10 kinds), 7 directive shapes (define/undef/include/timescale/default_nettype/celldefine/endcelldefine), include_path/nettype_value/time_literal, conditional-compilation tree (5 nodes), condition_expr/condition_atom (12 kinds), macro_formals/formal/default_value/default_atom (8 kinds) / body/body_fragment (9 kinds), passthrough lines. **NOTE:** the `pp_if_branch.keyword` shape in this baseline was defective (`SVPP-0001`, the inline-alternation-`$N` `"<invalid_sequence_access>"` malformation) — see schema `2` for the correction. |
| 0.1.0 | 1.0.0 | Foundation baseline. Grammar (`grammars/systemverilog_preprocessor.ebnf`) with the `systemverilog_preprocessor_file -> {type, items}` root only. AST dump is the recursive-envelope shape across all other rules. |

## Resolved Defects — `SVPP-0001` (fixed in release 1.0.2, schema 2)

- **`SVPP-0001` — `pp_if_branch.keyword` `<invalid_sequence_access>`
  (`Released`, fixed in parser release `1.0.2` / schema `2`).**
  *Historical (release `1.0.1`, schema `1`):* for `` `ifdef`` /
  `` `ifndef`` conditional input, `items[].body.if_branch.keyword`
  surfaced a malformed nested object containing three
  `"<invalid_sequence_access>"` strings instead of the keyword token.
  Root cause: `pp_if_branch := (kw_ifdef | kw_ifndef) macro_name … ->
  {keyword: $1, …}` bound `$1` to an **inline alternation group**, the
  same emit-time defect class fixed for `rtl_const_expr` in
  RTL-CE-Slice-2 (and tracked for `rtl_frontend` / `vhdl`
  `binop_chain`). The `` `define`` / non-conditional surface was
  unaffected. **Fix (proven RTL-CE-Slice-2 / `systemverilog.ebnf`
  idiom):** the inline alternation is lifted into a **named** rule
  `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} | kw_ifndef -> {kind:
  "ifndef"}`; `pp_if_branch`'s annotation is unchanged
  (`{keyword: $1, …}`) but `$1` now binds the clean named rule. The
  parser is regenerated and the corrected shape is machine-locked. For
  input `` `ifdef X\n`define A 1\n`else\n`define B 2\n`endif\n`` the
  fixed `if_branch.keyword` is now `{kind: "ifdef"}` (would be
  `{kind: "ifndef"}` for an `` `ifndef`` guard) — a real typed polarity
  discriminator with **no** `<invalid_sequence_access>` anywhere.
  Consumers read the conditional polarity directly from
  `if_branch.keyword.kind`; the guard macro stays at the outer
  `if_branch.macro`. The honest pre-fix history is kept in the
  [Schema Versioning](#schema-versioning) table (schema `1.0.0` row) and
  the conditional worked example's schema-`2` transition note; tracked
  (status `Released`) in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
  (`SVPP-0001`). Documented in
  [the conditional worked example](../systemverilog_preprocessor_parser_book/src/examples-conditional.md).

## Resolved Defects — `SVPP-0002` (fixed in release 1.0.4, schema unchanged 3)

- **`SVPP-0002` — macro body / default-value content rules were not
  comment-aware; valid SystemVerilog with a backtick inside a block
  comment in a macro body or function-macro default was wrongly
  rejected (`Released`, fixed in parser release `1.0.4`, AST-dump
  schema **unchanged** `3`).**
  *Historical (releases `1.0.1`–`1.0.3`):* `macro_default_text` and
  `macro_body_text` were `:= inline_trivia /[^`(),?:\r\n]+/`. That
  content regex excludes a backtick **and is not comment-aware**, so
  it greedily consumed a comment's opening `/*` then halted at a
  backtick *inside* the `block_comment`, splitting it; nothing could
  resume at the dangling `` `*/ ``, so `macro_body+` /
  `macro_default_value+` ended short and `pp_define` could not reach
  `newline`. Repro (against `1.0.3`):
  `printf '`define X a /*`*/\n' | parseability_probe --parse
  systemverilog_preprocessor /dev/stdin` → "Parser did not consume
  full input". The byte-identical input without the backtick parsed;
  the defect did not reproduce outside the macro body/default region.
  A comment is lexically transparent, so this was **valid SV wrongly
  rejected**. Pre-existing (the rules predate `SVPP-0001` /
  POST-SV-AUDIT.2.1; not campaign-caused — proven generatively inert
  by `PGEN-SV-EXH-PROOF-0005`/`-0006`); surfaced by the SV-EXH-PROOF.2.3
  preprocessor closed-loop.
  *Fixed (`1.0.4`):* both rules made comment-aware —
  `/(?:\/\*([^*]|\*+[^*\/])*\*+\/|[^`(),?:\r\n])+/` (the proven
  `systemverilog.ebnf` `timeunit_separator_trivia` / `block_comment`
  idiom, no lookahead). A `/* … */` comment is matched **atomically**
  by the leading alternative; the unchanged `[^`(),?:\r\n]` branch
  still excludes a bare backtick (so `macro_token_paste` /
  `macro_stringize` / `macro_reference` still split real
  `` ` ``-tokens). Accepts strictly more, narrows nothing; the rules
  are un-annotated so the annotation inventory is unchanged (`66`/`28`)
  and every previously-parseable input yields a byte-identical AST —
  hence **no schema bump** (schema `3` spans `1.0.3`–`1.0.4`). Tracked
  (status `Released`) in
  `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0002`);
  locked by the `macro_body_comment_backtick` sample in
  `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`
  and `make -C rust SHELL=/opt/homebrew/bin/bash
  systemverilog_preprocessor_parser_book_gate`.

## AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT) — `macro_formals` Category-A raw-envelope → clean list; schema 2 → 3

Landed 2026-05-17. The POST-SV-AUDIT static classification pass
(`docs/POST_SV_AUDIT_LEDGER.md`, leaf POST-SV-AUDIT.2.1, tracked
`PGEN-POST-SV-AUDIT-0002`) found one static-conclusive Category-A
raw-envelope misuse in `grammars/systemverilog_preprocessor.ebnf`. It
is corrected, the parser is regenerated, and the manifest inventory is
re-locked. This is **not** a released-parser bug (no
`<invalid_sequence_access>`, no crash) — it is a deliberate
audit-driven AST-shape correction, so it carries a schema bump but is
**not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
(that ledger is reserved for the `<invalid_sequence_access>`
corruption/crash class — `SVPP-0001` et al.).

- **`macro_formals` raw `{first, rest}` envelope (Category-A,
  `PGEN-POST-SV-AUDIT-0002`).** For a parameterised `` `define``,
  `pp_define.formals` is the typed `macro_formals` object. The
  `1.0.1`/`1.0.2` grammar was

  ```ebnf
  macro_formals := lparen macro_formal (comma macro_formal)* rparen
                -> {first: $2, rest: $3}
  ```

  which bound `rest` to the **raw iteration envelope**
  `[[comma, macro_formal], …]` — every consumer had to index past the
  `comma` separator on each iteration to reach the next `macro_formal`.
  This is a static-conclusive Category-A pure-single-token-separator
  list misuse (the `comma` carries no payload a consumer needs). **Fix
  (the canonical `object_properties` extraction-spread idiom):** drop
  the separator and emit a clean flat list:

  ```ebnf
  macro_formals := lparen macro_formal (comma macro_formal)* rparen
                -> [$2, $3::2*]
  ```

  `macro_formals` is still one rule with one annotation — only its
  annotation form changed (`annotation_type` `return_object` →
  `return_array`; `normalized_text` `{first: $2, rest: $3}` →
  `[$2, $3::2*]`). Real captured before→after for input
  `` `define M(a, b, c) a+b+c `` (verified via `parseability_probe`), at
  `pp_define.formals`:

  - **Before (≤ schema 2 / release 1.0.2):**
    `{"first": {"default": [], "name": [[], "a"]}, "rest": [[[[], ","], {"default": [], "name": [[" "], "b"]}], [[[], ","], {"default": [], "name": [[" "], "c"]}]]}`
  - **After (schema 3 / release 1.0.3):**
    `[{"default": [], "name": [[], "a"]}, {"default": [], "name": [[" "], "b"]}, {"default": [], "name": [[" "], "c"]}]`
    — a clean flat list of `macro_formal` `{name, default}` objects, in
    source order. No `<invalid_sequence_access>` anywhere (this is a
    clean Category-A shape improvement, **not** the inline-alternation
    corruption class of `SVPP-0001`).

  Consumers now iterate `pp_define.formals` directly as the
  `macro_formal` array — no `.first` / `.rest` split, no separator to
  skip. A consumer written against `1.0.2` that walked
  `formals.first` + `formals.rest[][1]` must repin to schema `3` and
  treat `formals` as a flat `macro_formal[]`.

Annotation count: **66** (unchanged — `macro_formals` was, and remains,
one rule / one annotation; no count delta). **28** distinct rules
(unchanged). The annotation-type mix is now **65 `return_object` + 1
`return_array`** (the single `return_array` is `macro_formals`; it was
the only `return_object` → `return_array` change). Same accept set (no
grammar acceptance change — purely the annotation-form change). Schema
bumped `2 → 3` because `pp_define.formals` (`macro_formals`) changed
shape in a consumer-visible way. Gate-locked:
`cargo test --lib --features generated_parsers systemverilog_preprocessor_ast_shape_contract`
and
`make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate`.

> **Audit-campaign note:** POST-SV-AUDIT.2.1 is the first Category-A
> fix of the POST-SV-AUDIT.2 worklist
> (`docs/POST_SV_AUDIT_LEDGER.md`). The remaining Category-A
> raw-envelope list rules in `rtl_frontend.ebnf`, `vhdl.ebnf`, and
> `systemverilog.ebnf` are tracked separately as their own
> POST-SV-AUDIT.2.x slices; this release corrects
> systemverilog_preprocessor only. Distinct from the `SVPP-0001`
> inline-alternation-`$N` corruption class — that was a released-parser
> bug (`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`); this
> Category-A correction is not.

## Release 1.0.2 / Contract 1.0.2 Highlights — SVPP-0001 correctness fix (pp_if_branch.keyword); schema 1 → 2

Landed 2026-05-16. A worked-example pass surfaced that the `1.0.1`
baseline shipped one return-annotation defect (`SVPP-0001`) that the
(root-keys-only) shape-contract regression lock did not catch. It is
fixed, the parser is regenerated, and the manifest inventory is
tightened to the full 66-entry surface so the corrected shape is now
machine-locked.

- **`pp_if_branch.keyword` `<invalid_sequence_access>` (SVPP-0001).**
  For any `` `ifdef`` / `` `ifndef`` conditional input,
  `items[].body.if_branch.keyword` emitted a malformed nested object
  containing three `"<invalid_sequence_access>"` strings instead of the
  keyword token. Root cause:
  `pp_if_branch := (kw_ifdef | kw_ifndef) macro_name directive_tail?
  newline pp_item* -> {keyword: $1, macro: $2, tail: $3, items: $5}`
  bound `$1` to an **inline alternation group**
  `(kw_ifdef | kw_ifndef)`, which corrupts the positional model so the
  rule's own annotation is mis-recursed. **Fix (proven RTL-CE-Slice-2 /
  `systemverilog.ebnf` op-chain idiom):** the inline alternation is
  lifted into a **named** rule:

  ```ebnf
  pp_if_keyword := kw_ifdef  -> {kind: "ifdef"}
                 | kw_ifndef -> {kind: "ifndef"}
  pp_if_branch  := pp_if_keyword macro_name directive_tail? newline pp_item*
                -> {keyword: $1, macro: $2, tail: $3, items: $5}
  ```

  `pp_if_branch`'s annotation is **unchanged** — only `$1` now binds
  the clean named `pp_if_keyword` rule instead of the inline group. The
  fixed shape for input
  `` `ifdef X\n`define A 1\n`else\n`define B 2\n`endif\n`` is
  `pp_conditional` = `{if_branch, elsif_branches, else_branch}`;
  `if_branch` = `{keyword: {kind: "ifdef"}, macro: [[" "], "X"],
  tail: [], items: [{kind: "define", …}]}`. `if_branch.keyword` is now
  the typed polarity object `{kind: "ifdef"}` (or `{kind: "ifndef"}`) —
  **no** `<invalid_sequence_access>` anywhere. Consumers read the
  `` `ifdef`` vs `` `ifndef`` polarity directly from
  `if_branch.keyword.kind`; the guard macro stays at the outer
  `if_branch.macro`.

Annotation count: **66** (was 64; +2 = the new `pp_if_keyword`
`return_object` branches `{kind: "ifdef"}` / `{kind: "ifndef"}`). **28**
distinct rules (was 27; +1 = `pp_if_keyword`). All 66 remain
`annotation_type: "return_object"`. Same accept set (no grammar
acceptance change — purely the alternation lift + its 2 branch
annotations). Schema bumped `1 → 2` because `pp_if_branch.keyword`
changed shape in a consumer-visible way (was the malformed
`<invalid_sequence_access>` object, now the typed `{kind}` polarity
discriminator). Gate-locked:
`cargo test --lib --features generated_parsers systemverilog_preprocessor_ast_shape_contract`
and
`make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate`.

> **Systemic note:** the inline-operator-alternation antipattern that
> caused `SVPP-0001` also exists in `grammars/rtl_frontend.ebnf` and
> `grammars/vhdl.ebnf` `binop_chain` levels (same
> `<invalid_sequence_access>` empirically confirmed for rtl_frontend
> `a + b`), and was the same root cause fixed for `rtl_const_expr` in
> RTL-CE-Slice-2 (`docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`
> Release 1.0.2 systemic note). Those families' corrections are tracked
> separately as their own slices + bug-ledger entries; this release
> fixes systemverilog_preprocessor only.

## Release 1.0.1 / Contract 1.0.1 Highlights — SVPP-Slice-1: full grammar typed (40+ rules / 63 annotations)

Single comprehensive slice landed on 2026-05-14 covering the entire grammar surface:

```ebnf
# File root (pre-existing)
systemverilog_preprocessor_file  -> {type: "systemverilog_preprocessor_file", items}

# Dispatch wrapper (10 kinds)
pp_item                          -> {kind: "define" | "undef" | "include" | "timescale"
                                          | "default_nettype" | "celldefine" | "endcelldefine"
                                          | "conditional" | "non_directive_line" | "blank_line",
                                     body?}

# Per-directive shapes (7)
pp_define                        -> {name, formals, body}
pp_undef                         -> {name, comment}
pp_include                       -> {path, comment}
pp_timescale                     -> {unit, precision, comment}
pp_default_nettype               -> {nettype, comment}
pp_celldefine                    -> {comment}
pp_endcelldefine                 -> {comment}

# Include path + nettype (2 kinds each)
include_path                     -> {kind: "quoted"|"angle", text}
nettype_value                    -> {kind: "identifier"|"none", body?}
time_literal                     -> {value, unit}

# Conditional compilation (5 nodes)
pp_conditional                   -> {if_branch, elsif_branches, else_branch}
pp_if_branch                     -> {keyword, macro, tail, items}
pp_elsif_branch                  -> {condition, items}
pp_else_branch                   -> {tail, items}
pp_endif                         -> {tail}

# Condition expression (12-kind atom)
condition_expr                   -> {atoms}
condition_atom                   -> {kind: "token_paste"|"stringize"|"macro_reference"|"text"
                                          |"lparen"|"rparen"|"comma"|"question"|"colon"
                                          |"logical_or"|"logical_and"|"bang", body?}

# Macro formals + default values (8-kind atom)
macro_formals                    -> {first, rest}   # 1.0.1/1.0.2 shape; corrected to [$2, $3::2*] (clean macro_formal list) in 1.0.3 / schema 3 — see "AST-Shape Corrections — 1.0.3"
macro_formal                     -> {name, default}
macro_default_value              -> {atoms}
macro_default_atom               -> {kind: "token_paste"|"stringize"|"macro_reference"|"text"
                                          |"lparen"|"rparen"|"question"|"colon", body?}

# Macro body fragment (9 kinds)
macro_body                       -> {fragments}
macro_body_fragment              -> {kind: "token_paste"|"stringize"|"macro_reference"|"text"
                                          |"lparen"|"rparen"|"comma"|"question"|"colon", body?}

# Passthrough lines
pp_non_directive_line            -> {text}
pp_blank_line                    -> {kind: "blank"}
```

Annotation count: **64** (was 1 / foundation baseline). Same accept set.

## AST Envelope and pp_item Dispatch

This section is the consumer-facing dispatch contract: how a downstream
integrator goes from the host AST-dump call to a typed
systemverilog_preprocessor tree, and how to branch on the top-level
discriminators. Every shape below is transcribed from the live inventory
`generated/systemverilog_preprocessor_return_annotations.json`
(`version: 1`, `grammar: "systemverilog_preprocessor"`,
`annotation_count: 66`, **28 distinct rules**), cross-checked against the
embedded copy in
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`
(content-identical on the `(rule, branch_index, annotation_type,
normalized_text)` tuples; the embedded copy omits only the diagnostic
`raw_text` field), and is consistent with the curated per-rule reference
at `docs/systemverilog_preprocessor_parser_book/src/rules-top-level.md`.

### The `AstDumpPayload` envelope

The AST-dump host entry points (the generic
`parse_grammar_profile_ast_dump*` family and the named-result form
`parse_grammar_profile_ast_dump_named`, used with grammar family
`systemverilog_preprocessor` / profile `default`) return — on success —
an `AstDumpPayload` (defined in `rust/src/embedding_api.rs`, contract in
`rust/docs/EMBEDDING_API_CONTRACT.md`). It is a canonical-JSON payload
string plus truncation metadata, with exactly four fields:

| Field | Type | Meaning |
|---|---|---|
| `dump_json` | string | The canonical (key-sorted) JSON encoding of the typed systemverilog_preprocessor AST. Parse this string to obtain the `systemverilog_preprocessor_file` root object described below. |
| `truncated` | bool | `false` for a complete dump; `true` when `max_ast_bytes` was exceeded and `dump_json` instead carries the truncation diagnostic envelope. |
| `full_bytes` | int | Byte length of the full encoded AST payload (before any truncation). |
| `emitted_bytes` | int | Byte length actually placed in `dump_json`. Equals `full_bytes` when not truncated. |

When `truncated` is `true`, `dump_json` is replaced by a deterministic
truncation diagnostic envelope (not the AST). That envelope carries
`pgen_dump_contract_version` (currently `1`), `kind:
"pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
"parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`. Consumers
must check `truncated` (or, equivalently, the presence of
`pgen_dump_contract_version` / `kind == "pgen_ast_dump_truncation"` in
the parsed `dump_json`) before treating `dump_json` as a
systemverilog_preprocessor AST. If `max_ast_bytes` is too small to fit
even the diagnostic envelope, the API returns `E_INVALID_LIMITS`
instead.

> Accuracy note: the live `AstDumpPayload` struct exposes precisely
> `dump_json` / `truncated` / `full_bytes` / `emitted_bytes`. The
> `pgen_dump_contract_version` / `schema_version` / `grammar` / `profile` /
> `root` keys are **not** members of `AstDumpPayload` itself —
> `pgen_dump_contract_version` appears only inside the truncation
> diagnostic envelope, the schema axis is the **AST-dump schema version
> `3`** tracked in [Schema Versioning](#schema-versioning), the grammar
> family is the fixed `systemverilog_preprocessor` label, and the profile
> is the fixed `default` profile (see [Stable Integration
> Surface](#stable-integration-surface)). The "root" is the parsed
> `systemverilog_preprocessor_file` object documented next. This contract
> documents the surface as it exists in `rust/src/embedding_api.rs`, not
> an idealized envelope. Where this prose and the inventory disagree, the
> inventory wins.

### The `systemverilog_preprocessor_file` root

The parsed `dump_json` is, for a successful systemverilog_preprocessor
parse, a single typed root object. Per
`grammars/systemverilog_preprocessor.ebnf` (lines 15–16):

```ebnf
systemverilog_preprocessor_file := pp_item*
-> {type: "systemverilog_preprocessor_file", items: $1}
```

```json
{
  "type": "systemverilog_preprocessor_file",
  "items": [ /* array of pp_item shapes, source order */ ]
}
```

Consumers dispatch on `obj["type"] == "systemverilog_preprocessor_file"`
at the root, then iterate `obj["items"]` — each element is one typed
`pp_item` object in source order. This is the only rule that carries a
`type` discriminator at the dispatch level; every other dispatcher uses
`kind`.

### The 10-branch `pp_item` dispatch

`pp_item` is the primary top-level dispatcher. It is a 10-branch
`kind`-tagged shape (`grammars/systemverilog_preprocessor.ebnf` lines
18–27). Consumers dispatch on `obj["kind"]`; every branch except the
three bodyless ones (`"celldefine"`, `"endcelldefine"`, `"blank_line"`)
carries a `body` holding the underlying typed shape:

```ebnf
pp_item := pp_define              -> {kind: "define",              body: $1}
         | pp_undef               -> {kind: "undef",               body: $1}
         | pp_include             -> {kind: "include",             body: $1}
         | pp_timescale           -> {kind: "timescale",           body: $1}
         | pp_default_nettype     -> {kind: "default_nettype",     body: $1}
         | pp_celldefine          -> {kind: "celldefine"}
         | pp_endcelldefine       -> {kind: "endcelldefine"}
         | pp_conditional         -> {kind: "conditional",         body: $1}
         | pp_non_directive_line  -> {kind: "non_directive_line",  body: $1}
         | pp_blank_line          -> {kind: "blank_line"}
```

| Branch | `kind` | `body` shape (fields) | Underlying rule (`grammars/systemverilog_preprocessor.ebnf`) |
|---|---|---|---|
| 0 | `"define"` | `{name, formals, body}` — `formals` is `[]` when there is no `(...)` formal list; `body` is `[]` for a bodyless `` `define`` | `pp_define` (line 33) |
| 1 | `"undef"` | `{name, comment}` | `pp_undef` (line 35) |
| 2 | `"include"` | `{path, comment}` — `path` is the typed `include_path` | `pp_include` (line 37) |
| 3 | `"timescale"` | `{unit, precision, comment}` — `unit` / `precision` are typed `time_literal` | `pp_timescale` (line 39) |
| 4 | `"default_nettype"` | `{nettype, comment}` — `nettype` is the typed `nettype_value` | `pp_default_nettype` (line 41) |
| 5 | `"celldefine"` | _(no `body` — bare `{kind: "celldefine"}`)_ | `pp_celldefine` (line 43) |
| 6 | `"endcelldefine"` | _(no `body` — bare `{kind: "endcelldefine"}`)_ | `pp_endcelldefine` (line 45) |
| 7 | `"conditional"` | `{if_branch, elsif_branches, else_branch}` — the conditional-compilation tree | `pp_conditional` (line 61) |
| 8 | `"non_directive_line"` | `{text}` — a passthrough source line | `pp_non_directive_line` (line 133) |
| 9 | `"blank_line"` | _(no `body` — bare `{kind: "blank_line"}`)_ | `pp_blank_line` (line 135) |

The inventory confirms exactly these **10** `pp_item` branches (one
`return_object` annotation per branch, branch indices 0–9); there is no
other `pp_item` `kind`. The three bodyless kinds are `"celldefine"`,
`"endcelldefine"`, and `"blank_line"` (their `pp_celldefine` /
`pp_endcelldefine` rules are still typed — they emit `{comment}` — but
`pp_item` discards the `body` for those two and for the
zero-information `pp_blank_line`).

### Per-directive shapes

Below the `pp_item` dispatch the seven non-conditional directive rules
emit named-field objects. The inventory confirms exactly **7** directive
shapes (`pp_define`, `pp_undef`, `pp_include`, `pp_timescale`,
`pp_default_nettype`, `pp_celldefine`, `pp_endcelldefine`), each a
single-branch `return_object`:

```ebnf
pp_define          := kw_define macro_name macro_formals? macro_body? newline?
                   -> {name: $2, formals: $3, body: $4}
pp_undef           := kw_undef macro_name directive_comment_tail newline?
                   -> {name: $2, comment: $3}
pp_include         := kw_include include_path directive_comment_tail newline?
                   -> {path: $2, comment: $3}
pp_timescale       := kw_timescale time_literal slash time_literal directive_comment_tail newline?
                   -> {unit: $2, precision: $4, comment: $5}
pp_default_nettype := kw_default_nettype nettype_value directive_comment_tail newline?
                   -> {nettype: $2, comment: $3}
pp_celldefine      := kw_celldefine directive_comment_tail newline?
                   -> {comment: $2}
pp_endcelldefine   := kw_endcelldefine directive_comment_tail newline?
                   -> {comment: $2}
```

| Rule (`grammars/systemverilog_preprocessor.ebnf`) | Shape | Notes |
|---|---|---|
| `pp_define` (line 33) | `{name, formals, body}` | `name` is the un-annotated `macro_name`/`identifier` envelope; `formals` is `[]` when no `(...)` formal list, else the clean `macro_formal[]` list (the `1.0.3` / schema `3` POST-SV-AUDIT Category-A `[$2, $3::2*]` extraction-spread — was the raw `{first, rest}` envelope at ≤ `1.0.2` / schema `2`; see [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--macro_formals-category-a-raw-envelope--clean-list-schema-2--3)); `body` is `[]` for a bodyless macro, else the typed `macro_body` `{fragments}` object. |
| `pp_undef` (line 35) | `{name, comment}` | `comment` is the `directive_comment_tail` envelope (`[]` when no trailing comment). |
| `pp_include` (line 37) | `{path, comment}` | `path` is the typed `include_path` (`{kind: "quoted"\|"angle", text}`). |
| `pp_timescale` (line 39) | `{unit, precision, comment}` | `unit` / `precision` are typed `time_literal` (`{value, unit}`). |
| `pp_default_nettype` (line 41) | `{nettype, comment}` | `nettype` is the typed `nettype_value` (`{kind: "identifier", body}` / `{kind: "none"}`). |
| `pp_celldefine` (line 43) | `{comment}` | The `pp_item` `"celldefine"` branch discards this `body`. |
| `pp_endcelldefine` (line 45) | `{comment}` | The `pp_item` `"endcelldefine"` branch discards this `body`. |

The supporting leaf shapes are `include_path` (2 kinds:
`{kind: "quoted", text}` / `{kind: "angle", text}`, lines 48–49),
`nettype_value` (2 kinds: `{kind: "identifier", body}` /
`{kind: "none"}`, lines 51–52), and `time_literal`
(`{value, unit}`, lines 54–55).

The conditional-compilation tree (`pp_conditional` →
`{if_branch, elsif_branches, else_branch}`, line 61) descends into
`pp_if_branch` (`{keyword, macro, tail, items}`, line 64),
`pp_elsif_branch` (`{condition, items}`, line 66), `pp_else_branch`
(`{tail, items}`, line 68), and `pp_endif` (`{tail}`, line 70); each
branch's `items` is a nested `pp_item*` array. The condition / macro
formal / macro body atom dispatchers (`condition_expr` /
`condition_atom`, `macro_formals` (the clean `macro_formal[]` list as of
`1.0.3` / schema `3`) / `macro_formal` / `macro_default_value` /
`macro_default_atom`, `macro_body` / `macro_body_fragment`) and the
passthrough lines (`pp_non_directive_line` → `{text}`, `pp_blank_line`
→ `{kind: "blank"}`) round out the 28-distinct-rule typed surface; their
full per-branch field lists are in
`docs/systemverilog_preprocessor_parser_book/src/rules-top-level.md`.

### Verified surface totals

The full typed surface of contract `1.0.3` is **66 return annotations
across 28 distinct rules** (**65 `annotation_type: "return_object"` + 1
`annotation_type: "return_array"`** — the single `return_array` is
`macro_formals`, the `1.0.3` POST-SV-AUDIT Category-A correction;
historically all 66 were `return_object` through `1.0.2`/schema `2`),
AST-dump schema version `3`, parser release `1.0.3`. These exact numbers
are transcribed from
`generated/systemverilog_preprocessor_return_annotations.json`
(`version: 1`, `grammar: "systemverilog_preprocessor"`,
`annotation_count: 66`; 28 distinct `rule` values) and its embedded copy
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`.
(The inventory-file `version: 1` is the inventory format version,
distinct from the AST-dump schema version `3` and the parser release
version `1.0.3`.) The `1.0.2` `SVPP-0001` fix added the new
`pp_if_keyword` rule (2 `return_object` branches, `{kind: "ifdef"}` /
`{kind: "ifndef"}`), taking the count `64 → 66` and distinct rules
`27 → 28`; the `1.0.3` POST-SV-AUDIT `macro_formals` correction did
**not** change the count (it is still one rule / one annotation) —
only its `annotation_type` `return_object` → `return_array` and
`normalized_text` `{first: $2, rest: $3}` → `[$2, $3::2*]`. The machine-checkable enumeration of every
`(rule, branch_index, annotation_type, normalized_text)` tuple is those
two artifacts; this contract section is curated; if this section and
either artifact disagree, the artifact wins, and this integration
contract wins over the per-family mdBook.

### Resolved defect — `SVPP-0001` (fixed in 1.0.2)

The released `1.0.1` parser shipped one accepted shape defect,
`SVPP-0001`, **fixed in parser release `1.0.2` / schema `2`** and
tracked (status `Released`) in
`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0001`) and at
the head of this contract under
[Resolved Defects — `SVPP-0001`](#resolved-defects--svpp-0001-fixed-in-release-102-schema-2).
*Historical (release `1.0.1`, schema `1`):* `pp_if_branch.keyword`
emitted `"<invalid_sequence_access>"` for `` `ifdef`` / `` `ifndef``
conditional input. Root cause:
`pp_if_branch := (kw_ifdef | kw_ifndef) macro_name … -> {keyword: $1,
…}` bound `$1` to an **inline alternation group** — the same
inline-alternation-`$N` emit-time defect class fixed for `rtl_const_expr`
in RTL-CE-Slice-2 (and tracked for `rtl_frontend` / `vhdl` `binop_chain`).
The `` `define`` / non-conditional surface was unaffected. **Fix
(proven RTL-CE-Slice-2 playbook):** the inline alternation is lifted into
the named rule `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} | kw_ifndef
-> {kind: "ifndef"}`; `pp_if_branch`'s annotation is unchanged
(`{keyword: $1, …}`) but `$1` now binds the clean named rule. The
verified post-fix output is clean — for
`` `ifdef X\n`define A 1\n`else\n`define B 2\n`endif\n`` the
`if_branch.keyword` is now `{kind: "ifdef"}` (or `{kind: "ifndef"}` for
an `` `ifndef`` guard), with **no** `<invalid_sequence_access>`
anywhere. Consumers read the conditional polarity directly from
`if_branch.keyword.kind`; the guard macro stays at the outer
`if_branch.macro`. The honest pre-fix history is retained in the
[Schema Versioning](#schema-versioning) schema `1.0.0` row.

## Conditional Compilation and Macro Body Fragments

This section is the consumer-facing walk contract for the two deep
substructures the [`pp_item` dispatch](#the-10-branch-pp_item-dispatch)
hands back: the **conditional-compilation tree** (reached via the
`pp_item` `"conditional"` branch → `pp_conditional`) and the
**macro-body / macro-default fragment streams** (reached via the
`pp_item` `"define"` branch → `pp_define` → `macro_body` and, for
parameterised macros, `macro_formal` → `macro_default_value`). Every
shape, `kind`, field name, branch count, and grammar line reference
below is transcribed from the live inventory
`generated/systemverilog_preprocessor_return_annotations.json`
(`version: 1`, `grammar: "systemverilog_preprocessor"`,
`annotation_count: 66`, **28 distinct rules**) and verified against
`grammars/systemverilog_preprocessor.ebnf`. The inventory is the ground
truth: where this prose and the inventory disagree, the inventory wins.

### The conditional-compilation tree

The `pp_item` `"conditional"` branch (branch 7 in the
[`pp_item` dispatch](#the-10-branch-pp_item-dispatch)) carries a `body`
that is the typed `pp_conditional` object. `pp_conditional` is the
single-branch root of the `` `ifdef`` / `` `ifndef`` … `` `elsif`` …
`` `else`` … `` `endif`` tree. Per
`grammars/systemverilog_preprocessor.ebnf` (lines 61–80):

```ebnf
pp_conditional  := pp_if_branch pp_elsif_branch* pp_else_branch? pp_endif
                -> {if_branch: $1, elsif_branches: $2, else_branch: $3}
pp_if_keyword   := kw_ifdef  -> {kind: "ifdef"}
                 | kw_ifndef -> {kind: "ifndef"}
pp_if_branch    := pp_if_keyword macro_name directive_tail? newline pp_item*
                -> {keyword: $1, macro: $2, tail: $3, items: $5}
pp_elsif_branch := kw_elsif condition_expr newline pp_item*
                -> {condition: $2, items: $4}
pp_else_branch  := kw_else directive_tail? newline pp_item*
                -> {tail: $2, items: $4}
pp_endif        := kw_endif directive_tail? newline?
                -> {tail: $2}
```

`pp_if_keyword` is the `1.0.2` `SVPP-0001` correctness fix (the proven
RTL-CE-Slice-2 / `systemverilog.ebnf` idiom): the previously-inline
`(kw_ifdef | kw_ifndef)` alternation is lifted into this **named**
2-branch rule so `pp_if_branch`'s bare `keyword: $1` captures cleanly.
It is the **only** dispatch rule in the conditional tree — a 2-branch
`kind`-tagged shape (`{kind: "ifdef"}` / `{kind: "ifndef"}`,
`branch_index` 0–1, both `return_object`). The other five rules
(`pp_conditional`, `pp_if_branch`, `pp_elsif_branch`, `pp_else_branch`,
`pp_endif`) each have exactly **one** branch (`branch_index: 0`), each
`annotation_type: "return_object"`, and carry no `type`/`kind`
discriminator — they are fixed-shape positional objects reached
structurally from the `pp_item` `"conditional"` `body`, not by tag
dispatch.

| Rule (`grammars/systemverilog_preprocessor.ebnf`) | Annotation | Branches | Fields | Meaning |
|---|---|---|---|---|
| `pp_conditional` (line 61) | `return_object` `{if_branch: $1, elsif_branches: $2, else_branch: $3}` | 1 | `if_branch`, `elsif_branches`, `else_branch` | `if_branch` is exactly one typed `pp_if_branch`. `elsif_branches` is an array (`[]` when there are no `` `elsif``) of typed `pp_elsif_branch`, in source order. `else_branch` is the single typed `pp_else_branch`, or `[]` when there is no `` `else`` (the `pp_else_branch?` optional). The closing `pp_endif` ($4) is **not** re-emitted as a field — it is consumed positionally and contributes only its `tail` semantics; there is no `endif` key on `pp_conditional`. |
| `pp_if_keyword` (lines 71–72) | `return_object` `{kind: "ifdef"}` (branch 0) / `{kind: "ifndef"}` (branch 1) | 2 | `kind` | The `1.0.2` `SVPP-0001` named-rule fix. A 2-branch `kind`-tagged polarity discriminator: branch 0 matches `kw_ifdef` → `{kind: "ifdef"}`, branch 1 matches `kw_ifndef` → `{kind: "ifndef"}`. This is exactly what `pp_if_branch.keyword` now holds. The two branches are the +2 entries that took the annotation count `64 → 66` and the distinct-rule count `27 → 28`. |
| `pp_if_branch` (line 74) | `return_object` `{keyword: $1, macro: $2, tail: $3, items: $5}` | 1 | `keyword`, `macro`, `tail`, `items` | `keyword` is `$1`, the typed `pp_if_keyword` polarity object — **`{kind: "ifdef"}` or `{kind: "ifndef"}`** as of the `1.0.2` `SVPP-0001` fix (was the malformed `<invalid_sequence_access>` object at `1.0.1` / schema `1` — see [Resolved defect — `SVPP-0001`](#resolved-defect--svpp-0001-fixed-in-102)). Read the conditional polarity from `if_branch.keyword.kind`. `macro` is the guard macro name (`macro_name`/`identifier` envelope). `tail` is the optional `directive_tail` envelope (`[]` when absent). `items` is a nested `pp_item*` array (`[]` when the branch body is empty) — recurse into the [`pp_item` dispatch](#the-10-branch-pp_item-dispatch) for each element. |
| `pp_elsif_branch` (line 76) | `return_object` `{condition: $2, items: $4}` | 1 | `condition`, `items` | `condition` is the typed `condition_expr` object (`{atoms}`) — a `condition_atom+` stream (see the condition-atom note below). `items` is a nested `pp_item*` array (`[]` when empty). Note the positional gap: `kw_elsif` is `$1`, `condition_expr` is `$2`, `newline` is `$3`, `pp_item*` is `$4`; `$1`/`$3` are consumed but not re-emitted. |
| `pp_else_branch` (line 78) | `return_object` `{tail: $2, items: $4}` | 1 | `tail`, `items` | `tail` is the optional `directive_tail` envelope (`[]` when absent). `items` is a nested `pp_item*` array (`[]` when empty). `kw_else` (`$1`) and `newline` (`$3`) are consumed positionally without their own fields. |
| `pp_endif` (line 80) | `return_object` `{tail: $2}` | 1 | `tail` | `tail` is the optional `directive_tail` envelope (`[]` when absent). `pp_endif` is consumed positionally by `pp_conditional` ($4) and is **not** surfaced as a `pp_conditional` field — a consumer that needs the `` `endif`` trailing comment must obtain it through a path that retains the `pp_endif` shape (e.g. a raw/structured dump), not from the typed `pp_conditional` object. |

**Consumer walk.** To reconstruct the
`` `ifdef``/`` `ifndef`` … `` `elsif`` … `` `else`` … `` `endif``
tree: dispatch a `pp_item` on `kind == "conditional"`, take
`body` (the `pp_conditional`). Read `body.if_branch` — its `macro` is
the guard identifier and its `keyword` distinguishes
`` `ifdef`` (defined-true) from `` `ifndef`` (defined-false) via
`body.if_branch.keyword.kind` (`"ifdef"` / `"ifndef"`), the typed
`pp_if_keyword` polarity object landed by the `1.0.2` `SVPP-0001`
fix (was defective at `1.0.1`; see
[Resolved defect — `SVPP-0001`](#resolved-defect--svpp-0001-fixed-in-102)).
Recurse `body.if_branch.items` as a `pp_item*` array for the if-true
body. Iterate
`body.elsif_branches` (possibly `[]`); for each, evaluate
`elsif.condition` (a `condition_expr` `{atoms}`) and recurse
`elsif.items`. If `body.else_branch` is not `[]`, recurse
`body.else_branch.items` for the fallback body. The closing
`` `endif`` is structurally implied (the tree is well-formed only when
a `pp_endif` was consumed) and is not a field.

#### `SVPP-0001` in the conditional-tree context (resolved in 1.0.2)

`pp_if_branch.keyword` is now the typed `pp_if_keyword` polarity object
**`{kind: "ifdef"}`** (or `{kind: "ifndef"}`). *Historical (release
`1.0.1`, schema `1`):* `keyword` (the `$1` bound to the then-inline
alternation `(kw_ifdef | kw_ifndef)`) surfaced, for any
`` `ifdef``/`` `ifndef`` input, a malformed nested object containing
three `"<invalid_sequence_access>"` strings instead of the keyword
token — the inline-alternation-`$N` emit-time defect class
(`SVPP-0001`). **Fixed in parser release `1.0.2` / schema `2`** by
lifting `(kw_ifdef | kw_ifndef)` into the named `pp_if_keyword` rule
(proven RTL-CE-Slice-2 playbook); the schema was bumped `1 → 2` because
`pp_if_branch.keyword` changed shape in a consumer-visible way. Tracked
(status `Released`) in
`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0001`) and at
the head of this contract under
[Resolved Defects — `SVPP-0001`](#resolved-defects--svpp-0001-fixed-in-release-102-schema-2)
and the
[Resolved defect — `SVPP-0001`](#resolved-defect--svpp-0001-fixed-in-102)
note of the AST-Envelope section. The fragment-specific consumer
guidance: read the conditional polarity directly from
`if_branch.keyword.kind` (`"ifdef"` = defined-true,
`"ifndef"` = defined-false); the guard macro is at the outer
`if_branch.macro`; `if_branch.tail`, `if_branch.items`, and every
`pp_elsif_branch` / `pp_else_branch` / `pp_endif` field are correct.
There is no longer any `<invalid_sequence_access>` anywhere in the
conditional tree; consumers written against `1.0.1` that treated
`if_branch.keyword` as opaque/text-only must repin to schema `2` and
switch to the `keyword.kind` discriminator.

### The `macro_body` fragment kinds

The `pp_item` `"define"` branch carries a `body` that is the typed
`pp_define` (`{name, formals, body}`, line 33). `pp_define.body` is, for
a non-bodyless macro, the typed `macro_body` object; for a bodyless
`` `define`` it is `[]` (the `macro_body?` optional). Per
`grammars/systemverilog_preprocessor.ebnf` (lines 112–124):

```ebnf
macro_body          := macro_body_fragment+
                    -> {fragments: $1}
macro_body_fragment := macro_token_paste -> {kind: "token_paste"}
                     | macro_stringize   -> {kind: "stringize"}
                     | macro_reference   -> {kind: "macro_reference", body: $1}
                     | macro_body_text   -> {kind: "text",   body: $1}
                     | lparen            -> {kind: "lparen"}
                     | rparen            -> {kind: "rparen"}
                     | comma             -> {kind: "comma"}
                     | question          -> {kind: "question"}
                     | colon             -> {kind: "colon"}
```

`macro_body` is a single-branch `return_object`
(`{fragments: $1}`, line 112): `fragments` is a non-empty array (the
`macro_body_fragment+`) of typed `macro_body_fragment` objects in
source order. `macro_body_fragment` is a `kind`-tagged dispatcher. The
inventory confirms exactly **9** `macro_body_fragment` branches
(`branch_index` 0–8, every one `annotation_type: "return_object"`) —
this **matches the expected 9**:

| Branch | `kind` | Fields | Captured source text | Grammar line |
|---|---|---|---|---|
| 0 | `"token_paste"` | _(none — bare `{kind}`)_ | The `` `` `` token-paste operator (`macro_token_paste := inline_trivia /``/`) | line 114 |
| 1 | `"stringize"` | _(none — bare `{kind}`)_ | The `` `" `` stringize operator (`macro_stringize := inline_trivia /`"/`) | line 115 |
| 2 | `"macro_reference"` | `body` | A nested macro reference `` `IDENT`` (`macro_reference := bt_identifier`); `body` is the referenced macro envelope | line 116 |
| 3 | `"text"` | `body` | A run of literal body text (`macro_body_text := inline_trivia /[^`(),?:\r\n]+/`); `body` is the captured text envelope | line 117 |
| 4 | `"lparen"` | _(none — bare `{kind}`)_ | A literal `(` | line 118 |
| 5 | `"rparen"` | _(none — bare `{kind}`)_ | A literal `)` | line 119 |
| 6 | `"comma"` | _(none — bare `{kind}`)_ | A literal `,` | line 120 |
| 7 | `"question"` | _(none — bare `{kind}`)_ | A literal `?` | line 121 |
| 8 | `"colon"` | _(none — bare `{kind}`)_ | A literal `:` | line 122 |

The structural punctuation kinds (`"lparen"`, `"rparen"`, `"comma"`,
`"question"`, `"colon"`) are emitted as discrete fragments rather than
folded into `"text"` so a consumer can reconstruct macro-argument
parentheses and `?:` ternary structure inside the un-expanded body
without re-lexing. Only `"macro_reference"` and `"text"` carry a
`body`; the other seven kinds are bare `{kind}` objects.

### The `macro_default_atom` kinds

For a parameterised macro, `pp_define.formals` is the typed
`macro_formals` — a clean `[macro_formal, …]` array (line 110, the
`1.0.3` POST-SV-AUDIT Category-A extraction-spread `[$2, $3::2*]`; at
≤ release `1.0.2` / schema `2` it was the raw `{first, rest}` iteration
envelope — see [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--macro_formals-category-a-raw-envelope--clean-list-schema-2--3));
each `macro_formal` (`{name, default}`, line 112) may carry a `default`
that is the typed `macro_default_value` object. Per
`grammars/systemverilog_preprocessor.ebnf` (lines 99–110):

```ebnf
macro_default_value := macro_default_atom+
                    -> {atoms: $1}
macro_default_atom  := macro_token_paste  -> {kind: "token_paste"}
                     | macro_stringize    -> {kind: "stringize"}
                     | macro_reference    -> {kind: "macro_reference", body: $1}
                     | macro_default_text -> {kind: "text",  body: $1}
                     | lparen             -> {kind: "lparen"}
                     | rparen             -> {kind: "rparen"}
                     | question           -> {kind: "question"}
                     | colon              -> {kind: "colon"}
```

`macro_default_value` is a single-branch `return_object`
(`{atoms: $1}`, line 99): `atoms` is a non-empty array (the
`macro_default_atom+`) of typed `macro_default_atom` objects in source
order. `macro_default_atom` is a `kind`-tagged dispatcher. The
inventory confirms exactly **8** `macro_default_atom` branches
(`branch_index` 0–7, every one `annotation_type: "return_object"`) —
this **matches the expected 8**. Note this is one fewer than
`macro_body_fragment`: `macro_default_atom` has **no `"comma"` kind**
(a comma terminates a default-argument value in the `macro_formals`
`lparen macro_formal (comma macro_formal)* rparen` grammar, so it is
not part of an atom stream), and its text leaf is `macro_default_text`
(line 110, char class `/[^`(),?:\r\n]+/`) rather than
`macro_body_text`:

| Branch | `kind` | Fields | Captured source text | Grammar line |
|---|---|---|---|---|
| 0 | `"token_paste"` | _(none — bare `{kind}`)_ | The `` `` `` token-paste operator | line 101 |
| 1 | `"stringize"` | _(none — bare `{kind}`)_ | The `` `" `` stringize operator | line 102 |
| 2 | `"macro_reference"` | `body` | A nested macro reference `` `IDENT``; `body` is the referenced macro envelope | line 103 |
| 3 | `"text"` | `body` | A run of literal default text (`macro_default_text := inline_trivia /[^`(),?:\r\n]+/`); `body` is the captured text envelope | line 104 |
| 4 | `"lparen"` | _(none — bare `{kind}`)_ | A literal `(` | line 105 |
| 5 | `"rparen"` | _(none — bare `{kind}`)_ | A literal `)` | line 106 |
| 6 | `"question"` | _(none — bare `{kind}`)_ | A literal `?` | line 107 |
| 7 | `"colon"` | _(none — bare `{kind}`)_ | A literal `:` | line 108 |

**Consumer guidance — composing default-argument atoms.** A
default-argument value is the `atoms` array of a `macro_default_value`,
reached via `pp_define.formals` (the clean `macro_formal[]` list as of
`1.0.3` / schema `3` — at ≤ `1.0.2` / schema `2` this was the raw
`macro_formals.first` / `macro_formals.rest[]` `{first, rest}` envelope;
see [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--macro_formals-category-a-raw-envelope--clean-list-schema-2--3))
→ each `macro_formal` → `macro_formal.default`
(`[]` when the formal has no `= default`). Concatenate the atoms in
order, dispatching each on `kind`: `"text"` / `"macro_reference"`
contribute their `body`; `"lparen"` / `"rparen"` / `"question"` /
`"colon"` contribute their literal punctuation (these let a consumer
reassemble a parenthesised or ternary default expression); `"token_paste"` /
`"stringize"` contribute the `` `` `` / `` `" `` operators. There is no
`"comma"` atom by construction — a comma always closes the current
default and begins the next `macro_formal`.

### Consumer guidance — walking a `pp_define` body

To walk a `` `define`` end to end: dispatch a `pp_item` on
`kind == "define"` and take `body` — the typed `pp_define`
(`{name, formals, body}`, line 33). `name` is the macro identifier
envelope. `formals` is `[]` for an object-like macro, else the typed
`macro_formals` — a clean flat `macro_formal[]` array (the `1.0.3` /
schema `3` POST-SV-AUDIT Category-A `[$2, $3::2*]` extraction-spread; at
≤ `1.0.2` / schema `2` this was the raw `{first, rest}` envelope where a
consumer had to walk `formals.first` then `formals.rest[][1]` past the
`comma` separator — see
[AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--macro_formals-category-a-raw-envelope--clean-list-schema-2--3)).
Iterate `pp_define.formals` directly (each element a typed
`macro_formal` `{name, default}`), recursing into
`macro_default_value.atoms` (the `macro_default_atom` stream above) for
any non-`[]` `default`. `pp_define.body` is `[]` for a bodyless macro,
else the typed `macro_body` (`{fragments}`); iterate `fragments`,
dispatching each typed `macro_body_fragment` on its `type`-equivalent
`kind` tag (`"text"` / `"macro_reference"` / `"token_paste"` /
`"stringize"` / `"lparen"` / `"rparen"` / `"comma"` / `"question"` /
`"colon"`) per the table above to reconstruct the un-expanded macro
body in source order. The only `type`-discriminated object on this
whole path is the `systemverilog_preprocessor_file` root; `pp_item`,
`macro_body_fragment`, and `macro_default_atom` all dispatch on `kind`.
This contract documents the surface as it exists in the inventory; the
schema axis is the AST-dump schema version **`3`** (see
[Schema Versioning](#schema-versioning)) and the parser release is
**`1.0.3`**, with **66 return annotations across 28 distinct rules**
(65 `return_object` + 1 `return_array`) — the count is unchanged by the
`1.0.3` POST-SV-AUDIT correction (`macro_formals` is still one rule /
one annotation; only its annotation form changed `return_object` →
`return_array`, `{first, rest}` → `[$2, $3::2*]`). The `1.0.2`
`SVPP-0001` fix touched only the conditional tree's
`pp_if_branch.keyword` / new `pp_if_keyword` rule.
Where this prose and the inventory disagree, the inventory wins.

## Source Of Truth
- Grammar source:
  - `grammars/systemverilog_preprocessor.ebnf`
- Runtime execution stage:
  - `rust/src/sv_preprocessor.rs`
- Generated-parser build discovery:
  - `rust/build.rs`
  - `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH`
- Current operational guide:
  - `PGEN_USER_GUIDE.md`
- Live status surface:
  - `LIVE_ACHIEVEMENT_STATUS.md`

## Stable Integration Surface
- Current downstream-facing contract is narrower than the main SystemVerilog/VHDL/regex host surface.
- The repository does expose generated-parser registry coverage for `systemverilog_preprocessor`, but it does not currently publish a dedicated general-purpose embedding API profile for it in `pgen::embedding_api`.
- The practical stable surface today is:
  - the Rust preprocessor execution/runtime module in `rust/src/sv_preprocessor.rs`
  - the executable quality and differential gates documented in `PGEN_USER_GUIDE.md`

## Build / Availability Requirements
- Do not treat internal parser-registry exposure as equivalent to a published general-purpose downstream host contract.
- If a downstream project needs a generic public embedding API for `systemverilog_preprocessor`, that should be treated as new product-surface work, not assumed from current internal registry availability.

## Validation / Release Gates
- `make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate`
- `make -C rust SHELL=/bin/bash sv_preprocessor_curated_differential_gate`
- `make -C rust SHELL=/bin/bash sv_preprocessor_template_differential_gate`

## Scope / Non-Goals
- This document is intentionally explicit that `systemverilog_preprocessor` does not yet have the same published host-embedding shape as `systemverilog`, `vhdl`, or `regex`.
- Downstream consumers should not couple themselves to internal generated parser modules as if they were already a stable public API.
- If a downstream integrator still reports a reproducible preprocessor/runtime bug, use `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` and log accepted released-parser issues in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Companion Documentation — sv_preprocessor Parser Integration mdBook

This contract is the **downstream integration surface**: the host-API
envelope, the `pp_item` dispatch / conditional-compilation / macro-body
shapes a consumer compiles against, and the release/schema axes. It does
not duplicate the per-rule walkthroughs or worked examples — those live
in the companion artifacts below. Each surface is authoritative for a
different thing; consult the matching one and respect the precedence
order stated at the end of this section.

| Surface | Path | Authoritative for |
|---|---|---|
| **This contract** | `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` | The downstream integration surface: AST-dump envelope, `systemverilog_preprocessor_file` root, the 10-branch `pp_item` dispatch, the conditional-compilation tree, and the macro-body / macro-default fragment streams. See [AST Envelope and pp_item Dispatch](#ast-envelope-and-pp_item-dispatch) and [Conditional Compilation and Macro Body Fragments](#conditional-compilation-and-macro-body-fragments). |
| **Per-parser mdBook** | `docs/systemverilog_preprocessor_parser_book/` (source `src/*.md`; tracked HTML at `docs/systemverilog_preprocessor_parser_book-html/`) | The per-rule reference and teaching surface: build recipe, public API, AST-envelope walkthrough, every rule shape, per-feature worked examples (including the conditional worked example with the `SVPP-0001` schema-`1`→`2` fix transition), schema-versioning timeline, glossary, changelog index. Curated, not machine-checked. Listed in `README.md` § "Per-Parser Integration Reference Books". |
| **Shape-contract manifest** | `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` | The machine-checkable shape lock embedded in the regression test. Content-identical to the live inventory on the `(rule, branch_index, annotation_type, normalized_text)` tuples (the embedded copy omits only the diagnostic `raw_text` field). Drift fails the AST-shape-contract test. |
| **Declared-annotation inventory** | `generated/systemverilog_preprocessor_return_annotations.json` | The live machine-checkable enumeration of every typed-shape annotation the systemverilog_preprocessor grammar emits (`version: 1`, `grammar: "systemverilog_preprocessor"`, `annotation_count: 66`, **28 distinct rules**; 65 `return_object` + 1 `return_array` — the `return_array` is `macro_formals`, the `1.0.3` POST-SV-AUDIT Category-A correction). The generator-side source of truth for the typed surface. |
| **Embedding-API contract** | `rust/docs/EMBEDDING_API_CONTRACT.md` | The canonical host-API truth: the `AstDumpPayload` struct (`dump_json` / `truncated` / `full_bytes` / `emitted_bytes`), the entry-point signatures, the truncation diagnostic envelope, and the stable diagnostics. The struct shape this contract documents is transcribed from there. |
| **Released-parser bug ledger** | `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | The accepted-bug log for the released systemverilog_preprocessor parser; the `SVPP-0001` defect lives there (status `Released`, fixed in parser release `1.0.2` / schema `2`). Consult before integrating around a suspected parser defect; file new accepted bugs here per `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`. |

Precedence when surfaces disagree (highest first): the **embedding-API
contract** (`rust/docs/EMBEDDING_API_CONTRACT.md`) wins for the host-API /
`AstDumpPayload` truth; the **declared-annotation inventory**
(`generated/systemverilog_preprocessor_return_annotations.json`) and its
embedded shape-contract manifest copy
(`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`)
win for the exact typed-shape enumeration; **this integration contract**
wins over the **per-parser mdBook** for downstream compliance. Report any
disagreement as a documentation bug rather than silently coding to the
lower-precedence surface.

### Gate Recipe

The exact, copy-pasteable per-family commands a downstream integrator or
releaser runs. Each is verified against the repo (`rust/Makefile`,
`docs/systemverilog_preprocessor_parser_book/src/build-recipe.md`,
`rust/src/ast_shape_contract.rs`); none are invented — do not substitute
flags.

**1. On-demand parser regen.** The systemverilog_preprocessor parser is
on-demand-only (not in the default `cargo test --features
generated_parsers` build). Build `ast_pipeline`, then regenerate the
parser from `grammars/systemverilog_preprocessor.ebnf` (run from
`rust/`, per
`docs/systemverilog_preprocessor_parser_book/src/build-recipe.md`
§ "Cold-clone build"):

```bash
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/systemverilog_preprocessor.ebnf \
    --generate-parser --output ../generated/systemverilog_preprocessor_parser.rs
```

To wire the regenerated parser into a cargo build, point
`PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH` at the absolute path of the
generated file before `cargo build --release --features
generated_parsers` (see
`docs/systemverilog_preprocessor_parser_book/src/build-recipe.md`
§ "Wiring into a downstream Cargo build").

**2. Per-family book gate.** Builds the sv_preprocessor parser book and
verifies the tracked HTML landing pages (Makefile target
`systemverilog_preprocessor_parser_book_gate`, `rust/Makefile`
line 751, which runs
`rust/scripts/systemverilog_preprocessor_parser_book_gate.sh`):

```bash
make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate
```

> Shell note: this contract (and the book's `build-recipe.md`)
> deliberately use the Homebrew bash-4+ path
> `SHELL=/opt/homebrew/bin/bash`, consistent with the other per-family
> contracts. `README.md` § "Per-Parser Integration Reference Books"
> still prints the `SHELL=/bin/bash systemverilog_preprocessor_parser_book_gate`
> form; that README-wide discrepancy is tracked as
> `DOC-README-SHELL-0001` — prefer the Homebrew path used here.

**3. AST-shape-contract regression lock.** With the generated backend
wired in (`PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH` exported), run
the shape-contract test that diffs the running generated parser against
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`
(test fn
`systemverilog_preprocessor_ast_shape_contract_holds_against_running_generated_parser`
in the `pgen::ast_shape_contract` library module,
`rust/src/ast_shape_contract.rs` line 817):

```bash
cargo test --lib --features generated_parsers systemverilog_preprocessor_ast_shape_contract
```

The substring `systemverilog_preprocessor_ast_shape_contract` selects
exactly the
`systemverilog_preprocessor_ast_shape_contract_holds_against_running_generated_parser`
test (it does not match the `vhdl_*` / `rtl_*` shape-contract tests in
the same module). Any drift between the running parser's emitted shapes
and the locked manifest fails this test, surfacing the change before
release. (The `1.0.2` `SVPP-0001` fix is locked here — the corrected
`pp_if_branch.keyword` / new `pp_if_keyword` shape is part of the locked
66-entry baseline; any reversion to the pre-fix
`<invalid_sequence_access>` shape fails this test immediately. The
schema was bumped `1 → 2` in same-commit lockstep with this fix.)

**4. Validation / release gates.** Anyone publishing a parser-release
version bump also runs the per-family gates enumerated in
[Validation / Release Gates](#validation--release-gates) (the
`sv_preprocessor_quality_gate`, `sv_preprocessor_curated_differential_gate`,
and `sv_preprocessor_template_differential_gate` targets). That section
is the canonical list; it is not repeated here.

## Glossary

Contract-scoped definitions of the terms a downstream integrator needs to
read this document. Where a term has a normative definition, this
contract is authoritative; the per-parser book's
[glossary](../systemverilog_preprocessor_parser_book/src/glossary.md)
paraphrases the same terms for quick lookup. Numbers below are pinned to
contract `1.0.3` / AST-dump schema `3` / parser release `1.0.3` /
**66 return annotations across 28 distinct rules** (65 `return_object`
+ 1 `return_array` — the `return_array` is `macro_formals`, the `1.0.3`
POST-SV-AUDIT Category-A correction).

- **`AstDumpPayload`** — the success return of the
  systemverilog_preprocessor AST-dump host entry points (defined in
  `rust/src/embedding_api.rs`, contract in
  `rust/docs/EMBEDDING_API_CONTRACT.md`). A canonical-JSON payload string
  plus truncation metadata, with **exactly four fields**: `dump_json`,
  `truncated`, `full_bytes`, `emitted_bytes`. It does **not** carry
  `root` / `schema_version` / `grammar` / `profile` members — see
  [The `AstDumpPayload` envelope](#the-astdumppayload-envelope) for the
  precise accuracy note.
- **`dump_json`** — the `AstDumpPayload` field holding the canonical
  (key-sorted) JSON encoding of the typed systemverilog_preprocessor
  AST. Parse this string to obtain the
  `systemverilog_preprocessor_file` root object. When `truncated` is
  `true` this string is replaced by the truncation diagnostic envelope,
  not the AST.
- **Truncation diagnostic envelope** — the deterministic JSON object
  that replaces the AST in `dump_json` when `max_ast_bytes` is exceeded.
  It carries `pgen_dump_contract_version` (currently `1`), `kind:
  "pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
  "parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`.
  Consumers must check `truncated` (or detect `kind ==
  "pgen_ast_dump_truncation"`) before treating `dump_json` as a
  systemverilog_preprocessor AST. `pgen_dump_contract_version` is a
  member of **this envelope only**, never of `AstDumpPayload` itself.
- **AST-dump schema version** — the integer version axis tracking the
  AST output shape, currently `3`, pinned by this contract (see
  [Schema Versioning](#schema-versioning)). It is **not** a field of
  `AstDumpPayload`; it is the contract-tracked axis. Bumped only when
  the emitted shape changes in a way consumers may need to adapt to (new
  annotation on a previously-unannotated rule, restructured annotation,
  user-visible grammar-shape change). It was bumped `1 → 2` by the
  `1.0.2` `SVPP-0001` correctness fix because `pp_if_branch.keyword`
  changed shape in a consumer-visible way (was the malformed
  `<invalid_sequence_access>` object at schema `1`, now the typed
  `{kind: "ifdef"|"ifndef"}` `pp_if_keyword` polarity discriminator),
  and bumped `2 → 3` by the `1.0.3` POST-SV-AUDIT Category-A correction
  because `pp_define.formals` (`macro_formals`) changed from the raw
  `{first, rest}` iteration envelope to the clean `[macro_formal, …]`
  list (`PGEN-POST-SV-AUDIT-0002`; a deliberate audit-driven shape
  correction, **not** a released-parser bug).
- **Parser release version** — the parser library's release identity,
  currently `1.0.3`. Bumped on every functional change (bug fixes, perf
  work, grammar changes). Moves independently of the schema version; the
  `1.0.3` release carries AST-dump schema `3`.
- **`pp_item` dispatch** — the primary top-level dispatcher: a
  **10-branch** `kind`-tagged shape
  (`grammars/systemverilog_preprocessor.ebnf` lines 18–27). Every parse
  roots at `{type: "systemverilog_preprocessor_file", items: [...]}`;
  each element of `items` is one typed `pp_item` object in source order.
  The `systemverilog_preprocessor_file` root is the only `type`-tagged
  dispatch object; `pp_item`, `condition_atom`, `macro_default_atom`,
  and `macro_body_fragment` all dispatch on `kind`. See
  [The 10-branch `pp_item` dispatch](#the-10-branch-pp_item-dispatch).
- **The 10 `pp_item` kinds** — `"define"`, `"undef"`, `"include"`,
  `"timescale"`, `"default_nettype"`, `"celldefine"`,
  `"endcelldefine"`, `"conditional"`, `"non_directive_line"`,
  `"blank_line"` (branch indices 0–9). Every kind except the three
  bodyless ones (`"celldefine"`, `"endcelldefine"`, `"blank_line"`)
  carries a `body` holding the underlying typed shape.
- **The 7 directive shapes** — the seven non-conditional directive
  rules below the `pp_item` dispatch, each a single-branch
  `return_object`: `pp_define` (`{name, formals, body}`), `pp_undef`
  (`{name, comment}`), `pp_include` (`{path, comment}`), `pp_timescale`
  (`{unit, precision, comment}`), `pp_default_nettype`
  (`{nettype, comment}`), `pp_celldefine` (`{comment}`),
  `pp_endcelldefine` (`{comment}`). See [Per-directive shapes](#per-directive-shapes).
- **Conditional-compilation tree** — the typed tree the
  `pp_item` `"conditional"` branch hands back: `pp_conditional`
  (`{if_branch, elsif_branches, else_branch}`) over `pp_if_branch`
  (`{keyword, macro, tail, items}` — where `keyword` is the typed
  `pp_if_keyword` polarity object `{kind: "ifdef"|"ifndef"}` as of the
  `1.0.2` `SVPP-0001` fix), `pp_elsif_branch` (`{condition, items}`),
  `pp_else_branch` (`{tail, items}`), and `pp_endif` (`{tail}`), plus
  the 2-branch `pp_if_keyword` dispatch rule
  (`{kind: "ifdef"}` / `{kind: "ifndef"}`) the `1.0.2` fix added. Each
  branch's `items` is a nested `pp_item*` array; the closing `pp_endif`
  is consumed positionally and is **not** re-emitted as a
  `pp_conditional` field. See
  [The conditional-compilation tree](#the-conditional-compilation-tree).
- **`macro_body_fragment` / `macro_default_atom`** — the two
  `kind`-tagged fragment-stream dispatchers. `macro_body` is
  `{fragments}` over **9** `macro_body_fragment` kinds (`"token_paste"`,
  `"stringize"`, `"macro_reference"`, `"text"`, `"lparen"`, `"rparen"`,
  `"comma"`, `"question"`, `"colon"`); `macro_default_value` is
  `{atoms}` over **8** `macro_default_atom` kinds (the same set **minus
  `"comma"`**, whose text leaf is `macro_default_text`). Only the
  `"macro_reference"` / `"text"` kinds carry a `body`; the others are
  bare `{kind}`. See [The `macro_body` fragment kinds](#the-macro_body-fragment-kinds)
  and [The `macro_default_atom` kinds](#the-macro_default_atom-kinds).
- **Recursive envelope** — the default JSON shape produced by
  un-annotated rules: a recursive composition of arrays (sequences,
  quantified iterations such as a branch's nested `pp_item*` `items`),
  strings (terminal/regex leaves — e.g. the `macro_name` / `identifier`
  envelope, the `directive_comment_tail` / `directive_tail` tail), and
  matched-branch passthroughs (for alternations). Un-matched optionals
  are the empty array `[]`, never `null`. It is what a consumer reaches
  when descending below the typed surface (the `name` / `comment` /
  `tail` envelopes; the malformed `pp_if_branch.keyword` nested object
  of `SVPP-0001`).
- **Shape-contract manifest** — the embedded machine-checkable shape
  lock `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`.
  Content-identical to the declared-annotation inventory on the
  `(rule, branch_index, annotation_type, normalized_text)` tuples (omits
  only the diagnostic `raw_text` field). Drift fails the
  `systemverilog_preprocessor_ast_shape_contract_holds_against_running_generated_parser`
  regression test (see [Gate Recipe](#gate-recipe)).
- **Declared-annotation inventory** — the live machine-checkable
  enumeration of every typed-shape annotation the
  systemverilog_preprocessor grammar emits:
  `generated/systemverilog_preprocessor_return_annotations.json`
  (`version: 1`, `grammar: "systemverilog_preprocessor"`,
  `annotation_count: 66`, **28 distinct rules**; 65 `return_object` + 1
  `return_array` — the `return_array` is `macro_formals`, the `1.0.3`
  POST-SV-AUDIT Category-A correction). The generator-side source of
  truth for the typed surface; mirrored by the embedded shape-contract
  manifest copy. (The `version: 1` field is the inventory-file format
  version, distinct from the AST-dump schema version `3` and the parser
  release version `1.0.3`.)
- **Generic host AST-dump surface** — the
  `parse_grammar_profile_ast_dump*` family
  (`parse_grammar_profile_ast_dump`, the `*_result` and `*_named`
  forms). The grammar-agnostic entry points that, for the
  `systemverilog_preprocessor` grammar + `default` profile, would return
  the `AstDumpPayload`. systemverilog_preprocessor has **no**
  named-convenience entry point and (per
  [Stable Integration Surface](#stable-integration-surface)) does **not**
  currently publish a dedicated general-purpose embedding-API profile in
  `pgen::embedding_api`; the practical stable surface today is the
  runtime module `rust/src/sv_preprocessor.rs` plus the executable
  gates. Signatures are in `rust/docs/EMBEDDING_API_CONTRACT.md`.
- **`SVPP-0001`** — the one accepted shape defect the `1.0.1` parser
  shipped, **fixed in parser release `1.0.2` / schema `2`**.
  *Historical:* `pp_if_branch.keyword` (the `$1` bound to the
  then-inline alternation `(kw_ifdef | kw_ifndef)`) surfaced, for any
  `` `ifdef``/`` `ifndef`` input, a malformed nested object containing
  three `"<invalid_sequence_access>"` strings instead of the keyword
  token — the inline-alternation-`$N` emit-time defect class also fixed
  for `rtl_const_expr` in RTL-CE-Slice-2 and tracked for
  `rtl_frontend` / `vhdl` `binop_chain`. **Fixed** by lifting
  `(kw_ifdef | kw_ifndef)` into the named rule `pp_if_keyword := kw_ifdef
  -> {kind: "ifdef"} | kw_ifndef -> {kind: "ifndef"}`
  (`grammars/systemverilog_preprocessor.ebnf` lines 71–72); `pp_if_branch`'s
  bare `keyword: $1` (line 74) now binds the clean named rule, so
  `if_branch.keyword` is the typed polarity object `{kind: "ifdef"}` /
  `{kind: "ifndef"}` with **no** `<invalid_sequence_access>` anywhere.
  Status `Released`; tracked in
  `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` and documented at
  [Resolved Defects — `SVPP-0001`](#resolved-defects--svpp-0001-fixed-in-release-102-schema-2)
  and in the
  [Resolved defect — `SVPP-0001`](#resolved-defect--svpp-0001-fixed-in-102)
  and
  [`SVPP-0001` in the conditional-tree context](#svpp-0001-in-the-conditional-tree-context-resolved-in-102)
  notes. Consumer guidance: read the conditional polarity from
  `if_branch.keyword.kind` and the guard macro from the outer
  `if_branch.macro`. The AST-dump schema version was bumped `1 → 2` by
  this fix; consumers written against `1.0.1` must repin to schema `2`.
