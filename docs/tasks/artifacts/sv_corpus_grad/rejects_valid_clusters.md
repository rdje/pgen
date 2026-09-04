# `systemverilog` stuck-point clusters (SV-CORPUS-GRAD.3.0 engine; family lane CORPUS-GRAD-ALL.2.1)

116 rows probed from the adjudication manifest; 89 distinct 3-token stuck signatures. Keyed on `furthest_position` (the DEEP locus), family profile `systemverilog`.

| # | signature | rows | example (stuck line) |
|---|---|---|---|
| 1 | `. ID ;` | 4 | `localparam type p0_t = if0.rq_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef2.v) |
| 2 | `input ID ;` | 4 | `input n1;` (verible/verible/verilog/tools/kythe/testdata/module_port_type_forwarding.sv) |
| 3 | `" ID ID` | 3 | `$fatal(1, "slv_aw_select_i is %d: AW has selected a slave that is not defined.\` (Surelog/tests/AssumeProp/dut.sv) |
| 4 | `ID ( "` | 3 | `$display("test");` (Surelog/tests/IndexedSelect/dut.sv) |
| 5 | `ID = ID` | 3 | `coverage_level clevel = cover_all // This argument should be bound to an` (Surelog/tests/Checker/dut.sv) |
| 6 | `## NUM ID` | 2 | `trans ##1 start_trans ##1 s ##1 end_trans;` (ispras-sv-tests/ieee-1800-2012/16/16.08_04.sv) |
| 7 | `( ID )` | 2 | `(states_t) state1;` (Surelog/tests/EnumParenType/dut.sv) |
| 8 | `) ID [` | 2 | `foreach(array[i][j])` (Surelog/tests/ForeachSquare/dut.sv) |
| 9 | `. ID )` | 2 | `if (instr matches (tagged Jmp .j) &&&` (slang/tests/regression/all.sv) |
| 10 | `. ID .` | 2 | `localparam type p0_rq2_t = if0.y_if0.rq2_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef_nested.v) |
| 11 | `. ID ID` | 2 | `assign #sub.delay dly4 = dly1 + 1;` (verilator/test_regress/t/t_delay.v) |
| 12 | `:: ID ;` | 2 | `#1 ->my_package::e;` (iverilog/ivtest/ivltests/sv_wildcard_import2.v) |
| 13 | `= ID )` | 2 | `rst = $inferred_disable);` (Surelog/tests/SequenceInst/dut.sv) |
| 14 | `@ ( ID` | 2 | `@($global_clock) a[*1:$] ##1 b);` (Surelog/tests/CheckerInst/dut.sv) |
| 15 | `ID ( ID` | 2 | `test dut(foo);` (iverilog/ivtest/ivltests/program5b.v) |
| 16 | `ID = NUM` | 2 | `module top #(int N = 5, M = N * 16, type T = int, T x = 0) ();` (ispras-sv-tests/ieee-1800-2012/06/06.20.01_02.sv) |
| 17 | `] ID ;` | 2 | `word [2] array;      // word[0:1] exposes the bug as well` (iverilog/ivtest/ivltests/display_bug.v) |
| 18 | `] [ NUM` | 2 | `logic [2][3] x;` (sv2v/test/core/multipack_struct_cast.sv) |
| 19 | `logic ; /` | 2 | `reg [63:0] logic; // OK: 'logic' is not a keyword in 1364-2001` (ispras-sv-tests/ieee-1800-2012/22/22.14.01_01.sv) |
| 20 | `with { ID` | 2 | `if (std::randomize(example.member[1]) with {example.member[1] inside {[0 : 3]};} == 0) $stop;` (verilator/test_regress/t/t_randomize_arraysel_membersel.v) |
| 21 | `!= type (` | 1 | `if (type(value1) != type(value2)) $stop;` (verilator/test_regress/t/t_type_expression_compare.v) |
| 22 | `" ID "` | 1 | `$display("""First "quoted"\nsecond\` (verilator/test_regress/t/t_display_qqq.v) |
| 23 | `" ID \` | 1 | `$display("broken \` (sv-tests/tests/chapter-5/5.9-string-broken-line.sv) |
| 24 | `" [ %` | 1 | `$display("[%0t] Embedded\` (verilator/test_regress/t/t_display.v) |
| 25 | `( ) ;` | 1 | `->b.get_event();` (verilator/test_regress/t/t_event_control_pass.v) |
| 26 | `( input logic` | 1 | `extern function myfunc (input logic val);` (verilator/test_regress/t/t_interface_modport_export.v) |
| 27 | `) ( ID` | 1 | `module ModuleA #(P=1,) (inp,);` (sv2v/test/core/trailing_commas.sv) |
| 28 | `) ; ID` | 1 | `m1 #PAR m1();` (verilator/test_regress/t/t_param_no_parentheses.v) |
| 29 | `) ; integer` | 1 | `mreal #1.2 mr ();` (verilator/test_regress/t/t_param.v) |
| 30 | `) ID ++` | 1 | `foreach (queue[i]) foreach (queue[i][j]) exp_count_que++;` (verilator/test_regress/t/t_foreach_array.v) |
| 31 | `) begin /` | 1 | `foreach (y) begin` (verible/verible/verilog/tools/kythe/testdata/foreach.sv) |
| 32 | `) this .` | 1 | `foreach(this.foo.x[i][j])` (verilator/test_regress/t/t_foreach_nested.v) |
| 33 | `) type (` | 1 | `case (type(T))` (sv-tests/tests/chapter-6/6.23--type_op_compare.sv) |
| 34 | `+ = NUM` | 1 | `for (x += 1; x < y; x++)` (sv2v/test/core/for_loop_inits.sv) |
| 35 | `- NUM ]` | 1 | `enum {ndef[-1]} ndef1;` (iverilog/ivtest/ivltests/pr3366217d.v) |
| 36 | `. ID (` | 1 | `output wire .ao_rename(ao),  .bo_rename(bo)` (verilator/test_regress/t/t_inst_port_complex_unsup.v) |
| 37 | `. ID [` | 1 | `localparam type p0_t = if0.rq_t[1:0];` (verilator/test_regress/t/t_lparam_assign_iface_typedef4.v) |
| 38 | `: NUM :` | 1 | `$setuphold (posedge clk, negedge d, 0:0:0, 0:0:0);` (verilator/test_regress/t/t_setuphold.v) |
| 39 | `:: ID {` | 1 | `typedef enum pkg1::uint8_t {` (verilator/test_regress/t/t_typedef_package.v) |
| 40 | `; ` ID` | 1 | `class test extends uvm_test;` (verilator/test_regress/t/t_uvm_hello.v) |
| 41 | `; assign ID` | 1 | `wreal wr;` (verilator/test_regress/t/t_vams_basic.v) |
| 42 | `; end endmodule` | 1 | `foreach (a[i]) ;` (iverilog/ivtest/ivltests/sf1289.v) |
| 43 | `; endtask` | 1 | `@x[y];` (verible/verible/verilog/tools/kythe/testdata/event_control.sv) |
| 44 | `; var ID` | 1 | `var C;` (iverilog/ivtest/ivltests/sv_type_identifier_variable_name.v) |
| 45 | `= ID (` | 1 | `input wire integer x = F(P); // defining a default here is non-standard` (sv2v/test/core/port_default.sv) |
| 46 | `= ID ,` | 1 | `d = d_d, // local input logic inferred, default actual` (ispras-sv-tests/ieee-1800-2012/16/16.08.02_01.sv) |
| 47 | `= ID -` | 1 | `j = i++ + (i = i - 1);` (ispras-sv-tests/ieee-1800-2012/11/11.04.02_01.sv) |
| 48 | `= NUM '` | 1 | `output [3:0] i65 = 65'd3 ** a; // IWI` (verilator/test_regress/t/t_math_pow6.v) |
| 49 | `= NUM ;` | 1 | `baz.get_bar().f.x = 4;` (verilator/test_regress/t/t_class_static_member_sel.v) |
| 50 | `== type (` | 1 | `if (type(real) == type(logic[12:0])) $stop;` (verilator/test_regress/t/t_type_compare.v) |
| 51 | `ID != =` | 1 | `if (T::VALUE !== 23) begin` (iverilog/ivtest/ivltests/sv_type_identifier_package_name.v) |
| 52 | `ID / *` | 1 | `'b` (sv2v/test/lex/number_literal_whitespace.sv) |
| 53 | `ID ; /` | 1 | `scalar_t pc;` (Surelog/tests/HierBitSlice/dut.sv) |
| 54 | `ID ; initial` | 1 | `environment env;` (verible/verible/verilog/tools/kythe/testdata/program.sv) |
| 55 | `ID = new` | 1 | `static class2 instance1 = new();` (verible/verible/verilog/tools/kythe/testdata/reversed_definition.sv) |
| 56 | `ID NUM ;` | 1 | `example line 10;` (verilator/test_regress/t/t_pipe_filter.v) |
| 57 | `NUM ( NUM` | 1 | `assert #0_0 (1);` (sv2v/test/nosim/assert.sv) |
| 58 | `NUM / *` | 1 | `/* comment */ 1          /* comment */ // comment` (iverilog/ivtest/ivltests/br_gh782b.v) |
| 59 | `NUM ; ID` | 1 | `a = 15'1;` (iverilog/ivtest/ivltests/pr3515542.v) |
| 60 | `[ ID :` | 1 | `function f4(int [var1:var2] t1 [var3:var4]);` (verible/verible/verilog/tools/kythe/testdata/function_port_dimensions.sv) |
| 61 | `[ ID ::` | 1 | `int [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH] var5 [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH];` (verible/verible/verilog/tools/kythe/testdata/primitive_type_with_dimensions.sv) |
| 62 | `[ NUM :` | 1 | `typedef enum [0:0] {` (sv2v/test/core/enum.sv) |
| 63 | `\ : NUM` | 1 | `0   0  x \` (verilator/test_regress/t/t_udp_nonsequential.v) |
| 64 | `] ID ,` | 1 | `logic [4] a, b;` (sv2v/test/core/delay.sv) |
| 65 | `] ID =` | 1 | `S [WIDTH] s = '{` (sv2v/test/core/struct_unit_array.sv) |
| 66 | `` ID ID` | 1 | ``default_nettype none` (verilator/test_regress/t/t_gate_primitives_implicit_net.v) |
| 67 | `assign ID =` | 1 | `assign c = d;` (Surelog/tests/TranslateOff/dut.sv) |
| 68 | `automatic = NUM` | 1 | `integer automatic = 2;` (sv2v/test/lex/begin_keywords.sv) |
| 69 | `begin : ID` | 1 | `begin : foo` (sv2v/test/core/struct_scope.sv) |
| 70 | `begin ID ID` | 1 | `begin` (verilator/test_regress/t/t_interface_gen11.v) |
| 71 | `begin case (` | 1 | `begin` (verilator/test_regress/t/t_gen_for.v) |
| 72 | `begin if (` | 1 | `begin` (verilator/test_regress/t/t_interface_gen6.v) |
| 73 | `begin initial ID` | 1 | `begin` (verilator/test_regress/t/t_interface_gen13.v) |
| 74 | `begin localparam ID` | 1 | `begin` (verilator/test_regress/t/t_simulate_array.v) |
| 75 | `begin logic [` | 1 | `begin` (verilator/test_regress/t/t_cover_fsm_sel_assign.v) |
| 76 | `bit , ID` | 1 | `reg	     bit, rst, clk;` (Surelog/tests/1364_2005/dut.v) |
| 77 | `bit ; initial` | 1 | `integer bit; initial bit = 1;` (verilator/test_regress/t/t_flag_language.v) |
| 78 | `do ) ;` | 1 | `module t(input do);` (verilator/test_regress/t/t_langext_order.v) |
| 79 | `endclass : ID` | 1 | `endclass : ClassB` (ispras-sv-tests/ieee-1800-2012/08/08.26.04_01.sv) |
| 80 | `inout wire reg` | 1 | `inout wire reg p;` (ispras-sv-tests/ieee-1800-2012/06/06.07.01_02.sv) |
| 81 | `int ID )` | 1 | `property check_if_1(var int cyc_mod_2);` (verilator/test_regress/t/t_property_named.v) |
| 82 | `medium . ID` | 1 | `assign o = medium.c;` (Surelog/tests/HierPathModule/dut.sv) |
| 83 | `modport ID (` | 1 | `modport client_mp (output .client_req(req[i]));` (sv2v/test/core/interface_generate.sv) |
| 84 | `module` | 1 | `endmodule` (sv-tests/tests/chapter-11/11.3.6--two_assign_in_expr.sv) |
| 85 | `output reg ID` | 1 | `output reg a;` (iverilog/ivtest/ivltests/module_output_port_sv_var2.v) |
| 86 | `vectored [ NUM` | 1 | `logic vectored [15:0] a = 0;` (sv-tests/tests/chapter-6/6.9.2--vector_vectored_inv.sv) |
| 87 | `{ } endgroup` | 1 | `{}` (verilator/test_regress/t/t_covergroup_unsup.v) |
| 88 | `|| ID inside` | 1 | `constraint B { w == 5; x inside {1,2} || x inside {4,5}; }` (verilator/test_regress/t/t_randomize_method_constraints.v) |
| 89 | `~ ID ==` | 1 | `constraint unary { !(-~c == 'h22); }` (verilator/test_regress/t/t_constraint_operators.v) |

