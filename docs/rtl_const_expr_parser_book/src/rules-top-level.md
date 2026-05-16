# Top-Level Rules

This chapter is the per-rule shape reference for the PGEN rtl_const_expr parser. It documents the `rtl_const_expr` root, the conditional / `binop_chain` / unary / primary expression hierarchy, and the `literal` and `identifier` leaf shapes, grouped by rule family.

> **Status:** parser release `1.0.2`, AST-dump schema version `2`. RTL-CE-Slice-1 typed the full `grammars/rtl_const_expr.ebnf` expression surface; the `1.0.2` correctness fix (named operator rules, clean `binop_chain` `rest`, clean `identifier` / `literal` text) brought the inventory to **26 return annotations on 18 distinct rules** (19 `return_object` + 7 `return_scalar`). Every shape in this chapter is drawn from the live inventory at `generated/rtl_const_expr_return_annotations.json` (cross-checked against the embedded inventory in `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` — content-identical: the `(rule, branch_index, annotation_type, normalized_text)` tuples match exactly; the live inventory additionally carries a per-entry `raw_text` field). That artifact, not this prose, is the machine-checkable source of truth.

## How to read this chapter

This is a **curated, grouped** reference — not a raw 26-line dump and not a copy of any RTL language spec. For each family it gives the `type` / `kind` discriminators and field lists the parser actually emits, transcribed from each rule's `normalized_text`. Where a rule has per-branch typing, the discriminator value names the matched branch; where a rule has a single sequence shape, the named fields are listed directly.

Two conventions appear throughout:

- **Typed object rules** emit an object literal with a `type` discriminator (`{type: "binop_chain", ...}`, `{type: "ternary", ...}`, `{type: "unary", ...}`, `{type: "literal", ...}`, `{type: "identifier", ...}`, `{type: "rtl_const_expr", ...}`). rtl_const_expr does **not** use a bare `kind` discriminator anywhere; every typed object carries `type`.
- **Passthrough / scalar-capture branches** carry a `return_scalar` annotation (`-> $1` / `-> $2`). Five are **passthroughs** that contribute no wrapper and surface the matched sub-rule's shape directly (`conditional_expr` branch 1, `primary_expr` branches 0/1/2, `unary_expr` branch 4). Two more (`based_integer` / `decimal_integer`, both `-> $2`) are leaf **scalar captures** that bind the matched regex span; they feed `literal.text` and never surface on their own. That is 7 `return_scalar` total.
- **Un-annotated alternation rules** — the five named operator rules `equality_op`, `relational_op`, `shift_op`, `additive_op`, `multiplicative_op` — carry **no** annotation and are therefore **not** in the 26-entry inventory. They exist only to lift the operator token out of each `binop_chain` tail so the iteration captures cleanly (see the `binop_chain` family below).

The annotation-language conventions (`$N`, `{field: value}`, string literals, `return_object` vs `return_scalar`) follow `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`.

## Entry point

| Profile | Entry rule | Description |
|---|---|---|
| `default` | `rtl_const_expr` | The synthesizable RTL constant-expression root. The only rtl_const_expr profile. |

There is no per-grammar convenience function for rtl_const_expr; the stable host surface is the generic `parse_grammar_profile*` family with `GrammarFamily::RtlConstExpr` + `GrammarProfile::Default` (and the `parse_grammar_profile_named` string overload with `"rtl_const_expr"` / `"default"`). See [Public API Surface](public-api.md).

## `rtl_const_expr` (root)

Per `grammars/rtl_const_expr.ebnf`:

```ebnf
rtl_const_expr := conditional_expr
              -> {type: "rtl_const_expr", expr: $1}
```

The annotation produces a typed JSON object at the root of every parse:

```json
{
  "type": "rtl_const_expr",
  "expr": { /* conditional_expr shape */ }
}
```

`expr` is the single typed child — whatever shape `conditional_expr` produces. Consumers walking the rtl_const_expr AST dispatch on `obj["type"] == "rtl_const_expr"` at the root, then descend into `obj["expr"]`. This is the only rule whose `type` is `"rtl_const_expr"`; every interior expression node carries a different `type`.

## Family: the conditional head

`conditional_expr` is the top of the expression hierarchy. It has two branches:

```ebnf
conditional_expr := logical_or_expr question conditional_expr colon conditional_expr
                 -> {type: "ternary", condition: $1, then_expr: $3, else_expr: $5}
conditional_expr := logical_or_expr
                 -> $1
```

