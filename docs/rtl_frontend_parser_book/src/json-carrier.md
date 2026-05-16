# The Json Carrier

This chapter explains how the rtl_frontend parser carries AST shapes in the JSON dump: the two carrier kinds, how typed shapes and recursive-envelope shapes coexist, the object / array / string / scalar mapping, the absent-optional convention, and the determinism guarantee. It is the conceptual companion to [Top-Level Rules](rules-top-level.md), which enumerates the concrete per-rule shapes, and to [AST Envelope Structure](ast-envelope.md), which documents the outer `AstDumpPayload`.

> **Note:** The rtl_frontend return-annotation campaign landed across seven slices — **RTL-FE-Slice-1..7, 156 annotations on 74 rules** (parser release `1.0.1`, schema version `1`). Almost every load-bearing rule is typed; the remaining un-annotated rules (terminal/regex leaves like `identifier`, and a few utility/passthrough rules) produce the recursive-envelope shape. The authoritative enumeration is `generated/rtl_frontend_return_annotations.json` (mirrored byte-for-byte by the embedded inventory in `rust/test_data/ast_shape_contract/rtl_frontend_v1.json`).

## Two carrier kinds

Every node in the rtl_frontend AST dump is carried in one of exactly two shapes.

### Typed shape (rules with a `-> ...` return annotation)

When a rule in `grammars/rtl_frontend.ebnf` carries a return annotation, the parser emits a typed JSON value derived from that annotation. For rtl_frontend this takes three sub-forms:

1. **Root object with `type`.** Only `rtl_frontend_file` carries a `type` discriminator at the dispatch level:

   ```json
   { "type": "rtl_frontend_file", "items": [ /* ... */ ] }
   ```

2. **`kind`-tagged dispatch object.** Most rtl_frontend dispatch rules use a `kind` discriminator (no `type`), reflecting the design choice of tightly-scoped per-rule dispatch. For example, `design_item`:

   ```json
   { "kind": "module", "body": { /* module_declaration shape */ } }
   ```

3. **Named-field object.** Single-sequence rules emit a flat object of named fields. For example, `module_declaration`:

   ```json
   { "name": <identifier-envelope>, "imports_pre": [], "parameters": [],
     "imports_post": [], "ports": [], "items": [] }
   ```

The expression rules combine forms: the ten `binop_chain` levels carry a `type` plus a `level` and the operand fields; `conditional_expr`'s ternary form carries a `type`; `primary_expr` carries a `kind`.

### Recursive-envelope shape (rules without annotations)

When a rule has no return annotation, the parser emits a JSON value derived mechanically from the rule's grammar shape:

- A **terminal literal** or **regex literal** produces a JSON string of the matched text.
- A **rule reference** produces whatever shape that rule produces.
- A **sequence** `a b c` produces a JSON array `[<a-shape>, <b-shape>, <c-shape>]`.
- An **alternation** produces the matched branch's shape directly.
- A **quantified rule** (`x*`, `x+`) produces a JSON array of the per-iteration shapes.
- An **optional rule** (`x?`) produces the matched shape if matched, or `[]` if un-matched.

In rtl_frontend the recursive-envelope shape is what you reach when you descend below the typed surface: `identifier` tokens, `named_data_type`, the passthrough forms of `conditional_expr` / `unary_expr`, and the few utility rules with no per-rule annotation. The `body` field of any `{kind, body}` dispatch object is whatever shape the matched sub-rule produces — typed if that sub-rule is itself annotated (it usually is), envelope otherwise.

> **Rule of thumb — un-annotated leaves are envelopes, not bare strings.** A field bound to an un-annotated rule (most importantly every identifier-valued field: `name`, `module_name`, `genvar`, …) surfaces as that rule's **recursive envelope**, not a bare JSON string. The identifier text is nested inside the envelope, reached by walking to the terminal. Do not assume `obj["name"]` is a string — see the worked example below for the real shape.

## Object / array / string / scalar mapping

| JSON value | Produced by |
|---|---|
| Object `{...}` | A typed return annotation that is an object literal (`{kind: ...}`, `{name: ..., data_type: ...}`, `{type: "binop_chain", ...}`, etc.). |
| Array `[...]` | A recursive-envelope sequence/quantified shape, **or** an un-matched optional field (`[]`), **or** the trailing `rest` iteration of a `{first, rest}` list rule. |
| String `"..."` | A matched terminal or regex leaf reached through an envelope (identifier text, literal text, keyword text). |
| `true` / `false` | A `BooleanLiteral` in a typed annotation. The current 156-annotation rtl_frontend surface uses **no** boolean literals; every `return_object` annotation is an object literal and the two `return_scalar` annotations are positional passthroughs. |
| Number | A `NumberLiteral` typed transform. Not used by the current rtl_frontend surface. |
| `null` | A `NullLiteral`. **Not used** by the current rtl_frontend surface — absent optional fields are carried as the empty array `[]`, never `null`. |

Two rtl_frontend-specific points consumers must internalize:

