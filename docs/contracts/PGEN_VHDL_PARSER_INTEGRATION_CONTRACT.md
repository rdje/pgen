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
