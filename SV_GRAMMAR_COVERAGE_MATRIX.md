# SV Grammar Coverage Matrix (IEEE 1800-2017 Anchored)

Last updated: 2026-02-27

## Purpose
Track `grammars/systemverilog.ebnf` coverage against IEEE 1800-2017 Annex-A-aligned syntax anchors, with explicit per-rule status and closure gaps.

## Status Legend
- `Seed Implemented`: rule exists and is exercised by current `EBNF -> JSON -> parser -> stimuli` flows.
- `Partial`: rule exists but anchor is incomplete, or referenced sub-rules are unresolved placeholders.
- `Missing`: anchor/rule family not represented yet in `grammars/systemverilog.ebnf`.

## Annex-A-Aligned Coverage Matrix (Seed `systemverilog_core_v0`)

| IEEE anchor (seed scope) | Grammar section | Rule count | Status | Notes |
| --- | --- | ---: | --- | --- |
| Syntax 23-1 / 24-1 / 25-1 / 26-1 (+ Clause 5 lexical baseline) | Compilation unit + top-level design-unit routing | 3 | Seed Implemented | `systemverilog_file`, `source_item`, `description`. |
| Syntax 23-1 / 24-1 / 25-1 / 26-1 | Module/interface/program/package/class declarations | 11 | Seed Implemented | Initial declaration headers + labels + class extension seed. |
| Syntax 23-x/24-x/25-x/26-x family | Body item routing | 14 | Partial | Includes references to unresolved placeholders (`modport_declaration`, `class_item`). |
| Syntax 23-2 and related parameter/port forms | Ports + parameters | 20 | Seed Implemented | ANSI/non-ANSI seed coverage and parameter/type assignment forms. |
| Syntax 23-5 / 23-6 (+ bind baseline) | Instantiation + bind | 17 | Seed Implemented | Module/interface/program instantiation + bind target forms. |
| Syntax 27-1 | Generate constructs | 8 | Seed Implemented | Conditional/loop generate baseline and generate-region wrappers. |
| Syntax 23/24/25/26 declaration families | Declarations | 32 | Partial | Contains unresolved placeholders (`block_item_declaration`, `checker_instantiation`, `kw_assert`). |
| Procedural statement baseline (core statement families) | Procedural constructs | 37 | Seed Implemented | `if/case/loop/timing/assign/call/jump` baseline. |
| Expression baseline | Expressions | 12 | Seed Implemented | Conditional, unary/binary, concatenation, call-containing primary forms. |
| Type baseline | Types | 18 | Seed Implemented | Integer/non-integer/scalar, enum, struct/union, packed/unpacked dimensions. |
| Hierarchical naming baseline | Hierarchical names | 4 | Seed Implemented | Rooted and segmented names with bit-select hooks. |
| Clause 5 lexical forms in seed scope | Attributes/directives/literals/identifiers | 23 | Seed Implemented | Identifier and literal families needed by current syntax baseline. |
| Lexical tokens + keywords used by seed grammar | Tokens and punctuation | 159 | Seed Implemented | Operator/keyword/punctuation inventory for current seed grammar. |
| Preprocessor grammar | SystemVerilog preprocessor | N/A | Tracked Separately | Covered by `grammars/systemverilog_preprocessor.ebnf` and `sv_preprocessor_quality_gate`. |

## Per-Rule Inventory (Current `grammars/systemverilog.ebnf`)

### Compilation unit + top-level routing (3, Seed Implemented)
`systemverilog_file`, `source_item`, `description`

### Module/interface/program/package/class declarations (11, Seed Implemented)
`module_declaration`, `module_keyword`, `module_header_ports`, `module_label`, `interface_declaration`, `program_declaration`, `package_declaration`, `package_label`, `class_declaration`, `class_extension`, `class_label`

### Items (14, Partial)
`module_item`, `non_port_module_item`, `interface_item`, `non_port_interface_item`, `program_item`, `non_port_program_item`, `program_generate_item`, `package_item`, `anonymous_program`, `anonymous_program_item`, `module_or_generate_item`, `interface_or_generate_item`, `module_or_generate_item_declaration`, `package_or_generate_item_declaration`

### Ports and parameters (20, Seed Implemented)
`parameter_port_list`, `parameter_port_declaration`, `parameter_declaration`, `local_parameter_declaration`, `list_of_param_assignments`, `param_assignment`, `list_of_type_assignments`, `type_assignment`, `list_of_ports`, `port`, `port_expression`, `port_reference`, `list_of_port_declarations`, `ansi_port_declaration`, `port_declaration`, `list_of_port_identifiers`, `port_identifier`, `port_direction`, `data_type_or_implicit`, `implicit_data_type`

