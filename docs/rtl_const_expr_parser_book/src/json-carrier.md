# The Json Carrier

This chapter explains how the rtl_const_expr parser carries AST shapes in the JSON dump: the two carrier kinds, how typed shapes and recursive-envelope shapes coexist, the object / array / string / scalar mapping, the absent-optional convention, and the determinism guarantee. It is the conceptual companion to [Top-Level Rules](rules-top-level.md), which enumerates the concrete per-rule shapes, and to [AST Envelope Structure](ast-envelope.md), which documents the outer `AstDumpPayload`.

> **Note:** parser release `1.0.2`, schema version `2`. RTL-CE-Slice-1 typed the full expression hierarchy; the `1.0.2` correctness fix brought the inventory to **26 annotations on 18 distinct rules**. The remaining un-annotated rules (the keyword/operator tokens, the five named operator alternation rules `equality_op` / `relational_op` / `shift_op` / `additive_op` / `multiplicative_op`, and `trivia`) produce the recursive-envelope shape but, in the typed surface, are absorbed into their annotated parent. The authoritative enumeration is `generated/rtl_const_expr_return_annotations.json` (mirrored content-identically by the embedded inventory in `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` — the `(rule, branch_index, annotation_type, normalized_text)` tuples match exactly; the live inventory additionally carries a per-entry `raw_text` field).

## Two carrier kinds

Every node in the rtl_const_expr AST dump is carried in one of exactly two shapes.

### Typed shape (rules with a `-> ...` return annotation)

When a rule in `grammars/rtl_const_expr.ebnf` carries a return annotation, the parser emits a typed JSON value derived from that annotation. For rtl_const_expr this takes two sub-forms:

1. **`type`-tagged object** (`annotation_type: "return_object"`, 19 of the 26 annotations). Every typed object in rtl_const_expr carries a `type` discriminator — there is no bare `kind` dispatcher anywhere in this grammar. The root and every interior expression node use `type`:

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

2. **Positional `return_scalar`** (`annotation_type: "return_scalar"`, 7 of the 26 annotations). Five are **positional passthroughs** (`conditional_expr` branch 1, `primary_expr` branches 0/1/2, `unary_expr` branch 4): the rule contributes no wrapper of its own; the node surfaces directly as whatever the referenced body element (`$1` or `$2`) produced. A non-ternary expression surfaces as its `logical_or_expr` `binop_chain`; a `primary_expr` surfaces as the underlying `literal` / `identifier` object or the inner parenthesized `conditional_expr`; a non-prefixed `unary_expr` surfaces as its `primary_expr`. The other two (`based_integer` / `decimal_integer`, both `-> $2`) are leaf **scalar captures**: each binds `$2`, the matched regex span, to a clean JSON string that feeds `literal.text` — they never surface on their own.

### Recursive-envelope shape (rules without annotations)

When a rule has no return annotation, the parser emits a JSON value derived mechanically from the rule's grammar shape:

- A **terminal literal** or **regex literal** produces a JSON string of the matched text.
- A **rule reference** produces whatever shape that rule produces.
- A **sequence** `a b c` produces a JSON array `[<a-shape>, <b-shape>, <c-shape>]`.
- An **alternation** produces the matched branch's shape directly.
- A **quantified rule** (`x*`, `x+`) produces a JSON array of the per-iteration shapes.
- An **optional rule** (`x?`) produces the matched shape if matched, or `[]` if un-matched.

In rtl_const_expr the recursive-envelope shape is what you reach in exactly two places:

- The **`rest` field of every `binop_chain`** — `rest` is bound to `$2`, the `( <named_op> operand )*` iteration. As of the `1.0.2` fix it is a **clean array** of per-iteration entries (it no longer emits `<invalid_sequence_access>`), **not** a typed `{op, rhs}` object. Each entry is a two-element array `[ <op-envelope>, <operand> ]`: `entry[0]` is the operator envelope (for a `trivia "<tok>"` operator token this is `["", "<tok>"]`, with the operator text at `entry[0][1]`); `entry[1]` is the next-level operand shape.
- The **un-annotated leaves** (`plus`, `question`, `lparen`, … the keyword/operator tokens; the five named operator alternation rules `equality_op` / `relational_op` / `shift_op` / `additive_op` / `multiplicative_op`; `trivia`). In the typed surface these are absorbed into their annotated parent: `literal.text` carries `based_integer` / `decimal_integer`'s clean `$2` capture, `identifier.text` carries the regex-matched string (bound to `$2`); `primary_expr` branch 2 drops the `( )` tokens entirely.

