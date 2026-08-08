# `systemverilog` stuck-point clusters (SV-CORPUS-GRAD.3.0 engine; family lane CORPUS-GRAD-ALL.2.1)

339 rows probed from the adjudication manifest; 200 distinct 3-token stuck signatures. Keyed on `furthest_position` (the DEEP locus), family profile `systemverilog`.

| # | signature | rows | example (stuck line) |
|---|---|---|---|
| 1 | `; / /` | 8 | `reg logic; // OK: "logic" is not a keyword in 1364-2001` (sv-tests/tests/chapter-5/5.6.4--compiler-directives-begin-keywords.sv) |
| 2 | `ID ) ;` | 8 | `virtual function void do_copy (uvm_object rhs);` (Surelog/tests/NamedEventHierPath/dut.sv) |
| 3 | `; endmodule` | 7 | `Interface i;` (sv2v/test/error/interface_modport_unlisted.sv) |
| 4 | `with { ID` | 7 | `if (std::randomize(example.member[1]) with {example.member[1] inside {[0 : 3]};} == 0) $stop;` (verilator/test_regress/t/t_randomize_arraysel_membersel.v) |
| 5 | `{ ID =` | 7 | `typedef enum [5:0] {` (sv2v/test/core/enum_scope.sv) |
| 6 | `# ( .` | 6 | `instance top use #(.WIDTH(32));` (ispras-sv-tests/ieee-1800-2012/33/33.04.03_02.sv) |
| 7 | `@ ( posedge` | 6 | `let p2(x) = $past(x,,,@(posedge clock));` (Surelog/tests/Assertions/dut.sv) |
| 8 | `ID = ID` | 6 | `coverage_level clevel = cover_all // This argument should be bound to an` (Surelog/tests/Checker/dut.sv) |
| 9 | `[ NUM :` | 6 | `bins c1 = ! binsof(a) intersect {[100:200]};// 4 cross products` (Surelog/tests/CrossItem/dut.sv) |
| 10 | `. ID )` | 5 | `tagged Jmp (tagged JmpU .a) : pc = pc + a;` (ispras-sv-tests/ieee-1800-2012/12/12.06.01_05.sv) |
| 11 | `ID = NUM` | 5 | `module top #(int N = 5, M = N * 16, type T = int, T x = 0) ();` (ispras-sv-tests/ieee-1800-2012/06/06.20.01_02.sv) |
| 12 | `] ; /` | 5 | `channel = channel[size:$]; // update the stream so it now lacks that packet` (ispras-sv-tests/ieee-1800-2012/06/06.24.03_03.sv) |
| 13 | `. ID ;` | 4 | `localparam type p0_t = if0.rq_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef2.v) |
| 14 | `: = NUM` | 4 | `x dist {100 := 1, 200 := 2, 300 := 5};` (ispras-sv-tests/ieee-1800-2012/18/18.05.04_01.sv) |
| 15 | `NUM ) NUM` | 4 | `(01)         0   :   ?   :   0   ;` (Surelog/tests/Udp/dut.sv) |
| 16 | `` ID ID` | 4 | ``pragma protect encoding=(enctype="raw")` (ispras-sv-tests/ieee-1800-2012/34/34.03.01_01.sv) |
| 17 | `" ID ID` | 3 | `$fatal(1, "slv_aw_select_i is %d: AW has selected a slave that is not defined.\` (Surelog/tests/AssumeProp/dut.sv) |
| 18 | `' ( ID` | 3 | `if ($clog2(RomSize)'(RomSize) == 5'd19) begin : gen_cast` (Surelog/tests/ClogCast/dut.sv) |
| 19 | `( ) ;` | 3 | `function void uvm_sequencer_base::start_phase_sequence();` (Surelog/tests/ClassMemberFunc/dut.sv) |
| 20 | `- NUM ]` | 3 | `enum {ndef[-1]} ndef1;` (iverilog/ivtest/ivltests/pr3366217d.v) |
| 21 | `. ID (` | 3 | `$fatal(.x("x"));` (sv2v/test/error/severity_task_arg.sv) |
| 22 | `: ID ID` | 3 | `generate begin:foo` (Surelog/tests/DefParamIndex/dut.sv) |
| 23 | `: ID for` | 3 | `begin : topgen` (verilator/test_regress/t/t_gen_local.v) |
| 24 | `: ID logic` | 3 | `begin : A` (sv2v/test/core/array.sv) |
| 25 | `: ID wire` | 3 | `begin : A` (Surelog/tests/GenerateBlock/dut.sv) |
| 26 | `ID ( "` | 3 | `$display("test");` (Surelog/tests/IndexedSelect/dut.sv) |
| 27 | `ID ( ID` | 3 | `extern function uvm_mem_mam_cfg reconfigure(uvm_mem_mam_cfg cfg = null);` (Surelog/tests/BitSelectHier/dut.sv) |
| 28 | `ID ; /` | 3 | `scalar_t pc;` (Surelog/tests/HierBitSlice/dut.sv) |
| 29 | `} endgroup endmodule` | 3 | `}` (ispras-sv-tests/ieee-1800-2012/19/19.06.01.04_01.sv) |
| 30 | `# ( )` | 2 | `instance top.a1 use #(); // set all parameters in instance 'a1'` (ispras-sv-tests/ieee-1800-2012/33/33.04.03_06.sv) |
| 31 | `## NUM ID` | 2 | `trans ##1 start_trans ##1 s ##1 end_trans;` (ispras-sv-tests/ieee-1800-2012/16/16.08_04.sv) |
| 32 | `( ID )` | 2 | `(states_t) state1;` (Surelog/tests/EnumParenType/dut.sv) |
| 33 | `) ; /` | 2 | `class Cls1 extends Base1(default);` (verilator/test_regress/t/t_class_extends_default.v) |
| 34 | `) ; ID` | 2 | `m();` (verilator/test_regress/t/t_inst_noname_bad.v) |
| 35 | `) ; endclass` | 2 | `);` (verilator/test_regress/t/t_param_type_id_bad.v) |
| 36 | `) ID [` | 2 | `foreach(array[i][j])` (Surelog/tests/ForeachSquare/dut.sv) |
| 37 | `) begin /` | 2 | `foreach (y) begin` (verible/verible/verilog/tools/kythe/testdata/foreach.sv) |
| 38 | `, ID ;` | 2 | `class Cls extends Base1, Base2;` (verilator/test_regress/t/t_class_extends_bad.v) |
| 39 | `. ID ,` | 2 | `if (e matches (tagged Jmp (tagged JmpC '{cc:.c, addr:.a}))) begin` (ispras-sv-tests/ieee-1800-2012/12/12.06.02_01.sv) |
| 40 | `. ID .` | 2 | `localparam type p0_rq2_t = if0.y_if0.rq2_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef_nested.v) |
| 41 | `. ID ID` | 2 | `assign #sub.delay dly4 = dly1 + 1;` (verilator/test_regress/t/t_delay.v) |
| 42 | `:: ID ;` | 2 | `#1 ->my_package::e;` (iverilog/ivtest/ivltests/sv_wildcard_import2.v) |
| 43 | `; ID .` | 2 | `sim.queues[Active].push_back('{is_update: 1});` (ispras-sv-tests/ieee-1800-2012/04/04.05_01.sv) |
| 44 | `; ID ID` | 2 | `tnoc_types#(2) i_types;` (Surelog/tests/InterfTypeBad/dut.sv) |
| 45 | `; logic [` | 2 | `input clk;` (verilator/test_regress/t/t_clk_concat5.v) |
| 46 | `= ID )` | 2 | `rst = $inferred_disable);` (Surelog/tests/SequenceInst/dut.sv) |
| 47 | `= NUM ;` | 2 | `output a = 1;` (sv2v/test/error/port_init_early.sv) |
| 48 | `== NUM )` | 2 | `assert((std::randomize(x) with {x inside {10, 20}; }) == 1)` (verilator/test_regress/t/t_randomize_inside_cond.v) |
| 49 | `@ ( ID` | 2 | `@($global_clock) a[*1:$] ##1 b);` (Surelog/tests/CheckerInst/dut.sv) |
| 50 | `ID ( .` | 2 | `t1 t (.mod_intf(mod_intfs[0]));` (verilator/test_regress/t/t_interface_gen11.v) |
| 51 | `ID ; ID` | 2 | `bp_bedrock_cmd_type_e         lce_cmd;` (Surelog/tests/BlackParrotConf/dut.sv) |
| 52 | `ID ; endmodule` | 2 | `tri1 vectored [15:0] a;` (sv-tests/tests/chapter-6/6.9.2--vector_vectored.sv) |
| 53 | `ID ; typedef` | 2 | `typedef some_other_type myalias;` (Surelog/tests/NetType/dut.sv) |
| 54 | `ID = '{` | 2 | `localparam bp_proc_param_s bp_default_cfg_p =` (Surelog/tests/NegParam/dut.sv) |
| 55 | `ID [ ID` | 2 | `protected uvm_phase m_type_overrides[$];` (Surelog/tests/ForeachClassParent/dut.sv) |
| 56 | `NUM : ?` | 2 | `? ?? 01 : ? : 1 ; // preset logic` (ispras-sv-tests/ieee-1800-2012/29/29.09_01.sv) |
| 57 | `[ NUM ]` | 2 | `dim2[0+:1][1] = 0;  // Bad: Range on non-bits` (verilator/test_regress/t/t_mem_multi_ref_bad.v) |
| 58 | `] ID ;` | 2 | `word [2] array;      // word[0:1] exposes the bug as well` (iverilog/ivtest/ivltests/display_bug.v) |
| 59 | `] [ NUM` | 2 | `logic [2][3] x;` (sv2v/test/core/multipack_struct_cast.sv) |
| 60 | `` ID NUM` | 2 | ``line 1 "5.6.4--compiler-directives-debug-line.sv" 1` (sv-tests/tests/chapter-5/5.6.4--compiler-directives-debug-line.sv) |
| 61 | `` ID wire` | 2 | ``default_nettype wire` (verilator/test_regress/t/t_lint_implicit_func_bad.v) |
| 62 | `enum ID ;` | 2 | `parameter type enum E_t;` (verilator/test_regress/t/t_param_type_fwd.v) |
| 63 | `function ID ID` | 2 | `function CrossQueueType myFunc1(int f_lim);` (Surelog/tests/CrossFunc/dut.sv) |
| 64 | `inside { ID` | 2 | `if (GetWhat inside {Get0, GetDefault}) begin : gen_zero` (Surelog/tests/InsideOp/dut.sv) |
| 65 | `tri ID ;` | 2 | `tri  a_bad;` (verilator/test_regress/t/t_udp_bad.v) |
| 66 | `{ ID ,` | 2 | `typedef enum [2:0] {` (verilator/test_regress/t/t_debug_emitv.v) |
| 67 | `!= NUM )` | 1 | `} != 1)` (verilator/test_regress/t/t_std_randomize_with.v) |
| 68 | `!= type (` | 1 | `if (type(value1) != type(value2)) $stop;` (verilator/test_regress/t/t_type_expression_compare.v) |
| 69 | `" " "` | 1 | `"""str` (verilator/test_regress/t/t_parse_eof_qqq_bad.v) |
| 70 | `" ID "` | 1 | `$display("""First "quoted"\nsecond\` (verilator/test_regress/t/t_display_qqq.v) |
| 71 | `" ID -` | 1 | `import "DPI-BAD" task dpix_twice;` (verilator/test_regress/t/t_dpi_type_bad.v) |
| 72 | `" ID \` | 1 | `$display("broken \` (sv-tests/tests/chapter-5/5.9-string-broken-line.sv) |
| 73 | `' ( '` | 1 | `localparam host_base_addr_gp         = (dev_id_width_gp+dev_addr_width_gp)'('h0010_0000);` (Surelog/tests/BlackBePipeInt/dut.sv) |
| 74 | `' ( {` | 1 | `parameter logic [2:0] PartInvDefault = 3'({` (Surelog/tests/ParamTypespec/dut.sv) |
| 75 | `' ID ]` | 1 | `A['x]` (sv2v/test/error/enum_range_x.sv) |
| 76 | `( ) )` | 1 | `initial $display(nested_class0::handle1::handle2.nested_function());` (verible/verible/verilog/tools/kythe/testdata/nested_member_access.sv) |
| 77 | `( ID [` | 1 | `modport client_mp (output .client_req(req[i]));` (sv2v/test/core/interface_generate.sv) |
| 78 | `( NUM )` | 1 | `(01) 1 0  :   0;` (verilator/test_regress/t/t_udp_bad_comb_trigger.v) |
| 79 | `( input logic` | 1 | `extern function myfunc (input logic val);` (verilator/test_regress/t/t_interface_modport_export.v) |
| 80 | `( ref bit` | 1 | `function void uvm_packer::get_packed_bits(ref bit unsigned stream[]);` (Surelog/tests/Assignments/dut.sv) |
| 81 | `) ( ID` | 1 | `module ModuleA #(P=1,) (inp,);` (sv2v/test/core/trailing_commas.sv) |
| 82 | `) ; endmodule` | 1 | `initial $display(C#()::X);` (sv2v/test/error/missing_class.sv) |
| 83 | `) ; integer` | 1 | `mreal #1.2 mr ();` (verilator/test_regress/t/t_param.v) |
| 84 | `) ID ++` | 1 | `foreach (queue[i]) foreach (queue[i][j]) exp_count_que++;` (verilator/test_regress/t/t_foreach_array.v) |
| 85 | `) NUM :` | 1 | `case (type(real))` (verilator/test_regress/t/t_type_compare_bad.v) |
| 86 | `) extends ID` | 1 | `class uvm_sequencer_analysis_fifo #(type RSP = uvm_sequence_item) extends uvm_tlm_fifo #(RSP);` (Surelog/tests/ExtendClassMember/dut.sv) |
| 87 | `) this .` | 1 | `foreach(this.foo.x[i][j])` (verilator/test_regress/t/t_foreach_nested.v) |
| 88 | `) type (` | 1 | `case (type(T))` (sv-tests/tests/chapter-6/6.23--type_op_compare.sv) |
| 89 | `) || ID` | 1 | `if (!bit'(std::randomize(x) with {unique {x};}) || x[0] == x[1]) $stop;` (verilator/test_regress/t/t_std_randomize_unsup_unq_arr.v) |
| 90 | `* ; ID` | 1 | `1 1 ?  :  ?: *;` (verilator/test_regress/t/t_udp_bad_illegal_output.v) |
| 91 | `+ = NUM` | 1 | `for (x += 1; x < y; x++)` (sv2v/test/core/for_loop_inits.sv) |
| 92 | `+ NUM )` | 1 | `void'(1+2);` (iverilog/ivtest/ivltests/sv_void_cast_fail3.v) |
| 93 | `, ID ,` | 1 | `reg	     bit, rst, clk;` (Surelog/tests/1364_2005/dut.v) |
| 94 | `, ID =` | 1 | `parameter type CT1 = Ctype1, CT2 = Ctype2;` (Surelog/tests/ClassTypeParamAlias/dut.sv) |
| 95 | `, NUM )` | 1 | `specparam PATHPULSE$clk$q = (2, 9), PATHPULSE$clr$q = (0, 4),` (ispras-sv-tests/ieee-1800-2012/30/30.07.01_01.sv) |
| 96 | `, parameter type` | 1 | `parameter type A = B,` (verilator/test_regress/t/t_recursive_typedef_bad.v) |
| 97 | `, type ID` | 1 | `virtual class uvm_sequence #(type REQ = uvm_sequence_item,` (Surelog/tests/ClassExtendParam/dut.sv) |
| 98 | `. * )` | 1 | `sub sub (foo, .*);` (verilator/test_regress/t/t_inst_2star_bad.v) |
| 99 | `. ID <=` | 1 | `].s_dyn <= '1;  // OK: direct nonblocking assignment to dynamically-sized array` (verilator/test_regress/t/t_assign_automatic_bad.v) |
| 100 | `. ID [` | 1 | `localparam type p0_t = if0.rq_t[1:0];` (verilator/test_regress/t/t_lparam_assign_iface_typedef4.v) |
| 101 | `: ID /` | 1 | `endclass : ClassB` (ispras-sv-tests/ieee-1800-2012/08/08.26.04_01.sv) |
| 102 | `: ID always` | 1 | `begin : gen_block1` (verible/verible/verilog/tools/lint/testdata/generate_begin_module.sv) |
| 103 | `: ID default` | 1 | `begin : g` (verilator/test_regress/t/t_default_disable_iff_gen_multi_bad.v) |
| 104 | `: ID end` | 1 | `begin : block` (verilator/test_regress/t/t_gen_duplicated_blocks_bad.v) |
| 105 | `: ID int` | 1 | `begin : A` (sv2v/test/core/cast.sv) |
| 106 | `: ID localparam` | 1 | `begin : patterns` (sv2v/test/core/inside_expr.sv) |
| 107 | `: ID string` | 1 | `begin : DSM` (verilator/test_regress/t/t_dpi_string.v) |
| 108 | `: NUM :` | 1 | `$setuphold (posedge clk, negedge d, 0:0:0, 0:0:0);` (verilator/test_regress/t/t_setuphold.v) |
| 109 | `: extends int` | 1 | `function :extends int get_e; return 1; endfunction  // Bad` (verilator/test_regress/t/t_class_override_bad.v) |
| 110 | `: initial int` | 1 | `function :initial int get_i; return 1; endfunction` (verilator/test_regress/t/t_class_override.v) |
| 111 | `:: ID {` | 1 | `typedef enum pkg1::uint8_t {` (verilator/test_regress/t/t_typedef_package.v) |
| 112 | `; ID (` | 1 | `p.m_pack[0].rand_mode(0);` (verilator/test_regress/t/t_randomize_rand_mode_bad.v) |
| 113 | `; ID :` | 1 | `input clk;` (verilator/test_regress/t/t_assert_imm_nz_bad.v) |
| 114 | `; end endgenerate` | 1 | `baz bam;` (verible/verible/verilog/tools/lint/testdata/generate_label_module.sv) |
| 115 | `; end endmodule` | 1 | `foreach (a[i]) ;` (iverilog/ivtest/ivltests/sf1289.v) |
| 116 | `; endclass ID` | 1 | `class bar extends pkg;` (verilator/test_regress/t/t_class_extends_pkg_bad.v) |
| 117 | `; endclass interface` | 1 | `class ClsBad1 implements NotIcls;` (verilator/test_regress/t/t_implements_noninterface_bad.v) |
| 118 | `; endclass module` | 1 | `class ClsI implements Inotfound;` (verilator/test_regress/t/t_implements_notfound_bad.v) |
| 119 | `; endmodule module` | 1 | `output bad_reout_port;` (verilator/test_regress/t/t_var_dup_bad.v) |
| 120 | `; endtask` | 1 | `@x[y];` (verible/verible/verilog/tools/kythe/testdata/event_control.sv) |
| 121 | `; endtask function` | 1 | `->write_events[index];` (verilator/test_regress/t/t_array_event.v) |
| 122 | `; if (` | 1 | `m_env.r_reg_model.p_s[p].n_doorbell.read(status, rdata, 0);` (verilator/test_regress/t/t_resize_lvalue.v) |
| 123 | `; initial ID` | 1 | `intf intf_i;` (verilator/test_regress/t/t_interface_paren_missing_bad.v) |
| 124 | `; initial bit` | 1 | `integer bit; initial bit = 1;` (verilator/test_regress/t/t_flag_language.v) |
| 125 | `; integer ID` | 1 | `dut inst[4];` (iverilog/ivtest/ivltests/br_gh553.v) |
| 126 | `; reg ID` | 1 | `input clk;` (verilator/test_regress/t/t_comb_do_not_convert_to.v) |
| 127 | `; typedef ID` | 1 | `tnoc_types  types;` (Surelog/tests/TNocBadType/dut.sv) |
| 128 | `; virtual function` | 1 | `class Hello #(type T = ihello) implements T;` (sv-tests/tests/chapter-8/8.26.4--illegal_implements_parameter.sv) |
| 129 | `; } endgroup` | 1 | `bins auto[size_var];` (verilator/test_regress/t/t_covergroup_autobins_bad.v) |
| 130 | `<< << <<` | 1 | `<<<<<<< HEAD  // Intentional test: This conflict marker should be here` (verilator/test_regress/t/t_lint_vcmarker_bad.v) |
| 131 | `<EOF>` | 1 | `(* attr` (verilator/test_regress/t/t_parse_eof_attr_bad.v) |
| 132 | `<NO-POSITION>` | 1 | `Error: failed to read input file 'stimuli/sv/subs/sv2v/test/lex/latin1.sv'

Caused by:
    stream did not contain valid ` (sv2v/test/lex/latin1.sv) |
| 133 | `= ( ID` | 1 | `assign active_command3[1:0][2:0][3:0] = (use_AnB) ?  command_A3[1:0][2:0][3:0] : command_B3[1:0][1:0][3:0];` (verilator/test_regress/t/t_mem_slice_bad.v) |
| 134 | `= ID (` | 1 | `input wire integer x = F(P); // defining a default here is non-standard` (sv2v/test/core/port_default.sv) |
| 135 | `= ID ,` | 1 | `d = d_d, // local input logic inferred, default actual` (ispras-sv-tests/ieee-1800-2012/16/16.08.02_01.sv) |
| 136 | `= ID -` | 1 | `j = i++ + (i = i - 1);` (ispras-sv-tests/ieee-1800-2012/11/11.04.02_01.sv) |
| 137 | `= ID [` | 1 | `output [2:1] z = r[2 :+ 1];` (verilator/test_regress/t/t_lint_colonplus_bad.v) |
| 138 | `= NUM '` | 1 | `output [3:0] i65 = 65'd3 ** a; // IWI` (verilator/test_regress/t/t_math_pow6.v) |
| 139 | `== type (` | 1 | `if (type(real) == type(logic[12:0])) $stop;` (verilator/test_regress/t/t_type_compare.v) |
| 140 | `? ; ID` | 1 | `?     :  ?;` (verilator/test_regress/t/t_timescale_udp.v) |
| 141 | `ID ( int` | 1 | `covergroup cg_bad with function foo(int x);` (verilator/test_regress/t/t_covergroup_with_function_foo_bad.v) |
| 142 | `ID , ID` | 1 | `task t(I dst, I src);` (sv2v/test/error/interface_name_func.sv) |
| 143 | `ID , input` | 1 | `input intf i,` (sv2v/test/core/interface_input_type.sv) |
| 144 | `ID , output` | 1 | `l0_if#(W) l0,` (verilator/test_regress/t/t_interface_nested_port_array.v) |
| 145 | `ID / *` | 1 | `'b` (sv2v/test/lex/number_literal_whitespace.sv) |
| 146 | `ID ; begin` | 1 | `logic [D-1:0][W-1:0] s;` (verilator/test_regress/t/t_cover_fsm_sel_assign.v) |
| 147 | `ID ; initial` | 1 | `environment env;` (verible/verible/verilog/tools/kythe/testdata/program.sv) |
| 148 | `ID ; logic` | 1 | `priv_lvl_e mpp;` (Surelog/tests/BadScope/dut.sv) |
| 149 | `ID ; module` | 1 | `typedef some_other_type myalias;` (sv-tests/tests/generic/typedef/typedef_test_8__bad.sv) |
| 150 | `ID ; output` | 1 | `output reg a;` (iverilog/ivtest/ivltests/module_output_port_sv_var2.v) |
| 151 | `ID ; }` | 1 | `uvm_reg_data_t data;` (Surelog/tests/IndexPartSelectBind/dut.sv) |
| 152 | `ID = new` | 1 | `static class2 instance1 = new();` (verible/verible/verilog/tools/kythe/testdata/reversed_definition.sv) |
| 153 | `ID = std` | 1 | `success = std::randomize(a, b) with {b - a > length;};` (ispras-sv-tests/ieee-1800-2012/18/18.12.01_01.sv) |
| 154 | `ID [ NUM` | 1 | `wire intf intf2  [2];` (Surelog/tests/ContAssign/dut.sv) |
| 155 | `ID eventually [` | 1 | `assert property (counter == 1 implies eventually[1: 2] counter == 3);` (verilator/test_regress/t/t_property_unsup.v) |
| 156 | `NUM ( NUM` | 1 | `assert #0_0 (1);` (sv2v/test/nosim/assert.sv) |
| 157 | `NUM ) :` | 1 | `(01) : ? : 0;` (verilator/test_regress/t/t_udp_sequential_x.v) |
| 158 | `NUM ) begin` | 1 | `if (1) begin` (verilator/test_regress/t/t_interface_gen6.v) |
| 159 | `NUM : NUM` | 1 | `00 : 0;` (iverilog/ivtest/ivltests/sv_type_identifier_udp_ansi_name.v) |
| 160 | `NUM ; /` | 1 | `localparam type bad2 = 2;  // Bad` (verilator/test_regress/t/t_param_type_bad.v) |
| 161 | `NUM ; ID` | 1 | `a = 15'1;` (iverilog/ivtest/ivltests/pr3515542.v) |
| 162 | `[ ID :` | 1 | `function f4(int [var1:var2] t1 [var3:var4]);` (verible/verible/verilog/tools/kythe/testdata/function_port_dimensions.sv) |
| 163 | `[ ID ::` | 1 | `int [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH] var5 [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH];` (verible/verible/verilog/tools/kythe/testdata/primitive_type_with_dimensions.sv) |
| 164 | `\ : NUM` | 1 | `0   0  x \` (verilator/test_regress/t/t_udp_nonsequential.v) |
| 165 | `] ; ID` | 1 | `q = q[1:$];` (sv-tests/tests/chapter-7/queues/pop_front_assign.sv) |
| 166 | `] ID ,` | 1 | `logic [4] a, b;` (sv2v/test/core/delay.sv) |
| 167 | `] ID =` | 1 | `S [WIDTH] s = '{` (sv2v/test/core/struct_unit_array.sv) |
| 168 | `] } ;` | 1 | `q = { q[0:1], 10, q[2:$] }; // q.insert(2, 10)` (sv-tests/tests/chapter-7/queues/insert_assign.sv) |
| 169 | `automatic = NUM` | 1 | `integer automatic = 2;` (sv2v/test/lex/begin_keywords.sv) |
| 170 | `begin : ID` | 1 | `begin : named` (verilator/test_regress/t/t_gen_if.v) |
| 171 | `default ) ;` | 1 | `function new(default);` (verilator/test_regress/t/t_class_new_default.v) |
| 172 | `default : ID` | 1 | `default: $stop;` (verilator/test_regress/t/t_case_inside_bad.v) |
| 173 | `do ) ;` | 1 | `module t(input do);` (verilator/test_regress/t/t_langext_order.v) |
| 174 | `endproperty endmodule` | 1 | `endproperty` (verilator/test_regress/t/t_property_case_bad.v) |
| 175 | `int ID )` | 1 | `property check_if_1(var int cyc_mod_2);` (verilator/test_regress/t/t_property_named.v) |
| 176 | `int ID ;` | 1 | `int badi;` (verilator/test_regress/t/t_implements_contents_bad.v) |
| 177 | `interface class ID` | 1 | `interface class bad_cannot_nest;` (verilator/test_regress/t/t_implements_nested_bad.v) |
| 178 | `logic ; /` | 1 | `reg [63:0] logic; // OK: 'logic' is not a keyword in 1364-2001` (ispras-sv-tests/ieee-1800-2012/22/22.14.01_01.sv) |
| 179 | `medium ( )` | 1 | `module medium();` (Surelog/tests/HierPathModule/dut.sv) |
| 180 | `module` | 1 | `endmodule` (sv-tests/tests/chapter-11/11.3.6--two_assign_in_expr.sv) |
| 181 | `new ; /` | 1 | `c = T::new; // This should fail, T is not a class` (iverilog/ivtest/ivltests/sv_class_new_typed_fail4.v) |
| 182 | `packed { ID` | 1 | `typedef union soft packed {` (verilator/test_regress/t/t_export_packed_struct.v) |
| 183 | `packed { logic` | 1 | `typedef union soft packed {` (verilator/test_regress/t/t_export_packed_struct2.v) |
| 184 | `reg ID ;` | 1 | `inout wire reg p;` (ispras-sv-tests/ieee-1800-2012/06/06.07.01_02.sv) |
| 185 | `return ID ;` | 1 | `return success;` (sv-tests/tests/chapter-18/18.12.1--adding-constraints-to-scope-variables_0.sv) |
| 186 | `tri0 ID ;` | 1 | `tri0 cyc;` (verilator/test_regress/t/t_tri_clocking.v) |
| 187 | `tri1 ID )` | 1 | `inout tri1 z` (verilator/test_regress/t/t_tri_gen.v) |
| 188 | `with ( ID` | 1 | `bins apple = X with (a + b < 257) matches 127;` (ispras-sv-tests/ieee-1800-2012/19/19.06.01.02_01.sv) |
| 189 | `{ / /` | 1 | `enum logic [1:0][1:0] {  // BAD enum type` (verilator/test_regress/t/t_enum_base_bad.v) |
| 190 | `{ ID >` | 1 | `total = q.sum() with (item) { item > 0; };` (verilator/test_regress/t/t_randomize_with_idlist_method_bad.v) |
| 191 | `{ bit [` | 1 | `union soft {` (verilator/test_regress/t/t_union_soft.v) |
| 192 | `| ( NUM` | 1 | `| (0                 <<  5)` (Surelog/tests/CastShift/dut.sv) |
| 193 | `| ID ;` | 1 | `assign y[0] = !|v;` (verilator/test_regress/t/t_lint_notredop_bad.v) |
| 194 | `|| ID inside` | 1 | `constraint B { w == 5; x inside {1,2} || x inside {4,5}; }` (verilator/test_regress/t/t_randomize_method_constraints.v) |
| 195 | `} ) ;` | 1 | `task tsk(int q[] = {});` (verilator/test_regress/t/t_queue_empty_pin.v) |
| 196 | `} + NUM` | 1 | `i = {} + 1;` (verilator/test_regress/t/t_queue_empty_bad.v) |
| 197 | `} ; if` | 1 | `words = {};` (iverilog/ivtest/ivltests/sv_queue3.v) |
| 198 | `} ID :` | 1 | `}` (verilator/test_regress/t/t_covergroup_cross_opt_unsup.v) |
| 199 | `} endgroup covergroup` | 1 | `}` (Surelog/tests/Covergroup/dut.sv) |
| 200 | `~ ID ==` | 1 | `constraint unary { !(-~c == 'h22); }` (verilator/test_regress/t/t_constraint_operators.v) |

