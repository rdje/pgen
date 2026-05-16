# AST Envelope Structure

The PGEN rtl_const_expr parser produces a JSON AST through `parse_grammar_profile_ast_dump_named` (grammar `"rtl_const_expr"`). This chapter documents the top-level structure of that JSON.

## The envelope

The `ast_dump` field of `NamedGrammarAstDumpOutcome` carries an `AstDumpPayload`:

```rust
pub struct AstDumpPayload {
    pub pgen_dump_contract_version: u32,  // currently 1
    pub schema_version: u32,              // 1 — see Schema Versioning
    pub grammar: String,                  // "rtl_const_expr"
    pub profile: String,                  // "default"
    pub root: JsonValue,                  // the actual AST tree
    pub truncated: bool,                  // true if max_ast_bytes was hit
}
```

The `root` field is the rtl_const_expr AST as a `serde_json::Value`. It is what this book's per-rule chapters describe.

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

The full typed surface as of contract `1.0.1` is **24 return
annotations across 16 distinct rules** (19 `return_object`, 5
`return_scalar` passthrough). It is **not** a `kind`-tagged surface:
every typed object carries a `type` discriminator, never a bare `kind`.
The five passthrough branches (`conditional_expr` branch 1,
`primary_expr` branches 0/1/2, `unary_expr` branch 4) carry no wrapper
of their own — they surface the matched sub-rule's shape directly.

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
<level>_expr := <next> ( <op> <next> )*
            -> {type: "binop_chain", level: "<level>", lhs: $1, rest: $2}
```

The ten levels (loosest to tightest binding, the order they nest):
`logical_or` → `logical_and` → `bit_or` → `bit_xor` → `bit_and` →
`equality` → `relational` → `shift` → `additive` → `multiplicative`.

```json
{ "type": "binop_chain", "level": "<level>", "lhs": <next-level-shape>, "rest": <rest-envelope> }
```

`lhs` (`$1`) is the leading operand (itself the next tighter level,
bottoming out at a typed `primary_expr` leaf). `rest` (`$2`) is the
**raw recursive-envelope iteration** of the `( <op> <next> )*` tail —
**not** an array of typed `{op, rhs}` objects. When the input had no
operator at that level, `rest` is the empty-iteration envelope and the
consumer simply unwraps `lhs`. Because `rest` is an envelope, the fold
must walk it rather than read `step["op"]` / `step["rhs"]`; the single
authoritative, inventory-accurate fold is in
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
literal    := ... -> {type: "literal", kind: "based",   text: $1}
            | ... -> {type: "literal", kind: "decimal", text: $1}
identifier := ... -> {type: "identifier", text: $1}
```

Full per-branch field lists are in [Top-Level Rules](rules-top-level.md).

## Two carrier kinds: typed and recursive-envelope

Per-rule, the AST dump produces JSON in one of two shapes:

- **Typed shape** — rules/branches with a `return_object` annotation
  (the root, `conditional_expr` branch 0, the ten `binop_chain` levels,
  `unary_expr` branches 0–3, `literal`, `identifier`).
- **Recursive-envelope shape** — un-annotated rules and the
  `return_scalar` passthrough branches surface a JSON value derived
  from grammar shape (sequence → array, alternation → matched-branch
  shape, quantified → iteration array). The `binop_chain` `rest` field
  and every un-annotated leaf (operator tokens, etc.) are
  envelope-shaped; see [The Json Carrier](json-carrier.md).

The 24-annotation / 16-distinct-rule surface (contract `1.0.1`) is
enumerated authoritatively in [Top-Level Rules](rules-top-level.md).

## Determinism

The AST dump is **deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical (no trailing zeros for integers, etc.).
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed) but the underlying JSON value is the same.

Any non-determinism in the dump is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Truncation

If `AstDumpOptions.max_ast_bytes` is set and the encoded JSON exceeds it, the dump is truncated and `truncated: true` is set on the payload. The truncated payload is still valid JSON (the truncation happens at a node boundary). Consumers should check the `truncated` flag and either bail or note the truncation.
