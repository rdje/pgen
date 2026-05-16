# Top-Level Rules

This chapter is the per-rule shape reference for the PGEN rtl_const_expr parser. It documents the `rtl_const_expr` root, the conditional / `binop_chain` / unary / primary expression hierarchy, and the `literal` and `identifier` leaf shapes, grouped by rule family.

> **Status:** RTL-CE-Slice-1 (parser release `1.0.1`, AST-dump schema version `1`) typed the full `grammars/rtl_const_expr.ebnf` expression surface in a single slice — **24 return annotations on 16 distinct rules**. Every shape in this chapter is drawn from the live inventory at `generated/rtl_const_expr_return_annotations.json` (cross-checked against the embedded inventory in `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` — content-identical: the `(rule, branch_index, annotation_type, normalized_text)` tuples match exactly; the live inventory additionally carries a per-entry `raw_text` field). That artifact, not this prose, is the machine-checkable source of truth.

## How to read this chapter

This is a **curated, grouped** reference — not a raw 24-line dump and not a copy of any RTL language spec. For each family it gives the `type` / `kind` discriminators and field lists the parser actually emits, transcribed from each rule's `normalized_text`. Where a rule has per-branch typing, the discriminator value names the matched branch; where a rule has a single sequence shape, the named fields are listed directly.

Two conventions appear throughout:

- **Typed object rules** emit an object literal with a `type` discriminator (`{type: "binop_chain", ...}`, `{type: "ternary", ...}`, `{type: "unary", ...}`, `{type: "literal", ...}`, `{type: "identifier", ...}`, `{type: "rtl_const_expr", ...}`). rtl_const_expr does **not** use a bare `kind` discriminator anywhere; every typed object carries `type`.
- **Passthrough branches** carry a `return_scalar` annotation (`-> $1` / `-> $2`): the rule contributes no wrapper of its own and surfaces the matched sub-rule's shape directly. rtl_const_expr has exactly **five** passthrough branches (`conditional_expr` branch 1, `primary_expr` branches 0/1/2, `unary_expr` branch 4).

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

Below `conditional_expr` the rtl_const_expr grammar is a **ten-level operator-precedence cascade**. Each of the ten binary levels carries the same `binop_chain` typed shape (independently counted from the inventory: exactly ten rules emit `{type: "binop_chain", ...}`):

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
equality_expr       := relational_expr  ( ( eqeq | ne ) relational_expr )*
                    -> {type: "binop_chain", level: "equality",       lhs: $1, rest: $2}
relational_expr     := shift_expr       ( ( le | lt | ge | gt ) shift_expr )*
                    -> {type: "binop_chain", level: "relational",     lhs: $1, rest: $2}
shift_expr          := additive_expr    ( ( shl | shr ) additive_expr )*
                    -> {type: "binop_chain", level: "shift",          lhs: $1, rest: $2}
additive_expr       := multiplicative_expr ( ( plus | minus ) multiplicative_expr )*
                    -> {type: "binop_chain", level: "additive",       lhs: $1, rest: $2}
multiplicative_expr := unary_expr       ( ( star | slash | percent ) unary_expr )*
                    -> {type: "binop_chain", level: "multiplicative", lhs: $1, rest: $2}
```

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

**Consumer-facing left-fold contract** (per the integration contract, Release 1.0.1 Highlights): every one of these ten rules emits `{type: "binop_chain", level, lhs, rest}` where:

- `lhs` is the leading operand at this precedence level — the typed value from the next-tighter level (`logical_or_expr.lhs` is a `logical_and_expr` `binop_chain`, and so on down to `multiplicative_expr.lhs`, a `unary_expr`).
- `rest` is the recursive-envelope iteration array of the `( op operand )*` tail. Each entry wraps one operator token plus one next-level operand. **It is not a typed `{op, rhs}` object** — it is the raw envelope of the iteration (see [The Json Carrier](json-carrier.md)). When the input had no operator at this level, `rest` is the empty array `[]` and the consumer simply unwraps `lhs`.
- `level` discriminates which operator family the node belongs to (e.g. `"logical_or"`, `"additive"`) so consumers can validate operator-vs-level conformance without re-deriving the tier from context.

To evaluate, consumers fold `rest` left-associatively onto `lhs`:

```pseudocode
value = lhs
for (op, operand) in rest:
    value = apply(op, value, operand)
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
| `literal` (2 kinds) | `{type: "literal", kind: "based", text: $1}` for a sized/based integer (branch 0; e.g. `8'hFF`, `4'b1010`, `12'sd5`); `{type: "literal", kind: "decimal", text: $1}` for a plain decimal integer (branch 1; e.g. `42`, `1_000`). `kind` is the literal-class discriminator; `text` is the matched literal source. |
| `identifier` | `{type: "identifier", text: $1}` — a single typed object. The grammar's `identifier` regex covers plain, dotted (`a.b`), and package-qualified (`pkg::name`) names; the whole matched span is the `text` value. |

The `text` field on `literal` and `identifier` is bound to `$1`, the matched terminal/regex span. Note this is the one place rtl_const_expr surfaces a leaf as a typed `{type, text}` object: `literal` and `identifier` are **annotated**, so reaching one yields a typed object whose `text` field carries the source text.

> **Un-annotated leaves are envelopes, not bare strings.** Sub-rules without their own return annotation (the keyword/operator tokens `plus`, `question`, `lparen`, …, the regex leaves `based_integer` / `decimal_integer`, and `trivia`) surface as their **recursive envelope** when reached, not as a bare JSON string. In the typed surface those leaves are absorbed into their annotated parent (`literal.text`, `identifier.text`, the dropped `( )` of `primary_expr` branch 2). Do not assume a value is a bare string unless it is the `text` field of a `literal` / `identifier` object — see [The Json Carrier](json-carrier.md) for the rule of thumb and a worked example.

## Total surface and the machine-checkable source

The full typed surface as of contract `1.0.1` is **24 return annotations across 16 distinct rules** (independently re-counted from the inventory: 19 `return_object` + 5 `return_scalar`; 10 of the 16 rules emit `binop_chain`). This chapter is a curated grouping; the authoritative, machine-checkable enumeration of every `(rule, branch_index, annotation_type, normalized_text)` tuple is:

- `generated/rtl_const_expr_return_annotations.json` — the live return-annotation inventory (`version: 1`, `grammar: "rtl_const_expr"`, `annotation_count: 24`).
- `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` — the embedded inventory used by the AST shape-contract regression lock (content-identical; 24 entries in `declared_annotation_inventory.annotations`, with a per-entry `raw_text` only on the live inventory).

If this chapter and either artifact disagree, the artifact wins — and the integration contract `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` wins over both.

## How to follow per-slice changes

Each shape-affecting slice after RTL-CE-Slice-1 gets a row in [Schema Versioning](schema-versioning.md) and a Highlights section in `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`. The [Changelog Index](changelog-index.md) ties them together. For two end-to-end captured shapes, see [Literal 42](examples-literal-42.md) and [Binary Addition](examples-binary-addition.md).
