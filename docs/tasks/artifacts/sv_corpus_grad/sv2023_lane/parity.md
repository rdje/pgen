# Production parity, IEEE 1800-2017 vs 1800-2023, for the sv_2023 lane's inherited `must_reject` rulings (SV-CORPUS-GRAD.14b.3)

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/production_parity.py --write`; `--check` re-derives and diffs.

A 2017-lane `must_reject` whose basis names a 1800-2017 production carries over to the 2023 lane mechanically when every cited production reads IDENTICALLY in 1800-2023 Annex A (footnote digits, PDF page furniture and spacing normalised); a DIFFERS or MISSING production is read by hand (`SV2023_REREAD_BY_HAND` in the adjudicator).

## Instrument identity

> Re-hash the rows; if any differs, this census no longer describes your tree — re-run it, never quote it.

| input | repo-root-relative path | sha256 |
|---|---|---|
| sv_2017 manifest | `stimuli/sv/characterization/adjudication_manifest.tsv` | `600f7126f4cd997e9c09ae26021c6303ba228b3b4dd25ecc9a7f51a367037aca` |
| IEEE 1800-2017 Annex A | `docs/systemverilog/2017/md/section-41-data-read-api.md` | `249ab571c2ef3237790a6bd9743f07a7595bdded337ca2cbb6d5e189b3b40d22` |
| IEEE 1800-2023 Annex A | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md` | `0b4f7fe935e9c499a56ccde8dc769ed626bf53d154f8a9585bd1daf371a9d4aa` |
| instrument | `docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/production_parity.py` | `26506e3786c05f76f116bd25b9b59a8fa3ff55eb0f044d988d7956b01fd93045` |

## The census

`PRODUCTION-PARITY: rows=73 all_cited_identical=48 some_differs=8 some_missing=3 no_production_named=16 productions=77`

| production | verdict |
|---|---|
| `class_constructor_declaration` | DIFFERS |
| `class_declaration` | DIFFERS |
| `constant_primary` | DIFFERS |
| `constraint_primary` | DIFFERS |
| `elaboration_system_task` | MISSING-2023 |
| `forward_type` | MISSING-2017 |
| `local_parameter_declaration` | DIFFERS |
| `parameter_declaration` | DIFFERS |
| `parameter_port_declaration` | DIFFERS |
| `type_assignment` | DIFFERS |
| `type_parameter_declaration` | MISSING-2017 |

