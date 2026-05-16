# The Json Carrier

This chapter explains how the rtl_const_expr parser carries AST shapes in the JSON dump: the two carrier kinds, how typed shapes and recursive-envelope shapes coexist, the object / array / string / scalar mapping, the absent-optional convention, and the determinism guarantee. It is the conceptual companion to [Top-Level Rules](rules-top-level.md), which enumerates the concrete per-rule shapes, and to [AST Envelope Structure](ast-envelope.md), which documents the outer `AstDumpPayload`.

> **Note:** The rtl_const_expr return-annotation campaign landed in a single slice — **RTL-CE-Slice-1, 24 annotations on 16 distinct rules** (parser release `1.0.1`, schema version `1`). The full expression hierarchy is typed; the remaining un-annotated rules (the keyword/operator tokens, the `based_integer` / `decimal_integer` regex leaves, and `trivia`) produce the recursive-envelope shape but, in the typed surface, are absorbed into their annotated parent. The authoritative enumeration is `generated/rtl_const_expr_return_annotations.json` (mirrored content-identically by the embedded inventory in `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` — the `(rule, branch_index, annotation_type, normalized_text)` tuples match exactly; the live inventory additionally carries a per-entry `raw_text` field).

## Two carrier kinds

Every node in the rtl_const_expr AST dump is carried in one of exactly two shapes.

### Typed shape (rules with a `-> ...` return annotation)

When a rule in `grammars/rtl_const_expr.ebnf` carries a return annotation, the parser emits a typed JSON value derived from that annotation. For rtl_const_expr this takes two sub-forms:

1. **`type`-tagged object** (`annotation_type: "return_object"`, 19 of the 24 annotations). Every typed object in rtl_const_expr carries a `type` discriminator — there is no bare `kind` dispatcher anywhere in this grammar. The root and every interior expression node use `type`:

   ```json
   { "type": "rtl_const_expr", "expr": { /* conditional_expr shape */ } }
   ```
   ```json
   { "type": "binop_chain", "level": "additive", "lhs": { /* ... */ }, "rest": [] }
   ```
   ```json
   { "type": "ternary",  "condition": { /* ... */ }, "then_expr": { /* ... */ }, "else_expr": { /* ... */ } }
   { "type": "unary",    "op": "minus", "expr": { /* ... */ } }
   { "type": "literal",  "kind": "decimal", "text": "42" }
   { "type": "identifier", "text": "WIDTH" }
   ```

   `literal` additionally carries a `kind` sub-discriminator (`"based"` / `"decimal"`); `binop_chain` carries a `level`; `unary` carries an `op`. These are secondary tags **inside** a `type`-tagged object, not a separate dispatch convention.

2. **Positional passthrough** (`annotation_type: "return_scalar"`, 5 of the 24 annotations: `conditional_expr` branch 1, `primary_expr` branches 0/1/2, `unary_expr` branch 4). The rule contributes no wrapper of its own; the node surfaces directly as whatever the referenced body element (`$1` or `$2`) produced. A non-ternary expression surfaces as its `logical_or_expr` `binop_chain`; a `primary_expr` surfaces as the underlying `literal` / `identifier` object or the inner parenthesized `conditional_expr`; a non-prefixed `unary_expr` surfaces as its `primary_expr`.

### Recursive-envelope shape (rules without annotations)

When a rule has no return annotation, the parser emits a JSON value derived mechanically from the rule's grammar shape:

- A **terminal literal** or **regex literal** produces a JSON string of the matched text.
- A **rule reference** produces whatever shape that rule produces.
- A **sequence** `a b c` produces a JSON array `[<a-shape>, <b-shape>, <c-shape>]`.
- An **alternation** produces the matched branch's shape directly.
- A **quantified rule** (`x*`, `x+`) produces a JSON array of the per-iteration shapes.
- An **optional rule** (`x?`) produces the matched shape if matched, or `[]` if un-matched.

In rtl_const_expr the recursive-envelope shape is what you reach in exactly two places:

- The **`rest` field of every `binop_chain`** — `rest` is bound to `$2`, the `( op operand )*` iteration. It is the recursive-envelope array of per-iteration shapes, **not** a typed `{op, rhs}` object. Each entry is the envelope of one `(operator operand)` iteration: the operator-token sub-shape followed by the next-level operand's shape.
- The **un-annotated leaves** (`plus`, `question`, `lparen`, … the keyword/operator tokens; `based_integer` / `decimal_integer`; `trivia`). In the typed surface these are absorbed into their annotated parent: `literal.text` and `identifier.text` carry the regex-matched string (bound to `$1`); `primary_expr` branch 2 drops the `( )` tokens entirely.

> **Rule of thumb — un-annotated leaves are envelopes, not bare strings.** A value reached through an un-annotated rule surfaces as that rule's **recursive envelope**, not a bare JSON string. The one place rtl_const_expr surfaces source text as a string is the `text` field of a `literal` or `identifier` object — those rules *are* annotated, and `text` is bound to the matched regex span. Do not assume any other slot is a scalar string; walk to the terminal (see the worked example and [Walking the AST](walking-the-ast.md#identifier-and-literal-extraction)).

## Object / array / string / scalar mapping

| JSON value | Produced by |
|---|---|
| Object `{...}` | A typed `return_object` annotation (`{type: "rtl_const_expr", ...}`, `{type: "binop_chain", ...}`, `{type: "ternary", ...}`, `{type: "unary", ...}`, `{type: "literal", ...}`, `{type: "identifier", ...}`). |
| Array `[...]` | A recursive-envelope sequence/quantified shape — most importantly the `rest` iteration of a `binop_chain` (the `( op operand )*` tail), and `rest == []` when no operator was present at that level. |
| String `"..."` | A matched terminal/regex leaf reached through an envelope — in the typed surface, the `text` field of a `literal` / `identifier` object (the matched literal or identifier source). |
| `true` / `false` | A `BooleanLiteral` in a typed annotation. The current 24-annotation rtl_const_expr surface uses **no** boolean literals; all 19 `return_object` annotations are object literals and the 5 `return_scalar` annotations are positional passthroughs. |
| Number | A `NumberLiteral` typed transform. Not used by the current rtl_const_expr surface — integer literals are carried as the string `text` of a `literal` object, not as JSON numbers. |
| `null` | A `NullLiteral`. **Not used** by the current rtl_const_expr surface — an absent iteration is carried as the empty array `[]`, never `null`. |

Two rtl_const_expr-specific points consumers must internalize:

