# The Json Carrier

This chapter explains how the sv_preprocessor parser carries AST shapes in the JSON dump: the two carrier kinds, how typed shapes and recursive-envelope shapes coexist, the object / array / string / scalar mapping, the absent-optional convention, and the determinism guarantee. It is the conceptual companion to [Top-Level Rules](rules-top-level.md), which enumerates the concrete per-rule shapes, and to [AST Envelope Structure](ast-envelope.md), which documents the outer `AstDumpPayload`.

> **Note:** The sv_preprocessor return-annotation campaign landed in a single comprehensive batch — **SVPP-Slice-1, 64 annotations across 27 rules** — followed by the `1.0.2` `SVPP-0001` correctness fix that lifted the inline `(kw_ifdef | kw_ifndef)` alternation into the named `pp_if_keyword` rule (taking the surface to **66 annotations across 28 rules**), and the `1.0.3` POST-SV-AUDIT Category-A fix that corrected `macro_formals` from the raw `{first, rest}` envelope to the clean `[$2, $3::2*]` list (parser release `1.0.3`, schema version `3`; the count stayed **66 / 28** — only `macro_formals`'s annotation form changed `return_object` → `return_array`, so the surface is now 65 `return_object` + 1 `return_array`). Every dispatcher and directive shape is typed; the remaining un-annotated rules (terminal/regex leaves like `identifier`, `macro_name`, `macro_body_text`, `unsigned_number`, `time_unit`, and the keyword/operator tokens) produce the recursive-envelope shape. The authoritative enumeration is `generated/systemverilog_preprocessor_return_annotations.json` (mirrored by the embedded inventory in `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` — same tuples; the contract-embedded copy omits only the cosmetic `raw_text` field).

## Two carrier kinds

Every node in the sv_preprocessor AST dump is carried in one of exactly two shapes.

### Typed shape (rules with a `-> ...` return annotation)

When a rule in `grammars/systemverilog_preprocessor.ebnf` carries a return annotation, the parser emits a typed JSON value derived from that annotation. For sv_preprocessor this takes three sub-forms:

1. **Root object with `type`.** Only `systemverilog_preprocessor_file` carries a `type` discriminator:

   ```json
   { "type": "systemverilog_preprocessor_file", "items": [ /* ... */ ] }
   ```

2. **`kind`-tagged dispatch object.** Every other sv_preprocessor dispatcher uses a `kind` discriminator (no `type`), reflecting the design choice of tightly-scoped per-rule dispatch. For example, `pp_item`:

   ```json
   { "kind": "define", "body": { /* pp_define shape */ } }
   ```

   Bodyless dispatch branches carry only the discriminator: `{ "kind": "celldefine" }`, `{ "kind": "blank_line" }`, the punctuation atoms `{ "kind": "lparen" }`, `{ "kind": "comma" }`, etc.

3. **Named-field object.** Single-sequence rules emit a flat object of named fields. For example, `pp_define` (real captured shape — see the worked example below):

   ```json
   { "name": [ [ " " ], "FOO" ], "formals": [], "body": { "fragments": [ /* ... */ ] } }
   ```

   `name` is `$2` of the `pp_define` sequence, bound to the **un-annotated** `macro_name` → `identifier` rule, so it carries that rule's recursive envelope `[ <trivia-prefix>, "<text>" ]` rather than a bare string. The text is at index `[1]`.

There is **no** fourth form: the sv_preprocessor surface uses no `return_scalar` passthrough annotations and no `binop_chain`-style expression carrier (the grammar is directive/line-oriented, not expression-precedence-oriented). All 66 annotations are `annotation_type: "return_object"`.

### Recursive-envelope shape (rules without annotations)

When a rule has no return annotation, the parser emits a JSON value derived mechanically from the rule's grammar shape:

- A **terminal literal** or **regex literal** produces a JSON string of the matched text.
- A **rule reference** produces whatever shape that rule produces.
- A **sequence** `a b c` produces a JSON array `[<a-shape>, <b-shape>, <c-shape>]`.
- An **alternation** produces the matched branch's shape directly.
- A **quantified rule** (`x*`, `x+`) produces a JSON array of the per-iteration shapes.
- An **optional rule** (`x?`) produces the matched shape if matched, or `[]` if un-matched.

In sv_preprocessor the recursive-envelope shape is what you reach when you descend below the typed surface: `identifier` / `macro_name` tokens, the `macro_body_text` / `condition_text` / `macro_default_text` runs, `unsigned_number`, `time_unit`, `macro_reference`, and the keyword/punctuation token rules. The `body` field of any `{kind, body}` dispatch object is whatever shape the matched sub-rule produces — typed if that sub-rule is itself annotated, envelope otherwise.

> **Rule of thumb — un-annotated leaves are envelopes, not bare strings.** A field bound to an un-annotated rule (most importantly every identifier-valued field: `name`, `macro`, and every `"text"` atom's `body`) surfaces as that rule's **recursive envelope**, not a bare JSON string. The text is nested inside the envelope, reached by walking to the terminal. Do not assume `obj["name"]` is a string — the worked example below shows the real captured shape, where `name` is `[ [ " " ], "FOO" ]` (the trivia prefix at `[0]`, the identifier text `"FOO"` at `[1]`), not the string `"FOO"`.

## Object / array / string / scalar mapping

| JSON value | Produced by |
|---|---|
| Object `{...}` | A typed return annotation that is an object literal (`{type: "systemverilog_preprocessor_file", ...}`, `{kind: ...}`, `{name: ..., formals: ...}`, etc.). |
| Array `[...]` | A recursive-envelope sequence/quantified shape, **or** an un-matched optional field (`[]`), **or** the typed `return_array` of `macro_formals` (the clean flat `macro_formal[]` list — the only `return_array` on the surface, as of `1.0.3` / schema `3`), **or** the un-annotated-leaf envelope `[ <prefix>, "<text>" ]`. |
| String `"..."` | A matched terminal or regex leaf reached through an envelope (identifier text, literal text, keyword text). |
| `true` / `false` | A `BooleanLiteral` in a typed annotation. The current 66-annotation sv_preprocessor surface uses **no** boolean literals; every annotation is an object literal except the single `return_array` (`macro_formals`, a clean `macro_formal[]` list as of `1.0.3` / schema `3`). |
| Number | A `NumberLiteral` typed transform. Not used by the current sv_preprocessor surface. |
| `null` | A `NullLiteral`. **Not used** by the current sv_preprocessor surface — absent optional fields are carried as the empty array `[]`, never `null` (verified empirically; see below). |

Two sv_preprocessor-specific points consumers must internalize:

- **Absent optionals are `[]`, never `null`.** Verified empirically against the live parser: for the input `` `define MAX`` (a bare define with neither a formal list nor a body), the captured `pp_define` shape is

  ```json
  { "name": [ [ " " ], "MAX" ], "formals": [], "body": [] }
  ```

  Both un-matched optionals (`macro_formals?`, `macro_body?`) are the empty array `[]`. Likewise an empty `` `ifdef X`` / `` `endif`` block captures `{ "elsif_branches": [], "else_branch": [] }` and a `pp_if_branch` with no trailing directive text captures `"tail": []`. Test for an empty array, not for `null` or a missing key.

- **`macro_formals` is the only separated list, and it is now a clean flat array.** As of release `1.0.3` / schema `3` `macro_formals` is a clean flat `macro_formal[]` array (the canonical `[$2, $3::2*]` extraction-spread — the only `return_array` annotation on the surface). The atom/fragment lists (`condition_expr.atoms`, `macro_default_value.atoms`, `macro_body.fragments`) are likewise plain `x+`-style arrays of typed atom objects. At ≤ release `1.0.2` / schema `2` `macro_formals` instead used the raw `{first: <leading macro_formal>, rest: <raw [[comma, macro_formal], …] iteration>}` separator envelope — a Category-A raw-envelope misuse that forced consumers to descend through the `comma` separator wrap. It was corrected to the flat list in `1.0.3` / schema `3` (POST-SV-AUDIT.2.1, `PGEN-POST-SV-AUDIT-0002`), mirroring the SystemVerilog grammar's slice-58 list flatten; see [Walking the AST](walking-the-ast.md#iterating-the-macro_formals-list) and the schema-`3` row of [Schema Versioning](schema-versioning.md).

## Worked example: a single define directive

Input:

```systemverilog
`define FOO 1
```

(`printf '`define FOO 1\n'`). This is the real captured output of the live `systemverilog_preprocessor` parser — the parsed `AstDumpPayload.dump_json` value (there is no `root` field; `serde_json::from_str(&payload.dump_json)` yields this typed root):

```json
{
  "type": "systemverilog_preprocessor_file",
  "items": [
    {
      "kind": "define",
      "body": {
        "name": [ [ " " ], "FOO" ],
        "formals": [],
        "body": {
          "fragments": [
            { "kind": "text", "body": [ [ " " ], "1" ] }
          ]
        }
      }
    }
  ]
}
```

Carrier walk:

- The root is the **typed root object** `{ "type": "systemverilog_preprocessor_file", "items": [ … ] }`.
- `items` is a **recursive-envelope array** (the `pp_item*` iteration) with one element.
- That element is a **`kind`-tagged dispatch object** `{ "kind": "define", "body": { … } }` — the `pp_item` `"define"` branch.
- The outer `body` is the **named-field object** for `pp_define`: `{ "name": …, "formals": …, "body": … }`.
- `name` is **not** the bare string `"FOO"`. It is the recursive envelope of the un-annotated `macro_name` → `identifier` rule: `[ [ " " ], "FOO" ]`. The leading `[ " " ]` is the inline-trivia prefix (the space after `` `define``); the identifier text `"FOO"` is at index `[1]`. A robust consumer walks to the terminal string rather than indexing a fixed depth (see [Walking the AST](walking-the-ast.md#identifier-and-text-extraction)).
- `formals` is an **empty array** (`[]`) because the optional `macro_formals?` was absent — the absent-optional convention.
- The inner `body` (the `pp_define` `body` field) is the typed `macro_body` object `{ "fragments": [ … ] }`.
- `fragments` is a **recursive-envelope array** (the `macro_body_fragment+` iteration) with one element: `{ "kind": "text", "body": [ [ " " ], "1" ] }` — the `macro_body_fragment` `"text"` branch. Its `body` is the un-annotated `macro_body_text` envelope `[ [ " " ], "1" ]`; the replacement text `"1"` is at index `[1]`.

The per-rule field names come straight from the live inventory's `normalized_text`; the value shapes above are the real captured output of `generated/systemverilog_preprocessor_parser.rs` for this input. This grounds the rule of thumb: an un-annotated identifier-or-text field is the leaf rule's recursive envelope, not a bare string — confirm structure against the inventory and the live parser, never assume a scalar. See [Single Define](examples-single-define.md) for the end-to-end worked example.

## No expression cascade

A consumer coming from the VHDL or rtl_frontend books should note: there is **no `binop_chain` carrier** here. The `` `elsif`` condition (`condition_expr`) is a flat `{atoms: [...]}` list of `condition_atom` objects — the `||` / `&&` / `!` / `?` / `:` tokens are individual untyped `{kind: "logical_or"}` / `{kind: "logical_and"}` / `{kind: "bang"}` / `{kind: "question"}` / `{kind: "colon"}` atoms in source order, not a parsed precedence tree. Any boolean evaluation of a conditional is the consumer's responsibility. The same flat-list shape backs `macro_default_value.atoms` and `macro_body.fragments`.

## Determinism

The AST dump is **byte-deterministic** for a given input + parser-release version:

- Object keys are emitted in canonical (alphabetical) order.
- Number formatting is canonical.
- No embedded timestamps or hashes.
- Whitespace is configurable via `AstDumpOptions.pretty` (compact vs pretty-printed), but the underlying JSON value is identical.

Re-running the parse on the same input produces an identical JSON value. This is a **hard guarantee** of the schema. Any non-determinism is a bug — report it via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Un-annotated-on-purpose rules

Some rules remain un-annotated by design — terminal/regex leaves (`identifier`, `macro_name`, `bt_identifier`/`macro_reference`, `unsigned_number`, `quoted_string`, `angle_path`, `time_unit`, the keyword and punctuation tokens) and the literal-text runs (`macro_body_text`, `macro_default_text`, `condition_text`, `non_directive_text`, `directive_tail`, `directive_comment_tail`). Their text is reachable through the recursive-envelope walk from the nearest typed parent (e.g. `include_path.kind == "quoted"` → `text` is the matched quoted-string envelope; `macro_body_fragment.kind == "text"` → `body` is the matched `macro_body_text` envelope).

## How to read the annotation text

The `normalized_text` field of each inventory entry is the EBNF `-> ...` clause:

- `$N` — positional reference to the Nth body element (1-indexed).
- `{field: value, ...}` — typed object literal (`annotation_type: "return_object"`; the sv_preprocessor surface uses this for **all 64** annotations).
- `[v1, v2, ...]` — array literal.
- `"text"` — string literal.

The full annotation-language grammar is `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`. The per-rule shapes those annotations produce are tabulated in [Top-Level Rules](rules-top-level.md).