| suite | relpath | verdict | productions |
|---|---|---|---|
| `Surelog` | `tests/ForeachSquare/dut.sv` | IDENTICAL | `loop_statement:IDENTICAL loop_variables:IDENTICAL struct_union_member:IDENTICAL` |
| `Surelog` | `tests/InvalidTypeParam/dut.sv` | UNCITED | — |
| `Surelog` | `tests/MaskNeg/dut.sv` | UNCITED | — |
| `Surelog` | `tests/ParamNoDefault/dut.sv` | UNCITED | — |
| `Surelog` | `tests/SynthForeach/dut.sv` | IDENTICAL | `loop_statement:IDENTICAL loop_variables:IDENTICAL struct_union_member:IDENTICAL` |
| `Surelog` | `tests/UnsizedPacked/dut.sv` | IDENTICAL | `packed_dimension:IDENTICAL variable_dimension:IDENTICAL` |
| `iverilog` | `ivtest/ivltests/br_gh390a.v` | DIFFERS | `class_constructor_declaration:DIFFERS` |
| `iverilog` | `ivtest/ivltests/sv_new_array_error.v` | IDENTICAL | `variable_decl_assignment:IDENTICAL` |
| `iverilog` | `ivtest/ivltests/sv_void_cast_fail3.v` | IDENTICAL | `subroutine_call_statement:IDENTICAL` |
| `sv2v` | `test/core/interface_genvar_shadow.sv` | UNCITED | — |
| `sv2v` | `test/core/interface_similar.sv` | UNCITED | — |
| `sv2v` | `test/error/enum_range_x.sv` | IDENTICAL | `enum_name_declaration:IDENTICAL integral_number:IDENTICAL unbased_unsized_literal:IDENTICAL` |
| `sv2v` | `test/error/interface_param_mismatch_type.sv` | UNCITED | — |
| `sv2v` | `test/error/module_param_mismatch_type.sv` | UNCITED | — |
| `sv2v` | `test/error/parameter_no_default_1.sv` | UNCITED | — |
| `sv2v` | `test/error/parameter_no_default_2.sv` | UNCITED | — |
| `sv2v` | `test/error/parameter_no_default_3.sv` | UNCITED | — |
| `sv2v` | `test/error/port_init_early.sv` | IDENTICAL | `output_declaration:IDENTICAL variable_port_type:IDENTICAL var_data_type:IDENTICAL list_of_variable_port_identifiers:IDENTICAL` |
| `sv2v` | `test/error/severity_task_arg.sv` | MISSING | `elaboration_system_task:MISSING-2023 finish_number:IDENTICAL` |
| `sv2v` | `test/top/main.sv` | UNCITED | — |
| `verible` | `verible/verilog/tools/kythe/testdata/array.sv` | IDENTICAL | `seq_block:IDENTICAL` |
| `verible` | `verible/verilog/tools/kythe/testdata/module_port_type_forwarding.sv` | IDENTICAL | `port:IDENTICAL list_of_ports:IDENTICAL module_declaration:IDENTICAL` |
| `verible` | `verible/verilog/tools/lint/testdata/shadow_parameter.sv` | IDENTICAL | `function_body_declaration:IDENTICAL` |
| `verilator` | `test_regress/t/t_assert_imm_nz_bad.v` | IDENTICAL | `deferred_immediate_assert_statement:IDENTICAL module_declaration:IDENTICAL module_item:IDENTICAL` |
| `verilator` | `test_regress/t/t_case_inside_bad.v` | IDENTICAL | `case_keyword:IDENTICAL case_statement:IDENTICAL` |
| `verilator` | `test_regress/t/t_class_extends_bad.v` | DIFFERS | `class_declaration:DIFFERS` |
| `verilator` | `test_regress/t/t_class_super_new_bad_nfirst.v` | DIFFERS | `class_constructor_declaration:DIFFERS` |
| `verilator` | `test_regress/t/t_clk_concat5.v` | IDENTICAL | `port:IDENTICAL list_of_ports:IDENTICAL module_declaration:IDENTICAL` |
| `verilator` | `test_regress/t/t_clk_concat6.v` | IDENTICAL | `port:IDENTICAL list_of_ports:IDENTICAL module_declaration:IDENTICAL` |
| `verilator` | `test_regress/t/t_comb_do_not_convert_to.v` | IDENTICAL | `port:IDENTICAL list_of_ports:IDENTICAL module_declaration:IDENTICAL` |
| `verilator` | `test_regress/t/t_covergroup_autobins_bad.v` | IDENTICAL | `bins_or_options:IDENTICAL` |
| `verilator` | `test_regress/t/t_covergroup_with_function_foo_bad.v` | IDENTICAL | `coverage_event:IDENTICAL` |
| `verilator` | `test_regress/t/t_dpi_type_bad.v` | IDENTICAL | `dpi_spec_string:IDENTICAL` |
| `verilator` | `test_regress/t/t_enum_base_bad.v` | IDENTICAL | `enum_base_type:IDENTICAL` |
| `verilator` | `test_regress/t/t_final.v` | IDENTICAL | `final_construct:IDENTICAL function_statement:IDENTICAL statement_or_null:IDENTICAL` |
| `verilator` | `test_regress/t/t_flag_language.v` | UNCITED | — |
| `verilator` | `test_regress/t/t_foreach_bad.v` | IDENTICAL | `loop_statement:IDENTICAL` |
| `verilator` | `test_regress/t/t_foreach_nindex_bad.v` | IDENTICAL | `statement_or_null:IDENTICAL loop_statement:IDENTICAL` |
| `verilator` | `test_regress/t/t_foreach_type_bad.v` | IDENTICAL | `loop_statement:IDENTICAL statement_or_null:IDENTICAL` |
| `verilator` | `test_regress/t/t_fuzz_genintf_bad.v` | IDENTICAL | `list_of_port_connections:IDENTICAL ordered_port_connection:IDENTICAL named_port_connection:IDENTICAL` |
| `verilator` | `test_regress/t/t_implements_contents_bad.v` | IDENTICAL | `interface_class_item:IDENTICAL` |
| `verilator` | `test_regress/t/t_implements_nested_bad.v` | IDENTICAL | `interface_class_item:IDENTICAL` |
| `verilator` | `test_regress/t/t_inst_2star_bad.v` | IDENTICAL | `named_port_connection:IDENTICAL list_of_port_connections:IDENTICAL` |
| `verilator` | `test_regress/t/t_inst_noname_bad.v` | IDENTICAL | `hierarchical_instance:IDENTICAL name_of_instance:IDENTICAL` |
| `verilator` | `test_regress/t/t_langext_order.v` | UNCITED | — |
| `verilator` | `test_regress/t/t_lint_colonplus_bad.v` | IDENTICAL | `output_declaration:IDENTICAL variable_port_type:IDENTICAL var_data_type:IDENTICAL list_of_variable_port_identifiers:IDENTICAL` |
| `verilator` | `test_regress/t/t_lint_notredop_bad.v` | IDENTICAL | `expression:IDENTICAL port_expression:IDENTICAL port_reference:IDENTICAL` |
| `verilator` | `test_regress/t/t_lint_paramnodefault.v` | UNCITED | — |
| `verilator` | `test_regress/t/t_lparam_assign_iface_typedef2.v` | IDENTICAL | `data_type:IDENTICAL` |
| `verilator` | `test_regress/t/t_lparam_assign_iface_typedef_nested_mod1.v` | IDENTICAL | `data_type:IDENTICAL` |
| `verilator` | `test_regress/t/t_lparam_assign_iface_typedef_nested_mpkg4.v` | IDENTICAL | `data_type:IDENTICAL` |
| `verilator` | `test_regress/t/t_lparam_assign_iface_typedef_nested_mpkg5.v` | IDENTICAL | `data_type:IDENTICAL` |
| `verilator` | `test_regress/t/t_mem_multi_ref_bad.v` | IDENTICAL | `select:IDENTICAL bit_select:IDENTICAL` |
| `verilator` | `test_regress/t/t_mem_slice_bad.v` | IDENTICAL | `bit_select:IDENTICAL` |
| `verilator` | `test_regress/t/t_param_implicit_bad.v` | DIFFERS | `parameter_port_declaration:DIFFERS` |
| `verilator` | `test_regress/t/t_param_type_bad.v` | DIFFERS | `local_parameter_declaration:DIFFERS type_assignment:DIFFERS` |
| `verilator` | `test_regress/t/t_param_type_fwd.v` | DIFFERS | `type_parameter_declaration:MISSING-2017 parameter_declaration:DIFFERS` |
| `verilator` | `test_regress/t/t_param_type_fwd_bad.v` | DIFFERS | `type_parameter_declaration:MISSING-2017 forward_type:MISSING-2017 parameter_declaration:DIFFERS` |
| `verilator` | `test_regress/t/t_prop_always_bad.v` | IDENTICAL | `property_expr:IDENTICAL` |
| `verilator` | `test_regress/t/t_property_case_bad.v` | IDENTICAL | `property_expr:IDENTICAL` |
| `verilator` | `test_regress/t/t_property_sexpr_bad.v` | IDENTICAL | `sequence_expr:IDENTICAL property_expr:IDENTICAL` |
| `verilator` | `test_regress/t/t_randomize_with_idlist_method_bad.v` | IDENTICAL | `array_manipulation_call:IDENTICAL randomize_call:IDENTICAL` |
| `verilator` | `test_regress/t/t_timescale_udp.v` | IDENTICAL | `combinational_entry:IDENTICAL output_symbol:IDENTICAL` |
| `verilator` | `test_regress/t/t_type_compare_bad.v` | DIFFERS | `case_expression:IDENTICAL constraint_primary:DIFFERS constraint_expression:IDENTICAL constant_primary:DIFFERS` |
| `verilator` | `test_regress/t/t_udp_bad.v` | IDENTICAL | `udp_port_declaration:IDENTICAL` |
| `verilator` | `test_regress/t/t_udp_bad_comb_trigger.v` | IDENTICAL | `combinational_entry:IDENTICAL sequential_entry:IDENTICAL sequential_body:IDENTICAL udp_initial_statement:IDENTICAL init_val:IDENTICAL seq_input_list:IDENTICAL level_input_list:IDENTICAL edge_input_list:IDENTICAL` |
| `verilator` | `test_regress/t/t_udp_bad_illegal_output.v` | IDENTICAL | `next_state:IDENTICAL output_symbol:IDENTICAL` |
| `verilator` | `test_regress/t/t_udp_bad_multi_output.v` | IDENTICAL | `udp_output_declaration:IDENTICAL` |
| `verilator` | `test_regress/t/t_udp_tableeof_bad.v` | IDENTICAL | `combinational_body:IDENTICAL udp_declaration:IDENTICAL` |
| `verilator` | `test_regress/t/t_var_dotted_dup_bad.v` | IDENTICAL | `generate_region:IDENTICAL module_or_generate_item:IDENTICAL generate_block:IDENTICAL` |
| `verilator` | `test_regress/t/t_var_dup_bad.v` | IDENTICAL | `module_declaration:IDENTICAL module_item:IDENTICAL` |
| `verilator` | `test_regress/t/t_var_lifetime_module_bad.v` | UNCITED | — |
| `verilator` | `test_regress/t/t_var_rsvd.v` | UNCITED | — |
