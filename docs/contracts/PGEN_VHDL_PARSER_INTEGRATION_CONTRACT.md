# docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `vhdl` parser family.

This is the document downstream projects embedding the PGEN VHDL parser should read first.

## Contract Identity
- Contract version:
  - `1.0.1`
- Parser release version:
  - `1.0.1`
- Embedding API contract baseline:
  - tracked under `rust/docs/EMBEDDING_API_CONTRACT.md`
- VHDL AST-dump schema version:
  - `1`
- Last updated:
  - `2026-05-15`
- Current grammar family label:
  - `vhdl`
- Per-family mdBook:
  - `docs/vhdl_parser_book/` (tracked HTML at `docs/vhdl_parser_book-html/`)
- Per-family gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_book_gate`

## Schema Versioning

The VHDL parser carries two version axes:

1. **Parser release version** (`1.0.1`). Tracks the parser library's release identity. Bumped on every functional change, including bug fixes, perf work, and grammar changes.
2. **AST-dump schema version** (`1`). Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

A single parser release can carry the same schema version as the previous release (no shape change) or a bumped schema version (shape changed). The two version numbers move independently.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 1.0.0 | 1.0.1 | **VHDL-Slice-1** — initial 249-annotation baseline. Design units, declarations, types, statements, expressions (binop_chain shape across the 5-level operator hierarchy), and literals all typed. |
| 0.1.0 | 1.0.0 | Foundation baseline. Grammar (`grammars/vhdl.ebnf`) un-annotated except for `vhdl_file -> {type, design_units}` root. AST dump is the recursive-envelope shape across all rules. |

Bump-trigger guidance:

- A new return annotation lands on a previously-unannotated rule → schema bump.
- An existing return annotation is restructured → schema bump.
- A grammar rule changes shape in a way that's user-visible → schema bump.
- Pure performance optimizations producing the same AST → NO bump.
- Internal codegen reorganization that doesn't reach the output → NO bump.

## Release 1.0.1 / Contract 1.0.1 Highlights — VHDL-Slice-1: full grammar typed (110+ rules / 249 annotations)

Initial typing campaign covering the entire vhdl.ebnf surface in one batch. The grammar is line-by-line annotated with consumer-facing typed shapes; consumers can dispatch on `kind` / `type` discriminators or address named fields directly. Coverage:

```ebnf
# File root
vhdl_file                       -> {type: "vhdl_file", design_units}

# Dispatch wrappers (10 kinds / 4 / 5 / 12 / 13 / 8 / 9 / 3 / 2 / 5 / 7 / 3 / 2 / 9 / 13 / 2 / 2 / 5 / 2 / 6 / 6 / 6 / ...)
design_unit                     -> {kind, body}    -- 10 kinds
context_item                    -> {kind, body}    -- 4 kinds
entity_declarative_item         -> {kind, body}    -- 5 kinds
architecture_declarative_item   -> {kind, body}    -- 12 kinds
package_declarative_item        -> {kind, body}    -- 13 kinds
package_body_declarative_item   -> {kind, body}    -- 8 kinds
subprogram_declarative_item     -> {kind, body}    -- 9 kinds
process_declarative_item        -> {kind, body}    -- 9 kinds
sequential_statement            -> {kind, body}    -- 13 kinds
concurrent_statement            -> {kind, body}    -- 7 kinds

# Keyword leaves
signal_mode                     -> {kind: "in" | "out" | "inout" | "buffer" | "linkage"}
view_mode                       -> {kind: "in" | "out" | "inout" | "buffer" | "linkage"}
parameter_mode                  -> {kind: "in" | "out" | "inout"}
logical_operator                -> {kind: "and" | "or" | "xor" | "nand" | "nor" | "xnor"}
relational_operator             -> {kind: "eq" | "ne" | "lt" | "le" | "gt" | "ge"}

# Design units
entity_declaration              -> {name, items, end_label}
architecture_body               -> {name, entity_name, items, statements, end_label}
package_declaration             -> {name, header, items, end_label}
package_body                    -> {name, items, end_label}
configuration_declaration       -> {name, entity_name, items, end_label}
context_declaration             -> {name, items, end_label}

# Generic / port / parameter interfaces
generic_clause                  -> {list}
generic_interface_list          -> {first, rest}
generic_interface_element       -> {kind, names, subtype, default}
port_clause                     -> {list}
port_interface_list             -> {first, rest}
port_interface_element          -> {names, mode, subtype, default}
parameter_list                  -> {first, rest}
parameter_interface_element     -> {names, mode, subtype, default}

# Declarations
signal_declaration              -> {names, subtype, default}
constant_declaration            -> {names, subtype, value}
variable_declaration            -> {names, subtype, default}
file_declaration                -> {names, subtype, open_mode, filename}
subtype_declaration             -> {name, subtype}
component_declaration           -> {name, generic, port, end_label}
type_declaration                -> {name, definition}
view_declaration                -> {name, subtype, elements}
alias_declaration               -> {name, target}

# Types
type_definition (3 kinds)       -> {kind: "enumeration"|"array"|"record", body}
enumeration_type_definition     -> {first, rest}
array_type_definition           -> {index_range, element_subtype}
record_type_definition          -> {elements}
record_element_declaration      -> {names, subtype}
subtype_indication              -> {type_mark, constraint}
constraint (2 kinds)            -> {kind: "range"|"index", body}
range_constraint                -> {range}
index_constraint                -> {first, rest}
discrete_range (5 kinds)        -> {kind: "range"|"attribute"|"name"|"subtype_range"|"subtype_box", ...}
range_expression (2 kinds)      -> {kind: "to"|"downto", low|high, high|low}

