# Top-Level Rules

This chapter is the per-rule shape reference for the PGEN rtl_frontend parser. It documents the `rtl_frontend_file` root, the `design_item` dispatch, and then enumerates the typed rule shapes grouped by rule family.

> **Status:** RTL-FE-Slice-1..7 typed the full `grammars/rtl_frontend.ebnf` surface across seven slices — **156 return annotations on 74 distinct rules**. The current parser release is `1.0.3` / AST-dump schema version `3` (the `1.0.2` `RTL-FE-0001` correctness fix to `binop_chain.rest`, then the `1.0.3` POST-SV-AUDIT batch: 15 Category-A list-shape corrections + the `RTL-FE-0002` `event_control_list` inline-alternation fix — see [Schema Versioning](schema-versioning.md); the annotation inventory is **unchanged at 156 / 74**, since the `*_op` / `event_separator` rules are un-annotated alternations and the 15 Category-A rules changed annotation form not count). Every shape in this chapter is drawn from the live inventory at `generated/rtl_frontend_return_annotations.json` (cross-checked against the embedded inventory in `rust/test_data/ast_shape_contract/rtl_frontend_v1.json` — identical content, 156 entries). That artifact, not this prose, is the machine-checkable source of truth.

## How to read this chapter

This is a **curated, grouped** reference — not a raw 156-line dump and not a copy of any SystemVerilog LRM. For each family it gives the `kind` discriminators and field lists that the parser actually emits, transcribed from each rule's normalized return-annotation text. Where a rule has per-branch typing, the `kind` value names the matched branch; where a rule has a single sequence shape, the named fields are listed directly.

Three conventions appear throughout:

