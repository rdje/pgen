# docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `systemverilog_preprocessor` frontend/parsing stage.

## Contract Identity
- Contract version:
  - `1.0.1`
- Parser release version:
  - `1.0.1`
- systemverilog_preprocessor AST-dump schema version:
  - `1`
- Last updated:
  - `2026-05-15`
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

1. **Parser release version** (`1.0.1`). Tracks the parser library's release identity.
2. **AST-dump schema version** (`1`). Tracks the AST output shape.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 1.0.0 | 1.0.1 | **SVPP-Slice-1** — initial 64-annotation baseline. pp_item dispatch (10 kinds), 7 directive shapes (define/undef/include/timescale/default_nettype/celldefine/endcelldefine), include_path/nettype_value/time_literal, conditional-compilation tree (5 nodes), condition_expr/condition_atom (12 kinds), macro_formals/formal/default_value/default_atom (8 kinds) / body/body_fragment (9 kinds), passthrough lines. |
| 0.1.0 | 1.0.0 | Foundation baseline. Grammar (`grammars/systemverilog_preprocessor.ebnf`) with the `systemverilog_preprocessor_file -> {type, items}` root only. AST dump is the recursive-envelope shape across all other rules. |

## Known Defects (release 1.0.1)

- **`SVPP-0001` — `pp_if_branch.keyword` `<invalid_sequence_access>`
  (`Root Caused`, fix not yet landed).** For `` `ifdef`` / `` `ifndef``
  conditional input, `items[].body.if_branch.keyword` is a malformed
  nested object containing three `"<invalid_sequence_access>"` strings
  instead of the keyword token. Root cause: `pp_if_branch := (kw_ifdef |
  kw_ifndef) macro_name … -> {keyword: $1, …}` binds `$1` to an
  **inline alternation group**, the same emit-time defect class fixed
  for `rtl_const_expr` in RTL-CE-Slice-2 (and tracked for
  `rtl_frontend` / `vhdl` `binop_chain`). The `` `define`` /
  non-conditional surface is unaffected. **Consumer workaround:** read
  the guard macro from the correct outer `if_branch.macro` (which is
  correct); treat `if_branch.keyword` as token-text-only and do not
  rely on its nested fields. Documented honestly in
  [the conditional worked example](../systemverilog_preprocessor_parser_book/src/examples-conditional.md);
  tracked in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
  (`SVPP-0001`). Scheduled fix: the systemic inline-alternation-`$N`
  correctness lane (lift `(kw_ifdef | kw_ifndef)` into a named rule per
  the proven RTL-CE-Slice-2 playbook; schema bump + lockstep).

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
macro_formals                    -> {first, rest}
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
`annotation_count: 64`, **27 distinct rules**), cross-checked against the
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
> `1`** tracked in [Schema Versioning](#schema-versioning), the grammar
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
| `pp_define` (line 33) | `{name, formals, body}` | `name` is the un-annotated `macro_name`/`identifier` envelope; `formals` is `[]` when no `(...)` formal list; `body` is `[]` for a bodyless macro, else the typed `macro_body` `{fragments}` object. |
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
`condition_atom`, `macro_formals` / `macro_formal` /
`macro_default_value` / `macro_default_atom`, `macro_body` /
`macro_body_fragment`) and the passthrough lines (`pp_non_directive_line`
→ `{text}`, `pp_blank_line` → `{kind: "blank"}`) round out the
27-distinct-rule typed surface; their full per-branch field lists are in
`docs/systemverilog_preprocessor_parser_book/src/rules-top-level.md`.

### Verified surface totals

The full typed surface of contract `1.0.1` is **64 return annotations
across 27 distinct rules** (all 64 are `annotation_type:
"return_object"`), AST-dump schema version `1`, parser release `1.0.1`.
These exact numbers are transcribed from
`generated/systemverilog_preprocessor_return_annotations.json`
(`version: 1`, `grammar: "systemverilog_preprocessor"`,
`annotation_count: 64`; 27 distinct `rule` values) and its embedded copy
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`.
(The inventory-file `version: 1` is the inventory format version,
distinct from the AST-dump schema version `1` and the parser release
version `1.0.1`.) The machine-checkable enumeration of every
`(rule, branch_index, annotation_type, normalized_text)` tuple is those
two artifacts; this contract section is curated; if this section and
either artifact disagree, the artifact wins, and this integration
contract wins over the per-family mdBook.

### Known defect — `SVPP-0001`

The released `1.0.1` parser ships one accepted shape defect, tracked in
`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0001`) and
documented at the head of this contract under
[Known Defects (release 1.0.1)](#known-defects-release-101):
`pp_if_branch.keyword` emits `"<invalid_sequence_access>"` for
`` `ifdef`` / `` `ifndef`` conditional input. Root cause:
`pp_if_branch := (kw_ifdef | kw_ifndef) macro_name … -> {keyword: $1,
…}` binds `$1` to an **inline alternation group** — the same
inline-alternation-`$N` emit-time defect class fixed for `rtl_const_expr`
in RTL-CE-Slice-2 (and tracked for `rtl_frontend` / `vhdl` `binop_chain`).
The `` `define`` / non-conditional surface is unaffected; the guard macro
is still recoverable from the correct outer `if_branch.macro`. **This
defect is NOT fixed in `1.0.1`** — the fix is deferred to the systemic
inline-alternation parser-correctness lane (lift `(kw_ifdef |
kw_ifndef)` into a named rule per the proven RTL-CE-Slice-2 playbook;
schema bump + lockstep at that time). The AST-dump schema version stays
**`1`** for release `1.0.1`.

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
