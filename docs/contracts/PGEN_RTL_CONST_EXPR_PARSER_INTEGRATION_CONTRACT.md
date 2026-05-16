# docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `rtl_const_expr` parser family.

This is the document downstream projects (primarily RTLSyn, for deterministic parameter / width / generate evaluation before elaboration) embedding the PGEN rtl_const_expr parser should read first.

## Contract Identity
- Contract version:
  - `1.0.2`
- Parser release version:
  - `1.0.2`
- Embedding API contract baseline:
  - tracked under `rust/docs/EMBEDDING_API_CONTRACT.md`
- rtl_const_expr AST-dump schema version:
  - `2` (breaking shape correction — see Release 1.0.2 Highlights)
- Annotation count:
  - `26` (19 `return_object` + 7 `return_scalar`; 18 distinct rules)
- Last updated:
  - `2026-05-16`
- Current grammar family label:
  - `rtl_const_expr`
- Per-family mdBook:
  - `docs/rtl_const_expr_parser_book/` (tracked HTML at `docs/rtl_const_expr_parser_book-html/`)
- Per-family gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash rtl_const_expr_parser_book_gate`
- Per-family ast-shape-contract manifest:
  - `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`

## Source Of Truth
- Grammar source:
  - `grammars/rtl_const_expr.ebnf`
- Standalone bootstrap crate:
  - `rtl_const_expr/`
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Build-time generated parser discovery:
  - `rust/build.rs`
  - `PGEN_RTL_CONST_EXPR_PARSER_PATH`

## Stable Integration Surface
- Grammar family:
  - `rtl_const_expr`
- Stable generic entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_ast_dump(...)`
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`

## Validation / Release Gates
- Per-family book gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash rtl_const_expr_parser_book_gate`
- AST-shape contract:
  - `cargo test --lib --features generated_parsers rtl_const_expr_ast_shape_contract`

## Schema Versioning

The rtl_const_expr parser carries two version axes:

1. **Parser release version** (`1.0.2`). Tracks the parser library's release identity.
2. **AST-dump schema version** (`2`). Tracks the AST output shape.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 2 | 1.0.2 | **RTL-CE correctness fix (breaking).** Three return-annotation bugs corrected, regenerated + gate-locked: (a) the 10 `binop_chain` levels no longer emit `"<invalid_sequence_access>"` on any operator input — the five multi-token inner operator alternations were lifted into named rules (`equality_op`, `relational_op`, `shift_op`, `additive_op`, `multiplicative_op`) mirroring the proven `systemverilog.ebnf` op-chain idiom, and `rest` is now a clean array of `[op-envelope, operand]` iteration entries; (b) `identifier.text` was `$1` (empty leading `trivia`) → `$2`, now the real name; (c) `based_integer` / `decimal_integer` were unannotated (surfacing the `["", "42"]` envelope) → annotated `-> $2`, so `literal.text` is a clean string. Annotation count `24 → 26` (the two new leaf scalar captures). Same accept set. |
| 1.0.0 | 1.0.1 | **RTL-CE-Slice-1** — initial 24-annotation baseline. Expression hierarchy (conditional + 10-rule binop_chain + unary + primary), literal (2 kinds), identifier all typed. **NOTE:** the binop_chain `rest` shape and `identifier`/`literal` `text` in this baseline were defective — see schema `2` for the correction. |
| 0.1.0 | 1.0.0 | Foundation baseline. Grammar (`grammars/rtl_const_expr.ebnf`) with the `rtl_const_expr -> {type, expr}` root, `unary_expr` per-branch typed shapes, `primary_expr` / `literal` typed shapes, and `identifier -> {type, text}` already in place; the 10 binop-chain rules were the unannotated tail. |

## Release 1.0.2 / Contract 1.0.2 Highlights — RTL-CE correctness fix (binop_chain / identifier / literal); schema 1 → 2

Landed 2026-05-16. A worked-example pass surfaced that the `1.0.1`
baseline shipped three return-annotation defects that the (root-keys-only)
shape-contract regression lock did not catch. All three are fixed,
the parser is regenerated, and the manifest inventory is tightened to
the full 26-entry surface so the corrected shapes are now machine-locked.

- **binop_chain `<invalid_sequence_access>` (Issue A).** For any input
  exercising an operator at any of the 10 levels (e.g. `a + b`), the
  `rest` field emitted the literal string `"<invalid_sequence_access>"`
  inside a malformed nested object. Root cause: the five multi-token
  levels used an inline operator alternation as the iteration's lead
  element (`additive_expr := multiplicative_expr ((plus|minus) multiplicative_expr)* -> {... rest:$2}`),
  which corrupts the positional model. **Fix (proven `systemverilog.ebnf`
  idiom):** the inline alternations are lifted into named rules
  (`equality_op := eqeq | ne`, `relational_op := le | lt | ge | gt`,
  `shift_op := shl | shr`, `additive_op := plus | minus`,
  `multiplicative_op := star | slash | percent`); every level is now
  `next (NAMED_op next)* -> {type:"binop_chain", level, lhs:$1, rest:$2}`
  with bare `$2`. `rest` is now a **clean array** of
  `[op-envelope, operand]` iteration entries (operator text at
  `entry[0][1]`), `[]` when no operator at that level. Verified on
  `a+b`, `a-b`, `a*b`, `a%b`, `a<<b`, `a<b`, `a==b`, `a||b`, `a&&b`,
  `a^b`, `a + b * c - d`, `a ? b + c : d * e`, `(a + b) * c`.