# Statements
process_statement               -> {label, sensitivity, items, statements, end_label}
concurrent_signal_assignment    -> {target, rhs}
target (2 kinds)                -> {kind: "name"|"aggregate", ...}
component_instantiation         -> {label, unit, generic_map, port_map}
instantiated_unit (3 kinds)     -> {kind: "component"|"entity"|"configuration", name}
port_map_aspect                 -> {associations}
association_list                -> {first, rest}
association_element             -> {formal, actual}
generate_statement (2 kinds)    -> {kind: "for"|"if", ...}
block_statement                 -> {label, items, statements, end_label}
signal_assignment_statement     -> {target, rhs}
variable_assignment_statement   -> {target, value}
procedure_call_statement        -> {name, params}
if_statement                    -> {condition, then_body, elsif_branches, else_body}
case_statement                  -> {value, alternatives}
case_statement_alternative      -> {choices, body}
loop_statement (3 kinds)        -> {kind: "while"|"for"|"infinite", ...}

# Expressions (5-level binop_chain hierarchy)
expression                      -> {type: "binop_chain", level: "logical",        lhs, rest}
relation                        -> {type: "binop_chain", level: "relational",     lhs, rest}
simple_expression               -> {type: "binop_chain", level: "additive",       sign, lhs, rest}
term                            -> {type: "binop_chain", level: "multiplicative", lhs, rest}
factor                          -> {type: "binop_chain", level: "power",          lhs, rest}
primary (7 kinds)               -> {kind: "literal"|"aggregate"|"attribute_name"|"function_call"|"name"|"parens"|"not", ...}

# Literals (6 kinds)
literal                         -> {kind: "physical"|"decimal"|"based"|"bit_string"|"string"|"character", body}
decimal_literal                 -> {value}
based_literal                   -> {base, value}
```

The `binop_chain` shape is the consumer-facing left-fold contract for VHDL's 5-level expression hierarchy. Each level emits `{type: "binop_chain", level, lhs, rest}` where `lhs` is the leading operand and `rest` is the iteration array of (op, operand) pairs; consumers fold left-associatively.

Annotation count: **249** (was 1 / pre-typing baseline). Same accept set.

## AST Envelope and `design_unit` Dispatch

This section is the consumer-facing dispatch contract: how a downstream
integrator goes from the host AST-dump call to a typed VHDL tree, and how to
branch on the top-level discriminators. Every shape below is transcribed from
the live inventory `generated/vhdl_return_annotations.json`
(`version: 1`, `grammar: "vhdl"`, `annotation_count: 249`), cross-checked
against the embedded copy in
`rust/test_data/ast_shape_contract/vhdl_v1.json` (identical content), and is
consistent with the curated per-rule reference at
`docs/vhdl_parser_book/src/rules-top-level.md`.

### The `AstDumpPayload` envelope

The AST-dump host entry points
(`parse_vhdl_1076_2019_ast_dump`, the generic
`parse_grammar_profile_ast_dump*` family, and the named-result form
`parse_grammar_profile_ast_dump_named`) return — on success — an
`AstDumpPayload` (defined in `rust/src/embedding_api.rs`, contract in
`rust/docs/EMBEDDING_API_CONTRACT.md`). It is a canonical-JSON payload string
plus truncation metadata, with exactly four fields:

| Field | Type | Meaning |
|---|---|---|
| `dump_json` | string | The canonical (key-sorted) JSON encoding of the typed VHDL AST. Parse this string to obtain the `vhdl_file` root object described below. |
| `truncated` | bool | `false` for a complete dump; `true` when `max_ast_bytes` was exceeded and `dump_json` instead carries the truncation diagnostic envelope. |
| `full_bytes` | int | Byte length of the full encoded AST payload (before any truncation). |
| `emitted_bytes` | int | Byte length actually placed in `dump_json`. Equals `full_bytes` when not truncated. |

When `truncated` is `true`, `dump_json` is replaced by a deterministic
truncation diagnostic envelope (not the AST). That envelope carries
`pgen_dump_contract_version` (currently `1`), `kind:
"pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
"parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`. Consumers must
check `truncated` (or, equivalently, the presence of
`pgen_dump_contract_version` / `kind == "pgen_ast_dump_truncation"` in the
parsed `dump_json`) before treating `dump_json` as a VHDL AST. If
`max_ast_bytes` is too small to fit even the diagnostic envelope, the API
returns `E_INVALID_LIMITS` instead.