> **Rule of thumb — un-annotated leaves are envelopes, not bare strings.** A value reached through an un-annotated rule surfaces as that rule's **recursive envelope**, not a bare JSON string. rtl_const_expr surfaces source text as a clean string in the `text` field of a `literal` or `identifier` object, and in `based_integer` / `decimal_integer`'s `-> $2` capture (which feeds `literal.text`) — those rules *are* annotated. Do not assume any other slot (e.g. an operator token inside a `binop_chain` `rest` entry) is a scalar string; walk to the terminal (see the worked example and [Walking the AST](walking-the-ast.md#identifier-and-literal-extraction)).

## Object / array / string / scalar mapping

| JSON value | Produced by |
|---|---|
| Object `{...}` | A typed `return_object` annotation (`{type: "rtl_const_expr", ...}`, `{type: "binop_chain", ...}`, `{type: "ternary", ...}`, `{type: "unary", ...}`, `{type: "literal", ...}`, `{type: "identifier", ...}`). |
| Array `[...]` | A recursive-envelope sequence/quantified shape — most importantly the `rest` iteration of a `binop_chain` (the `( op operand )*` tail), and `rest == []` when no operator was present at that level. |
| String `"..."` | A matched terminal/regex leaf. In the typed surface this is the clean `text` field of a `literal` / `identifier` object (the matched literal or identifier source), or `based_integer` / `decimal_integer`'s `-> $2` capture feeding `literal.text`. The operator text inside a `binop_chain` `rest` entry is reachable as `entry[0][1]`. |
| `true` / `false` | A `BooleanLiteral` in a typed annotation. The current 26-annotation rtl_const_expr surface uses **no** boolean literals; all 19 `return_object` annotations are object literals and the 7 `return_scalar` annotations are positional passthroughs / leaf scalar captures. |
| Number | A `NumberLiteral` typed transform. Not used by the current rtl_const_expr surface — integer literals are carried as the clean string `text` of a `literal` object, not as JSON numbers. |
| `null` | A `NullLiteral`. **Not used** by the current rtl_const_expr surface — an absent iteration is carried as the empty array `[]`, never `null`. |

Two rtl_const_expr-specific points consumers must internalize:

- **An empty `rest` is `[]`, never `null` and never absent.** Every `binop_chain` object always has a `rest` key. When the input had no operator at that precedence level, `rest` is the empty array `[]` and the node is a pure wrapper around `lhs`. Test for an empty array (and unwrap `lhs`), not for `null` or a missing key. This is the only "absent-optional"-style convention in rtl_const_expr — the grammar has no `x?` optionals, so `[]` always means "an empty `*` iteration", specifically an empty `binop_chain` `rest`.
- **There is no `{first, rest}` list convention.** Unlike rtl_frontend / VHDL, rtl_const_expr has no separated-list rules. The only iteration in the grammar is the `( <named_op> operand )*` tail inside each `binop_chain`, carried as the clean `rest` array of `[ <op-envelope>, <operand> ]` entries. Consumers descend per entry (read the operator from `entry[0]`, the operand from `entry[1]`) rather than reading a flattened pair array — see [Walking the AST](walking-the-ast.md#folding-the-binop_chain-expression-hierarchy).

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
- `literal` matched `decimal_integer`, so it is the **typed object** `{ "type": "literal", "kind": "decimal", "text": "42" }`. `text` is `$1` — the value of `literal`'s `decimal_integer` body element, which is `decimal_integer`'s own `-> $2` clean capture, the string `"42"`. (Pre-`1.0.2` this was the envelope `["", "42"]` because `decimal_integer` was un-annotated; the `1.0.2` fix annotated it `-> $2`.)

So `42` produces a `rtl_const_expr` root whose `expr` is a ten-deep stack of empty `binop_chain` wrappers bottoming out at the captured node `{"kind": "decimal", "text": "42", "type": "literal"}` (object keys emitted alphabetically). The full captured payload is in [Literal 42](examples-literal-42.md); a chain with an actual operator (non-empty `rest`) is in [Binary Addition](examples-binary-addition.md). The field names above come straight from the live inventory's `normalized_text` — confirm structure against the inventory and the live parser, never assume a scalar where a typed object or envelope is produced.

## The expression family (`binop_chain`)

The ten precedence levels (`logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr`) all carry the same `binop_chain` carrier so a single consumer fold handles the whole tree:

```json
{ "type": "binop_chain", "level": "additive", "lhs": <next-level-shape>, "rest": [ [ ["", "+"], <operand> ] ] }
```

`lhs` is the leading operand (itself a `binop_chain` of the next-tighter level, bottoming out at `unary_expr` → a typed `primary_expr` resolution). `rest` is a **clean array** of `( <named_op> operand )` iterations — **not** a typed `{op, rhs}` object, and as of `1.0.2` no longer `<invalid_sequence_access>`. Each entry is `[ <op-envelope>, <operand> ]`: `entry[0]` is the operator envelope (`["", "+"]` for a `trivia "+"` token; operator text at `entry[0][1]`) and `entry[1]` is the next-level operand. There is **no** `sign` field on any rtl_const_expr level (unlike VHDL's `simple_expression`); prefix `+` / `-` / `!` / `~` are handled by the `unary_expr` rule below the cascade. Consumers fold `rest` left-associatively onto `lhs`. The full level/field/operator table is in [Top-Level Rules](rules-top-level.md#family-expressions--the-binop_chain-contract); the fold code is in [Walking the AST](walking-the-ast.md).

`conditional_expr` (branch 1) and `unary_expr` (branch 4) are **passthrough** when their distinguishing syntax is absent (the `-> $1` `return_scalar` branches), and all three `primary_expr` branches are passthrough; a non-ternary, non-prefixed expression surfaces directly as its `logical_or_expr` `binop_chain`, and a leaf operand surfaces as its `literal` / `identifier` object. Consumers must therefore accept a `type`-tagged object (`binop_chain` / `ternary` / `unary` / `literal` / `identifier`) in any expression slot — dispatch on `obj["type"]`.

## Determinism

The AST dump is **byte-deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical.
- No embedded timestamps or hashes.
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed), but the underlying JSON value is identical.

Re-running the parse on the same input produces an identical JSON value. This is a **hard guarantee** of the schema. Any non-determinism is a bug — report it via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Un-annotated-on-purpose rules

Some rules remain un-annotated by design — the keyword/operator tokens (`plus`, `minus`, `question`, `colon`, `lparen`, …), the five named operator alternation rules (`equality_op`, `relational_op`, `shift_op`, `additive_op`, `multiplicative_op`), and `trivia`. (The regex leaves `based_integer` / `decimal_integer` are **now annotated** `-> $2` as of `1.0.2` and feed `literal.text` as clean strings — they are no longer in this un-annotated set.) In the typed surface the leaf source text is absorbed into the nearest typed parent: a literal's source is `literal.text`, an identifier's source is `identifier.text`, a parenthesized expression drops its `( )` and surfaces the inner `conditional_expr`. The operator tokens are reachable through the recursive-envelope walk of a `binop_chain`'s `rest` iteration (`entry[0]` is the op-envelope). There is no rtl_const_expr rule that surfaces a bare keyword/operator string at the typed level.

## How to read the annotation text

The `normalized_text` field of each inventory entry is the EBNF `-> ...` clause:

- `$N` — positional reference to the Nth body element (1-indexed).
- `{field: value, ...}` — typed object literal (`annotation_type: "return_object"`; 19 entries).
- `$N` alone — positional `return_scalar` (7 entries): five positional passthroughs (`conditional_expr` branch 1, `primary_expr` branches 0/1/2, `unary_expr` branch 4) plus two leaf scalar captures (`based_integer` / `decimal_integer`, both `-> $2`).
- `"text"` — string literal (the `level` / `kind` / `op` discriminator values).

The full annotation-language grammar is `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`. The per-rule shapes those annotations produce are tabulated in [Top-Level Rules](rules-top-level.md).
