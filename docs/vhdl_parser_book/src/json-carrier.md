# The Json Carrier

This chapter explains how the VHDL parser carries AST shapes in the JSON dump: the two carrier kinds, how typed shapes and recursive-envelope shapes coexist, the object / array / string / scalar mapping, and the determinism guarantee. It is the conceptual companion to [Top-Level Rules](rules-top-level.md), which enumerates the concrete per-rule shapes.

> **Note:** The VHDL return-annotation campaign landed in a single comprehensive batch — **VHDL-Slice-1, 249 annotations across 110 rules** (parser release `1.0.1`, schema version `1`). Almost every load-bearing rule is typed; the remaining un-annotated rules (terminal/regex leaves and a few utility rules) produce the recursive-envelope shape described in [AST Envelope Structure](ast-envelope.md). The authoritative enumeration is `generated/vhdl_return_annotations.json` (mirrored byte-for-byte by the embedded inventory in `rust/test_data/ast_shape_contract/vhdl_v1.json`).

## Two carrier kinds

Every node in the VHDL AST dump is carried in one of exactly two shapes.

### Typed shape (rules with a `-> ...` return annotation)

When a rule in `grammars/vhdl.ebnf` carries a return annotation, the parser emits a typed JSON value derived from that annotation. For VHDL this takes three sub-forms:

1. **Root object with `type`.** Only `vhdl_file` carries a `type` discriminator at the dispatch level:

   ```json
   { "type": "vhdl_file", "design_units": [ /* ... */ ] }
   ```

2. **`kind`-tagged dispatch object.** Most VHDL dispatch rules use a `kind` discriminator (no `type`), reflecting the design choice of tightly-scoped per-rule dispatch. For example, `design_unit`:

   ```json
   { "kind": "entity", "body": { /* entity_declaration shape */ } }
   ```

3. **Named-field object.** Single-sequence rules emit a flat object of named fields. For example, `entity_declaration`:

   ```json
   { "name": "e", "items": [], "end_label": [] }
   ```