### Instantiation + bind (17, Seed Implemented)
`module_instantiation`, `interface_instantiation`, `program_instantiation`, `parameter_value_assignment`, `list_of_parameter_assignments`, `ordered_parameter_assignment`, `named_parameter_assignment`, `hierarchical_instance`, `name_of_instance`, `list_of_port_connections`, `ordered_port_connection`, `named_port_connection`, `bind_directive`, `bind_target_scope`, `bind_target_instance`, `bind_target_instance_list`, `bind_instantiation`

### Generate (8, Seed Implemented)
`generate_region`, `generate_item`, `conditional_generate_construct`, `loop_generate_construct`, `genvar_initialization`, `genvar_expression`, `genvar_iteration`, `generate_block`

### Declarations (32, Partial)
`package_import_declaration`, `package_import_item`, `package_export_declaration`, `timeunits_declaration`, `net_declaration`, `net_type`, `list_of_net_decl_assignments`, `net_decl_assignment`, `data_declaration`, `list_of_variable_decl_assignments`, `variable_decl_assignment`, `variable_dimension`, `task_declaration`, `task_label`, `function_declaration`, `function_label`, `function_data_type_or_implicit`, `tf_port_list`, `tf_port_item`, `class_constructor_declaration`, `checker_declaration`, `checker_label`, `checker_item`, `covergroup_declaration`, `group_label`, `covergroup_item`, `dpi_import_export`, `extern_constraint_declaration`, `constraint_block`, `constraint_expression_list`, `assertion_item_declaration`, `assertion_item`

### Procedural constructs (37, Seed Implemented)
`continuous_assign`, `list_of_net_assignments`, `net_assignment`, `net_lvalue`, `initial_construct`, `final_construct`, `always_construct`, `statement_or_null`, `statement`, `begin_end_block`, `block_label`, `if_statement`, `case_statement`, `case_keyword`, `case_item`, `case_item_expression_list`, `loop_statement`, `for_initialization`, `for_variable_declaration`, `for_step`, `for_step_assignment`, `procedural_timing_statement`, `timing_control`, `delay_control`, `event_control`, `event_expression`, `jump_statement`, `blocking_assignment`, `nonblocking_assignment`, `delay_or_event_control`, `assignment_operator`, `subroutine_call_statement`, `subroutine_call`, `function_subroutine_call`, `method_call`, `system_tf_call`, `list_of_arguments`

### Expressions (12, Seed Implemented)
`expression`, `constant_expression`, `param_expression`, `conditional_expression`, `binary_expression`, `binary_operator`, `unary_expression`, `unary_operator`, `primary`, `concatenation`, `multiple_concatenation`, `variable_lvalue`

### Types (18, Seed Implemented)
`data_type`, `integer_vector_type`, `integer_atom_type`, `non_integer_type`, `signing`, `struct_union_type`, `struct_union_member`, `list_of_variable_identifiers`, `enum_data_type`, `enum_base_type`, `enum_name_declaration`, `class_type`, `list_of_type_arguments`, `type_or_expr`, `packed_dimension`, `unpacked_dimension`, `constant_range`, `constant_bit_select`

### Hierarchical names (4, Seed Implemented)
`hierarchical_identifier`, `hierarchical_segment`, `hierarchy_separator`, `root_keyword`

### Attributes/directives/literals/identifiers (23, Seed Implemented)
`attribute_instance`, `attr_spec`, `compiler_directive`, `identifier`, `simple_identifier`, `escaped_identifier`, `module_identifier`, `interface_identifier`, `program_identifier`, `package_identifier`, `instance_identifier`, `parameter_identifier`, `function_identifier`, `task_identifier`, `number`, `integral_number`, `real_number`, `unsigned_number`, `time_literal`, `time_unit`, `unbased_unsized_literal`, `string_literal`, `lifetime`

### Tokens + punctuation + keywords (159, Seed Implemented)
Token and keyword inventory is defined under the `Tokens and punctuation` section of `grammars/systemverilog.ebnf` and currently contains 159 rules.

## Known Closure Gaps (Detected from Rule-Reference Scan)

Current unresolved references in `grammars/systemverilog.ebnf`:
- `block_item_declaration`
- `checker_instantiation`
- `class_item`
- `kw_assert`
- `modport_declaration`

These keep the seed grammar in `Partial` status for item/declaration closure and should be closed before strict syntax-closure promotion for SV.

## How to Refresh this Matrix

1. Recompute grouped rule inventory after grammar edits.
2. Re-run unresolved-reference scan and update the gap list.
3. Re-run:
   - `make -C rust SHELL=/bin/bash hdl_frontend_readiness`
   - `make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
4. Update status rows and notes only from executable evidence (no speculative status promotion).