- **Absent optionals are `[]`, never `null`.** A `parameter_declaration_tail` with no `= <expr>` emits `"default": []`. An `enum_item` with no `= <expr>` emits `"value": []`. A `port_group` with no `data_type` emits `"data_type": []`. A plain signal's `signal_reference.path` is `[]`. A module with no parameter list emits `"parameters": []`. Test for an empty array, not for `null` or a missing key. (This was verified empirically against the live parser output — see the worked example.)
- **`{first, rest}` is the list convention, used uniformly.** Separated lists (`port_list`, `parameter_declaration_sequence`, `genvar_declaration`, `net_declaration`, `scoped_identifier`, `event_control_list`, `module_instantiation`, `concatenation_expr`, `parameter_override_list`, `port_connection_list`, `struct_union_field`, `enum_type`, …) emit `{first: <head>, rest: <iteration-of-the-(sep X)*-tail>}`. `rest` is a recursive-envelope array; each entry is the envelope of one `(separator element)` iteration. Unlike the SystemVerilog grammar — where a slice-58 audit flattened these to clean `[$N, $M::2*]` arrays — the rtl_frontend grammar uses the `{first, rest}` shape uniformly across RTL-FE-Slice-1..7, so consumers should expect to descend through the separator wrap when iterating an rtl_frontend list. (A future flattening slice, if it lands, will get its own [Schema Versioning](schema-versioning.md) row.)

## Worked example: a minimal module

Input:

```systemverilog
module m; endmodule
```

This is the real captured output of the live `rtl_frontend` parser (the value obtained by parsing the `AstDumpPayload.dump_json` string):

- The root is the **typed root object** `{ "type": "rtl_frontend_file", "items": [ … ] }`.
- `items` is a **recursive-envelope array** (the `design_item*` iteration) with one element.
- That element is a **`kind`-tagged dispatch object** `{ "kind": "module", "body": { … } }`.
- `body` is the **named-field object** for `module_declaration`:

  ```json
  {
    "name":         <identifier-envelope ending in [ [], "m" ]>,
    "imports_pre":  [],
    "parameters":   [],
    "imports_post": [],
    "ports":        [],
    "items":        []
  }
  ```

- `name` is **not** a bare string `"m"`. It is the recursive envelope of the `kw_module identifier` sequence position bound to `$2` of the `module_declaration` rule, threaded through the un-annotated `identifier` rule. In the live output it is a nested array whose deeply-nested tail is `[ [], "m" ]` — the identifier text `"m"` is at index `[1]` of that innermost two-element pair. A robust consumer walks to the terminal string rather than indexing a fixed depth (see [Walking the AST](walking-the-ast.md#identifier-extraction)).
- `imports_pre`, `imports_post` are **empty arrays** (the `import_declaration*` iterations matched zero items).
- `parameters` is an **empty array** (`[]`) because the optional `#( … )` parameter clause was absent — the absent-optional convention.
- `ports` is an **empty array** (`[]`) because the optional `( … )` ANSI port list was absent.
- `items` is an **empty array** (the `module_item*` iteration matched zero items).

The per-rule field names come straight from the live inventory's `normalized_text`; the value shapes above are the real captured output of `generated/rtl_frontend_parser.rs` for this input. This grounds the rule of thumb: an un-annotated identifier field is the identifier rule's recursive envelope, not a bare string — confirm structure against the inventory and the live parser, never assume a scalar.

## The expression family (`binop_chain`)

The ten expression-precedence rules (`logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr`) all carry the same `binop_chain` carrier so a single consumer fold handles the whole tree:

```json
{ "type": "binop_chain", "level": "additive", "lhs": <next-level-shape>, "rest": [ /* (op, operand) iterations */ ] }
```

`lhs` is the leading operand (itself a `binop_chain` of the next-tighter level, bottoming out at `unary_expr` → a typed `primary_expr`). `rest` is a recursive-envelope array of `(operator, operand)` iterations. There is **no** `sign` field on any rtl_frontend level (unlike VHDL's `simple_expression`); prefix `+` / `-` / `!` / `~` are handled by the `unary_expr` rule below the cascade. Consumers fold `rest` left-associatively onto `lhs`. The full level/field/operator table is in [Top-Level Rules](rules-top-level.md#family-expressions--the-binop_chain-contract); the fold code is in [Walking the AST](walking-the-ast.md).

`conditional_expr` (ternary) and `unary_expr` are **passthrough** when their distinguishing syntax is absent (the `-> $1` `return_scalar` branches): a non-ternary, non-prefixed expression surfaces directly as its `logical_or_expr` `binop_chain` or its `primary_expr` `{kind, ...}` object, with no wrapper. Consumers must therefore accept a `type`-tagged object, a `kind`-tagged object, or a passthrough child in any expression slot.

## Determinism

The AST dump is **byte-deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical.
- No embedded timestamps or hashes.
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed), but the underlying JSON value is identical.

Re-running the parse on the same input produces an identical JSON value. This is a **hard guarantee** of the schema. Any non-determinism is a bug — report it via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Un-annotated-on-purpose rules

Some rules remain un-annotated by design — typically terminal/regex leaves (`identifier` and the keyword/operator tokens), `named_data_type`, and the passthrough forms of `conditional_expr` / `unary_expr` whose envelope shape is already the most useful representation. Their text is reachable through the recursive-envelope walk from the nearest typed parent (e.g. `data_type.kind == "named"` → `body` is the `named_data_type` envelope; `literal.kind == "decimal"` → `text` is the matched literal text).

## How to read the annotation text

The `normalized_text` field of each inventory entry is the EBNF `-> ...` clause:

- `$N` — positional reference to the Nth body element (1-indexed).
- `{field: value, ...}` — typed object literal (`annotation_type: "return_object"`).
- `$N` alone — positional passthrough (`annotation_type: "return_scalar"`; rtl_frontend uses exactly two: `conditional_expr` and `unary_expr` passthrough branches).
- `[v1, v2, ...]` — array literal.
- `"text"` — string literal.

The full annotation-language grammar is `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`. The per-rule shapes those annotations produce are tabulated in [Top-Level Rules](rules-top-level.md).