| Branch | `annotation_type` | Shape |
|---|---|---|
| 0 | `return_object` | `{type: "ternary", condition: <logical_or_expr>, then_expr: <conditional_expr>, else_expr: <conditional_expr>}` — the `c ? t : e` form. Both arms (`then_expr`, `else_expr`) recurse into `conditional_expr`, so a ternary can nest. |
| 1 | `return_scalar` (`-> $1`) | **Passthrough.** A non-ternary expression surfaces directly as its `logical_or_expr` `binop_chain` with no `conditional_expr` wrapper. |

Because branch 1 is passthrough, consumers must accept that any expression slot (`rtl_const_expr.expr`, a ternary arm, a parenthesized sub-expression) may hold a `ternary` object **or** directly a `binop_chain` — dispatch on `obj["type"]` (see [Walking the AST](walking-the-ast.md)).

## Family: expressions — the `binop_chain` contract

Below `conditional_expr` the rtl_const_expr grammar is a **ten-level operator-precedence cascade**. Each of the ten binary levels carries the same `binop_chain` typed shape (independently counted from the inventory: exactly ten rules emit `{type: "binop_chain", ...}`). Every level's tail is `( <op> <next> )*` where `<op>` is a **named** rule, so the iteration captures cleanly as `rest: $2`:

```ebnf
logical_or_expr     := logical_and_expr ( logical_or  logical_and_expr )*
                    -> {type: "binop_chain", level: "logical_or",     lhs: $1, rest: $2}
logical_and_expr    := bit_or_expr      ( logical_and bit_or_expr     )*
                    -> {type: "binop_chain", level: "logical_and",    lhs: $1, rest: $2}
bit_or_expr         := bit_xor_expr     ( bit_or      bit_xor_expr    )*
                    -> {type: "binop_chain", level: "bit_or",         lhs: $1, rest: $2}
bit_xor_expr        := bit_and_expr     ( bit_xor     bit_and_expr    )*
                    -> {type: "binop_chain", level: "bit_xor",        lhs: $1, rest: $2}
bit_and_expr        := equality_expr    ( bit_and     equality_expr   )*
                    -> {type: "binop_chain", level: "bit_and",        lhs: $1, rest: $2}
equality_expr       := relational_expr  ( equality_op       relational_expr )*
                    -> {type: "binop_chain", level: "equality",       lhs: $1, rest: $2}
relational_expr     := shift_expr       ( relational_op     shift_expr )*
                    -> {type: "binop_chain", level: "relational",     lhs: $1, rest: $2}
shift_expr          := additive_expr    ( shift_op          additive_expr )*
                    -> {type: "binop_chain", level: "shift",          lhs: $1, rest: $2}
additive_expr       := multiplicative_expr ( additive_op    multiplicative_expr )*
                    -> {type: "binop_chain", level: "additive",       lhs: $1, rest: $2}
multiplicative_expr := unary_expr       ( multiplicative_op unary_expr )*
                    -> {type: "binop_chain", level: "multiplicative", lhs: $1, rest: $2}
```

The five inner operator alternations are lifted into **named, un-annotated** rules (this is the `1.0.2` correctness fix — see [Schema Versioning](schema-versioning.md)):

```ebnf
equality_op       := eqeq | ne
relational_op     := le | lt | ge | gt
shift_op          := shl | shr
additive_op       := plus | minus
multiplicative_op := star | slash | percent
```

| Named operator rule | Used by level | Token alternatives |
|---|---|---|
| `equality_op` | `equality_expr` | `==` / `!=` |
| `relational_op` | `relational_expr` | `<=` / `<` / `>=` / `>` |
| `shift_op` | `shift_expr` | `<<` / `>>` |
| `additive_op` | `additive_expr` | `+` / `-` |
| `multiplicative_op` | `multiplicative_expr` | `*` / `/` / `%` |

These five rules carry **no** return annotation (they are not in the 26-entry inventory). The remaining five single-token levels (`logical_or`, `logical_and`, `bit_or`, `bit_xor`, `bit_and`) already used a single named token rule and were unchanged. Lifting the operator into a named rule is what makes `rest: $2` capture as a clean per-iteration array instead of emitting `<invalid_sequence_access>` (the pre-`1.0.2` bug).

| Level (`level`) | Rule | Operators (lowest binding first; the table is in grammar/precedence order) |
|---|---|---|
| `"logical_or"` | `logical_or_expr` | `\|\|` |
| `"logical_and"` | `logical_and_expr` | `&&` |
| `"bit_or"` | `bit_or_expr` | `\|` |
| `"bit_xor"` | `bit_xor_expr` | `^` |
| `"bit_and"` | `bit_and_expr` | `&` |
| `"equality"` | `equality_expr` | `==` / `!=` |
| `"relational"` | `relational_expr` | `<=` / `<` / `>=` / `>` |
| `"shift"` | `shift_expr` | `<<` / `>>` |
| `"additive"` | `additive_expr` | `+` / `-` |
| `"multiplicative"` | `multiplicative_expr` | `*` / `/` / `%` |