> Accuracy note: the live `AstDumpPayload` struct exposes precisely
> `dump_json` / `truncated` / `full_bytes` / `emitted_bytes`. The
> `pgen_dump_contract_version` / `schema_version` / `grammar` / `profile` /
> `root` keys are **not** members of `AstDumpPayload` itself —
> `pgen_dump_contract_version` appears only inside the truncation diagnostic
> envelope, the schema axis is the **AST-dump schema version `1`** tracked in
> [Schema Versioning](#schema-versioning) (and surfaced for the regex sibling
> via `parser_embedding_api_contract().regex_ast_dump_schema_version`), the
> grammar family is the fixed `vhdl` label, and the profile is the fixed
> `vhdl_1076_2019` stable profile (see [Stable Integration
> Surface](#stable-integration-surface)). The "root" is the parsed
> `vhdl_file` object documented next. This contract documents the surface as
> it exists in `rust/src/embedding_api.rs`, not an idealized envelope.

### The `vhdl_file` root

The parsed `dump_json` is, for a successful VHDL parse, a single typed root
object. Per `grammars/vhdl.ebnf` (line 11–12):

```ebnf
vhdl_file := design_unit*
          -> {type: "vhdl_file", design_units: $1}
```

```json
{
  "type": "vhdl_file",
  "design_units": [ /* array of design_unit shapes, source order */ ]
}
```

Consumers dispatch on `obj["type"] == "vhdl_file"` at the root, then iterate
`obj["design_units"]` — each element is one typed `design_unit` object in
source order.

### The 10-branch `design_unit` dispatch

`design_unit` is the primary top-level dispatcher. It is a 10-branch
`kind`-tagged shape (`grammars/vhdl.ebnf` line 14). Consumers dispatch on
`obj["kind"]`; every branch except `"semi"` carries a `body` holding the
underlying typed shape:

```ebnf
design_unit := library_clause              -> {kind: "library",            body: $1}
             | use_clause                  -> {kind: "use",                body: $1}
             | context_reference_clause    -> {kind: "context_reference",  body: $1}
             | entity_declaration          -> {kind: "entity",             body: $1}
             | architecture_body           -> {kind: "architecture",       body: $1}
             | package_declaration         -> {kind: "package",            body: $1}
             | package_body                -> {kind: "package_body",       body: $1}
             | configuration_declaration   -> {kind: "configuration",      body: $1}
             | context_declaration         -> {kind: "context",            body: $1}
             | semi                        -> {kind: "semi"}
```

| `kind` | `body` shape (fields) | Underlying rule (`grammars/vhdl.ebnf`) |
|---|---|---|
| `"library"` | `{first, rest}` — first identifier + comma-separated identifier iteration | `library_clause` (line 29) |
| `"use"` | `{first, rest}` — first selected name + comma-separated iteration | `use_clause` (line 31) |
| `"context_reference"` | `{name}` — the referenced context name | `context_reference_clause` (line 33) |
| `"entity"` | `{name, items, end_label}` — `items` is the `entity_declarative_item*` array; `end_label` is the optional trailing identifier | `entity_declaration` (line 51) |
| `"architecture"` | `{name, entity_name, items, statements, end_label}` — `items` are declarative items, `statements` are concurrent statements | `architecture_body` (line 84) |
| `"package"` | `{name, header, items, end_label}` — `header` is the optional `package_header` | `package_declaration` (line 100) |
| `"package_body"` | `{name, items, end_label}` | `package_body` (line 102) |
| `"configuration"` | `{name, entity_name, items, end_label}` — `items` is the `configuration_item*` array | `configuration_declaration` (line 144) |
| `"context"` | `{name, items, end_label}` — `items` is the `context_item*` array | `context_declaration` (line 35) |
| `"semi"` | _(no `body` — a lone top-level `;` separator)_ | `semi` (line 427) |

The `"semi"` branch is the only bodyless one: it carries solely
`{kind: "semi"}` for a stray top-level `;`. Every other branch carries
`{kind, body}` where `body` is the typed shape of the named rule. The full
per-family shapes (declarations, types, statements, the five-level
`binop_chain` expression hierarchy, literals) are enumerated in
`docs/vhdl_parser_book/src/rules-top-level.md`; the machine-checkable
enumeration of every `(rule, branch_index, annotation_type, normalized_text)`
tuple is `generated/vhdl_return_annotations.json` and its embedded copy
`rust/test_data/ast_shape_contract/vhdl_v1.json`.

## Declarations, Types, Statements, and Expressions

This section is the consumer-facing per-family shape contract: for every
VHDL rule family it enumerates the `kind` discriminator(s) and the exact
field list the parser emits. Every `kind` value, field name, and branch
count below is transcribed from the live inventory
`generated/vhdl_return_annotations.json`
(`version: 1`, `grammar: "vhdl"`, `annotation_count: 249`,
**110 distinct rules**), cross-checked against the embedded
shape-contract manifest `rust/test_data/ast_shape_contract/vhdl_v1.json`
(content-identical on the `(rule, branch_index, annotation_type,
normalized_text)` tuples; the embedded copy omits only the diagnostic
`raw_text` field), and consistent with the curated per-rule reference at
`docs/vhdl_parser_book/src/rules-top-level.md`. The top-level `vhdl_file`
root and the 10-branch `design_unit` dispatcher are documented in
[AST Envelope and `design_unit` Dispatch](#ast-envelope-and-design_unit-dispatch)
and are not repeated here.

Two emission conventions recur:

- **Dispatch rules** emit `{kind: "<branch>", body: $N}` (or per-branch
  named fields). Consumers dispatch on `obj["kind"]`. A bodyless
  `{kind: "semi"}` (and, for `sequential_statement`, `{kind: "null"}`)
  marks the lone-`;` / `null` branch — there is no `body`.
- **`{first, rest}` list rules** emit a head element plus the raw
  iteration envelope of the `(sep X)*` tail. This is VHDL's separated-list
  convention; `first` is the head, `rest` is the trailing-iteration array.

Field selectors below name JSON keys exactly as emitted; a field bound to
an optional grammar element is `[]` when that element is absent (e.g.
`signal_declaration.default`, `subtype_indication.constraint`,
`return_statement.value`, `function_specification.impure`).

### Context clauses and selected names

| Rule (`grammars/vhdl.ebnf`) | Shape |
|---|---|
| `library_clause` (line 29) | `{first, rest}` — first identifier + comma-separated identifier iteration. |
| `use_clause` (line 31) | `{first, rest}` — first selected name + comma-separated iteration. |
| `context_reference_clause` (line 33) | `{name}` — the referenced context name. |
| `context_item` (line 37, 4 kinds) | `{kind: "library", body}` / `{kind: "use", body}` / `{kind: "context_reference", body}` / `{kind: "semi"}` (bodyless). |
| `selected_name` (line 42) | `{first, rest}` — leading name segment + dotted-suffix iteration. |

### Design units

| Rule (`grammars/vhdl.ebnf`) | Shape |
|---|---|
| `entity_declaration` (line 51) | `{name, items, end_label}` — `items` is the `entity_declarative_item*` array; `end_label` the optional trailing identifier after `end`. |
| `architecture_body` (line 84) | `{name, entity_name, items, statements, end_label}` — `items` are declarative items, `statements` are concurrent statements. |
| `package_declaration` (line 100) | `{name, header, items, end_label}` — `header` is the optional `package_header`. |
| `package_body` (line 102) | `{name, items, end_label}`. |
| `configuration_declaration` (line 144) | `{name, entity_name, items, end_label}` — `items` is the `configuration_item*` array. |
| `context_declaration` (line 35) | `{name, items, end_label}` — `items` is the `context_item*` array. |
| `configuration_item` (line 146, 3 kinds) | `{kind: "for", target}` / `{kind: "use", body}` / `{kind: "semi"}` (bodyless). |

### Declarative-item dispatch rules

Each declarative region is a `kind`-tagged dispatch over the declarations
it admits. Every branch emits `{kind, body}` except the bodyless
`{kind: "semi"}` branch.

| Rule (`grammars/vhdl.ebnf`) | `kind` values (branch order) |
|---|---|
| `entity_declarative_item` (line 54, 5) | `"generic"`, `"port"`, `"component"`, `"use"`, `"semi"` |
| `architecture_declarative_item` (line 87, 12) | `"signal"`, `"constant"`, `"file"`, `"type"`, `"subtype"`, `"component"`, `"subprogram_declaration"`, `"subprogram_body"`, `"package_instantiation"`, `"alias"`, `"use"`, `"semi"` |
| `package_declarative_item` (line 107, 13) | `"subprogram_declaration"`, `"subprogram_body"`, `"type"`, `"subtype"`, `"constant"`, `"signal"`, `"file"`, `"component"`, `"package_instantiation"`, `"view"`, `"alias"`, `"use"`, `"semi"` |
| `package_body_declarative_item` (line 121, 8) | `"subprogram_declaration"`, `"subprogram_body"`, `"constant"`, `"signal"`, `"file"`, `"alias"`, `"use"`, `"semi"` |
| `subprogram_declarative_item` (line 161, 9) | `"variable"`, `"file"`, `"constant"`, `"type"`, `"subtype"`, `"subprogram_declaration"`, `"subprogram_body"`, `"use"`, `"semi"` |
| `process_declarative_item` (line 279, 9) | `"variable"`, `"file"`, `"constant"`, `"type"`, `"subtype"`, `"subprogram_declaration"`, `"subprogram_body"`, `"use"`, `"semi"` |

### Generic / port / parameter interfaces

| Rule (`grammars/vhdl.ebnf`) | Shape |
|---|---|
| `generic_clause` (line 60) | `{list}` — the inner `generic_interface_list`. |
| `generic_interface_list` | `{first, rest}`. |
| `generic_interface_element` (2 kinds) | `{kind: "value", names, subtype, default}` / `{kind: "package", body}`. |
| `port_clause` (line 71) | `{list}` — the inner `port_interface_list`. |
| `port_interface_list` | `{first, rest}`. |
| `port_interface_element` | `{names, mode, subtype, default}` — `mode` is the typed `signal_mode`. |
| `parameter_list` | `{first, rest}`. |
| `parameter_interface_element` | `{names, mode, subtype, default}` — `mode` is the typed `parameter_mode`. |
| `generic_map_aspect` | `{associations}` — the inner `association_list`. |
| `port_map_aspect` | `{associations}`. |
| `association_list` | `{first, rest}`. |
| `association_element` | `{formal, actual}`. |
| `actual_part` (2 kinds) | `{kind: "expression", body}` / `{kind: "open"}` (bodyless). |
| `actual_parameter_part` | `{first, rest}`. |
| `actual_parameter_element` (2 kinds) | `{kind: "association", body}` / `{kind: "range_expression", body}`. |
| `identifier_list` | `{first, rest}`. |

#### Mode keyword leaves

Each is a bare `{kind}` object — no `body`; the keyword token is
redundant with `kind`.

| Rule (`grammars/vhdl.ebnf`) | `kind` values |
|---|---|
| `signal_mode` (line 78, 5) | `"in"`, `"out"`, `"inout"`, `"buffer"`, `"linkage"` |
| `view_mode` (line 136, 5) | `"in"`, `"out"`, `"inout"`, `"buffer"`, `"linkage"` |
| `parameter_mode` (line 176, 3) | `"in"`, `"out"`, `"inout"` |

### Declarations

| Rule (`grammars/vhdl.ebnf`) | Shape |
|---|---|
| `signal_declaration` (line 184) | `{names, subtype, default}` — `default` is `[]` when no `:= expr` initializer. |
| `constant_declaration` (line 186) | `{names, subtype, value}`. |
| `variable_declaration` (line 188) | `{names, subtype, default}`. |
| `file_declaration` | `{names, subtype, open_mode, filename}`. |
| `subtype_declaration` | `{name, subtype}`. |
| `component_declaration` | `{name, generic, port, end_label}`. |
| `type_declaration` (line 197) | `{name, definition}` — `definition` is the typed `type_definition`. |
| `view_declaration` | `{name, subtype, elements}`. |
| `view_element` | `{name, mode}` — `mode` is the typed `view_mode`. |
| `alias_declaration` | `{name, target}`. |
| `package_header` | `{generic}`. |
| `package_generic_interface_element` | `{name, base, map}`. |
| `package_instantiation_declaration` | `{name, base, map}`. |
| `subprogram_declaration` (2 kinds) | `{kind: "procedure", spec}` / `{kind: "function", spec}`. |
| `subprogram_body` (2 kinds) | `{kind: "procedure", spec, items, statements, end_label}` / `{kind: "function", spec, items, statements, end_label}`. |
| `procedure_specification` | `{name, parameters}`. |
| `function_specification` | `{impure, name, parameters, return_type}` — `impure` is `[]` for a pure function, set when `impure` is present. |

### Types and constraints

| Rule (`grammars/vhdl.ebnf`) | Shape |
|---|---|
| `type_definition` (3 kinds) | `{kind: "enumeration", body}` / `{kind: "array", body}` / `{kind: "record", body}`. |
| `enumeration_type_definition` | `{first, rest}`. |
| `array_type_definition` | `{index_range, element_subtype}`. |
| `record_type_definition` | `{elements}`. |
| `record_element_declaration` | `{names, subtype}`. |
| `subtype_indication` (line 212) | `{type_mark, constraint}` — `constraint` is `[]` when the subtype is unconstrained. |
| `constraint` (line 214, 2 kinds) | `{kind: "range", body}` / `{kind: "index", body}`. |
| `range_constraint` | `{range}`. |
| `index_constraint` | `{first, rest}`. |
| `discrete_range` (line 220, 5 kinds) | `{kind: "range", body}` / `{kind: "attribute", body}` / `{kind: "name", body}` / `{kind: "subtype_range", subtype, range}` / `{kind: "subtype_box", subtype}`. |
| `range_expression` (line 226, 2 kinds) | `{kind: "to", low, high}` / `{kind: "downto", high, low}`. |

### Concurrent statements

| Rule (`grammars/vhdl.ebnf`) | Shape |
|---|---|
| `concurrent_statement` (line 233, 7 kinds) | `{kind, body}` for `"signal_assignment"`, `"assert"`, `"process"`, `"component_instantiation"`, `"generate"`, `"block"`; `{kind: "semi"}` (bodyless) for the lone-`;` branch. |
| `concurrent_signal_assignment_statement` | `{target, rhs}`. |
| `process_statement` (line 270) | `{label, sensitivity, items, statements, end_label}`. |
| `process_label` | `{name}`. |
| `sensitivity_clause` | `{list}` — the inner `sensitivity_list`. |
| `sensitivity_list` | `{first, rest}`. |
| `component_instantiation_statement` | `{label, unit, generic_map, port_map}` — `unit` is the typed `instantiated_unit`. |
| `instantiated_unit` (3 kinds) | `{kind: "component", name}` / `{kind: "entity", name}` / `{kind: "configuration", name}`. |
| `generate_statement` (2 kinds) | `{kind: "for", label, var, range, statements}` / `{kind: "if", label, condition, statements}`. |
| `block_statement` | `{label, items, statements, end_label}`. |

### Sequential statements

| Rule (`grammars/vhdl.ebnf`) | Shape |
|---|---|
| `sequential_statement` (line 289, 13 kinds) | `{kind, body}` for `"signal_assignment"`, `"variable_assignment"`, `"if"`, `"case"`, `"loop"`, `"exit"`, `"wait"`, `"assert"`, `"report"`, `"return"`, `"procedure_call"`; `{kind: "null"}` and `{kind: "semi"}` are bodyless. |
| `signal_assignment_statement` | `{target, rhs}`. |
| `signal_assignment_rhs` | `{value, conditional}` — `conditional` carries the optional `when`-clause tail. |
| `variable_assignment_statement` | `{target, value}`. |
| `target` (2 kinds) | `{kind: "name", name, params}` / `{kind: "aggregate", first, rest}`. |
| `if_statement` (line 316) | `{condition, then_body, elsif_branches, else_body}` — `elsif_branches` is the `(elsif … then …)*` iteration; `else_body` is `[]` when there is no `else`. |
| `case_statement` (line 319) | `{value, alternatives}`. |
| `case_statement_alternative` | `{choices, body}`. |
| `loop_statement` (line 328, 3 kinds) | `{kind: "while", condition, body}` / `{kind: "for", var, range, body}` / `{kind: "infinite", body}`. |
| `exit_statement` | `{condition_or_label}`. |
| `wait_statement` | `{timeout}` — the optional `for expression` clause. |
| `assert_statement` | `{condition, report, severity}`. |
| `report_statement` | `{message, severity}`. |
| `return_statement` | `{value}` — `value` is `[]` for a bare `return;`. |
| `procedure_call_statement` | `{name, params}`. |

### Expressions — the `binop_chain` left-fold contract

VHDL's expression grammar is a **five-level operator-precedence cascade**
(`grammars/vhdl.ebnf` lines 348–357):
`expression` → `relation` → `simple_expression` → `term` → `factor`.
Every level emits the same typed shape — a `binop_chain` object — so a
single consumer fold routine handles the entire expression tree:

```ebnf
expression        := relation (logical_operator relation)*
                  -> {type: "binop_chain", level: "logical",        lhs: $1, rest: $2}
relation          := simple_expression (relational_operator simple_expression)?
                  -> {type: "binop_chain", level: "relational",     lhs: $1, rest: $2}
simple_expression := (plus | minus)? term ((plus | minus | ampersand) term)*
                  -> {type: "binop_chain", level: "additive",       sign: $1, lhs: $2, rest: $3}
term              := factor ((star | slash | kw_mod | kw_rem) factor)*
                  -> {type: "binop_chain", level: "multiplicative", lhs: $1, rest: $2}
factor            := primary (power primary)?
                  -> {type: "binop_chain", level: "power",          lhs: $1, rest: $2}
```

| Level (`level`) | Rule (`grammars/vhdl.ebnf`) | Fields | Operators |
|---|---|---|---|
| `"logical"` | `expression` (line 348) | `lhs`, `rest` | `and` / `or` / `xor` / `nand` / `nor` / `xnor` |
| `"relational"` | `relation` (line 350) | `lhs`, `rest` | `=` / `/=` / `<` / `<=` / `>` / `>=` |
| `"additive"` | `simple_expression` (line 352) | `sign`, `lhs`, `rest` | unary `+`/`-` (`sign`); binary `+` / `-` / `&` |
| `"multiplicative"` | `term` (line 354) | `lhs`, `rest` | `*` / `/` / `mod` / `rem` |
| `"power"` | `factor` (line 356) | `lhs`, `rest` | `**` |

**Consumer left-fold rule (normative).** Every level emits
`{type: "binop_chain", level, lhs, rest}` where `lhs` is the leading
operand and `rest` is the iteration array of `(op, operand)` pairs.
**Consumers MUST fold `rest` left-associatively onto `lhs`** —
evaluate `lhs`, then for each `(op, operand)` pair in `rest` (in array
order) apply `op` with the running result as the left side and `operand`
as the right side. This left-fold is identical at all five levels by
construction, so one fold routine walks the whole expression tree.

Two level-specific notes:

- The **`"additive"`** level (`simple_expression`) carries an extra
  leading `sign` field for the optional unary `+`/`-`; `sign` is `[]`
  when no leading sign is present. Apply `sign` to the folded result of
  `lhs`/`rest`.
- The **`"relational"`** level's `rest` is **at most one `(op, operand)`
  pair** — the grammar uses `?`, not `*` (`relation := simple_expression
  (relational_operator simple_expression)?`). The `"logical"`,
  `"additive"`, and `"multiplicative"` levels iterate `*` (zero or more
  pairs); `"power"` (`factor`) likewise uses `?` and so carries at most
  one pair.

#### Operator leaves

`logical_operator` and `relational_operator` are typed bare-`{kind}`
leaves (no `body`); the `simple_expression`, `term`, and `factor`
operators are matched inline as literal tokens within `rest`, not via a
separate typed operator rule.

| Rule (`grammars/vhdl.ebnf`) | `kind` values |
|---|---|
| `logical_operator` (line 382, 6) | `"and"`, `"or"`, `"xor"`, `"nand"`, `"nor"`, `"xnor"` |
| `relational_operator` (line 389, 6) | `"eq"`, `"ne"`, `"lt"`, `"le"`, `"gt"`, `"ge"` |

### `primary` and aggregates

`primary` is the leaf-operand dispatcher at the bottom of the
`binop_chain` cascade.

| Rule (`grammars/vhdl.ebnf`) | Shape |
|---|---|
| `primary` (line 359, 7 kinds) | `{kind: "literal", body}` / `{kind: "aggregate", body}` / `{kind: "attribute_name", body}` / `{kind: "function_call", name, params}` / `{kind: "name", body}` / `{kind: "parens", expr}` / `{kind: "not", expr}`. |
| `attribute_name` | `{prefix, prefix_params, attribute, attribute_params}`. |
| `aggregate` (line 370, 2 kinds) | `{kind: "named_first", first_choices, first_value, rest}` / `{kind: "positional_first", first_value, second, rest}`. |
| `aggregate_element_association` (2 kinds) | `{kind: "named", choices, value}` / `{kind: "positional", value}`. |
| `aggregate_choice_list` | `{first, rest}`. |
| `aggregate_choice` (5 kinds) | `{kind: "others"}` (bodyless) / `{kind: "name", body}` / `{kind: "decimal", body}` / `{kind: "character", body}` / `{kind: "string", body}`. |
| `choices` | `{first, rest}`. |
| `choice` (2 kinds) | `{kind: "expression", body}` / `{kind: "others"}` (bodyless). |

### Literals

| Rule (`grammars/vhdl.ebnf`) | Shape |
|---|---|
| `literal` (line 396, 6 kinds) | `{kind, body}` for `"physical"`, `"decimal"`, `"based"`, `"bit_string"`, `"string"`, `"character"`. |
| `decimal_literal` (line 404) | `{value}`. |
| `based_literal` (line 406) | `{base, value}`. |

`physical_literal`, `bit_string_literal`, `string_literal`, and
`character_literal` are matched as terminal/regex leaves and carried
through the recursive-envelope shape (the string of matched text); they
carry no per-rule annotation — only the dispatch in `literal` is typed.

The above enumerates the full typed surface of contract `1.0.1`
(**249 annotations across 110 distinct rules**, schema version `1`).
This contract section is curated; the authoritative machine-checkable
enumeration of every `(rule, branch_index, annotation_type,
normalized_text)` tuple is `generated/vhdl_return_annotations.json` and
its embedded copy `rust/test_data/ast_shape_contract/vhdl_v1.json`. The
per-rule reference at `docs/vhdl_parser_book/src/rules-top-level.md`
mirrors these family groupings; if any disagree, the inventory artifact
wins, and this integration contract wins over the book.

## Source Of Truth
- Grammar source:
  - `grammars/vhdl.ebnf`
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Build-time generated parser discovery:
  - `rust/build.rs`
  - `PGEN_VHDL_PARSER_PATH`
- Live closure/status surface:
  - `LIVE_ACHIEVEMENT_STATUS.md`
- Machine-checkable shape inventory:
  - `generated/vhdl_return_annotations.json`
  - `rust/test_data/ast_shape_contract/vhdl_v1.json`

## Stable Integration Surface
- Grammar family:
  - `vhdl`
- Stable host profile:
  - `vhdl_1076_2019`
- Stable convenience entry points:
  - `parse_vhdl_1076_2019(...)`
  - `parse_vhdl_1076_2019_ast_dump(...)`
- Stable generic entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_ast_dump(...)`
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`

## Build / Availability Requirements
- Downstream consumers should inspect `parser_embedding_api_contract().supports_vhdl_generated_backend` during startup or build validation.
- The generated backend is resolved by `rust/build.rs`, not by importing internal generated parser modules directly.

## Validation / Release Gates
- Public host API stability:
  - `make -C rust SHELL=/bin/bash embedding_api_gate`
  - `make -C rust SHELL=/bin/bash nexsim_parser_embedding_contract_gate`
- Family closure/proof:
  - `make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_family_status_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_family_status_contract_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash vhdl_combined_telemetry_contract_gate`

## Scope / Non-Goals
- The stable downstream contract is the host-oriented embedding API, not internal generated parser modules or internal AST types.
- `vhdl` is still an `In Progress` family in the live tracker, so downstream integrators should treat the embedding surface as real but still pay attention to the current live blocker list in `LIVE_ACHIEVEMENT_STATUS.md`.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Companion Documentation — VHDL Parser Integration mdBook

This contract is the **downstream integration surface**: the host-API
envelope, the dispatch/rule-family shapes a consumer compiles against, and
the release/schema axes. It does not duplicate the per-rule walkthroughs or
worked examples — those live in the companion artifacts below. Each surface
is authoritative for a different thing; consult the matching one and respect
the precedence order stated at the end of this section.

| Surface | Path | Authoritative for |
|---|---|---|
| **This contract** | `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` | The downstream integration surface: AST-dump envelope, `vhdl_file` root, the 10-branch `design_unit` dispatch, and the per-family rule shapes (declarations, types, statements, the 5-level `binop_chain` expression hierarchy, literals). See [AST Envelope and `design_unit` Dispatch](#ast-envelope-and-design_unit-dispatch) and [Declarations, Types, Statements, and Expressions](#declarations-types-statements-and-expressions). |
| **Per-parser mdBook** | `docs/vhdl_parser_book/` (source `src/*.md`; tracked HTML at `docs/vhdl_parser_book-html/`) | The per-rule reference and teaching surface: build recipe, public API, AST-envelope walkthrough, every rule shape, per-feature worked examples, schema-versioning timeline, glossary, changelog index. Curated, not machine-checked. |
| **Shape-contract manifest** | `rust/test_data/ast_shape_contract/vhdl_v1.json` | The machine-checkable shape lock embedded in the regression test. Content-identical to the live inventory on the `(rule, branch_index, annotation_type, normalized_text)` tuples (the embedded copy omits only the diagnostic `raw_text` field). Drift fails the AST-shape-contract test. |
| **Declared-annotation inventory** | `generated/vhdl_return_annotations.json` | The live machine-checkable enumeration of every typed-shape annotation the VHDL grammar emits (`version: 1`, `grammar: "vhdl"`, `annotation_count: 249`, **110 distinct rules**). The generator-side source of truth for the typed surface. |
| **Embedding-API contract** | `rust/docs/EMBEDDING_API_CONTRACT.md` | The canonical host-API truth: the `AstDumpPayload` struct (`dump_json` / `truncated` / `full_bytes` / `emitted_bytes`), the entry-point signatures, the truncation diagnostic envelope, and the stable diagnostics. The struct shape this contract documents is transcribed from there. |
| **Released-parser bug ledger** | `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | The accepted-bug log for the released VHDL parser. Consult before integrating around a suspected parser defect; file new accepted bugs here per `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`. |

Precedence when surfaces disagree (highest first): the **embedding-API
contract** (`rust/docs/EMBEDDING_API_CONTRACT.md`) wins for the host-API /
`AstDumpPayload` truth; the **declared-annotation inventory**
(`generated/vhdl_return_annotations.json`) and its embedded
shape-contract manifest copy win for the exact typed-shape enumeration;
**this integration contract** wins over the **per-parser mdBook** for
downstream compliance. Report any disagreement as a documentation bug
rather than silently coding to the lower-precedence surface.

### Gate Recipe

The exact, copy-pasteable per-family commands a downstream integrator or
releaser runs. Each is verified against the repo (`rust/Makefile`,
`docs/vhdl_parser_book/src/build-recipe.md`,
`rust/src/ast_shape_contract.rs`); none are invented — do not substitute
flags.

**1. On-demand parser regen.** The VHDL parser is on-demand-only (not in the
default build). Build `ast_pipeline`, then regenerate the parser from
`grammars/vhdl.ebnf` (run from `rust/`, per
`docs/vhdl_parser_book/src/build-recipe.md`):

```bash
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/vhdl.ebnf \
    --generate-parser --output ../generated/vhdl_parser.rs
```

To wire the regenerated parser into a cargo build, point
`PGEN_VHDL_PARSER_PATH` at the absolute path of the generated file before
`cargo build --release --features generated_parsers` (see
`docs/vhdl_parser_book/src/build-recipe.md` § "Wiring into a downstream
Cargo build").

**2. Per-family book gate.** Builds the VHDL parser book and verifies the
tracked HTML landing pages (Makefile target `vhdl_parser_book_gate`,
`rust/Makefile` line 733):

```bash
make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_book_gate
```

**3. AST-shape-contract regression lock.** With the generated backend wired
in (`PGEN_VHDL_PARSER_PATH` exported), run the shape-contract test that
diffs the running generated parser against
`rust/test_data/ast_shape_contract/vhdl_v1.json` (test fn
`vhdl_ast_shape_contract_holds_against_running_generated_parser` in the
`pgen::ast_shape_contract` library module,
`rust/src/ast_shape_contract.rs` line 839):

```bash
cargo test --lib --features generated_parsers vhdl_ast_shape_contract
```

The substring `vhdl_ast_shape_contract` selects exactly the
`vhdl_ast_shape_contract_holds_against_running_generated_parser` test.
Any drift between the running parser's emitted shapes and the locked
manifest fails this test, surfacing the change before release.

**4. Family closure / proof gates.** Anyone publishing a parser-release
version bump also runs the closure/proof gates enumerated in
[Validation / Release Gates](#validation--release-gates) (e.g.
`make -C rust SHELL=/opt/homebrew/bin/bash vhdl_parser_family_status_gate`).
That section is the full list; it is not repeated here.

## Glossary

Contract-scoped definitions of the terms a downstream integrator needs to
read this document. Where a term has a normative definition, this contract
is authoritative; the per-parser book's
[glossary](../vhdl_parser_book/src/glossary.md) paraphrases the same terms
for quick lookup. Numbers below are pinned to contract `1.0.1` /
schema `1` / **249 annotations across 110 distinct rules**.

- **`AstDumpPayload`** — the success return of the VHDL AST-dump host
  entry points (defined in `rust/src/embedding_api.rs`, contract in
  `rust/docs/EMBEDDING_API_CONTRACT.md`). A canonical-JSON payload string
  plus truncation metadata, with **exactly four fields**: `dump_json`,
  `truncated`, `full_bytes`, `emitted_bytes`. It does **not** carry
  `root` / `schema_version` / `grammar` / `profile` members — see
  [The `AstDumpPayload` envelope](#the-astdumppayload-envelope) for the
  precise accuracy note.
- **`dump_json`** — the `AstDumpPayload` field holding the canonical
  (key-sorted) JSON encoding of the typed VHDL AST. Parse this string to
  obtain the `vhdl_file` root object. When `truncated` is `true` this
  string is replaced by the truncation diagnostic envelope, not the AST.
- **Truncation diagnostic envelope** — the deterministic JSON object that
  replaces the AST in `dump_json` when `max_ast_bytes` is exceeded. It
  carries `pgen_dump_contract_version` (currently `1`), `kind:
  "pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
  "parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`.
  Consumers must check `truncated` (or detect `kind ==
  "pgen_ast_dump_truncation"`) before treating `dump_json` as a VHDL AST.
- **AST-dump schema version** — the integer version axis tracking the AST
  output shape, currently `1`. Bumped only when the emitted shape changes
  in a way consumers may need to adapt to (new annotation on a
  previously-unannotated rule, restructured annotation, user-visible
  grammar-shape change). Pure perf work / internal codegen
  reorganization do not bump it. See [Schema Versioning](#schema-versioning).
- **Parser release version** — the parser library's release identity,
  currently `1.0.1`. Bumped on every functional change (bug fixes, perf
  work, grammar changes). Moves independently of the schema version.
- **`design_unit` dispatch** — the primary top-level dispatcher: a
  10-branch `kind`-tagged shape (`"library"`, `"use"`,
  `"context_reference"`, `"entity"`, `"architecture"`, `"package"`,
  `"package_body"`, `"configuration"`, `"context"`, `"semi"`). Every
  branch except the bodyless `"semi"` carries a `body`. Every parse
  roots at `{type: "vhdl_file", design_units: [...]}`; each element of
  `design_units` is one `design_unit` object. See
  [The 10-branch `design_unit` dispatch](#the-10-branch-design_unit-dispatch).
- **`binop_chain` left-fold** — the consumer-facing contract for VHDL's
  five-level operator-precedence cascade (`expression` → `relation` →
  `simple_expression` → `term` → `factor`). Every level emits
  `{type: "binop_chain", level, lhs, rest}`; consumers MUST fold `rest`
  left-associatively onto `lhs`. One fold routine walks the whole
  expression tree. See
  [Expressions — the `binop_chain` left-fold contract](#expressions--the-binop_chain-left-fold-contract).
- **Shape-contract manifest** — the embedded machine-checkable shape lock
  `rust/test_data/ast_shape_contract/vhdl_v1.json`. Content-identical to
  the declared-annotation inventory on the `(rule, branch_index,
  annotation_type, normalized_text)` tuples (omits only the diagnostic
  `raw_text` field). Drift fails the
  `vhdl_ast_shape_contract_holds_against_running_generated_parser`
  regression test (see [Gate Recipe](#gate-recipe)).
- **Declared-annotation inventory** — the live machine-checkable
  enumeration of every typed-shape annotation the VHDL grammar emits:
  `generated/vhdl_return_annotations.json` (`version: 1`,
  `grammar: "vhdl"`, `annotation_count: 249`, **110 distinct rules**).
  The generator-side source of truth for the typed surface; mirrored by
  the embedded shape-contract manifest copy.
- **Recursive envelope** — the default JSON shape produced by
  un-annotated rules: a recursive composition of arrays (sequences,
  quantified iterations, the `rest` tail of a `{first, rest}` list),
  strings (terminal/regex leaves), and matched-branch passthroughs (for
  alternations). Un-matched optionals are the empty array `[]`, never
  `null`. It is what a consumer reaches when descending below the typed
  surface (identifier tokens; physical / bit-string / string / character
  literals; the few utility rules with no per-rule annotation).
- **Generic host AST-dump surface** — the
  `parse_grammar_profile_ast_dump*` family
  (`parse_grammar_profile_ast_dump`,
  `parse_grammar_profile_ast_dump_with_limits`, the `*_result` and
  `*_named` forms). The grammar-agnostic entry points that, for the
  `vhdl` grammar + `vhdl_1076_2019` profile, return the same
  `AstDumpPayload` as the named convenience entry point
  `parse_vhdl_1076_2019_ast_dump`. Signatures are in
  `rust/docs/EMBEDDING_API_CONTRACT.md`; the stable entry-point list is
  in [Stable Integration Surface](#stable-integration-surface).
