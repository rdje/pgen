# AST Envelope Structure

The PGEN rtl_const_expr parser produces a JSON AST through `parse_grammar_profile_ast_dump_named` (grammar `"rtl_const_expr"`). This chapter documents the top-level structure of that JSON.

## The envelope

The AST-dump host entry points (the generic
`parse_grammar_profile_ast_dump*` family and the named-result form
`parse_grammar_profile_ast_dump_named`) return — on success — an
`AstDumpPayload`, defined in `rust/src/embedding_api.rs` (the
authoritative API contract is `rust/docs/EMBEDDING_API_CONTRACT.md`). It
is a **canonical-JSON payload string plus truncation metadata** — it has
**exactly four fields**:

```rust
pub struct AstDumpPayload {
    pub dump_json: String,    // canonical (key-sorted) JSON encoding of the typed AST
    pub truncated: bool,      // true if max_ast_bytes was exceeded
    pub full_bytes: usize,    // byte length of the full encoded AST (pre-truncation)
    pub emitted_bytes: usize, // byte length actually placed in dump_json
}
```

`dump_json` is a **JSON string you must parse** (e.g.
`serde_json::from_str`) to obtain the `rtl_const_expr` **root object**
that this book's per-rule chapters describe. There is no `root` /
`schema_version` / `grammar` / `profile` field on `AstDumpPayload`
itself: the AST-dump schema version is the `2` tracked in
[Schema Versioning](schema-versioning.md) (the post-`1.0.2`
correctness-fix schema); the grammar/profile are fixed
(`rtl_const_expr` / `default`).

When `truncated` is `true`, `dump_json` is **not** the AST — it is a
deterministic truncation diagnostic envelope carrying
`pgen_dump_contract_version` (currently `1`),
`kind: "pgen_ast_dump_truncation"`, `truncated: true`,
`dump_kind: "parser_return_ast"`, `max_bytes`, `full_bytes`, and
`reason`. Consumers must check `truncated` (or, equivalently, the
presence of `pgen_dump_contract_version` / `kind ==
"pgen_ast_dump_truncation"` in the parsed `dump_json`) before treating
`dump_json` as an rtl_const_expr AST. If `max_ast_bytes` is too small to
fit even the diagnostic envelope, the API returns `E_INVALID_LIMITS`.
The downstream integration contract
`docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`
("AST Envelope and Expression Hierarchy") is the authoritative
restatement of this for consumers.

The parsed `dump_json` is what this book's per-rule chapters describe.

> **Authoritative shapes.** Every shape in this chapter is the live
> output of `generated/rtl_const_expr_parser.rs`, drawn from the
> return-annotation inventory `generated/rtl_const_expr_return_annotations.json`
> (content-mirrored by `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`).
> The per-rule enumeration and the consumer-facing fold live in
> [Top-Level Rules](rules-top-level.md) and [Walking the AST](walking-the-ast.md);
> this chapter is the structural overview. Where prose and the
> inventory disagree, the inventory wins.

## The root rule

The rtl_const_expr grammar root is the `rtl_const_expr` rule. It is
**typed** — it wraps the single `conditional_expr` child:

```ebnf
rtl_const_expr := conditional_expr
              -> {type: "rtl_const_expr", expr: $1}
```

```json
{ "type": "rtl_const_expr", "expr": { /* conditional_expr shape */ } }
```

Dispatch on `root["type"] == "rtl_const_expr"`, then descend into
`root["expr"]`.

## The typed surface at a glance

The full typed surface as of contract `1.0.2` is **26 return
annotations across 18 distinct rules** (19 `return_object`, 7
`return_scalar`). It is **not** a `kind`-tagged surface: every typed
object carries a `type` discriminator, never a bare `kind`. Of the 7
`return_scalar` annotations, five are passthrough branches
(`conditional_expr` branch 1, `primary_expr` branches 0/1/2,
`unary_expr` branch 4) that carry no wrapper and surface the matched
sub-rule's shape directly; the other two (`based_integer` /
`decimal_integer`, both `-> $2`) are leaf scalar captures feeding
`literal.text`. The five named operator alternation rules (`equality_op`,
`relational_op`, `shift_op`, `additive_op`, `multiplicative_op`) are
**un-annotated** and not part of the 26.

## conditional_expr (ternary)

The top of the expression hierarchy, two branches:

```ebnf
conditional_expr := logical_or_expr question conditional_expr colon conditional_expr
                 -> {type: "ternary", condition: $1, then_expr: $3, else_expr: $5}
conditional_expr := logical_or_expr
                 -> $1
```

- Branch 0 (`?:` present): `{type: "ternary", condition, then_expr,
  else_expr}`. Both arms recurse into `conditional_expr`.
- Branch 1 (no `?:`): **passthrough** — the node *is* the inner
  `logical_or_expr` `binop_chain` value, with **no** `conditional_expr`
  wrapper (not a `{kind: "passthrough"}` object). Any expression slot
  may therefore hold a `ternary` object *or* directly a `binop_chain` —
  dispatch on `obj["type"]`.

## The ten-level binop_chain cascade

Below `conditional_expr` is a ten-level left-associative
operator-precedence cascade. Each level emits the same `binop_chain`
shape:

```ebnf
<level>_expr := <next> ( <named_op> <next> )*
            -> {type: "binop_chain", level: "<level>", lhs: $1, rest: $2}
```

The ten levels (loosest to tightest binding, the order they nest):
`logical_or` → `logical_and` → `bit_or` → `bit_xor` → `bit_and` →
`equality` → `relational` → `shift` → `additive` → `multiplicative`.
The five inner multi-token operator alternations are lifted into named
rules (`equality_op := eqeq | ne`, `relational_op := le | lt | ge | gt`,
`shift_op := shl | shr`, `additive_op := plus | minus`,
`multiplicative_op := star | slash | percent`); the other five levels
use a single named token rule. This is the `1.0.2` correctness fix —
see [Schema Versioning](schema-versioning.md).

```json
{ "type": "binop_chain", "level": "<level>", "lhs": <next-level-shape>, "rest": [ [ <op-envelope>, <operand> ], ... ] }
```

`lhs` (`$1`) is the leading operand (itself the next tighter level,
bottoming out at a typed `primary_expr` leaf). `rest` (`$2`) is a
**clean array** of `( <named_op> <next> )` iterations — **not** an array
of typed `{op, rhs}` objects, and (as of `1.0.2`) never
`"<invalid_sequence_access>"`. Each entry is `[ <op-envelope>, <operand> ]`:
`entry[0]` is the operator envelope (`["", "+"]` for a `trivia "+"`
token; operator text at `entry[0][1]`) and `entry[1]` is the next-level
operand. When the input had no operator at that level, `rest` is the
empty array `[]` and the consumer simply unwraps `lhs`. Because each
entry is the `[op-envelope, operand]` pair (not a `{op, rhs}` object),
the fold must read `entry[0]` / `entry[1]` rather than
`step["op"]` / `step["rhs"]`; the single authoritative,
inventory-accurate fold is in
[Walking the AST](walking-the-ast.md#folding-the-binop_chain-expression-hierarchy)
— consumers should use that one rather than reimplement it here.

## unary_expr (prefix)

```ebnf
unary_expr := plus     unary_expr -> {type: "unary", op: "plus",        expr: $2}
            | minus    unary_expr -> {type: "unary", op: "minus",       expr: $2}
            | bang     unary_expr -> {type: "unary", op: "logical_not", expr: $2}
            | tilde    unary_expr -> {type: "unary", op: "bit_not",     expr: $2}
            | primary_expr        -> $1
```

Branches 0–3 emit `{type: "unary", op: <"plus"|"minus"|"logical_not"|"bit_not">, expr}`.
Branch 4 is **passthrough** — the node *is* the `primary_expr` value,
with no `unary` wrapper.

## primary_expr, literal, identifier (leaves)

`primary_expr` has three branches, **all passthrough** (`return_scalar`):
it never contributes a wrapper of its own — it surfaces the matched
literal, identifier, or parenthesized-expression shape directly. The two
typed leaves it bottoms out at are:

```ebnf
based_integer   := trivia /…/ -> $2     # clean digit string, feeds literal.text
decimal_integer := trivia /…/ -> $2     # clean digit string, feeds literal.text
literal    := based_integer   -> {type: "literal", kind: "based",   text: $1}
            | decimal_integer -> {type: "literal", kind: "decimal", text: $1}
identifier := trivia /…/      -> {type: "identifier", text: $2}
```

`literal.text` (`$1` = the `based_integer` / `decimal_integer` body
element) is now a **clean string** because those leaf rules are
annotated `-> $2` as of `1.0.2` (were unannotated, surfacing the
envelope `["", "42"]`). `identifier.text` binds `$2` (was `$1`, the
empty leading `trivia`) so it is the real name. Full per-branch field
lists are in [Top-Level Rules](rules-top-level.md).

## Two carrier kinds: typed and recursive-envelope

Per-rule, the AST dump produces JSON in one of two shapes:

- **Typed shape** — rules/branches with a `return_object` annotation
  (the root, `conditional_expr` branch 0, the ten `binop_chain` levels,
  `unary_expr` branches 0–3, `literal`, `identifier`).
- **Recursive-envelope shape** — un-annotated rules (including the five
  named operator alternation rules) and the `return_scalar` passthrough
  branches surface a JSON value derived from grammar shape (sequence →
  array, alternation → matched-branch shape, quantified → iteration
  array). Each `binop_chain` `rest` entry's `entry[0]` op-envelope and
  every un-annotated leaf (operator tokens, etc.) are envelope-shaped;
  see [The Json Carrier](json-carrier.md).

The 26-annotation / 18-distinct-rule surface (contract `1.0.2`) is
enumerated authoritatively in [Top-Level Rules](rules-top-level.md).

## Determinism

The AST dump is **deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical (no trailing zeros for integers, etc.).
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed) but the underlying JSON value is the same.

Any non-determinism in the dump is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Truncation

If `AstDumpOptions.max_ast_bytes` is set and the encoded JSON exceeds it, the dump is truncated and `truncated: true` is set on the payload. The truncated payload is still valid JSON (the truncation happens at a node boundary). Consumers should check the `truncated` flag and either bail or note the truncation.