**Consumer-facing left-fold contract** (per the integration contract): every one of these ten rules emits `{type: "binop_chain", level, lhs, rest}` where:

- `lhs` is the leading operand at this precedence level — the typed value from the next-tighter level (`logical_or_expr.lhs` is a `logical_and_expr` `binop_chain`, and so on down to `multiplicative_expr.lhs`, a `unary_expr`).
- `rest` is a **clean array** of per-iteration entries for the `( <op> <next> )*` tail. As of the `1.0.2` fix it no longer emits `<invalid_sequence_access>`. **Each entry is a two-element array `[ <op-envelope>, <operand> ]`:**
  - `entry[0]` is the operator envelope — the recursive envelope of the named op rule, which for the `trivia "<tok>"` operator tokens is the two-element array `["", "<tok>"]` (element 0 is the empty `trivia` capture, element 1 is the operator text). For `a + b` the additive `rest` is `[ [ ["", "+"], <operand> ] ]`; the operator text is at `entry[0][1]`.
  - `entry[1]` is the next-level operand — itself a `binop_chain` of the next-tighter level.
  - **It is not a typed `{op, rhs}` object** — it is the raw envelope of the iteration (see [The Json Carrier](json-carrier.md)). When the input had no operator at this level, `rest` is the empty array `[]` and the consumer simply unwraps `lhs`.
- `level` discriminates which operator family the node belongs to (e.g. `"logical_or"`, `"additive"`) so consumers can validate operator-vs-level conformance without re-deriving the tier from context.

To evaluate, consumers fold `rest` left-associatively onto `lhs`, reading each entry's operator from `entry[0]` and its operand from `entry[1]`:

```pseudocode
value = lhs
for entry in rest:                 # entry == [ op_envelope, operand ]
    op      = terminal_text(entry[0])   # e.g. "+" from ["", "+"]
    operand = entry[1]
    value   = apply(op, value, operand)
```