- **Dispatch rules** emit `{kind: "<branch>", body: $N}` (or named fields per branch). Consumers dispatch on `obj["kind"]`. The bodyless `{kind: "semi"}` branch carries a stray `;` with no `body`.
- **Separated-list rules** emit a **clean flat array** of the element type in source order — the canonical extraction-spread `[$N, $M::K*]` idiom (the semantically-irrelevant `,` / `or` / `::` separator is dropped). The five bare-list rules (`parameter_declaration_sequence`, `port_list`, `genvar_declaration`, `concatenation_expr`, `scoped_identifier`) emit a **top-level array**; the rules with a leading discriminator/type field (`port_group`, `module_instantiation`, `parameter_override_list`, `port_connection_list`, `net_declaration`, `assignment_target` concat, `repetition_expr`, `enum_type`, `struct_union_field`) keep those fields and carry the element list in a single named field (`ports` / `instances` / `items` / `names`). This is the `1.0.3` / schema `3` shape — the POST-SV-AUDIT.2.2 Category-A correction (`PGEN-POST-SV-AUDIT-0003`). At ≤ `1.0.2` / schema `2` these rules emitted the raw `{first, rest}` (resp. `{…, first, rest}`) envelope where `rest` was the raw `[[sep, item], …]` recursive iteration a consumer had to walk past; that history is kept in [Schema Versioning](schema-versioning.md). Consumers repinning to schema `3` treat the field (or the rule's whole value, for the bare-list rules) as a flat element array — no `.first` / `.rest` split, no separator to skip.
- **`binop_chain` expression rules** emit `{type: "binop_chain", level, lhs, rest}` — the consumer-facing left-fold contract documented below.

The annotation-language conventions (`$N`, `{field: value}`, `[...]`, string literals) follow `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`.

## Entry point

| Profile | Entry rule | Description |
|---|---|---|
| `default` | `rtl_frontend_file` | The synthesizable RTL subset source file. The only rtl_frontend profile. |

There is no per-grammar convenience function for rtl_frontend; the stable host surface is the generic `parse_grammar_profile*` family with `GrammarFamily::RtlFrontend` + `GrammarProfile::Default` (and the `parse_grammar_profile_named` string overload with `"rtl_frontend"` / `"default"`). See [Public API Surface](public-api.md).

## `rtl_frontend_file` (root)

Per `grammars/rtl_frontend.ebnf`:

```ebnf
rtl_frontend_file := trivia design_item* trivia
                  -> {type: "rtl_frontend_file", items: $2}
```

The annotation produces a typed JSON object at the root of every parse:

```json
{
  "type": "rtl_frontend_file",
  "items": [/* array of design_item shapes */]
}
```

`items` is the array of typed `design_item` objects, one per top-level design item in source order. Consumers walking the rtl_frontend AST dispatch on `obj["type"] == "rtl_frontend_file"` at the root, then iterate `obj["items"]`. This is the only rule that carries a `type` discriminator at the dispatch level; every other dispatcher uses `kind`.

## `design_item` dispatch

`design_item` is the primary top-level dispatcher — a 4-branch `kind`-tagged shape covering every rtl_frontend top-level form:

```ebnf
design_item := typedef_declaration -> {kind: "typedef", body: $1}
             | package_declaration -> {kind: "package", body: $1}
             | module_declaration  -> {kind: "module",  body: $1}
             | semi                -> {kind: "semi"}
```

| `kind` | `body` shape |
|---|---|
| `"typedef"` | `typedef_declaration` — `{data_type, packed_range, name}` |
| `"package"` | `package_declaration` — `{name, items}` |
| `"module"` | `module_declaration` — `{name, imports_pre, parameters, imports_post, ports, items}` |
| `"semi"` | _(no body — lone `;` separator)_ |

## Family: dispatch wrappers

Below `design_item` the grammar uses three more `{kind, body?}` dispatchers. All emit `{kind, body: $1}` per branch except the bodyless `{kind: "semi"}` branch.

| Rule | Branches | `kind` values |
|---|---|---|
| `module_item` | 10 | `"parameter"`, `"import"`, `"typedef"`, `"genvar"`, `"module_instantiation"`, `"net"`, `"continuous_assign"`, `"procedural_block"`, `"generate_region"`, `"semi"` |
| `generate_item` | 11 | `"parameter"`, `"import"`, `"typedef"`, `"genvar"`, `"module_instantiation"`, `"net"`, `"continuous_assign"`, `"procedural_block"`, `"generate_if"`, `"generate_for"`, `"semi"` |
| `package_item` | 3 | `"typedef"`, `"parameter"`, `"semi"` |

`module_item` and `generate_item` admit the same construct families but differ at the tail: `module_item` admits `"generate_region"`, while `generate_item` (used inside a generate region) admits `"generate_if"` / `"generate_for"` directly.

## Family: modules and packages

| Rule | Shape |
|---|---|
| `module_declaration` | `{name, imports_pre, parameters, imports_post, ports, items}` — `imports_pre` / `imports_post` are the pre- and post-parameter `import_declaration*` arrays; `parameters` is the optional `( # ( parameter_declaration_sequence )? )?` envelope (`[]` when absent); `ports` is the optional ANSI port-list envelope; `items` is the `module_item*` array. |
| `package_declaration` | `{name, items}` — `items` is the `package_item*` array. |
| `import_declaration` | `{package, member}` — `member` is the imported symbol or `*` wildcard. |
| `typedef_declaration` | `{data_type, packed_range, name}` — `packed_range` is `[]` when no `[msb:lsb]` is present. |
| `generate_region` | `{items}` — the `generate_item*` array of a `generate … endgenerate` block. |

## Family: generate constructs

| Rule | Shape |
|---|---|
| `generate_if` | `{cond, then_body, else_body}` — `else_body` is `[]` when there is no `else`. |
| `generate_for` | `{genvar, init_var, init_value, condition, step_var, step_value, body}`. |
| `generate_body` (2 kinds) | `{kind: "block", label, items}` for a `begin … end` (labeled) form / `{kind: "single", body}` for a single nested generate item. |

## Family: parameters

| Rule | Shape |
|---|---|
| `parameter_declaration_statement` | `{body}` — wraps a `parameter_declaration_sequence`. |
| `parameter_declaration_sequence` | `[parameter_declaration_group, …]` — clean flat array of the comma-separated `parameter_declaration_group` list (`1.0.3` / schema `3`; was `{first, rest}` at ≤ `1.0.2`). |
| `parameter_declaration_group` | `{head, tail}` — `head` carries the leading flavor/type, `tail` carries the continued declarators. |
| `parameter_declaration_head` (2 kinds) | `{kind: "typed", flavor, data_type, name, default}` / `{kind: "untyped", flavor, name, default}`. |
| `parameter_declaration_tail` (4 kinds) | `{kind: "typed_with_flavor", flavor, data_type, name, default}` / `{kind: "untyped_with_flavor", flavor, name, default}` / `{kind: "typed", data_type, name, default}` / `{kind: "untyped", name, default}`. |
| `parameter_flavor` (2 kinds) | `{kind: "parameter"}` / `{kind: "localparam"}` — bare `{kind}` keyword leaf. |
| `parameter_override` (2 kinds) | `{kind: "named", name, value}` / `{kind: "positional", value}`. |
| `parameter_override_list` (2 kinds) | `{kind: "named", items}` / `{kind: "positional", items}` — `items` is the clean flat `parameter_override[]` list (`1.0.3` / schema `3`; was `{kind, first, rest}` at ≤ `1.0.2`). |

The `default` field on every parameter shape is `[]` when no `= <expr>` initializer is present.

## Family: ports

| Rule | Shape |
|---|---|
| `port_list` | `[port_group, …]` — clean flat array of the comma-separated `port_group` list (`1.0.3` / schema `3`; was `{first, rest}` at ≤ `1.0.2`). |
| `port_group` | `{direction, data_type, packed_range, ports}` — `direction` is the typed `port_direction`; `data_type` / `packed_range` are `[]` when omitted; `ports` is the clean flat `port_item[]` declarator list (`1.0.3` / schema `3`; was `{direction, data_type, packed_range, first, rest}` at ≤ `1.0.2`). |
| `port_item` | `{name, dims}` — `dims` is `[]` when the port is unpacked-scalar. |
| `port_direction` (3 kinds) | `{kind: "input"}` / `{kind: "output"}` / `{kind: "inout"}` — bare `{kind}` keyword leaf. |
| `port_direction_token` (3 kinds) | `{kind: "input"}` / `{kind: "output"}` / `{kind: "inout"}` — the negative-lookahead guard token used in the port-continuation iteration; same `kind` set as `port_direction`. |

## Family: net / signal / instance declarations

| Rule | Shape |
|---|---|
| `net_declaration` | `{data_type, packed_range, items}` — `items` is the clean flat `net_item[]` declarator list (`1.0.3` / schema `3`; was `{data_type, packed_range, first, rest}` at ≤ `1.0.2`). |
| `net_item` | `{name, dims}` — `dims` is `[]` for a scalar net. |
| `genvar_declaration` | `[identifier, …]` — clean flat array of the comma-separated genvar identifiers (`1.0.3` / schema `3`; was `{first, rest}` at ≤ `1.0.2`). |
| `continuous_assign` | `{lvalue, value}` — `assign lvalue = value;`. |
| `module_instantiation` | `{module_name, parameters, instances}` — `parameters` is the optional `#( parameter_override_list? )` envelope (`[]` when absent); `instances` is the clean flat `instance_item[]` list (`1.0.3` / schema `3`; was `{module_name, parameters, first, rest}` at ≤ `1.0.2`). |
| `instance_item` | `{name, dims, connections}` — `dims` is `[]` for a non-array instance; `connections` is the `port_connection_list`. |

## Family: module instantiation port connections

| Rule | Shape |
|---|---|
| `port_connection_list` (2 kinds) | `{kind: "named", items}` / `{kind: "positional", items}` — `items` is the clean flat `port_connection[]` list (`1.0.3` / schema `3`; was `{kind, first, rest}` at ≤ `1.0.2`). |
| `port_connection` (3 kinds) | `{kind: "wildcard"}` (the `.*` form, bodyless) / `{kind: "named", name, value}` / `{kind: "positional", value}`. |

## Family: procedural / always blocks

| Rule | Shape |
|---|---|
| `procedural_block` (4 kinds) | `{kind: "always_comb", statement}` / `{kind: "always_latch", statement}` / `{kind: "always_ff", event_control, statement}` / `{kind: "always", event, statement}`. |
| `always_star_event` (2 kinds) | `{kind: "at_paren_star"}` (`@(*)`) / `{kind: "bare_star"}` (`@*`). |
| `event_control_list` | `[event_control_item, …]` — clean flat array of the comma/`or`-separated `event_control_item` list (`1.0.3` / schema `3`; was the corrupt `{first, rest}` that surfaced `"<invalid_sequence_access>"` for multi-entry sensitivity input at ≤ `1.0.2` — the `RTL-FE-0002` inline-alternation fix, the inline `( comma \| kw_or )` lifted to the un-annotated `event_separator` rule and dropped; see [Schema Versioning](schema-versioning.md)). |
| `event_control_item` | `{edge, expr}` — `edge` is the typed `event_edge` (`[]` for a level-sensitive signal). |
| `event_edge` (2 kinds) | `{kind: "posedge"}` / `{kind: "negedge"}` — bare `{kind}` keyword leaf. |
| `statement` (4 kinds) | `{kind: "semi"}` / `{kind: "block", label, items}` / `{kind: "if", cond, then_body, else_body}` / `{kind: "assignment", lvalue, operator, value}`. |
| `always_ff_statement` (4 kinds) | Same 4-kind shape as `statement` (`"semi"` / `"block"` / `"if"` / `"assignment"`); the `always_ff` body grammar restricts the assignment operator to the nonblocking arrow. |
| `assignment_operator` (2 kinds) | `{kind: "blocking"}` (`=`) / `{kind: "nonblocking"}` (`<=`) — bare `{kind}` keyword leaf. |
| `assignment_target` (3 kinds) | `{kind: "concat", items}` (`{a, b}` LHS — `items` is the clean flat target list; `1.0.3` / schema `3`, was `{kind: "concat", first, rest}` at ≤ `1.0.2`) / `{kind: "ranged", body}` / `{kind: "signal", body}`. |

For `"if"` shapes (`statement` / `always_ff_statement`), `else_body` is `[]` when there is no `else`. For `"block"`, `label` is `[]` when the `begin` has no `: label`.

## Family: expressions — the `binop_chain` contract

The rtl_frontend expression grammar is a **ten-level operator-precedence cascade** plus a ternary head and a unary tier. The ten binary levels each carry the same `binop_chain` typed shape:

```ebnf
rtl_expr            := conditional_expr
conditional_expr    := logical_or_expr ? conditional_expr : conditional_expr
                    -> {type: "ternary", condition: $1, then_expr: $3, else_expr: $5}
                     | logical_or_expr  -> $1                       -- passthrough

# Named operator rules — un-annotated alternations (the RTL-FE-0001 fix,
# schema 2; NOT in the 156-annotation inventory):
equality_op         := eqeq | ne
relational_op       := less_equal | lt | ge | gt
shift_op            := shl | shr
additive_op         := plus | minus
multiplicative_op   := star | slash | percent

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
equality_expr       := relational_expr  ( equality_op       relational_expr     )*
                    -> {type: "binop_chain", level: "equality",       lhs: $1, rest: $2}
relational_expr     := shift_expr       ( relational_op     shift_expr          )*
                    -> {type: "binop_chain", level: "relational",     lhs: $1, rest: $2}
shift_expr          := additive_expr    ( shift_op          additive_expr       )*
                    -> {type: "binop_chain", level: "shift",          lhs: $1, rest: $2}
additive_expr       := multiplicative_expr ( additive_op    multiplicative_expr )*
                    -> {type: "binop_chain", level: "additive",       lhs: $1, rest: $2}
multiplicative_expr := unary_expr       ( multiplicative_op unary_expr          )*
                    -> {type: "binop_chain", level: "multiplicative", lhs: $1, rest: $2}
```

> **`RTL-FE-0001` (resolved in `1.0.2` / schema `2`).** At release
> `1.0.1` / schema `1` the five lower levels used **inline operator
> alternations** as the iteration lead (`equality_expr := relational_expr
> ( ( == | != ) relational_expr )*`, and likewise `relational` / `shift`
> / `additive` / `multiplicative`). A bare positional `rest: $2`
> referencing an inline `( a | b )` group corrupts the positional model,
> so for any multi-operand input the level's `rest` emitted
> `"<invalid_sequence_access>"` plus a malformed nested object. **Fixed
> in `1.0.2`** by lifting the five inline alternations into the **named,
> un-annotated** op-rules shown above — the proven RTL-CE-Slice-2 /
> `systemverilog.ebnf` `binary_operator` idiom. The five `binop_chain`
> level annotations are **unchanged**; only the inline `( a | b )`
> became a named rule, so `rest` is now the clean `[ [op-envelope],
> operand ]` array (operator token text at `entry[0][1]`, `[]` for no
> operator). Because the five `*_op` rules are un-annotated, the
> annotation inventory is **unchanged at 156 / 74**. See
> [Worked Example: Binary Addition](examples-binary-addition.md) for the
> captured schema-`2` shape and the kept pre-fix history; tracked
> (status `Released`) as `RTL-FE-0001` in
> `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

| Level (`level`) | Rule | Operators |
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

**Consumer-facing left-fold contract** (per the integration contract, Release 1.0.2 Highlights): every one of these ten rules emits `{type: "binop_chain", level, lhs, rest}` where `lhs` is the leading operand (itself a `binop_chain` of the next-tighter level) and `rest` is a **clean array of iteration entries**, one per `(op operand)` repetition of `<next> ( <NAMED_op> <next> )*`. Each entry is a two-element array: `entry[0]` is the named op-rule envelope with the operator **token text at `entry[0][1]`** (`"+"`, `"-"`, …) and an empty `trivia` at `entry[0][0]` (`[]` when no leading trivia); `entry[1]` is the right-hand operand. For `a + b` the `additive`-level `rest` is the single entry `[ [ [], "+" ], {type:"binop_chain", level:"multiplicative", lhs:<b>, rest:[]} ]` — the **identical** consumer-fold contract as `rtl_const_expr`'s `binop_chain`. (This is the `RTL-FE-0001` corrected, gate-locked shape — at `1.0.1` / schema `1` it was the malformed `<invalid_sequence_access>` + nested object; see the note above and [Worked Example: Binary Addition](examples-binary-addition.md).) **Consumers fold `rest` left-associatively onto `lhs`**: evaluate `lhs`, then for each entry apply the operator at `entry[0][1]` with the running result on the left and `entry[1]` on the right. There is no `sign` field on any rtl_frontend level (unlike VHDL's `simple_expression`); the unary `+` / `-` / `!` / `~` operators live in `unary_expr`, below `multiplicative_expr`. All ten levels iterate `*`, so `rest` may hold any number of entries (including zero — a single operand surfaces as a `binop_chain` whose `rest` is `[]`).

The cascade bottoms out at `unary_expr` → `primary_expr`. This `binop_chain` shape is identical across all ten levels precisely so that a single consumer fold routine handles the entire binary-expression tree. See [Walking the AST](walking-the-ast.md) for a worked left-fold.

### Ternary, unary, and primary

| Rule | Shape |
|---|---|
| `conditional_expr` (2 forms) | `{type: "ternary", condition, then_expr, else_expr}` for the `c ? t : e` form; **passthrough** (`-> $1`, a `return_scalar` annotation) when there is no `?`, so a non-ternary expression surfaces as its `logical_or_expr` `binop_chain` directly with no `conditional_expr` wrapper. |
| `unary_expr` (5 forms) | `{type: "unary", op: "plus", expr}` / `{op: "minus"}` / `{op: "logical_not"}` (`!`) / `{op: "bit_not"}` (`~`); the fifth branch is **passthrough** (`-> $1`, `return_scalar`) — a non-prefixed operand surfaces as its `primary_expr` directly with no `unary` wrapper. |
| `primary_expr` (6 kinds) | `{kind: "repetition", body}` / `{kind: "concatenation", body}` / `{kind: "ranged_signal", body}` / `{kind: "signal", body}` / `{kind: "literal", body}` / `{kind: "parens", expr}`. |

Because `conditional_expr` and `unary_expr` are passthrough when their distinguishing syntax is absent, consumers must accept that any expression slot may hold a `binop_chain`, a `ternary`, a `unary`, **or** directly a `primary_expr` `{kind, ...}` object — dispatch on the presence of `type` vs `kind` (see [Walking the AST](walking-the-ast.md)).

### Signal references and operands

| Rule | Shape |
|---|---|
| `signal_reference` | `{name, path}` — `name` is the `scoped_identifier`; `path` is the `signal_path_op*` array (`[]` for a plain signal). |
| `ranged_signal_reference` | `{name, path, msb, lsb}` — a part-select `sig[msb:lsb]`. |
| `scoped_identifier` | `[identifier, …]` — clean flat array of the `pkg::name` scope-resolution chain; a single-element array for an unqualified identifier (`1.0.3` / schema `3`; was `{first, rest}` with `rest` `[]` for an unqualified identifier at ≤ `1.0.2`). |
| `signal_path_op` (2 kinds) | `{kind: "member", name}` (`.field`) / `{kind: "index", index}` (`[expr]`). |
| `concatenation_expr` | `[operand, …]` — clean flat array of the `{a, b, …}` operands (`1.0.3` / schema `3`; was `{first, rest}` at ≤ `1.0.2`). |
| `repetition_expr` | `{count, items}` — `{count{a, …}}`; `items` is the clean flat operand list (`1.0.3` / schema `3`; was `{count, first, rest}` at ≤ `1.0.2`). |
| `literal` (3 kinds) | `{kind: "based", text}` / `{kind: "decimal", text}` / `{kind: "real", body}`. |

## Family: data types

| Rule | Shape |
|---|---|
| `data_type` (6 kinds) | `{kind: "enum", body}` / `{kind: "union", body}` / `{kind: "struct", body}` / `{kind: "builtin", body}` / `{kind: "package", body}` / `{kind: "named", body}`. |
| `builtin_data_type` (9 kinds) | bare `{kind}` for `"bit"`, `"byte"`, `"shortint"`, `"int"`, `"integer"`, `"longint"`, `"logic"`, `"reg"`, `"wire"`. |
| `package_qualified_type` | `{package, name}` — `pkg::type_name`. |
| `enum_type` | `{base, packed_range, items}` — `base` is the typed `enum_base_type`; `items` is the clean flat `enum_item[]` list (`1.0.3` / schema `3`; was `{base, packed_range, first, rest}` at ≤ `1.0.2`). |
| `enum_base_type` (3 kinds) | `{kind: "builtin", body}` / `{kind: "package", body}` / `{kind: "named", body}`. |
| `enum_item` | `{name, value}` — `value` is `[]` when there is no `= <expr>`. |
| `struct_type` | `{packed, fields}` — `packed` is `[]` when the struct is unpacked; `fields` is the `struct_union_field` list. |
| `union_type` | `{packed, fields}` — same shape as `struct_type`. |
| `struct_union_field` | `{data_type, packed_range, names}` — `names` is the clean flat field-name declarator list (`1.0.3` / schema `3`; was `{data_type, packed_range, first, rest}` at ≤ `1.0.2`). |
| `struct_union_field_name` (2 kinds) | `{kind: "identifier", body}` / `{kind: "byte"}` (the `byte` reserved-word field-name form, bodyless). |
| `packed_range` | `{msb, lsb}` — `[msb:lsb]`. |

`named_data_type` (a bare identifier type reference) is **un-annotated** — it surfaces through the recursive envelope of its `identifier`, reached as `data_type.body` when `data_type.kind == "named"`. See [The Json Carrier](json-carrier.md).

## Total surface and the machine-checkable source

The full typed surface as of contract `1.0.3` is **156 return annotations across 74 distinct rules** (independently re-counted from the inventory below; neither the `1.0.2` `RTL-FE-0001` fix nor the `1.0.3` POST-SV-AUDIT batch — the 15 Category-A list-shape corrections + the `RTL-FE-0002` `event_control_list` fix — changed this count: the `*_op` / `event_separator` rules are un-annotated alternations and the 15 Category-A rules changed annotation form not count). This chapter is a curated grouping; the authoritative, machine-checkable enumeration of every `(rule, branch_index, annotation_type, normalized_text)` tuple is:

- `generated/rtl_frontend_return_annotations.json` — the live return-annotation inventory (`version: 1`, `grammar: "rtl_frontend"`, `annotation_count: 156`).
- `rust/test_data/ast_shape_contract/rtl_frontend_v1.json` — the embedded inventory used by the AST shape-contract regression lock (identical content; 156 entries in `declared_annotation_inventory.annotations`).

If this chapter and either artifact disagree, the artifact wins — and the integration contract `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` wins over both.

## How to follow per-slice changes

Each shape-affecting slice after RTL-FE-Slice-7 gets a row in [Schema Versioning](schema-versioning.md) and a Highlights section in `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md`. The [Changelog Index](changelog-index.md) ties them together.
