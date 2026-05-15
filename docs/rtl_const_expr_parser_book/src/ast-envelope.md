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

## The root rule

The rtl_const_expr grammar root is `rtl_const_expr`, which immediately delegates to `conditional_expr`:

```ebnf
rtl_const_expr := conditional_expr
```

## The 10-level precedence-climbing binop chain

The rtl_const_expr grammar is a precedence-climbing chain of left-associative binary operators plus a prefix-unary plus primary. The 10 binary levels (highest to lowest precedence) are:

1. `multiplicative_expr` — `*`, `/`, `%`
2. `additive_expr` — `+`, `-`
3. `shift_expr` — `<<`, `>>`
4. `relational_expr` — `<`, `<=`, `>`, `>=`
5. `equality_expr` — `==`, `!=`
6. `bit_and_expr` — `&`
7. `bit_xor_expr` — `^`
8. `bit_or_expr` — `|`
9. `logical_and_expr` — `&&`
10. `logical_or_expr` — `||`

Each level produces a typed `binop_chain` shape:

```json
{
  "type": "binop_chain",
  "level": "<level-name>",
  "lhs": <next-level-shape>,
  "rest": [
    {"op": "<operator>", "rhs": <next-level-shape>},
    {"op": "<operator>", "rhs": <next-level-shape>}
  ]
}
```

`rest` is empty when the input had no operator at that level — in that case the consumer simply unwraps `lhs`.

## conditional_expr (ternary)

Above the 10 binary levels is the ternary `?:`:

```ebnf
conditional_expr := logical_or_expr question conditional_expr colon conditional_expr
conditional_expr := logical_or_expr
```

The typed shape is a `kind`-tagged variant:

- `{kind: "ternary", cond, then, else}` when `?:` is present.
- `{kind: "passthrough", expr}` when only the LHS matched.

## unary_expr (prefix)

Below the 10 binary levels is the unary prefix layer:

```ebnf
unary_expr := plus  unary_expr   -> {kind: "plus",  expr: $2}
            | minus unary_expr   -> {kind: "minus", expr: $2}
            | bang  unary_expr   -> {kind: "bang",  expr: $2}
            | tilde unary_expr   -> {kind: "tilde", expr: $2}
            | primary_expr       -> {kind: "primary", expr: $1}
```

## primary_expr

The leaf layer is `primary_expr`, dispatching on literal vs identifier vs parenthesized expression.

## Consumer fold for binop_chain

```rust
fn fold(node: &serde_json::Value) -> EvaluationTree {
    let kind = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if kind != "binop_chain" {
        return interpret_other(node);
    }
    let mut acc = fold(node.get("lhs").unwrap());
    let rest = node.get("rest").and_then(|v| v.as_array()).unwrap();
    for step in rest {
        let op = step.get("op").and_then(|v| v.as_str()).unwrap();
        let rhs = fold(step.get("rhs").unwrap());
        acc = EvaluationTree::Binop { op: op.to_string(), lhs: Box::new(acc), rhs: Box::new(rhs) };
    }
    acc
}
```

See [Walking the AST](walking-the-ast.md) for the full walker pattern, including the binop_chain consumer-fold across the 10 levels.

## Two carrier kinds: typed and recursive-envelope

Per-rule, the AST dump produces JSON in one of two shapes:

- **Typed shape** — rules with `-> {...}` annotations (binop_chain levels, unary_expr, primary_expr, etc.).
- **Recursive-envelope shape** — rules without annotations produce a JSON value derived from grammar shape (sequence → array, alternation → matched-branch shape, etc.).

The 10-annotation surface (as of contract 1.0.1) covers all binary chain levels. The remaining rules produce recursive-envelope arrays.

## Determinism

The AST dump is **deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical (no trailing zeros for integers, etc.).
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed) but the underlying JSON value is the same.

Any non-determinism in the dump is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Truncation

If `AstDumpOptions.max_ast_bytes` is set and the encoded JSON exceeds it, the dump is truncated and `truncated: true` is set on the payload. The truncated payload is still valid JSON (the truncation happens at a node boundary). Consumers should check the `truncated` flag and either bail or note the truncation.