- **An empty `rest` is `[]`, never `null` and never absent.** Every `binop_chain` object always has a `rest` key. When the input had no operator at that precedence level, `rest` is the empty array `[]` and the node is a pure wrapper around `lhs`. Test for an empty array (and unwrap `lhs`), not for `null` or a missing key. This is the only "absent-optional"-style convention in rtl_const_expr — the grammar has no `x?` optionals, so `[]` always means "an empty `*` iteration", specifically an empty `binop_chain` `rest`.
- **There is no `{first, rest}` list convention.** Unlike rtl_frontend / VHDL, rtl_const_expr has no separated-list rules. The only iteration in the grammar is the `( op operand )*` tail inside each `binop_chain`, carried as the raw recursive-envelope `rest` array. Consumers descend through that envelope per iteration (operator sub-shape then operand) rather than reading a flattened pair array — see [Walking the AST](walking-the-ast.md#folding-the-binop_chain-expression-hierarchy).

## Worked example: a decimal literal

Input:

```text
42
```

Walking the typed surface from the root:

- The root is the **typed root object** `{ "type": "rtl_const_expr", "expr": <conditional_expr> }`.
- `conditional_expr` has no `?:`, so its branch-1 **passthrough** surfaces the `logical_or_expr` `binop_chain` directly with no `ternary` wrapper.
- Each of the ten `binop_chain` levels has no operator, so each is `{ "type": "binop_chain", "level": "<level>", "lhs": <next>, "rest": [] }` — `rest` is the **empty array `[]`**, and the node is a pure wrapper around `lhs`. The cascade nests ten deep down to `multiplicative_expr`.
- `multiplicative_expr.lhs` is a `unary_expr`. With no prefix operator, `unary_expr` branch 4 is **passthrough** to `primary_expr`.
- `primary_expr` branch 0 is **passthrough** to `literal`.
- `literal` matched `decimal_integer`, so it is the **typed object** `{ "type": "literal", "kind": "decimal", "text": "42" }`. `text` is the one place a bare string appears — the matched regex span `"42"`, bound to `$1`.

So `42` produces a `rtl_const_expr` root whose `expr` is a ten-deep stack of empty `binop_chain` wrappers bottoming out at `{type: "literal", kind: "decimal", text: "42"}`. The full captured payload is in [Literal 42](examples-literal-42.md); a chain with an actual operator (non-empty `rest`) is in [Binary Addition](examples-binary-addition.md). The field names above come straight from the live inventory's `normalized_text` — confirm structure against the inventory and the live parser, never assume a scalar where a typed object or envelope is produced.

## The expression family (`binop_chain`)

The ten precedence levels (`logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr`) all carry the same `binop_chain` carrier so a single consumer fold handles the whole tree:

```json
{ "type": "binop_chain", "level": "additive", "lhs": <next-level-shape>, "rest": [ /* (op, operand) iterations */ ] }
```

`lhs` is the leading operand (itself a `binop_chain` of the next-tighter level, bottoming out at `unary_expr` → a typed `primary_expr` resolution). `rest` is a recursive-envelope array of `( operator operand )` iterations — **not** a typed `{op, rhs}` object. There is **no** `sign` field on any rtl_const_expr level (unlike VHDL's `simple_expression`); prefix `+` / `-` / `!` / `~` are handled by the `unary_expr` rule below the cascade. Consumers fold `rest` left-associatively onto `lhs`. The full level/field/operator table is in [Top-Level Rules](rules-top-level.md#family-expressions--the-binop_chain-contract); the fold code is in [Walking the AST](walking-the-ast.md).

`conditional_expr` (branch 1) and `unary_expr` (branch 4) are **passthrough** when their distinguishing syntax is absent (the `-> $1` `return_scalar` branches), and all three `primary_expr` branches are passthrough; a non-ternary, non-prefixed expression surfaces directly as its `logical_or_expr` `binop_chain`, and a leaf operand surfaces as its `literal` / `identifier` object. Consumers must therefore accept a `type`-tagged object (`binop_chain` / `ternary` / `unary` / `literal` / `identifier`) in any expression slot — dispatch on `obj["type"]`.

## Determinism

The AST dump is **byte-deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical.
- No embedded timestamps or hashes.
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed), but the underlying JSON value is identical.

Re-running the parse on the same input produces an identical JSON value. This is a **hard guarantee** of the schema. Any non-determinism is a bug — report it via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Un-annotated-on-purpose rules

Some rules remain un-annotated by design — the keyword/operator tokens (`plus`, `minus`, `question`, `colon`, `lparen`, …), the regex leaves (`based_integer`, `decimal_integer`), and `trivia`. In the typed surface their text is absorbed into the nearest typed parent: a literal's source is `literal.text`, an identifier's source is `identifier.text`, a parenthesized expression drops its `( )` and surfaces the inner `conditional_expr`. The operator tokens are reachable through the recursive-envelope walk of a `binop_chain`'s `rest` iteration. There is no rtl_const_expr rule that surfaces a bare keyword/operator string at the typed level.

## How to read the annotation text

The `normalized_text` field of each inventory entry is the EBNF `-> ...` clause:

- `$N` — positional reference to the Nth body element (1-indexed).
- `{field: value, ...}` — typed object literal (`annotation_type: "return_object"`; 19 entries).
- `$N` alone — positional passthrough (`annotation_type: "return_scalar"`; rtl_const_expr uses exactly five: `conditional_expr` branch 1, `primary_expr` branches 0/1/2, `unary_expr` branch 4).
- `"text"` — string literal (the `level` / `kind` / `op` discriminator values).

The full annotation-language grammar is `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`. The per-rule shapes those annotations produce are tabulated in [Top-Level Rules](rules-top-level.md).
