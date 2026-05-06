# Top-Level Rules

This chapter describes the entry points of the SystemVerilog grammar and the AST shape they produce.

> **Status:** SV-Slice-1 (parser release `1.0.1`) typed `systemverilog_file` and `systemverilog_parseable_file`. SV-Slice-2 (parser release `1.0.2`) flattened `source_text` via `[$1**]`. SV-Slice-3 (parser release `1.0.3`) typed `source_text_item` per-branch. SV-Slice-4 (parser release `1.0.4`) typed `description` per-branch with `kind:` discriminator. SV-Slice-5 (parser release `1.0.5`) made `compiler_directive` transparent passthrough. SV-Slice-6 batch (parser release `1.0.6`) typed `attribute_instance` (`{first, rest}` shape) and `module_declaration_sv_2017` / `module_declaration_sv_2023` per-branch. SV-Slice-7 batch (parser release `1.0.7`) typed `module_keyword`, `lifetime`, `module_ansi_header`, `module_nonansi_header` — the module-header sub-tree. SV-Slice-8 batch (parser release `1.0.8`) typed the 4 identifier-leaf rules (`simple_identifier`, `escaped_identifier`, `non_keyword_identifier`, `simple_identifier_no_scope`) so clean identifier strings propagate through every typed parent rule's name field. SV-Slice-9 batch (parser release `1.0.9`) typed interface declarations (full mirror of module pattern). SV-Slice-10 batch (parser release `1.0.10`) typed class declarations, `package_declaration`, and program declarations (5 per-branch kinds each). SV-Slice-11 batch (parser release `1.0.11`) typed `program_ansi_header` and `program_nonansi_header`. SV-Slice-12 batch (parser release `1.0.12`) typed the UDP declaration family. SV-Slice-13 batch (parser release `1.0.13`) typed `bind_directive` (2 kinds), `bind_instantiation` (4 kinds), and `package_item` (4 kinds). SV-Slice-14 batch (parser release `1.0.14`) typed the rest of the bind sub-tree, `interface_class_declaration`, and `config_declaration`. SV-Slice-15 batch (parser release `1.0.15`) typed the port-list family + `anonymous_program` and `package_export_declaration`. SV-Slice-16 batch (parser release `1.0.16`) typed `port`, `port_direction`, `package_import_declaration`, `package_import_item`. SV-Slice-17 batch (parser release `1.0.17`) typed the UDP body sub-tree. SV-Slice-18 batch (parser release `1.0.18`) typed UDP truth-table entries. SV-Slice-19 batch (parser release `1.0.19`) typed the module-items dispatch tree (`module_item`, `module_or_generate_item`, `module_or_generate_item_declaration`, `non_port_module_item`, `continuous_assign`) — every `header.items` / `body.items` field on every typed module/interface/program declaration now exposes kind-discriminated dispatch. SV-Slice-20 batch (parser release `1.0.20`) mirrored that batch onto interface/program (`interface_item`, `interface_or_generate_item`, `non_port_interface_item`, `program_item`, `non_port_program_item`) — interface and program walks now match the module walk's typed-dispatch level (5 rules / 19 annotations). SV-Slice-21 batch (parser release `1.0.21`) typed `module_common_item` (both profiles) and `package_or_generate_item_declaration` (both profiles) — biggest batch yet at 4 rules / 55 annotations; closes the cascading walk path that SV-Slice-19/20 set up so every reachable common-item and package-or-generate-item-declaration discriminates its matched sub-construct. SV-Slice-22 batch (parser release `1.0.22`) typed the generate sub-tree: `generate_region` (`{items}`), `generate_item` (3 kinds), and `generate_block` (anonymous / labeled / generate_item passthrough) — closes the generate-construct walk path; 3 rules / 7 annotations. SV-Slice-23 batch (parser release `1.0.23`) typed the generate-construct internals: `loop_generate_construct` (`{init, condition, step, block}`), `conditional_generate_construct` (2 kinds), `if_generate_construct` (`{condition, then_block, else_clause}`), NEW helper rule `if_generate_else_clause` (2 kinds — workaround for task #38), `case_generate_construct` (`{expr, items}`), `case_generate_item` (2 kinds); 6 rules / 9 annotations + 1 new helper rule. SV-Slice-24 batch (parser release `1.0.24`) typed the assertion + genvar dispatch: `assertion_item` (2 kinds), `assertion_item_declaration` (3 kinds), `concurrent_assertion_item` (2 kinds), `genvar_initialization` (`{genvar_keyword, name, value}`), `genvar_iteration` (3 kinds), `assignment_operator` (13 kinds — bare `{kind}`), `inc_or_dec_operator` (2 kinds — bare `{kind}`); 7 rules / 26 annotations. SV-Slice-25 batch (parser release `1.0.25`) typed data/function/task declarations + bodies: `data_declaration_sv_2017/2023` (4 kinds each), `function_declaration_sv_2017/2023` (`{lifetime, body}` / `{dynamic_override, lifetime, body}`), `function_body_declaration` (`{return_type, name, items, statements, end_label}`), `task_declaration_sv_2017/2023` (parallel to function), `task_body_declaration` (no return_type); 8 rules / 14 annotations. SV-Slice-26 batch (parser release `1.0.26`) typed `net_declaration_sv_2017/2023` (3 kinds each: wire / alias / interconnect) using NEW helper rules `net_strength` (2 kinds: drive / charge) and `net_vector_scalar` (2 kinds: vectored / scalared, bare `{kind}`) — extracted from inline parens-Or to dodge task #38; 4 rules / 10 annotations + 2 helper rules. SV-Slice-27 batch (parser release `1.0.27`) typed the class body sub-tree: `class_item_sv_2017/2023` (8/9 kinds), `class_item_qualifier` (3 kinds bare), `class_constraint` (2 kinds), `class_property` (2 kinds: decl / const), `class_method` (6 kinds: task / function / pure_virtual / extern / constructor / extern_constructor); 6 rules / 30 annotations. SV-Slice-28 batch (parser release `1.0.28`) typed class qualifiers: `method_qualifier` (2 kinds: virtual / class_item_qualifier), `property_qualifier` (2 kinds: random / class_item_qualifier), `random_qualifier` (2 kinds bare: rand / randc); 3 rules / 6 annotations — completes SV-Slice-27's class body picture. SV-Slice-29 batch (parser release `1.0.29`) typed the concurrent assertion + constraint family: `concurrent_assertion_statement` (5 kinds), 6 individual property statements (assert / assume / cover_property / cover_sequence / restrict / expect), `constraint_block`, `constraint_block_item` (2 kinds), `constraint_declaration_sv_2017/2023`, `constraint_expression` (6 kinds), `constraint_prototype_sv_2017/2023`, `constraint_prototype_qualifier` (2 kinds bare), `constraint_set` (2 kinds); 16 rules / 28 annotations. SV-Slice-30 batch (parser release `1.0.30`) typed deferred immediate assertions: `deferred_immediate_assertion_item` (`{label, body}`), `deferred_immediate_assertion_statement` (3 kinds: assert / assume / cover), and the three statement rules each with 2 kinds (zero_delay / final); 5 rules / 10 annotations. SV-Slice-31 batch (parser release `1.0.31`) typed action_block + statement framing: `action_block` (2 kinds: always / with_else), `statement` (`{label, attributes, body}`), `statement_or_null` and `function_statement_or_null` (each 2 kinds), `tf_item_declaration` (2 kinds); 5 rules / 9 annotations — closes assertion action / function-task body walk paths. SV-Slice-32 batch (parser release `1.0.32`) typed statement_item dispatch: `statement_item_sv_2017` (20 kinds), `statement_item_sv_2023` (19 kinds — drops inc_or_dec_expression per LRM 2023), `block_item_declaration` (4 kinds); 3 rules / 43 annotations — crosses the 400-annotation milestone. SV-Slice-33 batch (parser release `1.0.33`) typed 7 of statement_item's procedural-statement forms: `disable_statement` (3 kinds), `jump_statement` (3 kinds), `wait_statement` (3 kinds), `event_trigger_sv_2017/2023` (2 kinds each), `procedural_timing_control_statement` (`{control, body}`), `procedural_timing_control` (3 kinds), `subroutine_call` (5 kinds), `subroutine_call_statement` (2 kinds), `seq_block` and `par_block` (each typed `{label, declarations, statements, ...}`); 11 rules / 26 annotations. SV-Slice-34 batch (parser release `1.0.34`) typed case + loop families: `case_statement` (`{unique_priority, keyword, expr, items}`), `case_keyword` (3 kinds bare), `case_item` / `case_pattern_item` / `case_inside_item_sv_2017/2023` (each 2 kinds), `loop_statement` (6 kinds: forever / repeat / while / for / do_while / foreach); 7 rules / 18 annotations. SV-Slice-35 batch (parser release `1.0.35`) typed `conditional_statement` (`{unique_priority, condition, then_body, else_body}`) using NEW helper rule `conditional_else_branch` (2 kinds: elseif / else) — closes the if-else dispatch path; third use of the helper-rule extraction pattern for task #38 workarounds. SV-Slice-36 batch (parser release `1.0.36`) typed assignments + procedural assertions + randcase: `nonblocking_assignment` (`{lvalue, control, value}`), `procedural_continuous_assignment` (6 kinds), `clocking_drive` (`{lvalue, cycle_delay, value}`), `randcase_statement` (`{items}`), `randcase_item` (`{weight, body}`), `procedural_assertion_statement` (3 kinds), `immediate_assertion_statement` (2 kinds), `variable_assignment` (`{lvalue, value}`); 8 rules / 16 annotations. After this slice 19 of statement_item's 19/20 kinds expose typed dispatch end-to-end (only blocking_assignment remains). SV-Slice-37 batch (parser release `1.0.37`) typed `blocking_assignment_sv_2017/2023` (4/5 kinds) using NEW helper rule `class_or_package_scope` (3 kinds — 4th use of the helper-rule extraction pattern); 3 rules / 12 annotations + 3 helper annotations. **After this slice, all 20 (sv_2017) / 19 (sv_2023) statement_item kinds expose typed dispatch end-to-end.** SV-Slice-38 batch (parser release `1.0.38`) typed `randsequence_statement_sv_2017/2023` (`{start, productions}`), `production_sv_2017` (`{return_type, name, ports, rules}`), and `production_item_sv_2017` (`{name, args}`); 4 rules / 4 annotations — closes the last raw-envelope statement_item kind. SV-Slice-39 batch (parser release `1.0.39`) typed the rs_* family (rs_case / rs_case_item / rs_code_block / rs_if_else / rs_prod / rs_production / rs_production_item / rs_production_list / rs_repeat / rs_rule / rs_weight_specification across both sv_2017 and sv_2023 profiles); 17 rules / 31 annotations — closes the random-sequence walk path end-to-end and crosses the 500-annotation milestone. SV-Slice-40 batch (parser release `1.0.40`) typed `simple_immediate_assertion_statement` (3 kinds), `simple_immediate_{assert,assume,cover}_statement` (each typed), `inc_or_dec_expression` (2 kinds: prefix / postfix), and `weight_specification_sv_2017` (3 kinds); 6 rules / 11 annotations. SV-Slice-41 batch (parser release `1.0.41`) typed the data_type family: `data_type` (15 kinds — pervasive impact across all data_type fields in the grammar), `data_type_or_implicit` / `data_type_or_void` / `data_type_or_incomplete_class_scoped_type_sv_2023` (each 2 kinds), `implicit_data_type` (`{signing, dims}`), `integer_atom_type` (6 kinds bare), `integer_vector_type` (3 kinds bare), `non_integer_type` (3 kinds bare), `integer_type` (2 kinds); 8 rules / 36 annotations. SV-Slice-42 batch (parser release `1.0.42`) typed signing + struct_union + enum + type_reference + class_type internals using NEW helper rules `union_modifier` and `class_type_head` (5th and 6th uses of the helper-rule extraction pattern); 9 rules / 21 annotations + 2 helper rules / 5 helper annotations. SV-Slice-43 batch (parser release `1.0.43`) typed the parameter_value_assignment + arguments family: `parameter_value_assignment_sv_2017/2023` (`{params}`), `list_of_parameter_assignments_sv_2017` / `list_of_parameter_value_assignments_sv_2023` (each 2 kinds), `named_parameter_assignment` / `named_argument` (each `{name, value}`), `list_of_arguments` (3 kinds), `list_of_arguments_ordered` / `_named` (each `{first, rest}`), `list_of_arguments_mixed` (`{head, named}`), `list_of_arguments_mixed_head` (2 kinds: single / chain); 10 rules / 16 annotations — crosses the 600-annotation milestone. SV-Slice-44 batch (parser release `1.0.44`) typed the small list_of_* family across 20 rules / 22 annotations: 12 simple `{first, rest}` rules (clocking / defparam / genvar / net / param / path / specparam / type / variable assignments), 3 `{first: {name, dims}, rest}` rules (interface / port / variable identifiers), 2 `{first: {name, dims, init}, rest}` rules (tf_variable / variable_port), 1 `{first, second, rest}` rule (cross_items), 2 2-kind dispatches (checker_port_connections, port_connections). SV-Slice-45 batch (parser release `1.0.45`) typed the pattern + cond_predicate family: `cond_predicate` (`{first, rest}`), `cond_pattern` (`{expression, pattern}`), `expression_or_cond_pattern` (2 kinds), `pattern_sv_2017` (6 kinds: variable_capture / wildcard / expression / tagged / ordered / named), `pattern_sv_2023` (7 kinds — adds parenthesized per LRM 2023), `assignment_pattern` (`{exprs}`); 6 rules / 18 annotations. SV-Slice-46 batch (parser release `1.0.46`) typed the expression family: `expression` (3 kinds), `expression_base` (3 kinds), `expression_operand` (3 kinds), `expression_or_dist`, `constant_expression` (`{first, rest, ternary}`), `constant_expression_operand` (2 kinds), `inside_expression_sv_2017/2023`, `conditional_expression`, `tagged_union_expression_sv_2017/2023`, `primary_literal` (4 kinds), `binary_operator` (29 kinds bare), `unary_operator` (11 kinds bare); 14 rules / 62 annotations — single largest impact slice. Crosses the 700-annotation milestone. SV-Slice-47 batch (parser release `1.0.47`) typed `primary_sv_2017` (15 kinds) and `constant_primary_sv_2017` (15 kinds) using NEW helper rules `primary_hier_scope_prefix`, `instance_or_class_scope`, `enum_id_scope_prefix` (7th/8th/9th uses of helper-rule extraction pattern); 2 rules / 30 annotations + 3 helper rules / 6 helper annotations.

## Entry points by profile

| Profile | Entry rule | Description |
|---|---|---|
| `sv_2017` | `systemverilog_file` (per LRM Annex A.1.1) | IEEE 1800-2017 source file. |
| `sv_2023` | `systemverilog_file` (per LRM Annex A.1.1, with 2023 deltas) | IEEE 1800-2023 source file. Same entry rule symbol; the 2023 grammar differs in interior rules where the LRM was extended. |

Both profiles share `grammars/systemverilog.ebnf` as the single source. The profile selection determines which top-level dispatcher rule is used at parse time.

## `systemverilog_file` (typed since SV-Slice-1)

Per `grammars/systemverilog.ebnf` line 184:

```ebnf
systemverilog_file := trivia source_text trivia
                   -> {type: "systemverilog_file", source_text: $2}
```

The annotation produces a typed JSON object at the root of every `sv_2017` / `sv_2023` parse. For an input like `"module m; endmodule\n"`:

```json
{
  "type": "systemverilog_file",
  "source_text": [/* source_text envelope */]
}
```

The `source_text` field is a flat array of `source_text_item` shapes (since SV-Slice-2 — see [`source_text` (flat array since SV-Slice-2)](#source_text-flat-array-since-sv-slice-2) below). Consumers walking the SV AST should dispatch on `obj["type"] == "systemverilog_file"` at the root level.

## `source_text` (flat array since SV-Slice-2)

Per `grammars/systemverilog.ebnf` line 2273:

```ebnf
source_text := source_text_item*
            -> [$1**]
```

The `[$1**]` flatten-spread idiom produces a clean flat array of `source_text_item` shapes. Pre-SV-Slice-2 this was the raw Quantified envelope of the iteration; consumers walking `obj["source_text"]` had to descend through a Quantified wrap. Post-fix the array is consumer-ready — iterate directly:

```rust
for item in obj["source_text"].as_array().unwrap() {
    walk_source_text_item(item);
}
```

For `module m; endmodule\n`, `source_text` has length 1 (the single `module_declaration` source-text item). For a multi-construct file (multiple modules + interfaces + packages), it carries one item per top-level construct in source order.

The inner `source_text_item` shapes are still **raw envelope** (Or of `description | local_parameter_declaration semi | parameter_declaration semi | package_import_declaration | bind_directive | ...`). Per-branch typing of source_text_item is a follow-up slice (will assign each branch a `kind:` discriminator).

See `rust/test_data/ast_shape_contract/systemverilog_v1.json` for the calibrated regression-lock sample.

## `description` (un-annotated)

`description` is the per-construct top-level alternative — module / interface / class / package / etc. Per LRM A.1.2:

```ebnf
description = module_declaration
            | udp_declaration
            | interface_declaration
            | program_declaration
            | package_declaration
            | (attribute_instance* package_item)
            | (attribute_instance* bind_directive)
            | config_declaration
```

Un-annotated `description` produces the matched-branch shape directly (no extra wrapping). When a slice annotates the per-branch dispatch, this rule will likely become:

```ebnf
description -> {type: "description", kind: "<branch>", body: $1}
```

(or similar) — actual annotation lands per-slice with the corresponding manifest update.

## After the first slice

Once the first annotation slice lands, this chapter will document:

- The `-> ...` annotation that landed on each top-level rule.
- A worked example (input → JSON tree) for `module m; endmodule\n`.
- The cumulative manifest entries.

For now, consult the [Walking the AST](walking-the-ast.md) walker pattern and use the recursive-envelope walk path for top-level rules.

## How to follow per-slice changes

Each annotation slice gets a row in [Schema Versioning](schema-versioning.md) and a Highlights section in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`. A row in the [Changelog Index](changelog-index.md) ties the two together with a one-paragraph summary.