The `binop_chain` expression rules combine forms 1 and 3 — they carry both a `type` and a `level` plus the operand fields (see [the expression family](#the-expression-family-binop_chain)).

### Recursive-envelope shape (rules without annotations)

When a rule has no return annotation, the parser emits a JSON value derived mechanically from the rule's grammar shape:

- A **terminal literal** or **regex literal** produces a JSON string of the matched text.
- A **rule reference** produces whatever shape that rule produces.
- A **sequence** `a b c` produces a JSON array `[<a-shape>, <b-shape>, <c-shape>]`.
- An **alternation** produces the matched branch's shape directly.
- A **quantified rule** (`x*`, `x+`) produces a JSON array of the per-iteration shapes.
- An **optional rule** (`x?`) produces the matched shape if matched, or `[]` if un-matched.

In VHDL, the recursive-envelope shape is what you reach when you descend below the typed surface: VHDL identifier tokens, physical / bit-string / string / character literals, and the few utility rules that have no per-rule annotation. The `body` field of any `{kind, body}` dispatch object is whatever shape the matched sub-rule produces — typed if that sub-rule is itself annotated (it usually is), envelope otherwise.

## Object / array / string / scalar mapping

| JSON value | Produced by |
|---|---|
| Object `{...}` | A typed return annotation that is an object literal (`{kind: ...}`, `{name: ..., subtype: ...}`, `{type: "binop_chain", ...}`, etc.). |
| Array `[...]` | A recursive-envelope sequence/quantified shape, **or** an un-matched optional field (`[]`), **or** the trailing `rest` iteration of a `{first, rest}` list rule. |
| String `"..."` | A matched terminal or regex leaf (identifier text, literal text, keyword text reached through an envelope). |
| `true` / `false` | A `BooleanLiteral` in a typed annotation. VHDL's current 249-annotation surface uses no boolean literals; all annotations are object literals. |
| Number | A `NumberLiteral` typed transform. Not used by the current VHDL surface. |
| `null` | A `NullLiteral`. Not used by the current VHDL surface — absent optional fields are carried as the empty array `[]`, not `null`. |

Two VHDL-specific points consumers must internalize:

- **Absent optionals are `[]`, never `null`.** A `signal_declaration` with no initializer emits `"default": []`. A bare `return;` emits `"value": []`. Test for an empty array, not for `null` or a missing key.
- **`{first, rest}` is the list convention.** Separated lists (`identifier_list`, `selected_name`, `association_list`, `library_clause`, `use_clause`, `parameter_list`, `choices`, `enumeration_type_definition`, …) emit `{first: <head>, rest: <iteration-of-the-(sep X)*-tail>}`. `rest` is a recursive-envelope array; each entry is the envelope of one `(separator element)` iteration. Unlike the SystemVerilog grammar — where a slice-58 audit flattened these to clean `[$N, $M::2*]` arrays — the VHDL grammar uses the `{first, rest}` shape uniformly across VHDL-Slice-1, so consumers should expect to descend through the separator wrap when iterating a VHDL list. (A future flattening slice, if it lands, will get its own [Schema Versioning](schema-versioning.md) row.)

## Worked example: a minimal entity

Input:

```vhdl
entity e is end e;
```

Carrier walk:

- The root is the **typed root object** `{ "type": "vhdl_file", "design_units": [ … ] }`.
- `design_units` is a **recursive-envelope array** (the `design_unit*` iteration) with one element.
- That element is a **`kind`-tagged dispatch object** `{ "kind": "entity", "body": { … } }`.
- `body` is the **named-field object** for `entity_declaration`: `{ "name": "e", "items": [], "end_label": [ … ] }`.
- `name` is a **string** (the identifier text reached through the envelope).
- `items` is an **empty array** (`entity_declarative_item*` matched zero items).
- `end_label` is an **array** carrying the optional trailing `identifier?` (the `e` after `end`), or `[]` if omitted.

The per-rule field names come straight from the live inventory's `normalized_text`. See [Examples Minimal Entity](examples-minimal-entity.md) for a pinned end-to-end production AST.

## The expression family (`binop_chain`)

The five expression-precedence rules (`expression` → `relation` → `simple_expression` → `term` → `factor`) all carry the same `binop_chain` carrier so a single consumer fold handles the whole tree:

```json
{ "type": "binop_chain", "level": "logical", "lhs": <relation-shape>, "rest": [ /* (op, relation) iterations */ ] }
```

`lhs` is the leading operand (itself a `binop_chain` of the next-tighter level, bottoming out at a typed `primary`). `rest` is a recursive-envelope array of `(operator, operand)` iterations; for `simple_expression` there is an additional leading `sign` field (`[]` when no unary `+`/`-`). Consumers fold `rest` left-associatively onto `lhs`. The full level/field/operator table is in [Top-Level Rules](rules-top-level.md#family-expressions--the-binop_chain-contract); the fold code is in [Walking the AST](walking-the-ast.md).

## Determinism

The AST dump is **byte-deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical.
- No embedded timestamps or hashes.
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed), but the underlying JSON value is identical.

Re-running the parse on the same input produces an identical JSON value. This is a **hard guarantee** of the schema. Any non-determinism is a bug — report it via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Un-annotated-on-purpose rules

Some rules remain un-annotated by design — typically terminal/regex leaves (identifiers, `physical_literal`, `bit_string_literal`, `string_literal`, `character_literal`) and utility rules whose envelope shape is already the most useful representation. Their text is reachable through the recursive-envelope walk from the nearest typed parent (e.g. `literal.kind == "string"` → `body` is the matched string-literal text).

## How to read the annotation text

The `normalized_text` field of each inventory entry is the EBNF `-> ...` clause:

- `$N` — positional reference to the Nth body element (1-indexed).
- `{field: value, ...}` — typed object literal.
- `[v1, v2, ...]` — array literal.
- `"text"` — string literal.

The full annotation-language grammar is `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`. The per-rule shapes those annotations produce are tabulated in [Top-Level Rules](rules-top-level.md).