- **`identifier.text` empty (Issue B).** `identifier := trivia /re/ -> {type:"identifier", text:$1}` —
  `$1` was the leading `trivia`, so every identifier `text` was `""`.
  **Fix:** `text: $2`. Verified `x1` → `{"type":"identifier","text":"x1"}`.
- **`literal.text` envelope (Issue C).** `based_integer` /
  `decimal_integer` (`:= trivia /re/`) were unannotated, so
  `literal.text` surfaced the envelope `["", "42"]`. **Fix:** annotate
  both leaves `-> $2`; `literal.text` is now a clean string. Verified
  `42` → `{"type":"literal","kind":"decimal","text":"42"}` and
  `8'hFF` → `kind:"based","text":"8'hFF"`.

Annotation count: **26** (was 24; +2 = the `based_integer` /
`decimal_integer` `-> $2` leaf scalar captures). 18 distinct rules; 19
`return_object` + 7 `return_scalar`. Same accept set (no grammar
acceptance change — purely annotation shaping). Schema bumped `1 → 2`
because `binop_chain.rest`, `identifier.text`, and `literal.text` all
changed shape in consumer-visible ways. Gate-locked:
`cargo test --lib --features generated_parsers rtl_const_expr_ast_shape_contract`
(samples=2 aligned=2, regression_lock_failures=0) and
`make -C rust SHELL=/opt/homebrew/bin/bash rtl_const_expr_parser_book_gate`.

> **Systemic note:** the inline-operator-alternation antipattern that
> caused Issue A also exists in `grammars/rtl_frontend.ebnf` and
> `grammars/vhdl.ebnf` binop_chain levels (same `<invalid_sequence_access>`
> empirically confirmed for rtl_frontend `a + b`). Those families'
> corrections are tracked separately as their own slices + bug-ledger
> entries; this release fixes rtl_const_expr only.

## Release 1.0.1 / Contract 1.0.1 Highlights — RTL-CE-Slice-1: binop_chain hierarchy typed (10 rules / 10 annotations)

Initial slice landed on 2026-05-14 (required the codegen outer-branch remap fix `PGEN-PIP-001` to make Pattern-A `digit ( sep | digit )*` work; see `feedback_codegen_outer_branch_remap.md`). The 10 binary-operator chain rules now emit a uniform left-fold shape:

```ebnf
# File root (pre-existing)
rtl_const_expr           -> {type: "rtl_const_expr", expr}

# Expression hierarchy (10 binop_chain levels, slice 1)
logical_or_expr          -> {type: "binop_chain", level: "logical_or",     lhs, rest}
logical_and_expr         -> {type: "binop_chain", level: "logical_and",    lhs, rest}
bit_or_expr              -> {type: "binop_chain", level: "bit_or",         lhs, rest}
bit_xor_expr             -> {type: "binop_chain", level: "bit_xor",        lhs, rest}
bit_and_expr             -> {type: "binop_chain", level: "bit_and",        lhs, rest}
equality_expr            -> {type: "binop_chain", level: "equality",       lhs, rest}
relational_expr          -> {type: "binop_chain", level: "relational",     lhs, rest}
shift_expr               -> {type: "binop_chain", level: "shift",          lhs, rest}
additive_expr            -> {type: "binop_chain", level: "additive",       lhs, rest}
multiplicative_expr      -> {type: "binop_chain", level: "multiplicative", lhs, rest}

# Pre-existing shapes (foundation baseline)
conditional_expr         -> {type: "ternary", condition, then_expr, else_expr}    | passthrough
unary_expr               -> {type: "unary", op: "plus"|"minus"|"logical_not"|"bit_not", expr}    | passthrough
primary_expr             -> passthrough on literal/identifier; {kind, expr} on lparen-rparen
literal                  -> {type: "literal", kind: "based"|"decimal", text}
identifier               -> {type: "identifier", text}
```

### Consumer guidance: the `binop_chain` shape

All 10 binary-operator chain rules emit the same shape: `{type: "binop_chain", level, lhs, rest}` where:

- `lhs` is the leading operand at this precedence level (typed value from the next-lower level).
- `rest` is the iteration array of `(op, operand)` pairs from `(op X)*`.

To evaluate, consumers fold left:

```pseudocode
value = lhs
for (op, operand) in rest:
    value = apply(op, value, operand)
```

`level` discriminates which operator family the node belongs to (e.g. "logical_or", "additive") so consumers can validate operator-vs-level conformance.

Annotation count: **24** (was 14 / foundation baseline). Same accept set.

## Scope / Non-Goals
- The stable downstream contract is the host-oriented embedding API, not internal generated parser modules or internal AST types.
- `rtl_const_expr` covers only **constant expressions** (decimal and sized-based integer literals, identifiers, unary `+ - ! ~`, binary arithmetic / shift / comparison / equality / bitwise / logical operators, ternary `?:`). For statements, modules, control flow → see `rtl_frontend`.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
