# rejects-valid stuck-point clusters (SV-CORPUS-GRAD.3.0)

315 rows probed from the adjudication manifest; 171 distinct 3-token stuck signatures.

| # | signature | rows | example (stuck line) |
|---|---|---|---|
| 1 | `class ID ;` | 18 | `interface class Bar; endclass` (sv-tests/tests/generic/class/class_test_28.sv) |
| 2 | `ID ; pure` | 12 | `interface class ihello;` (sv-tests/tests/chapter-8/8.26.2--implements.sv) |
| 3 | `; / /` | 8 | `reg logic; // OK: "logic" is not a keyword in 1364-2001` (sv-tests/tests/chapter-5/5.6.4--compiler-directives-begin-keywords.sv) |
| 4 | `ID ) ;` | 8 | `modport modp(input clk, rst);` (verilator/test_regress/t/t_interface_array_parameter_access.v) |
| 5 | `[ NUM :` | 8 | `length dist { [0:1], [2:5] :/ 2, 6 := 6, 7 := 10, 1};` (verilator/test_regress/t/t_constraint_json_only.v) |
| 6 | `ID NUM ;` | 7 | `a = -8'd 6;  // this defines the two's-complement of 6,` (sv-tests/tests/chapter-5/5.7.1--integers-signed.sv) |
| 7 | `with { ID` | 7 | `if (std::randomize(example.member[1]) with {example.member[1] inside {[0 : 3]};} == 0) $stop;` (verilator/test_regress/t/t_randomize_arraysel_membersel.v) |
| 8 | `ID ; typedef` | 6 | `interface class ihello;` (sv-tests/tests/chapter-8/8.26.3--type_access_extends.sv) |
| 9 | `interface class ID` | 6 | `package Package;` (sv-tests/tests/generic/class/class_test_25.sv) |
| 10 | `{ ID =` | 6 | `typedef enum [15:0] {` (verilator/test_regress/t/t_cast.v) |
| 11 | `> ID )` | 5 | `assert property ( @(posedge clk) disable iff (a) b |-> c );` (sv-tests/tests/chapter-16/16.12--property-disable-iff.sv) |
| 12 | `> ID ==` | 5 | `@(posedge clk) cyc % 2 == cyc_mod_2 |=> val == expected;` (verilator/test_regress/t/t_assert_clock_event_unsup.v) |
| 13 | `ID , ID` | 5 | `modport slave(input req, addr, clk, output gnt, input data);` (verilator/test_regress/t/t_interface_import_param.v) |
| 14 | `` ID ID` | 5 | `module ts();` (sv-tests/tests/chapter-5/5.6.4--compiler-directives-pragma.sv) |
| 15 | `' ID NUM` | 4 | `parameter int foo = 32 'd 1;` (sv-tests/tests/generic/number/number_test_28.sv) |
| 16 | `, weak1 )` | 4 | `assign (supply0, weak1) #(1:0:1, 2:1:0) w = 2;` (slang/tests/regression/all.sv) |
| 17 | `. ID ;` | 4 | `localparam type p0_t = if0.rq_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef2.v) |
| 18 | `| -> ID` | 4 | `assert property (@(posedge clk) a [* 25700000] |-> b);` (verilator/test_regress/t/t_assert_consec_rep_unroll_limit_bad.v) |
| 19 | `: = NUM` | 3 | `constraint c { b dist {3 := 1, 10 := 2}; }` (sv-tests/tests/chapter-18/18.5.4--distribution_0.sv) |
| 20 | `: ID for` | 3 | `begin : topgen` (verilator/test_regress/t/t_gen_local.v) |
| 21 | `> ( ID` | 3 | `assert property (@(posedge clk) fire |-> (cnt == 0))` (verilator/test_regress/t/t_assert_preponed_nba.v) |
| 22 | `> ID [` | 3 | `@(posedge clk) a |-> b [-> 1: n];` (verilator/test_regress/t/t_assert_rep_range_bad.v) |
| 23 | `ID # (` | 3 | `interface class ic1#(type T = logic);` (sv-tests/tests/chapter-8/8.26.6.2--parameter_type_conflict.sv) |
| 24 | `ID = NUM` | 3 | `tri1 scalared [15:0] a = 0;` (sv-tests/tests/chapter-6/6.9.2--vector_scalared.sv) |
| 25 | `| -> ##` | 3 | `@(posedge clk) (valid, x = in) |-> ##4 (out == x + 3);` (sv-tests/tests/chapter-16/16.10--property-local-var-fail.sv) |
| 26 | `' ( ID` | 2 | `if ((cyc[3+32'(t1&&t2)+:2] == cyc[5+32'(t3||t4)+:2]) || cyc[31]) $write("");` (verilator/test_regress/t/t_cover_expr.v) |
| 27 | `) ; /` | 2 | `class Cls1 extends Base1(default);` (verilator/test_regress/t/t_class_extends_default.v) |
| 28 | `) ; ID` | 2 | `m();` (verilator/test_regress/t/t_inst_noname_bad.v) |
| 29 | `) ; endclass` | 2 | `parameter type P_T = i` (verilator/test_regress/t/t_param_type_id_bad.v) |
| 30 | `) begin /` | 2 | `foreach (y) begin` (verible/verible/verilog/tools/kythe/testdata/foreach.sv) |
| 31 | `, ID ;` | 2 | `class Cls extends Base1, Base2;` (verilator/test_regress/t/t_class_extends_bad.v) |
| 32 | `- NUM ]` | 2 | `q = q[0:$-1]; // void'(q.pop_back()) or q.delete(q.size-1)` (sv-tests/tests/chapter-7/queues/pop_back_assing.sv) |
| 33 | `. ID (` | 2 | `j.e(0),` (verilator/test_regress/t/t_fuzz_genintf_bad.v) |
| 34 | `. ID .` | 2 | `localparam type p0_rq2_t = if0.y_if0.rq2_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef_nested.v) |
| 35 | `. ID ID` | 2 | `assign #sub.delay dly4 = dly1 + 1;` (verilator/test_regress/t/t_delay.v) |
| 36 | `: ID ID` | 2 | `begin : TestIf` (verilator/test_regress/t/t_interface_gen10.v) |
| 37 | `; ID (` | 2 | `p.m_pack[0].rand_mode(0);` (verilator/test_regress/t/t_randomize_rand_mode_bad.v) |
| 38 | `; endmodule` | 2 | `input n1;` (verible/verible/verilog/tools/kythe/testdata/module_port_type_forwarding.sv) |
| 39 | `; logic [` | 2 | `input clk;` (verilator/test_regress/t/t_clk_concat5.v) |
| 40 | `== NUM )` | 2 | `assert((std::randomize(x) with {x inside {10, 20}; }) == 1)` (verilator/test_regress/t/t_randomize_inside_cond.v) |
| 41 | `ID ( "` | 2 | `$display(":assert: (%d == %d)", b, (e+c+2));` (sv-tests/tests/chapter-11/11.3.6--two_assign_in_expr-sim.sv) |
| 42 | `ID ( .` | 2 | `t1 t (.mod_intf(mod_intfs[0]));` (verilator/test_regress/t/t_interface_gen11.v) |
| 43 | `ID , output` | 2 | `modport source(input clk, rst, output data);` (verilator/test_regress/t/t_interface_and_struct_pattern.v) |
| 44 | `[ NUM ]` | 2 | `dim2[0+:1][1] = 0;  // Bad: Range on non-bits` (verilator/test_regress/t/t_mem_multi_ref_bad.v) |
| 45 | `` ID ,` | 2 | `initial $display("At %s @ %d\n", `__FILE__, `__LINE__);` (sv-tests/tests/chapter-5/5.6.4--compiler-directives-debug.sv) |
| 46 | `` ID wire` | 2 | `endfunction` (verilator/test_regress/t/t_lint_implicit_func_bad.v) |
| 47 | `case ( ID` | 2 | `unique0 case (i)` (verilator/test_regress/t/t_langext_2.v) |
| 48 | `class ID #` | 2 | `interface class Bar #(parameter N); endclass` (sv-tests/tests/generic/class/class_test_26.sv) |
| 49 | `if ( ID` | 2 | `unique0 if(a == 0) b = 1;` (sv-tests/tests/chapter-12/12.4.2--unique0_if.sv) |
| 50 | `tri ID ;` | 2 | `primitive udp_x (a_bad, b, c_bad);` (verilator/test_regress/t/t_udp_bad.v) |
| 51 | `{ ID ,` | 2 | `typedef enum [2:0] {` (verilator/test_regress/t/t_debug_emitv.v) |
| 52 | `!= NUM )` | 1 | `} != 1)` (verilator/test_regress/t/t_std_randomize_with.v) |
| 53 | `!= type (` | 1 | `if (type(value1) != type(value2)) $stop;` (verilator/test_regress/t/t_type_expression_compare.v) |
| 54 | `" " "` | 1 | `` (verilator/test_regress/t/t_parse_eof_qqq_bad.v) |
| 55 | `" ID "` | 1 | `$display("""First "quoted"\nsecond\` (verilator/test_regress/t/t_display_qqq.v) |
| 56 | `" ID -` | 1 | `import "DPI-BAD" task dpix_twice;` (verilator/test_regress/t/t_dpi_type_bad.v) |
| 57 | `" ID ID` | 1 | `string bad_literal = "Humpty Dumpty sat on a wall. \` (verible/verible/verilog/tools/lint/testdata/forbid_line_continuations.sv) |
| 58 | `" ID \` | 1 | `$display("broken \` (sv-tests/tests/chapter-5/5.9-string-broken-line.sv) |
| 59 | `# ( )` | 1 | `instance t.m32 use #();` (verilator/test_regress/t/t_config_rules.v) |
| 60 | `# ( .` | 1 | `instance t.u_1a use #(.P1(), .P2("override.u_a.p2"));` (verilator/test_regress/t/t_config_param.v) |
| 61 | `' ID ID` | 1 | `parameter int foo = 1 'b x;` (sv-tests/tests/generic/number/number_test_11.sv) |
| 62 | `( ) )` | 1 | `initial $display(nested_class0::handle1::handle2.nested_function());` (verible/verible/verilog/tools/kythe/testdata/nested_member_access.sv) |
| 63 | `( ) ;` | 1 | `->b.get_event();` (verilator/test_regress/t/t_event_control_pass.v) |
| 64 | `( ID )` | 1 | `extern function ext_f_imp(in);` (verilator/test_regress/t/t_class_extern.v) |
| 65 | `( NUM )` | 1 | `0 ? 1  :   1;` (verilator/test_regress/t/t_udp_bad_comb_trigger.v) |
| 66 | `( input logic` | 1 | `extern function myfunc (input logic val);` (verilator/test_regress/t/t_interface_modport_export.v) |
| 67 | `) ; integer` | 1 | `mreal #1.2 mr ();` (verilator/test_regress/t/t_param.v) |
| 68 | `) ID (` | 1 | `pullup (supply1) pu1 (a);` (verilator/test_regress/t/t_tri_pull_unsup.v) |
| 69 | `) ID ++` | 1 | `foreach (queue[i]) foreach (queue[i][j]) exp_count_que++;` (verilator/test_regress/t/t_foreach_array.v) |
| 70 | `) NUM :` | 1 | `case (type(real))` (verilator/test_regress/t/t_type_compare_bad.v) |
| 71 | `) this .` | 1 | `foreach(this.foo.x[i][j])` (verilator/test_regress/t/t_foreach_nested.v) |
| 72 | `) type (` | 1 | `case (type(T))` (sv-tests/tests/chapter-6/6.23--type_op_compare.sv) |
| 73 | `) || ID` | 1 | `if (!bit'(std::randomize(x) with {unique {x};}) || x[0] == x[1]) $stop;` (verilator/test_regress/t/t_std_randomize_unsup_unq_arr.v) |
| 74 | `* ; ID` | 1 | `1 1 ?  :  ?: *;` (verilator/test_regress/t/t_udp_bad_illegal_output.v) |
| 75 | `, highz1 )` | 1 | `wire (weak0, highz1) a = 1;` (verilator/test_regress/t/t_strength_highz.v) |
| 76 | `, parameter type` | 1 | `parameter type A = B,` (verilator/test_regress/t/t_recursive_typedef_bad.v) |
| 77 | `, strong1 )` | 1 | `assign (weak0, strong1) a = clk ? 'z : '0;` (verilator/test_regress/t/t_strength_2_uneq_assign.v) |
| 78 | `, supply1 )` | 1 | `assign (highz0, supply1) nt00 = a[0];` (verilator/test_regress/t/t_gate_strength.v) |
| 79 | `. * )` | 1 | `sub sub (foo, .*);` (verilator/test_regress/t/t_inst_2star_bad.v) |
| 80 | `. ID <=` | 1 | `].s_dyn <= '1;  // OK: direct nonblocking assignment to dynamically-sized array` (verilator/test_regress/t/t_assign_automatic_bad.v) |
| 81 | `. ID [` | 1 | `localparam type p0_t = if0.rq_t[1:0];` (verilator/test_regress/t/t_lparam_assign_iface_typedef4.v) |
| 82 | `: ID always` | 1 | `begin : gen_block1` (verible/verible/verilog/tools/lint/testdata/generate_begin_module.sv) |
| 83 | `: ID default` | 1 | `begin : g` (verilator/test_regress/t/t_default_disable_iff_gen_multi_bad.v) |
| 84 | `: ID end` | 1 | `begin : block` (verilator/test_regress/t/t_gen_duplicated_blocks_bad.v) |
| 85 | `: ID string` | 1 | `begin : DSM` (verilator/test_regress/t/t_dpi_string.v) |
| 86 | `: NUM :` | 1 | `$setuphold (posedge clk, negedge d, 0:0:0, 0:0:0);` (verilator/test_regress/t/t_setuphold.v) |
| 87 | `: extends int` | 1 | `function :extends int get_e; return 1; endfunction  // Bad` (verilator/test_regress/t/t_class_override_bad.v) |
| 88 | `: initial int` | 1 | `function :initial int get_i; return 1; endfunction` (verilator/test_regress/t/t_class_override.v) |
| 89 | `:: ID {` | 1 | `typedef enum pkg1::uint8_t {` (verilator/test_regress/t/t_typedef_package.v) |
| 90 | `; ID .` | 1 | `p.m_dyn_arr[0].rand_mode(0);` (verilator/test_regress/t/t_randomize_rand_mode_unsup.v) |
| 91 | `; ID :` | 1 | `input clk;` (verilator/test_regress/t/t_assert_imm_nz_bad.v) |
| 92 | `; ID ID` | 1 | `wreal vout[0:1]  /*verilator split_var*/;` (verilator/test_regress/t/t_split_var_3_wreal.v) |
| 93 | `; assign ID` | 1 | `wreal wr;` (verilator/test_regress/t/t_vams_basic.v) |
| 94 | `; end endgenerate` | 1 | `baz bam;` (verible/verible/verilog/tools/lint/testdata/generate_label_module.sv) |
| 95 | `; endclass ID` | 1 | `class bar extends pkg;` (verilator/test_regress/t/t_class_extends_pkg_bad.v) |
| 96 | `; endclass interface` | 1 | `class ClsBad1 implements NotIcls;` (verilator/test_regress/t/t_implements_noninterface_bad.v) |
| 97 | `; endclass module` | 1 | `class ClsI implements Inotfound;` (verilator/test_regress/t/t_implements_notfound_bad.v) |
| 98 | `; endmodule module` | 1 | `output bad_reout_port;` (verilator/test_regress/t/t_var_dup_bad.v) |
| 99 | `; endtask` | 1 | `@x[y];` (verible/verible/verilog/tools/kythe/testdata/event_control.sv) |
| 100 | `; endtask function` | 1 | `->write_events[index];` (verilator/test_regress/t/t_array_event.v) |
| 101 | `; if (` | 1 | `m_env.r_reg_model.p_s[p].n_doorbell.read(status, rdata, 0);` (verilator/test_regress/t/t_resize_lvalue.v) |
| 102 | `; initial ID` | 1 | `intf intf_i;` (verilator/test_regress/t/t_interface_paren_missing_bad.v) |
| 103 | `; initial bit` | 1 | `integer bit; initial bit = 1;` (verilator/test_regress/t/t_flag_language.v) |
| 104 | `; reg ID` | 1 | `input clk;` (verilator/test_regress/t/t_comb_do_not_convert_to.v) |
| 105 | `; supply1 ID` | 1 | `supply0       gnd;` (verilator/test_regress/t/t_gate_fdup.v) |
| 106 | `; } endgroup` | 1 | `bins auto[size_var];` (verilator/test_regress/t/t_covergroup_autobins_bad.v) |
| 107 | `<< << <<` | 1 | `module t;` (verilator/test_regress/t/t_lint_vcmarker_bad.v) |
| 108 | `<EOF>` | 1 | `(* attr` (verilator/test_regress/t/t_parse_eof_attr_bad.v) |
| 109 | `= ( ID` | 1 | `assign active_command3[1:0][2:0][3:0] = (use_AnB) ?  command_A3[1:0][2:0][3:0] : command_B3[1:0][1:0][3:0];` (verilator/test_regress/t/t_mem_slice_bad.v) |
| 110 | `= ID [` | 1 | `output [2:1] z = r[2 :+ 1];` (verilator/test_regress/t/t_lint_colonplus_bad.v) |
| 111 | `= NUM '` | 1 | `output [3:0] i65 = 65'd3 ** a; // IWI` (verilator/test_regress/t/t_math_pow6.v) |
| 112 | `= NUM ;` | 1 | `baz.get_bar().f.x = 4;` (verilator/test_regress/t/t_class_static_member_sel.v) |
| 113 | `== type (` | 1 | `if (type(real) == type(logic[12:0])) $stop;` (verilator/test_regress/t/t_type_compare.v) |
| 114 | `> ## NUM` | 1 | `@(posedge clk) (valid) |-> ##2 (out == in + 3);` (verilator/test_regress/t/t_assert_property_stop_bad.v) |
| 115 | `> ## [` | 1 | `a1: assert property (@(posedge clk) a |-> ##[1:cyc] b);` (verilator/test_regress/t/t_property_sexpr_range_delay_bad.v) |
| 116 | `> ID (` | 1 | `assert property (@(posedge clk) cyc % 2 == 0 |=> $past(val) == 0)` (verilator/test_regress/t/t_assert_past.v) |
| 117 | `? ; ID` | 1 | `?     :  ?;` (verilator/test_regress/t/t_timescale_udp.v) |
| 118 | `@ ( posedge` | 1 | `if ($past(d, 1, 1, @(posedge clk))) $stop;  // Unsup` (verilator/test_regress/t/t_past_unsup.v) |
| 119 | `ID ( int` | 1 | `covergroup cg_bad with function foo(int x);` (verilator/test_regress/t/t_covergroup_with_function_foo_bad.v) |
| 120 | `ID , import` | 1 | `modport Tx(output sop, data, import unpack);` (verilator/test_regress/t/t_varref_scope_in_interface.v) |
| 121 | `ID , input` | 1 | `modport initiator(output tdata, tvalid, input tready);` (verilator/test_regress/t/t_paramgraph_bits_iface_typedef.v) |
| 122 | `ID ; /` | 1 | `i badi;  // Bad` (verilator/test_regress/t/t_struct_type_bad.v) |
| 123 | `ID ; ID` | 1 | `typedef my_iface#(T) self_t;` (verilator/test_regress/t/t_iface_self_ref_typedef.v) |
| 124 | `ID ; begin` | 1 | `logic [D-1:0][W-1:0] s;` (verilator/test_regress/t/t_cover_fsm_sel_assign.v) |
| 125 | `ID ; endmodule` | 1 | `tri1 vectored [15:0] a;` (sv-tests/tests/chapter-6/6.9.2--vector_vectored.sv) |
| 126 | `ID ; initial` | 1 | `environment env;` (verible/verible/verilog/tools/kythe/testdata/program.sv) |
| 127 | `ID ; module` | 1 | `typedef some_other_type myalias;` (sv-tests/tests/generic/typedef/typedef_test_8__bad.sv) |
| 128 | `ID = ID` | 1 | `localparam something = fun();` (verilator/test_regress/t/t_simulate_array.v) |
| 129 | `ID = new` | 1 | `static class2 instance1 = new();` (verible/verible/verilog/tools/kythe/testdata/reversed_definition.sv) |
| 130 | `ID ? NUM` | 1 | `2'b ?1: out0 = 3'd0;` (verilator/test_regress/t/t_priority_case.v) |
| 131 | `ID ID ;` | 1 | `a = 'h x;   // yields xxx` (sv-tests/tests/chapter-5/5.7.1--integers-left-padding.sv) |
| 132 | `ID NUM )` | 1 | `if ((32'hffff0000 >> w32) != 32'h 000ffff0) $stop;` (verilator/test_regress/t/t_math_const.v) |
| 133 | `ID eventually [` | 1 | `assert property (counter == 1 implies eventually[1: 2] counter == 3);` (verilator/test_regress/t/t_property_unsup.v) |
| 134 | `NUM ) :` | 1 | `(01) : ? : 0;` (verilator/test_regress/t/t_udp_sequential_x.v) |
| 135 | `NUM ) NUM` | 1 | `(10) 1 : ? : 1 ;` (verilator/test_regress/t/t_udp_sequential.v) |
| 136 | `NUM ) begin` | 1 | `if (1) begin` (verilator/test_regress/t/t_interface_gen6.v) |
| 137 | `NUM ; /` | 1 | `localparam type bad2 = 2;  // Bad` (verilator/test_regress/t/t_param_type_bad.v) |
| 138 | `[ ID :` | 1 | `function f4(int [var1:var2] t1 [var3:var4]);` (verible/verible/verilog/tools/kythe/testdata/function_port_dimensions.sv) |
| 139 | `[ ID ::` | 1 | `int [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH] var5 [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH];` (verible/verible/verilog/tools/kythe/testdata/primitive_type_with_dimensions.sv) |
| 140 | `\ : NUM` | 1 | `0   0  x \` (verilator/test_regress/t/t_udp_nonsequential.v) |
| 141 | `] ; /` | 1 | `q = q[1:$]; // q.delete(0)` (sv-tests/tests/chapter-7/queues/delete_assign.sv) |
| 142 | `] ; ID` | 1 | `q = q[1:$];` (sv-tests/tests/chapter-7/queues/pop_front_assign.sv) |
| 143 | `] } ;` | 1 | `q = { q[0:1], 10, q[2:$] }; // q.insert(2, 10)` (sv-tests/tests/chapter-7/queues/insert_assign.sv) |
| 144 | `` ID )` | 1 | `t_tri0 tri0a (.line(`__LINE__), .expval(1'b0)); // Pin missing` (verilator/test_regress/t/t_tri_unconn.v) |
| 145 | `` ID ;` | 1 | `localparam string FILENAME = `__FILE__;` (verilator/test_regress/t/t_stop_winos_bad.v) |
| 146 | `` ID NUM` | 1 | `module directives();` (sv-tests/tests/chapter-5/5.6.4--compiler-directives-debug-line.sv) |
| 147 | `begin : ID` | 1 | `end` (verilator/test_regress/t/t_gen_if.v) |
| 148 | `default ) ;` | 1 | `function new(default);` (verilator/test_regress/t/t_class_new_default.v) |
| 149 | `default : ID` | 1 | `casex (1'bx) inside` (verilator/test_regress/t/t_case_inside_bad.v) |
| 150 | `do ) ;` | 1 | `module t(input do);` (verilator/test_regress/t/t_langext_order.v) |
| 151 | `endproperty endmodule` | 1 | `endcase` (verilator/test_regress/t/t_property_case_bad.v) |
| 152 | `enum ID ;` | 1 | `parameter type enum E_t;` (verilator/test_regress/t/t_param_type_fwd_bad.v) |
| 153 | `inside { ID` | 1 | `localparam HIT_INSIDE = HIT_LP inside {CHECKLIST_P};` (verilator/test_regress/t/t_inside_unpacked_param.v) |
| 154 | `module` | 1 | `endmodule` (sv-tests/tests/chapter-11/11.3.6--two_assign_in_expr.sv) |
| 155 | `packed { ID` | 1 | `typedef union soft packed {` (verilator/test_regress/t/t_export_packed_struct.v) |
| 156 | `packed { logic` | 1 | `typedef union soft packed {` (verilator/test_regress/t/t_export_packed_struct2.v) |
| 157 | `return ID ;` | 1 | `return success;` (sv-tests/tests/chapter-18/18.12.1--adding-constraints-to-scope-variables_0.sv) |
| 158 | `tri0 ID ;` | 1 | `wire rst;` (verilator/test_regress/t/t_tri_clocking.v) |
| 159 | `tri1 ID )` | 1 | `inout tri1 z` (verilator/test_regress/t/t_tri_gen.v) |
| 160 | `weak0 , weak1` | 1 | `wire (weak0, weak1) [W-1:0] b = '1;` (verilator/test_regress/t/t_strength_equal_strength.v) |
| 161 | `{ / /` | 1 | `enum logic [1:0][1:0] {  // BAD enum type` (verilator/test_regress/t/t_enum_base_bad.v) |
| 162 | `{ ID >` | 1 | `total = q.sum() with (item) { item > 0; };` (verilator/test_regress/t/t_randomize_with_idlist_method_bad.v) |
| 163 | `{ bit [` | 1 | `union soft {` (verilator/test_regress/t/t_union_soft.v) |
| 164 | `| -> (` | 1 | `) |-> (snap == cyc);` (verilator/test_regress/t/t_property_local_var_delay.v) |
| 165 | `| -> NUM` | 1 | `assert property (@(posedge clk) a [*n] |-> 1);` (verilator/test_regress/t/t_assert_consec_rep_bad.v) |
| 166 | `| ID ;` | 1 | `assign y[0] = !|v;` (verilator/test_regress/t/t_lint_notredop_bad.v) |
| 167 | `|| ID inside` | 1 | `constraint B { w == 5; x inside {1,2} || x inside {4,5}; }` (verilator/test_regress/t/t_randomize_method_constraints.v) |
| 168 | `} ) ;` | 1 | `task tsk(int q[] = {});` (verilator/test_regress/t/t_queue_empty_pin.v) |
| 169 | `} + NUM` | 1 | `i = {} + 1;` (verilator/test_regress/t/t_queue_empty_bad.v) |
| 170 | `} ID :` | 1 | `}` (verilator/test_regress/t/t_covergroup_cross_opt_unsup.v) |
| 171 | `~ ID ==` | 1 | `constraint unary { !(-~c == 'h22); }` (verilator/test_regress/t/t_constraint_operators.v) |

