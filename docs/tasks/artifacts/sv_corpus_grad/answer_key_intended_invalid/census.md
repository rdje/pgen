# Suite-intended-invalid `must_accept` census — SV-CORPUS-GRAD.13x

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py`
>
> ⛔ **A MEASUREMENT, NOT A RULING.** Every row's `expected` / `adjudication` is read back from the
> tracked manifest; the rulings live in `stimuli/sv/adjudicate_external_corpus.py`'s pin tables and
> move only on an LRM citation (`.13k`'s discipline). This table shows the population before and
> after each ruling slice.

**Population: 48 rows** = `_bad|/error/|should_fail|_fail` on a `divergence:unexplained_rejects_valid` + `must_accept` row (47) + named EXTRA (1).

| suite | rows |
|---|---:|
| `verilator` | 41 |
| `sv2v` | 5 |
| `iverilog` | 2 |

| adjudication (manifest, as of this run) | rows |
|---|---:|
| `divergence:unexplained_rejects_valid` | 48 |

Tracked `positions.tsv` agrees with the live probe on **48 of 48** rows.
Edition contrast (`sv_2017` fail, `sv_2023` pass): **2** rows — `test_regress/t/t_class_override_bad.v`, `test_regress/t/t_param_type_fwd_bad.v`

## Rows

| # | suite | relpath | line:col | sv_2017 | sv_2023 | source at furthest | upstream testimony | expected | adjudication |
|---:|---|---|---|---|---|---|---|---|---|
| 1 | `iverilog` | `ivtest/ivltests/sv_class_new_typed_fail4.v` | 13:11 | fail@207 | fail@207 | `c = T::new; // This should fail, T is not a class` |  | `must_accept` | `divergence:unexplained_rejects_valid` |
| 2 | `iverilog` | `ivtest/ivltests/sv_void_cast_fail3.v` | 6:11 | fail@103 | fail@103 | `void'(1+2);` |  | `must_accept` | `divergence:unexplained_rejects_valid` |
| 3 | `sv2v` | `test/error/enum_range_x.sv` | 4:10 | fail@87 | fail@87 | `A['x]` | expected an integral number, but found 'x | `must_accept` | `divergence:unexplained_rejects_valid` |
| 4 | `sv2v` | `test/error/port_init_early.sv` | 3:12 | fail@148 | fail@148 | `output a = 1;` | declarations 'output a = 1' and 'logic a' are incompatible due to invalid initialization at port declaration | `must_accept` | `divergence:unexplained_rejects_valid` |
| 5 | `sv2v` | `test/error/severity_task_arg.sv` | 4:11 | fail@102 | fail@102 | `$fatal(.x("x"));` | unexpected keyword argument | `must_accept` | `divergence:unexplained_rejects_valid` |
| 6 | `sv2v` | `test/error/typedef_not_type_net.sv` | 5:7 | fail@132 | fail@132 | `w x;` | expected typename, but found net identifier "w" | `must_accept` | `divergence:unexplained_rejects_valid` |
| 7 | `sv2v` | `test/error/typedef_not_type_var.sv` | 5:7 | fail@131 | fail@131 | `v x;` | expected typename, but found var identifier "v" | `must_accept` | `divergence:unexplained_rejects_valid` |
| 8 | `verilator` | `test_regress/t/t_assert_imm_nz_bad.v` | 9:2 | fail@229 | fail@229 | `);` | %Error: t/t_assert_imm_nz_bad.v:14:11: Deferred assertions must use '#0' (IEEE 1800-2023 16.4) | `must_accept` | `divergence:unexplained_rejects_valid` |
| 9 | `verilator` | `test_regress/t/t_assign_automatic_bad.v` | 60:5 | fail@2126 | fail@2126 | `].s_dyn <= '1;  // OK: direct nonblocking assignment to dynamically-si` | %Error: t/t_assign_automatic_bad.v:36:10: Dynamically-sized variable not allowed in continuous assignment (IEE | `must_accept` | `divergence:unexplained_rejects_valid` |
| 10 | `verilator` | `test_regress/t/t_case_inside_bad.v` | 9:23 | fail@252 | fail@252 | `casex (1'bx) inside` | %Error: t/t_case_inside_bad.v:9:18: Illegal to have inside on a casex/casez | `must_accept` | `divergence:unexplained_rejects_valid` |
| 11 | `verilator` | `test_regress/t/t_class_extends_bad.v` | 13:23 | fail@272 | fail@272 | `class Cls extends Base1, Base2;` | %Error: t/t_class_extends_bad.v:13:26: Multiple inheritance illegal on non-interface classes (IEEE 1800-2023 8 | `must_accept` | `divergence:unexplained_rejects_valid` |
| 12 | `verilator` | `test_regress/t/t_class_extends_pkg_bad.v` | 22:23 | fail@413 | fail@413 | `class bar extends pkg;` | %Error-UNSUPPORTED: t/t_class_extends_pkg_bad.v:22:21: Unsupported: PACKAGE 'pkg' in 'class extends' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 13 | `verilator` | `test_regress/t/t_class_override_bad.v` | 23:10 | fail@819 | pass@-1 | `function :extends int get_e; return 1; endfunction  // Bad` | %Error: t/t_class_override_bad.v:23:25: Member 'get_e' marked ':extends' but no base class function is being e | `must_accept` | `divergence:unexplained_rejects_valid` |
| 14 | `verilator` | `test_regress/t/t_covergroup_autobins_bad.v` | 17:25 | fail@471 | fail@471 | `bins auto[size_var];` | %Error: t/t_covergroup_autobins_bad.v:17:12: Automatic bins array size must be a constant | `must_accept` | `divergence:unexplained_rejects_valid` |
| 15 | `verilator` | `test_regress/t/t_covergroup_with_function_foo_bad.v` | 8:33 | fail@240 | fail@240 | `covergroup cg_bad with function foo(int x);` | %Error: t/t_covergroup_with_function_foo_bad.v:8:35: Coverage sampling function must be named 'sample' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 16 | `verilator` | `test_regress/t/t_dpi_type_bad.v` | 11:8 | fail@371 | fail@371 | `import "DPI-BAD" task dpix_twice;` | %Error-UNSUPPORTED: t/t_dpi_type_bad.v:11:3: Unsupported DPI type 'DPI-BAD': Use 'DPI-C' (IEEE 1800-2023 35.5. | `must_accept` | `divergence:unexplained_rejects_valid` |
| 17 | `verilator` | `test_regress/t/t_enum_base_bad.v` | 25:18 | fail@498 | fail@498 | `enum logic [1:0][1:0] {  // BAD enum type` | %Error: t/t_enum_base_bad.v:11:11: Enum data type must be an integer atom or vector type (IEEE 1800-2023 6.19) | `must_accept` | `divergence:unexplained_rejects_valid` |
| 18 | `verilator` | `test_regress/t/t_foreach_bad.v` | 14:18 | fail@290 | fail@290 | `foreach (array);  // no index` | %Error: t/t_foreach_bad.v:14:5: Foreach missing bracketed loop variable is no-operation (IEEE 1800-2023 12.7.3 | `must_accept` | `divergence:unexplained_rejects_valid` |
| 19 | `verilator` | `test_regress/t/t_foreach_nindex_bad.v` | 12:37 | fail@287 | fail@287 | `foreach (array[i, j, badk, badl]);  // bad` | %Error: t/t_foreach_nindex_bad.v:12:32: foreach loop variables exceed number of indices of array | `must_accept` | `divergence:unexplained_rejects_valid` |
| 20 | `verilator` | `test_regress/t/t_foreach_type_bad.v` | 19:18 | fail@303 | fail@303 | `foreach (c[i]);  // bad type` | %Error: t/t_foreach_type_bad.v:19:16: Illegal 'foreach' loop on CLASSREFDTYPE 'Cls' data type | `must_accept` | `divergence:unexplained_rejects_valid` |
| 21 | `verilator` | `test_regress/t/t_fuzz_genintf_bad.v` | 23:13 | fail@398 | fail@398 | `j.e(0),` | %Error: t/t_fuzz_genintf_bad.v:23:8: Mixing positional and .*/named instantiation connection (IEEE 1800-2023 2 | `must_accept` | `divergence:unexplained_rejects_valid` |
| 22 | `verilator` | `test_regress/t/t_implements_contents_bad.v` | 7:21 | fail@224 | fail@224 | `interface class Icls;` | %Error: t/t_implements_contents_bad.v:8:7: Interface class cannot contain non-parameter members (IEEE 1800-202 | `must_accept` | `divergence:unexplained_rejects_valid` |
| 23 | `verilator` | `test_regress/t/t_implements_nested_bad.v` | 8:23 | fail@237 | fail@237 | `interface class inte;` | %Error: t/t_implements_nested_bad.v:9:15: Interface class shall not be nested within another interface class.  | `must_accept` | `divergence:unexplained_rejects_valid` |
| 24 | `verilator` | `test_regress/t/t_implements_noninterface_bad.v` | 10:32 | fail@260 | fail@260 | `class ClsBad1 implements NotIcls;` | %Error: t/t_implements_noninterface_bad.v:10:26: Attempting to implement from non-interface class 'NotIcls' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 25 | `verilator` | `test_regress/t/t_implements_notfound_bad.v` | 7:31 | fail@234 | fail@234 | `class ClsI implements Inotfound;` | %Error: t/t_implements_notfound_bad.v:7:23: Class for 'implements' not found: 'Inotfound' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 26 | `verilator` | `test_regress/t/t_inst_2star_bad.v` | 14:15 | fail@275 | fail@275 | `sub sub (foo, .*);` | %Error: t/t_inst_2star_bad.v:12:16: Duplicate .* in an instance (IEEE 1800-2023 23.3.2) | `must_accept` | `divergence:unexplained_rejects_valid` |
| 27 | `verilator` | `test_regress/t/t_inst_noname_bad.v` | 8:4 | fail@218 | fail@218 | `m();` | %Error: t/t_inst_noname_bad.v:8:3: Instance of module must be named | `must_accept` | `divergence:unexplained_rejects_valid` |
| 28 | `verilator` | `test_regress/t/t_lint_colonplus_bad.v` | 13:16 | fail@289 | fail@289 | `output [2:1] z = r[2 :+ 1];` | %Warning-COLONPLUS: t/t_lint_colonplus_bad.v:13:24: Perhaps instead of ':+' the intent was '+:'? | `must_accept` | `divergence:unexplained_rejects_valid` |
| 29 | `verilator` | `test_regress/t/t_lint_notredop_bad.v` | 12:17 | fail@278 | fail@278 | `assign y[0] = !\|v;` | %Error-NOTREDOP: t/t_lint_notredop_bad.v:12:17: Logical not directly before reduction operator is illegal | `must_accept` | `divergence:unexplained_rejects_valid` |
| 30 | `verilator` | `test_regress/t/t_mem_multi_ref_bad.v` | 19:14 | fail@513 | fail@513 | `dim2[0+:1][1] = 0;  // Bad: Range on non-bits` | %Error: t/t_mem_multi_ref_bad.v:15:9: Illegal bit or array select; type does not have a bit range, or bad dime | `must_accept` | `divergence:unexplained_rejects_valid` |
| 31 | `verilator` | `test_regress/t/t_mem_slice_bad.v` | 36:29 | fail@969 | fail@969 | `assign active_command3[1:0][2:0][3:0] = (use_AnB) ?  command_A3[1:0][2` | %Error: t/t_mem_slice_bad.v:36:30: Slice selection index '[2:0]' outside data type's '[1:0]' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 32 | `verilator` | `test_regress/t/t_param_implicit_bad.v` | 9:14 | fail@297 | fail@297 | `module sub1 #([7:0] PAR1 = 1);  // <--- Error: requires 'parameter'` | %Error: t/t_param_implicit_bad.v:9:15: parameter port declarations require 'parameter' keyword before implicit | `must_accept` | `divergence:unexplained_rejects_valid` |
| 33 | `verilator` | `test_regress/t/t_param_type_bad.v` | 9:24 | fail@275 | fail@275 | `localparam type bad2 = 2;  // Bad` | %Error: t/t_param_type_bad.v:9:26: Expecting a data type, not a constant: 2 | `must_accept` | `divergence:unexplained_rejects_valid` |
| 34 | `verilator` | `test_regress/t/t_param_type_fwd_bad.v` | 10:16 | fail@251 | pass@-1 | `parameter type enum E_t;` | %Error: t/t_param_type_fwd_bad.v:25:10: Parameter type expression type 'int' violates parameter's forwarding t | `must_accept` | `divergence:unexplained_rejects_valid` |
| 35 | `verilator` | `test_regress/t/t_prop_always_bad.v` | 17:27 | fail@466 | fail@466 | `assert property (s_always a);` | %Error: t/t_prop_always_bad.v:13:20: always range low bound must be non-negative (IEEE 1800-2023 16.12.11) | `must_accept` | `divergence:unexplained_rejects_valid` |
| 36 | `verilator` | `test_regress/t/t_property_case_bad.v` | 18:14 | fail@356 | fail@356 | `case (sel)` | %Error: t/t_property_case_bad.v:13:7: Multiple default statements in property case statement | `must_accept` | `divergence:unexplained_rejects_valid` |
| 37 | `verilator` | `test_regress/t/t_property_sexpr_bad.v` | 20:37 | fail@398 | fail@398 | `assert property (@(posedge clk) ##1 not val);` | %Error: t/t_property_sexpr_bad.v:20:39: Unexpected 'not' in sequence expression context | `must_accept` | `divergence:unexplained_rejects_valid` |
| 38 | `verilator` | `test_regress/t/t_randomize_with_idlist_method_bad.v` | 13:31 | fail@322 | fail@322 | `total = q.sum() with (item) { item > 0; };` | %Error: t/t_randomize_with_idlist_method_bad.v:13:21: 'with (...) { ... }' constraint block is only valid for  | `must_accept` | `divergence:unexplained_rejects_valid` |
| 39 | `verilator` | `test_regress/t/t_recursive_typedef_bad.v` | 12:24 | fail@422 | fail@422 | `parameter type A = B,` | %Error: t/t_recursive_typedef_bad.v:12:20: Recursive type definition | `must_accept` | `divergence:unexplained_rejects_valid` |
| 40 | `verilator` | `test_regress/t/t_timescale_udp.v` (extra) | 39:7 | fail@605 | fail@605 | `?     :  ?;` | (no golden .out) | `must_accept` | `divergence:unexplained_rejects_valid` |
| 41 | `verilator` | `test_regress/t/t_type_compare_bad.v` | 11:20 | fail@325 | fail@325 | `case (type(real))` | %Error: t/t_type_compare_bad.v:12:7: Case(type) statement requires items that have type() items | `must_accept` | `divergence:unexplained_rejects_valid` |
| 42 | `verilator` | `test_regress/t/t_udp_bad.v` | 14:34 | fail@291 | fail@291 | `primitive udp_x (a_bad, b, c_bad);` | %Warning-PINMISSING: t/t_udp_bad.v:10:9: Instance has missing pin: 'c_bad' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 43 | `verilator` | `test_regress/t/t_udp_bad_comb_trigger.v` | 13:17 | fail@312 | fail@312 | `0 ? 1  :   1;` | %Error: t/t_udp_bad_comb_trigger.v:14:6: There should not be a edge trigger for combinational UDP table line | `must_accept` | `divergence:unexplained_rejects_valid` |
| 44 | `verilator` | `test_regress/t/t_udp_bad_illegal_output.v` | 16:16 | fail@378 | fail@378 | `1 1 ?  :  ?: *;` | %Error: t/t_udp_bad_illegal_output.v:9:10: For sequential UDP, the output must be of 'reg' data type | `must_accept` | `divergence:unexplained_rejects_valid` |
| 45 | `verilator` | `test_regress/t/t_udp_bad_multi_output.v` | 8:14 | fail@258 | fail@258 | `output dout1, dout2;` | %Error: t/t_udp_bad_multi_output.v:8:17: 2 output ports for UDP table, there must be one output port | `must_accept` | `divergence:unexplained_rejects_valid` |
| 46 | `verilator` | `test_regress/t/t_udp_tableeof_bad.v` | 7:34 | fail@237 | fail@237 | `primitive udp_x (a_bad, b, c_bad);` | (no golden .out) | `must_accept` | `divergence:unexplained_rejects_valid` |
| 47 | `verilator` | `test_regress/t/t_var_dotted_dup_bad.v` | 9:10 | fail@224 | fail@224 | `generate` | %Error: t/t_var_dotted_dup_bad.v:14:15: Duplicate declaration of instance: 'dccm_bank' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 48 | `verilator` | `test_regress/t/t_var_dup_bad.v` | 61:2 | fail@770 | fail@770 | `);` | %Error: t/t_var_dup_bad.v:16:7: Duplicate declaration of signal: 'a' | `must_accept` | `divergence:unexplained_rejects_valid` |

## Instrument identity

Re-hash these inputs; if any hash differs from the row below, **this report no longer
describes your tree** and the honest act is to re-measure, not to quote.

| input | sha256 |
| --- | --- |
| `stimuli/sv/characterization/adjudication_manifest.tsv` | `e0f052741ee58d84713514925afe3a5a15e8808496bc14c7b85e801a8f83cdb9` |
| `stimuli/sv/characterization/positions.tsv` | `75dd16941ff3de8113bd0f9144031e5fa8e00980384a6c4c2633636af199538d` |
| `docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py` | `23f68af1ded2b07a8137cbd94a445d776b6e0ecf739494a07701a53d0330e48b` |
| `rust/target/release/parseability_probe` (parse binary) | `5f670862bad6f80de1f10402ea7c85be80e23a362f73e472cf2abbe76a6c6bac` |