All ten levels iterate `*`, so `rest` may hold any number of pairs. There is **no** `sign` field on any rtl_const_expr level (unlike VHDL's `simple_expression`); the prefix `+` / `-` / `!` / `~` operators live in `unary_expr`, below `multiplicative_expr`. The cascade bottoms out at `unary_expr` → `primary_expr`. This `binop_chain` shape is identical across all ten levels precisely so that a single consumer fold routine handles the entire binary-expression tree. See [Walking the AST](walking-the-ast.md) for a worked left-fold.

## Family: the unary prefix tier

`unary_expr` sits below `multiplicative_expr` and has five branches:

```ebnf
unary_expr := plus  unary_expr  -> {type: "unary", op: "plus",        expr: $2}
unary_expr := minus unary_expr  -> {type: "unary", op: "minus",       expr: $2}
unary_expr := bang  unary_expr  -> {type: "unary", op: "logical_not", expr: $2}
unary_expr := tilde unary_expr  -> {type: "unary", op: "bit_not",     expr: $2}
unary_expr := primary_expr      -> $1
```

| Branch | `annotation_type` | Shape |
|---|---|---|
| 0 | `return_object` | `{type: "unary", op: "plus", expr: <unary_expr>}` (prefix `+`) |
| 1 | `return_object` | `{type: "unary", op: "minus", expr: <unary_expr>}` (prefix `-`) |
| 2 | `return_object` | `{type: "unary", op: "logical_not", expr: <unary_expr>}` (prefix `!`) |
| 3 | `return_object` | `{type: "unary", op: "bit_not", expr: <unary_expr>}` (prefix `~`) |
| 4 | `return_scalar` (`-> $1`) | **Passthrough.** A non-prefixed operand surfaces directly as its `primary_expr` shape with no `unary` wrapper. |

The four operator branches recurse through `expr` into `unary_expr` again (so `--x`, `!~x` chains nest). The `op` value is the typed operator name — `"plus"`, `"minus"`, `"logical_not"`, `"bit_not"` — **not** the raw token. Because branch 4 is passthrough, a non-prefixed operand surfaces as its `primary_expr` directly.

## Family: the primary tier

`primary_expr` is the leaf of the expression cascade. All three branches are `return_scalar` passthroughs:

```ebnf
primary_expr := literal                     -> $1
primary_expr := identifier                  -> $1
primary_expr := lparen conditional_expr rparen -> $2
```

| Branch | `annotation_type` | Shape |
|---|---|---|
| 0 | `return_scalar` (`-> $1`) | **Passthrough** to the `literal` typed object. |
| 1 | `return_scalar` (`-> $1`) | **Passthrough** to the `identifier` typed object. |
| 2 | `return_scalar` (`-> $2`) | **Passthrough** to the parenthesized inner `conditional_expr`. The surrounding `( )` tokens are dropped; the node surfaces as whatever `conditional_expr` produced. |

`primary_expr` therefore never appears as its own wrapper object in the dump — it always resolves to the typed shape of `literal`, `identifier`, or the parenthesized `conditional_expr`. Consumers must treat any operand slot as potentially holding a `literal`, an `identifier`, or any expression node (a parenthesized expression unwraps to the inner `conditional_expr`'s shape).

## Family: literals and identifiers

| Rule | Shape |
|---|---|
| `based_integer` (leaf scalar) | `based_integer := trivia /…/ -> $2`. `return_scalar` capturing `$2`, the matched regex span — a clean string (e.g. `"8'hFF"`). Feeds `literal.text` for the `"based"` branch; never surfaces on its own. |
| `decimal_integer` (leaf scalar) | `decimal_integer := trivia /…/ -> $2`. `return_scalar` capturing `$2`, the matched digits — a clean string (e.g. `"42"`). Feeds `literal.text` for the `"decimal"` branch; never surfaces on its own. |
| `literal` (2 kinds) | `{type: "literal", kind: "based", text: $1}` for a sized/based integer (branch 0; `$1` is `based_integer`'s clean `$2` capture — e.g. `8'hFF`, `4'b1010`, `12'sd5`); `{type: "literal", kind: "decimal", text: $1}` for a plain decimal integer (branch 1; `$1` is `decimal_integer`'s clean capture — e.g. `42`, `1_000`). `kind` is the literal-class discriminator; `text` is a clean source string. |
| `identifier` | `{type: "identifier", text: $2}` — a single typed object. `text` is bound to `$2` (the regex span); `$1` is the leading `trivia`. The grammar's `identifier` regex covers plain, dotted (`a.b`), and package-qualified (`pkg::name`) names; the whole matched span is the `text` value, e.g. `{"type":"identifier","text":"WIDTH"}`. |

The `text` field is a **clean JSON string** on both `literal` and `identifier`. The `1.0.2` fix made these correct: `identifier` now binds `text: $2` (was `$1`, which captured the empty leading `trivia`), and `based_integer` / `decimal_integer` are now annotated `-> $2`, so `literal.text` is the plain digit/literal string (was the envelope `["", "42"]`).

> **Un-annotated leaves are envelopes, not bare strings.** Sub-rules without their own return annotation (the keyword/operator tokens `plus`, `question`, `lparen`, …, the five named operator alternation rules `equality_op` / `relational_op` / `shift_op` / `additive_op` / `multiplicative_op`, and `trivia`) surface as their **recursive envelope** when reached, not as a bare JSON string. The operator tokens are reachable through the `binop_chain` `rest` walk (`entry[0]` is the op-envelope). In the typed surface the leaf source text is absorbed into its annotated parent (`literal.text`, `identifier.text`, the dropped `( )` of `primary_expr` branch 2). Note `based_integer` / `decimal_integer` *are* now annotated (`-> $2`), so their captured span is a clean string. See [The Json Carrier](json-carrier.md) for the rule of thumb and a worked example.

## Total surface and the machine-checkable source

The full typed surface as of contract `1.0.2` is **26 return annotations across 18 distinct rules** (independently re-counted from the inventory: 19 `return_object` + 7 `return_scalar`; 10 of the 18 rules emit `binop_chain`). The five named operator rules (`equality_op` … `multiplicative_op`) are **un-annotated** and therefore not part of the 26. This chapter is a curated grouping; the authoritative, machine-checkable enumeration of every `(rule, branch_index, annotation_type, normalized_text)` tuple is:

- `generated/rtl_const_expr_return_annotations.json` — the live return-annotation inventory (`version: 1`, `grammar: "rtl_const_expr"`, `annotation_count: 26`).
- `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` — the embedded inventory used by the AST shape-contract regression lock (content-identical; 26 entries in `declared_annotation_inventory.annotations`, with a per-entry `raw_text` only on the live inventory).

If this chapter and either artifact disagree, the artifact wins — and the integration contract `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` wins over both.

## How to follow per-slice changes

Each shape-affecting slice after RTL-CE-Slice-1 gets a row in [Schema Versioning](schema-versioning.md) and a Highlights section in `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`. The [Changelog Index](changelog-index.md) ties them together. For two end-to-end captured shapes, see [Literal 42](examples-literal-42.md) and [Binary Addition](examples-binary-addition.md).
