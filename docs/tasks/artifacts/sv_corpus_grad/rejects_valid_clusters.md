# `systemverilog` stuck-point clusters (SV-CORPUS-GRAD.3.0 engine; family lane CORPUS-GRAD-ALL.2.1)

284 rows probed from the adjudication manifest; 177 distinct 3-token stuck signatures. Keyed on `furthest_position` (the DEEP locus), family profile `systemverilog`.

| # | signature | rows | example (stuck line) |
|---|---|---|---|
| 1 | `; / /` | 8 | `reg logic; // OK: "logic" is not a keyword in 1364-2001` (sv-tests/tests/chapter-5/5.6.4--compiler-directives-begin-keywords.sv) |
| 2 | `ID ) ;` | 8 | `virtual function void do_copy (uvm_object rhs);` (Surelog/tests/NamedEventHierPath/dut.sv) |
| 3 | `; endmodule` | 7 | `Interface i;` (sv2v/test/error/interface_modport_unlisted.sv) |
| 4 | `with { ID` | 7 | `if (std::randomize(example.member[1]) with {example.member[1] inside {[0 : 3]};} == 0) $stop;` (verilator/test_regress/t/t_randomize_arraysel_membersel.v) |
| 5 | `@ ( posedge` | 6 | `let p2(x) = $past(x,,,@(posedge clock));` (Surelog/tests/Assertions/dut.sv) |
| 6 | `ID = ID` | 6 | `coverage_level clevel = cover_all // This argument should be bound to an` (Surelog/tests/Checker/dut.sv) |
| 7 | `. ID )` | 5 | `tagged Jmp (tagged JmpU .a) : pc = pc + a;` (ispras-sv-tests/ieee-1800-2012/12/12.06.01_05.sv) |
| 8 | `ID = NUM` | 5 | `module top #(int N = 5, M = N * 16, type T = int, T x = 0) ();` (ispras-sv-tests/ieee-1800-2012/06/06.20.01_02.sv) |
| 9 | `] ; /` | 5 | `channel = channel[size:$]; // update the stream so it now lacks that packet` (ispras-sv-tests/ieee-1800-2012/06/06.24.03_03.sv) |
| 10 | `. ID ;` | 4 | `localparam type p0_t = if0.rq_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef2.v) |
| 11 | `NUM ) NUM` | 4 | `(01)         0   :   ?   :   0   ;` (Surelog/tests/Udp/dut.sv) |
| 12 | `[ NUM :` | 4 | `bins c1 = ! binsof(a) intersect {[100:200]};// 4 cross products` (Surelog/tests/CrossItem/dut.sv) |
| 13 | `" ID ID` | 3 | `$fatal(1, "slv_aw_select_i is %d: AW has selected a slave that is not defined.\` (Surelog/tests/AssumeProp/dut.sv) |
| 14 | `' ( ID` | 3 | `if ($clog2(RomSize)'(RomSize) == 5'd19) begin : gen_cast` (Surelog/tests/ClogCast/dut.sv) |
| 15 | `( ) ;` | 3 | `function void uvm_sequencer_base::start_phase_sequence();` (Surelog/tests/ClassMemberFunc/dut.sv) |
| 16 | `- NUM ]` | 3 | `enum {ndef[-1]} ndef1;` (iverilog/ivtest/ivltests/pr3366217d.v) |
| 17 | `. ID (` | 3 | `$fatal(.x("x"));` (sv2v/test/error/severity_task_arg.sv) |
| 18 | `ID ( "` | 3 | `$display("test");` (Surelog/tests/IndexedSelect/dut.sv) |
| 19 | `ID ( ID` | 3 | `extern function uvm_mem_mam_cfg reconfigure(uvm_mem_mam_cfg cfg = null);` (Surelog/tests/BitSelectHier/dut.sv) |
| 20 | `ID ; /` | 3 | `scalar_t pc;` (Surelog/tests/HierBitSlice/dut.sv) |
| 21 | `} endgroup endmodule` | 3 | `}` (ispras-sv-tests/ieee-1800-2012/19/19.06.01.04_01.sv) |
| 22 | `## NUM ID` | 2 | `trans ##1 start_trans ##1 s ##1 end_trans;` (ispras-sv-tests/ieee-1800-2012/16/16.08_04.sv) |
| 23 | `( ID )` | 2 | `(states_t) state1;` (Surelog/tests/EnumParenType/dut.sv) |
| 24 | `) ; /` | 2 | `class Cls1 extends Base1(default);` (verilator/test_regress/t/t_class_extends_default.v) |
| 25 | `) ; ID` | 2 | `m();` (verilator/test_regress/t/t_inst_noname_bad.v) |
| 26 | `) ; endclass` | 2 | `);` (verilator/test_regress/t/t_param_type_id_bad.v) |
| 27 | `) ID [` | 2 | `foreach(array[i][j])` (Surelog/tests/ForeachSquare/dut.sv) |
| 28 | `) begin /` | 2 | `foreach (y) begin` (verible/verible/verilog/tools/kythe/testdata/foreach.sv) |
| 29 | `, ID ;` | 2 | `class Cls extends Base1, Base2;` (verilator/test_regress/t/t_class_extends_bad.v) |
| 30 | `. ID ,` | 2 | `if (e matches (tagged Jmp (tagged JmpC '{cc:.c, addr:.a}))) begin` (ispras-sv-tests/ieee-1800-2012/12/12.06.02_01.sv) |
| 31 | `. ID .` | 2 | `localparam type p0_rq2_t = if0.y_if0.rq2_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef_nested.v) |
| 32 | `. ID ID` | 2 | `assign #sub.delay dly4 = dly1 + 1;` (verilator/test_regress/t/t_delay.v) |
| 33 | `:: ID ;` | 2 | `#1 ->my_package::e;` (iverilog/ivtest/ivltests/sv_wildcard_import2.v) |
| 34 | `; ID .` | 2 | `sim.queues[Active].push_back('{is_update: 1});` (ispras-sv-tests/ieee-1800-2012/04/04.05_01.sv) |
| 35 | `; ID ID` | 2 | `tnoc_types#(2) i_types;` (Surelog/tests/InterfTypeBad/dut.sv) |
| 36 | `; logic [` | 2 | `input clk;` (verilator/test_regress/t/t_clk_concat5.v) |
| 37 | `= ID )` | 2 | `rst = $inferred_disable);` (Surelog/tests/SequenceInst/dut.sv) |
| 38 | `= NUM ;` | 2 | `output a = 1;` (sv2v/test/error/port_init_early.sv) |
| 39 | `== NUM )` | 2 | `assert((std::randomize(x) with {x inside {10, 20}; }) == 1)` (verilator/test_regress/t/t_randomize_inside_cond.v) |
| 40 | `@ ( ID` | 2 | `@($global_clock) a[*1:$] ##1 b);` (Surelog/tests/CheckerInst/dut.sv) |
| 41 | `ID ( .` | 2 | `t1 t (.mod_intf(mod_intfs[0]));` (verilator/test_regress/t/t_interface_gen11.v) |
| 42 | `ID ; ID` | 2 | `bp_bedrock_cmd_type_e         lce_cmd;` (Surelog/tests/BlackParrotConf/dut.sv) |
| 43 | `ID ; endmodule` | 2 | `tri1 vectored [15:0] a;` (sv-tests/tests/chapter-6/6.9.2--vector_vectored.sv) |
| 44 | `ID ; typedef` | 2 | `typedef some_other_type myalias;` (Surelog/tests/NetType/dut.sv) |
| 45 | `ID = '{` | 2 | `localparam bp_proc_param_s bp_default_cfg_p =` (Surelog/tests/NegParam/dut.sv) |
| 46 | `ID [ ID` | 2 | `protected uvm_phase m_type_overrides[$];` (Surelog/tests/ForeachClassParent/dut.sv) |
| 47 | `NUM : ?` | 2 | `? ?? 01 : ? : 1 ; // preset logic` (ispras-sv-tests/ieee-1800-2012/29/29.09_01.sv) |
| 48 | `[ NUM ]` | 2 | `dim2[0+:1][1] = 0;  // Bad: Range on non-bits` (verilator/test_regress/t/t_mem_multi_ref_bad.v) |
| 49 | `] ID ;` | 2 | `word [2] array;      // word[0:1] exposes the bug as well` (iverilog/ivtest/ivltests/display_bug.v) |
| 50 | `] [ NUM` | 2 | `logic [2][3] x;` (sv2v/test/core/multipack_struct_cast.sv) |
| 51 | `enum ID ;` | 2 | `parameter type enum E_t;` (verilator/test_regress/t/t_param_type_fwd.v) |
| 52 | `function ID ID` | 2 | `function CrossQueueType myFunc1(int f_lim);` (Surelog/tests/CrossFunc/dut.sv) |
| 53 | `tri ID ;` | 2 | `tri  a_bad;` (verilator/test_regress/t/t_udp_bad.v) |
| 54 | `!= NUM )` | 1 | `} != 1)` (verilator/test_regress/t/t_std_randomize_with.v) |
| 55 | `!= type (` | 1 | `if (type(value1) != type(value2)) $stop;` (verilator/test_regress/t/t_type_expression_compare.v) |
| 56 | `" ID "` | 1 | `$display("""First "quoted"\nsecond\` (verilator/test_regress/t/t_display_qqq.v) |
| 57 | `" ID -` | 1 | `import "DPI-BAD" task dpix_twice;` (verilator/test_regress/t/t_dpi_type_bad.v) |
| 58 | `" ID \` | 1 | `$display("broken \` (sv-tests/tests/chapter-5/5.9-string-broken-line.sv) |
| 59 | `' ( '` | 1 | `localparam host_base_addr_gp         = (dev_id_width_gp+dev_addr_width_gp)'('h0010_0000);` (Surelog/tests/BlackBePipeInt/dut.sv) |
| 60 | `' ( {` | 1 | `parameter logic [2:0] PartInvDefault = 3'({` (Surelog/tests/ParamTypespec/dut.sv) |
| 61 | `' ID ]` | 1 | `A['x]` (sv2v/test/error/enum_range_x.sv) |
| 62 | `( ) )` | 1 | `initial $display(nested_class0::handle1::handle2.nested_function());` (verible/verible/verilog/tools/kythe/testdata/nested_member_access.sv) |
| 63 | `( ID [` | 1 | `modport client_mp (output .client_req(req[i]));` (sv2v/test/core/interface_generate.sv) |
| 64 | `( NUM )` | 1 | `(01) 1 0  :   0;` (verilator/test_regress/t/t_udp_bad_comb_trigger.v) |
| 65 | `( input logic` | 1 | `extern function myfunc (input logic val);` (verilator/test_regress/t/t_interface_modport_export.v) |
| 66 | `( ref bit` | 1 | `function void uvm_packer::get_packed_bits(ref bit unsigned stream[]);` (Surelog/tests/Assignments/dut.sv) |
| 67 | `) ( ID` | 1 | `module ModuleA #(P=1,) (inp,);` (sv2v/test/core/trailing_commas.sv) |
| 68 | `) ; endmodule` | 1 | `initial $display(C#()::X);` (sv2v/test/error/missing_class.sv) |
| 69 | `) ; integer` | 1 | `mreal #1.2 mr ();` (verilator/test_regress/t/t_param.v) |
| 70 | `) ID ++` | 1 | `foreach (queue[i]) foreach (queue[i][j]) exp_count_que++;` (verilator/test_regress/t/t_foreach_array.v) |
| 71 | `) NUM :` | 1 | `case (type(real))` (verilator/test_regress/t/t_type_compare_bad.v) |
| 72 | `) extends ID` | 1 | `class uvm_sequencer_analysis_fifo #(type RSP = uvm_sequence_item) extends uvm_tlm_fifo #(RSP);` (Surelog/tests/ExtendClassMember/dut.sv) |
| 73 | `) this .` | 1 | `foreach(this.foo.x[i][j])` (verilator/test_regress/t/t_foreach_nested.v) |
| 74 | `) type (` | 1 | `case (type(T))` (sv-tests/tests/chapter-6/6.23--type_op_compare.sv) |
| 75 | `) || ID` | 1 | `if (!bit'(std::randomize(x) with {unique {x};}) || x[0] == x[1]) $stop;` (verilator/test_regress/t/t_std_randomize_unsup_unq_arr.v) |
| 76 | `* ; ID` | 1 | `1 1 ?  :  ?: *;` (verilator/test_regress/t/t_udp_bad_illegal_output.v) |
| 77 | `+ = NUM` | 1 | `for (x += 1; x < y; x++)` (sv2v/test/core/for_loop_inits.sv) |
| 78 | `+ NUM )` | 1 | `void'(1+2);` (iverilog/ivtest/ivltests/sv_void_cast_fail3.v) |
| 79 | `, ID ,` | 1 | `reg	     bit, rst, clk;` (Surelog/tests/1364_2005/dut.v) |
| 80 | `, ID =` | 1 | `parameter type CT1 = Ctype1, CT2 = Ctype2;` (Surelog/tests/ClassTypeParamAlias/dut.sv) |
| 81 | `, NUM )` | 1 | `specparam PATHPULSE$clk$q = (2, 9), PATHPULSE$clr$q = (0, 4),` (ispras-sv-tests/ieee-1800-2012/30/30.07.01_01.sv) |
| 82 | `, parameter type` | 1 | `parameter type A = B,` (verilator/test_regress/t/t_recursive_typedef_bad.v) |
| 83 | `, type ID` | 1 | `virtual class uvm_sequence #(type REQ = uvm_sequence_item,` (Surelog/tests/ClassExtendParam/dut.sv) |
| 84 | `. * )` | 1 | `sub sub (foo, .*);` (verilator/test_regress/t/t_inst_2star_bad.v) |
| 85 | `. ID <=` | 1 | `].s_dyn <= '1;  // OK: direct nonblocking assignment to dynamically-sized array` (verilator/test_regress/t/t_assign_automatic_bad.v) |
| 86 | `. ID [` | 1 | `localparam type p0_t = if0.rq_t[1:0];` (verilator/test_regress/t/t_lparam_assign_iface_typedef4.v) |
| 87 | `: ID /` | 1 | `endclass : ClassB` (ispras-sv-tests/ieee-1800-2012/08/08.26.04_01.sv) |
| 88 | `: NUM :` | 1 | `$setuphold (posedge clk, negedge d, 0:0:0, 0:0:0);` (verilator/test_regress/t/t_setuphold.v) |
| 89 | `: extends int` | 1 | `function :extends int get_e; return 1; endfunction  // Bad` (verilator/test_regress/t/t_class_override_bad.v) |
| 90 | `: initial int` | 1 | `function :initial int get_i; return 1; endfunction` (verilator/test_regress/t/t_class_override.v) |
| 91 | `:: ID {` | 1 | `typedef enum pkg1::uint8_t {` (verilator/test_regress/t/t_typedef_package.v) |
| 92 | `; ID (` | 1 | `p.m_pack[0].rand_mode(0);` (verilator/test_regress/t/t_randomize_rand_mode_bad.v) |
| 93 | `; ID :` | 1 | `input clk;` (verilator/test_regress/t/t_assert_imm_nz_bad.v) |
| 94 | `; end endgenerate` | 1 | `baz bam;` (verible/verible/verilog/tools/lint/testdata/generate_label_module.sv) |
| 95 | `; end endmodule` | 1 | `foreach (a[i]) ;` (iverilog/ivtest/ivltests/sf1289.v) |
| 96 | `; endclass ID` | 1 | `class bar extends pkg;` (verilator/test_regress/t/t_class_extends_pkg_bad.v) |
| 97 | `; endclass interface` | 1 | `class ClsBad1 implements NotIcls;` (verilator/test_regress/t/t_implements_noninterface_bad.v) |
| 98 | `; endclass module` | 1 | `class ClsI implements Inotfound;` (verilator/test_regress/t/t_implements_notfound_bad.v) |
| 99 | `; endmodule module` | 1 | `output bad_reout_port;` (verilator/test_regress/t/t_var_dup_bad.v) |
| 100 | `; endtask` | 1 | `@x[y];` (verible/verible/verilog/tools/kythe/testdata/event_control.sv) |
| 101 | `; endtask function` | 1 | `->write_events[index];` (verilator/test_regress/t/t_array_event.v) |
| 102 | `; if (` | 1 | `m_env.r_reg_model.p_s[p].n_doorbell.read(status, rdata, 0);` (verilator/test_regress/t/t_resize_lvalue.v) |
| 103 | `; initial ID` | 1 | `intf intf_i;` (verilator/test_regress/t/t_interface_paren_missing_bad.v) |
| 104 | `; initial bit` | 1 | `integer bit; initial bit = 1;` (verilator/test_regress/t/t_flag_language.v) |
| 105 | `; integer ID` | 1 | `dut inst[4];` (iverilog/ivtest/ivltests/br_gh553.v) |
| 106 | `; reg ID` | 1 | `input clk;` (verilator/test_regress/t/t_comb_do_not_convert_to.v) |
| 107 | `; typedef ID` | 1 | `tnoc_types  types;` (Surelog/tests/TNocBadType/dut.sv) |
| 108 | `; virtual function` | 1 | `class Hello #(type T = ihello) implements T;` (sv-tests/tests/chapter-8/8.26.4--illegal_implements_parameter.sv) |
| 109 | `; } endgroup` | 1 | `bins auto[size_var];` (verilator/test_regress/t/t_covergroup_autobins_bad.v) |
| 110 | `<NO-POSITION>` | 1 | `Error: failed to read input file 'stimuli/sv/subs/sv2v/test/lex/latin1.sv'

Caused by:
    stream did not contain valid ` (sv2v/test/lex/latin1.sv) |
| 111 | `= ( ID` | 1 | `assign active_command3[1:0][2:0][3:0] = (use_AnB) ?  command_A3[1:0][2:0][3:0] : command_B3[1:0][1:0][3:0];` (verilator/test_regress/t/t_mem_slice_bad.v) |
| 112 | `= ID (` | 1 | `input wire integer x = F(P); // defining a default here is non-standard` (sv2v/test/core/port_default.sv) |
| 113 | `= ID ,` | 1 | `d = d_d, // local input logic inferred, default actual` (ispras-sv-tests/ieee-1800-2012/16/16.08.02_01.sv) |
| 114 | `= ID -` | 1 | `j = i++ + (i = i - 1);` (ispras-sv-tests/ieee-1800-2012/11/11.04.02_01.sv) |
| 115 | `= ID [` | 1 | `output [2:1] z = r[2 :+ 1];` (verilator/test_regress/t/t_lint_colonplus_bad.v) |
| 116 | `= NUM '` | 1 | `output [3:0] i65 = 65'd3 ** a; // IWI` (verilator/test_regress/t/t_math_pow6.v) |
| 117 | `== type (` | 1 | `if (type(real) == type(logic[12:0])) $stop;` (verilator/test_regress/t/t_type_compare.v) |
| 118 | `? ; ID` | 1 | `?     :  ?;` (verilator/test_regress/t/t_timescale_udp.v) |
| 119 | `ID ( int` | 1 | `covergroup cg_bad with function foo(int x);` (verilator/test_regress/t/t_covergroup_with_function_foo_bad.v) |
| 120 | `ID , ID` | 1 | `task t(I dst, I src);` (sv2v/test/error/interface_name_func.sv) |
| 121 | `ID , input` | 1 | `input intf i,` (sv2v/test/core/interface_input_type.sv) |
| 122 | `ID , output` | 1 | `l0_if#(W) l0,` (verilator/test_regress/t/t_interface_nested_port_array.v) |
| 123 | `ID / *` | 1 | `'b` (sv2v/test/lex/number_literal_whitespace.sv) |
| 124 | `ID ; begin` | 1 | `logic [D-1:0][W-1:0] s;` (verilator/test_regress/t/t_cover_fsm_sel_assign.v) |
| 125 | `ID ; initial` | 1 | `environment env;` (verible/verible/verilog/tools/kythe/testdata/program.sv) |
| 126 | `ID ; logic` | 1 | `priv_lvl_e mpp;` (Surelog/tests/BadScope/dut.sv) |
| 127 | `ID ; module` | 1 | `typedef some_other_type myalias;` (sv-tests/tests/generic/typedef/typedef_test_8__bad.sv) |
| 128 | `ID ; output` | 1 | `output reg a;` (iverilog/ivtest/ivltests/module_output_port_sv_var2.v) |
| 129 | `ID ; }` | 1 | `uvm_reg_data_t data;` (Surelog/tests/IndexPartSelectBind/dut.sv) |
| 130 | `ID = new` | 1 | `static class2 instance1 = new();` (verible/verible/verilog/tools/kythe/testdata/reversed_definition.sv) |
| 131 | `ID = std` | 1 | `success = std::randomize(a, b) with {b - a > length;};` (ispras-sv-tests/ieee-1800-2012/18/18.12.01_01.sv) |
| 132 | `ID [ NUM` | 1 | `wire intf intf2  [2];` (Surelog/tests/ContAssign/dut.sv) |
| 133 | `ID eventually [` | 1 | `assert property (counter == 1 implies eventually[1: 2] counter == 3);` (verilator/test_regress/t/t_property_unsup.v) |
| 134 | `NUM ( NUM` | 1 | `assert #0_0 (1);` (sv2v/test/nosim/assert.sv) |
| 135 | `NUM ) :` | 1 | `(01) : ? : 0;` (verilator/test_regress/t/t_udp_sequential_x.v) |
| 136 | `NUM ) begin` | 1 | `if (1) begin` (verilator/test_regress/t/t_interface_gen6.v) |
| 137 | `NUM : NUM` | 1 | `00 : 0;` (iverilog/ivtest/ivltests/sv_type_identifier_udp_ansi_name.v) |
| 138 | `NUM ; /` | 1 | `localparam type bad2 = 2;  // Bad` (verilator/test_regress/t/t_param_type_bad.v) |
| 139 | `NUM ; ID` | 1 | `a = 15'1;` (iverilog/ivtest/ivltests/pr3515542.v) |
| 140 | `[ ID :` | 1 | `function f4(int [var1:var2] t1 [var3:var4]);` (verible/verible/verilog/tools/kythe/testdata/function_port_dimensions.sv) |
| 141 | `[ ID ::` | 1 | `int [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH] var5 [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH];` (verible/verible/verilog/tools/kythe/testdata/primitive_type_with_dimensions.sv) |
| 142 | `\ : NUM` | 1 | `0   0  x \` (verilator/test_regress/t/t_udp_nonsequential.v) |
| 143 | `] ; ID` | 1 | `q = q[1:$];` (sv-tests/tests/chapter-7/queues/pop_front_assign.sv) |
| 144 | `] ID ,` | 1 | `logic [4] a, b;` (sv2v/test/core/delay.sv) |
| 145 | `] ID =` | 1 | `S [WIDTH] s = '{` (sv2v/test/core/struct_unit_array.sv) |
| 146 | `] } ;` | 1 | `q = { q[0:1], 10, q[2:$] }; // q.insert(2, 10)` (sv-tests/tests/chapter-7/queues/insert_assign.sv) |
| 147 | `automatic = NUM` | 1 | `integer automatic = 2;` (sv2v/test/lex/begin_keywords.sv) |
| 148 | `default ) ;` | 1 | `function new(default);` (verilator/test_regress/t/t_class_new_default.v) |
| 149 | `default : ID` | 1 | `default: $stop;` (verilator/test_regress/t/t_case_inside_bad.v) |
| 150 | `do ) ;` | 1 | `module t(input do);` (verilator/test_regress/t/t_langext_order.v) |
| 151 | `endproperty endmodule` | 1 | `endproperty` (verilator/test_regress/t/t_property_case_bad.v) |
| 152 | `int ID )` | 1 | `property check_if_1(var int cyc_mod_2);` (verilator/test_regress/t/t_property_named.v) |
| 153 | `int ID ;` | 1 | `int badi;` (verilator/test_regress/t/t_implements_contents_bad.v) |
| 154 | `interface class ID` | 1 | `interface class bad_cannot_nest;` (verilator/test_regress/t/t_implements_nested_bad.v) |
| 155 | `logic ; /` | 1 | `reg [63:0] logic; // OK: 'logic' is not a keyword in 1364-2001` (ispras-sv-tests/ieee-1800-2012/22/22.14.01_01.sv) |
| 156 | `medium ( )` | 1 | `module medium();` (Surelog/tests/HierPathModule/dut.sv) |
| 157 | `module` | 1 | `endmodule` (sv-tests/tests/chapter-11/11.3.6--two_assign_in_expr.sv) |
| 158 | `new ; /` | 1 | `c = T::new; // This should fail, T is not a class` (iverilog/ivtest/ivltests/sv_class_new_typed_fail4.v) |
| 159 | `packed { ID` | 1 | `typedef union soft packed {` (verilator/test_regress/t/t_export_packed_struct.v) |
| 160 | `packed { logic` | 1 | `typedef union soft packed {` (verilator/test_regress/t/t_export_packed_struct2.v) |
| 161 | `reg ID ;` | 1 | `inout wire reg p;` (ispras-sv-tests/ieee-1800-2012/06/06.07.01_02.sv) |
| 162 | `return ID ;` | 1 | `return success;` (sv-tests/tests/chapter-18/18.12.1--adding-constraints-to-scope-variables_0.sv) |
| 163 | `tri0 ID ;` | 1 | `tri0 cyc;` (verilator/test_regress/t/t_tri_clocking.v) |
| 164 | `tri1 ID )` | 1 | `inout tri1 z` (verilator/test_regress/t/t_tri_gen.v) |
| 165 | `with ( ID` | 1 | `bins apple = X with (a + b < 257) matches 127;` (ispras-sv-tests/ieee-1800-2012/19/19.06.01.02_01.sv) |
| 166 | `{ / /` | 1 | `enum logic [1:0][1:0] {  // BAD enum type` (verilator/test_regress/t/t_enum_base_bad.v) |
| 167 | `{ ID >` | 1 | `total = q.sum() with (item) { item > 0; };` (verilator/test_regress/t/t_randomize_with_idlist_method_bad.v) |
| 168 | `{ bit [` | 1 | `union soft {` (verilator/test_regress/t/t_union_soft.v) |
| 169 | `| ( NUM` | 1 | `| (0                 <<  5)` (Surelog/tests/CastShift/dut.sv) |
| 170 | `| ID ;` | 1 | `assign y[0] = !|v;` (verilator/test_regress/t/t_lint_notredop_bad.v) |
| 171 | `|| ID inside` | 1 | `constraint B { w == 5; x inside {1,2} || x inside {4,5}; }` (verilator/test_regress/t/t_randomize_method_constraints.v) |
| 172 | `} ) ;` | 1 | `task tsk(int q[] = {});` (verilator/test_regress/t/t_queue_empty_pin.v) |
| 173 | `} + NUM` | 1 | `i = {} + 1;` (verilator/test_regress/t/t_queue_empty_bad.v) |
| 174 | `} ; if` | 1 | `words = {};` (iverilog/ivtest/ivltests/sv_queue3.v) |
| 175 | `} ID :` | 1 | `}` (verilator/test_regress/t/t_covergroup_cross_opt_unsup.v) |
| 176 | `} endgroup covergroup` | 1 | `}` (Surelog/tests/Covergroup/dut.sv) |
| 177 | `~ ID ==` | 1 | `constraint unary { !(-~c == 'h22); }` (verilator/test_regress/t/t_constraint_operators.v) |

