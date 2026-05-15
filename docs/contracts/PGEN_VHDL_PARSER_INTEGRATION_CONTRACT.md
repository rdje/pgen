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

## Source Of Truth

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
