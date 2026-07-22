# rejects-valid stuck-point clusters (SV-CORPUS-GRAD.3.0)

543 rows probed from the adjudication manifest; 247 distinct 3-token stuck signatures.

| # | signature | rows | example (stuck line) |
|---|---|---|---|
| 1 | `[ NUM :` | 22 | `bins c1 = ! binsof(a) intersect {[100:200]};// 4 cross products` (Surelog/tests/CrossItem/dut.sv) |
| 2 | `ID ) ;` | 16 | `modport A (output .P(r[3:0]), input .Q(x), R);` (Surelog/tests/InterfaceModExp/dut.sv) |
| 3 | `` ID )` | 15 | `$display(`__FILE__);` (iverilog/ivtest/ivltests/fileline.v) |
| 4 | `ID , ID` | 13 | `modport slave(input req, addr, mode, start, clk,` (ispras-sv-tests/ieee-1800-2012/25/25.05.01_01.sv) |
| 5 | `` ID ,` | 11 | `$display("FAILED(%0d). Expected 1, got %b", `__LINE__, a);` (iverilog/ivtest/ivltests/br_gh1321.v) |
| 6 | `> ID )` | 9 | `a1: assert property (@(posedge clk) a |-> p3);` (ispras-sv-tests/ieee-1800-2012/16/16.12.01_01.sv) |
| 7 | `' ID NUM` | 8 | `parameter logic [31:0]     IdcodeValue = 32'h 0000_0001` (Surelog/tests/BindStmt2/dut.sv) |
| 8 | `; / /` | 8 | `reg logic; // OK: "logic" is not a keyword in 1364-2001` (sv-tests/tests/chapter-5/5.6.4--compiler-directives-begin-keywords.sv) |
| 9 | `> ( ID` | 8 | `aw_select: assume property( @(posedge clk_i) (slv_req_i.aw_valid |->` (Surelog/tests/AssumeProp/dut.sv) |
| 10 | `> ID ==` | 8 | `assert property (up ##[*] up |=> cnt == $past(cnt, 2) + 8'd 2);` (Surelog/tests/UnitSVA/dut.sv) |
| 11 | `' ( ID` | 7 | `if (addr_q < $clog2(RomSize)'(RomSize)) begin` (Surelog/tests/ClogCast/dut.sv) |
| 12 | `; endmodule` | 7 | `Interface i;` (sv2v/test/error/interface_modport_unlisted.sv) |
| 13 | `> ID ;` | 7 | `@($global_clock) req |=> ack;` (Surelog/tests/CheckerInst/dut.sv) |
| 14 | `ID NUM ;` | 7 | `a = -8'd 6;  // this defines the two's-complement of 6,` (sv-tests/tests/chapter-5/5.7.1--integers-signed.sv) |
| 15 | `with { ID` | 7 | `if (std::randomize(example.member[1]) with {example.member[1] inside {[0 : 3]};} == 0) $stop;` (verilator/test_regress/t/t_randomize_arraysel_membersel.v) |
| 16 | `{ ID =` | 7 | `typedef enum [5:0] {` (sv2v/test/core/enum_scope.sv) |
| 17 | `# ( .` | 6 | `instance top use #(.WIDTH(32));` (ispras-sv-tests/ieee-1800-2012/33/33.04.03_02.sv) |
| 18 | `> ID (` | 6 | `a1: assert property (@clk $changing_gclk(sig) |-> $falling_gclk(clk))` (ispras-sv-tests/ieee-1800-2012/16/16.09.04_02.sv) |
| 19 | `> ID [` | 6 | `$rose(a) |-> q[0];` (ispras-sv-tests/ieee-1800-2012/16/16.06_01.sv) |
| 20 | `ID = ID` | 6 | `coverage_level clevel = cover_all // This argument should be bound to an` (Surelog/tests/Checker/dut.sv) |
| 21 | `` ID ID` | 6 | `output b;` (ispras-sv-tests/ieee-1800-2012/34/34.03.01_01.sv) |
| 22 | `| -> ID` | 6 | `$falling_gclk(clk) ##1 (!$falling_gclk(clk)[*1:$]) |-> $steady_gclk(sig));` (ispras-sv-tests/ieee-1800-2012/16/16.09.04_03.sv) |
| 23 | `, weak1 )` | 5 | `assign (highz0, weak1)    inout_i1 = pu;` (Surelog/tests/MultContAssign/dut.sv) |
| 24 | `@ ( posedge` | 5 | `let p2(x) = $past(x,,,@(posedge clock));` (Surelog/tests/LetExpr/dut.sv) |
| 25 | `ID , output` | 5 | `modport master(input a, b, output c, d);` (ispras-sv-tests/ieee-1800-2012/25/25.05_01.sv) |
| 26 | `ID = NUM` | 5 | `module top #(int N = 5, M = N * 16, type T = int, T x = 0) ();` (ispras-sv-tests/ieee-1800-2012/06/06.20.01_02.sv) |
| 27 | `] ; /` | 5 | `channel = channel[size:$]; // update the stream so it now lacks that packet` (ispras-sv-tests/ieee-1800-2012/06/06.24.03_03.sv) |
| 28 | `| -> ##` | 5 | `(valid_in, x = pipe_in) |-> ##5 (pipe_out1 == (x + 1));` (ispras-sv-tests/ieee-1800-2012/16/16.10_03.sv) |
| 29 | `. ID (` | 4 | `input .x(x[2*WIDTH-1:WIDTH]), .y(x[WIDTH-1:0])` (sv2v/test/core/interface_modport.sv) |
| 30 | `. ID )` | 4 | `tagged Jmp (tagged JmpU .a) : pc = pc + a;` (ispras-sv-tests/ieee-1800-2012/12/12.06.01_05.sv) |
| 31 | `. ID ;` | 4 | `localparam type p0_t = if0.rq_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef2.v) |
| 32 | `: = NUM` | 4 | `x dist {100 := 1, 200 := 2, 300 := 5};` (ispras-sv-tests/ieee-1800-2012/18/18.05.04_01.sv) |
| 33 | `NUM ) NUM` | 4 | `(01)         0   :   ?   :   0   ;` (Surelog/tests/Udp/dut.sv) |
| 34 | `case ( ID` | 4 | `unique0 case(a) // values 3, 5, 6, 7 do not cause a violation report` (ispras-sv-tests/ieee-1800-2012/12/12.05.03_01.sv) |
| 35 | `| -> (` | 4 | `assert property (@(clk) go ##1 get[*2] |-> (reject_on(stop) put[->2]));` (ispras-sv-tests/ieee-1800-2012/16/16.12.14_01.sv) |
| 36 | `( ) ;` | 3 | `function void uvm_sequencer_base::start_phase_sequence();` (Surelog/tests/ClassMemberFunc/dut.sv) |
| 37 | `, supply1 )` | 3 | `wire (supply0, supply1) x;` (sv2v/test/error/drive_strength_uninit.sv) |
| 38 | `- NUM ]` | 3 | `enum {ndef[-1]} ndef1;` (iverilog/ivtest/ivltests/pr3366217d.v) |
| 39 | `: ID ID` | 3 | `generate begin:foo` (Surelog/tests/DefParamIndex/dut.sv) |
| 40 | `: ID for` | 3 | `begin : topgen` (verilator/test_regress/t/t_gen_local.v) |
| 41 | `: ID logic` | 3 | `begin : A` (sv2v/test/core/array.sv) |
| 42 | `: ID wire` | 3 | `begin : A` (Surelog/tests/GenerateBlock/dut.sv) |
| 43 | `> ! ID` | 3 | `same(x, y) |=> !same(x, y);` (ispras-sv-tests/ieee-1800-2012/11/11.13_07.sv) |
| 44 | `> ## NUM` | 3 | `assert property ( @(posedge Clock)  strap_en_i |=> ##0 !strap_en_i [*]);` (Surelog/tests/Assert/dut.sv) |
| 45 | `ID ( "` | 3 | `for (i = 0; i < FOO_REPL[0+:32]; i=i+1) begin : genfoo` (Surelog/tests/IndexedSelect/dut.sv) |
| 46 | `ID ( ID` | 3 | `extern function uvm_mem_mam_cfg reconfigure(uvm_mem_mam_cfg cfg = null);` (Surelog/tests/BitSelectHier/dut.sv) |
| 47 | `ID ; /` | 3 | `scalar_t pc;` (Surelog/tests/HierBitSlice/dut.sv) |
| 48 | `[ ID :` | 3 | `@(posedge clk) a ##[r1:r2] b ##2 1'b1 |=> d;` (Surelog/tests/PoundDelay/dut.sv) |
| 49 | `} endgroup endmodule` | 3 | `bins one = '{'{1, 2}, '{3, 4}, '{5, 6}};` (ispras-sv-tests/ieee-1800-2012/19/19.06.01.04_01.sv) |
| 50 | `" ID ID` | 2 | `$display("Humpty Dumpty sat on a wall. \` (ispras-sv-tests/ieee-1800-2012/05/05.09_01.sv) |
| 51 | `# ( )` | 2 | `instance top.a1 use #(); // set all parameters in instance 'a1'` (ispras-sv-tests/ieee-1800-2012/33/33.04.03_06.sv) |
| 52 | `## NUM ID` | 2 | `trans ##1 start_trans ##1 s ##1 end_trans;` (ispras-sv-tests/ieee-1800-2012/16/16.08_04.sv) |
| 53 | `' ( '` | 2 | `localparam host_base_addr_gp         = (dev_id_width_gp+dev_addr_width_gp)'('h0010_0000);` (Surelog/tests/BlackBePipeInt/dut.sv) |
| 54 | `( ID )` | 2 | `states_t state;` (Surelog/tests/EnumParenType/dut.sv) |
| 55 | `) ; /` | 2 | `class Cls1 extends Base1(default);` (verilator/test_regress/t/t_class_extends_default.v) |
| 56 | `) ; ID` | 2 | `m();` (verilator/test_regress/t/t_inst_noname_bad.v) |
| 57 | `) ; endclass` | 2 | `parameter type P_T = i` (verilator/test_regress/t/t_param_type_id_bad.v) |
| 58 | `) ID (` | 2 | `pullup (strong1) p1 (neta), p2 (netb);` (ispras-sv-tests/ieee-1800-2012/28/28.10_01.sv) |
| 59 | `) ID [` | 2 | `foreach(array[i][j])` (Surelog/tests/ForeachSquare/dut.sv) |
| 60 | `) begin /` | 2 | `foreach (y) begin` (verible/verible/verilog/tools/kythe/testdata/foreach.sv) |
| 61 | `, ID ;` | 2 | `class Cls extends Base1, Base2;` (verilator/test_regress/t/t_class_extends_bad.v) |
| 62 | `, pull0 )` | 2 | `wire (strong1, pull0) mynet = enable;` (ispras-sv-tests/ieee-1800-2012/10/10.03.01_01.sv) |
| 63 | `, strong0 )` | 2 | `nor (highz1, strong0) n1(out1, in1, in2);` (ispras-sv-tests/ieee-1800-2012/28/28.03.02_01.sv) |
| 64 | `. ID ,` | 2 | `if (e matches (tagged Jmp (tagged JmpC '{cc:.c, addr:.a}))) begin` (ispras-sv-tests/ieee-1800-2012/12/12.06.02_01.sv) |
| 65 | `. ID .` | 2 | `localparam type p0_rq2_t = if0.y_if0.rq2_t;` (verilator/test_regress/t/t_lparam_assign_iface_typedef_nested.v) |
| 66 | `. ID ID` | 2 | `assign #sub.delay dly4 = dly1 + 1;` (verilator/test_regress/t/t_delay.v) |
| 67 | `:: ID ;` | 2 | `#1 ->my_package::e;` (iverilog/ivtest/ivltests/sv_wildcard_import2.v) |
| 68 | `; ID (` | 2 | `p.m_pack[0].rand_mode(0);` (verilator/test_regress/t/t_randomize_rand_mode_bad.v) |
| 69 | `; ID .` | 2 | `sim.queues[Active].push_back('{is_update: 1});` (ispras-sv-tests/ieee-1800-2012/04/04.05_01.sv) |
| 70 | `; ID ID` | 2 | `tnoc_types#(2) i_types;` (Surelog/tests/InterfTypeBad/dut.sv) |
| 71 | `; logic [` | 2 | `input clk;` (verilator/test_regress/t/t_clk_concat5.v) |
| 72 | `; parameter ID` | 2 | `parameter bp_page_offset_width_gp = ( ((bp_page_size_in_bytes_gp)==1) ? 1 : $clog2((bp_page_size_in_bytes_gp)));` (Surelog/tests/BlackParrotConf/dut.sv) |
| 73 | `= ID )` | 2 | `rst = $inferred_disable);` (Surelog/tests/SequenceInst/dut.sv) |
| 74 | `= NUM ;` | 2 | `output a = 1;` (sv2v/test/error/port_init_early.sv) |
| 75 | `== NUM )` | 2 | `assert((std::randomize(x) with {x inside {10, 20}; }) == 1)` (verilator/test_regress/t/t_randomize_inside_cond.v) |
| 76 | `> ## [` | 2 | `$rose(data_phase) |-> ##[1:5] ((irdy == 0) && ($fell(trdy) || $fell(stop)));` (ispras-sv-tests/ieee-1800-2012/16/16.12.06_01.sv) |
| 77 | `> @ (` | 2 | `@(posedge clk0) s3 |=> @(posedge clk1) s2;` (ispras-sv-tests/ieee-1800-2012/16/16.13.02_01.sv) |
| 78 | `> ID ##` | 2 | `@(posedge clk) a |-> b ##1 c ##1 d;` (ispras-sv-tests/ieee-1800-2012/16/16.12.20_01.sv) |
| 79 | `ID ( .` | 2 | `t1 t (.mod_intf(mod_intfs[0]));` (verilator/test_regress/t/t_interface_gen11.v) |
| 80 | `ID , input` | 2 | `input intf i,` (sv2v/test/core/interface_input_type.sv) |
| 81 | `ID ; endmodule` | 2 | `tri1 vectored [15:0] a;` (sv-tests/tests/chapter-6/6.9.2--vector_vectored.sv) |
| 82 | `ID ; typedef` | 2 | `typedef some_other_type myalias;` (Surelog/tests/NetType/dut.sv) |
| 83 | `ID = '{` | 2 | `localparam bp_proc_param_s bp_default_cfg_p =` (Surelog/tests/NegParam/dut.sv) |
| 84 | `ID ID ;` | 2 | `a = 'h x; // yields xxx` (ispras-sv-tests/ieee-1800-2012/05/05.07.01_04.sv) |
| 85 | `ID NUM )` | 2 | `Prefix: 96'(96'h 4c52_5443_5f4d_4f52_4001_0001)` (Surelog/tests/LargeHexCast/dut.sv) |
| 86 | `ID [ ID` | 2 | `protected uvm_phase m_type_overrides[$];` (Surelog/tests/ForeachClassParent/dut.sv) |
| 87 | `NUM : ?` | 2 | `? ?? 01 : ? : 1 ; // preset logic` (ispras-sv-tests/ieee-1800-2012/29/29.09_01.sv) |
| 88 | `[ NUM ]` | 2 | `dim2[0+:1][1] = 0;  // Bad: Range on non-bits` (verilator/test_regress/t/t_mem_multi_ref_bad.v) |
| 89 | `] ID ;` | 2 | `word [2] array;      // word[0:1] exposes the bug as well` (iverilog/ivtest/ivltests/display_bug.v) |
| 90 | `] [ NUM` | 2 | `logic [2][3] x;` (sv2v/test/core/multipack_struct_cast.sv) |
| 91 | `` ID NUM` | 2 | `module directives();` (sv-tests/tests/chapter-5/5.6.4--compiler-directives-debug-line.sv) |
| 92 | `` ID wire` | 2 | `endfunction` (verilator/test_regress/t/t_lint_implicit_func_bad.v) |
| 93 | `enum ID ;` | 2 | `parameter type enum E_t;` (verilator/test_regress/t/t_param_type_fwd.v) |
| 94 | `function ID ID` | 2 | `{` (Surelog/tests/CrossFunc/dut.sv) |
| 95 | `if ( ID` | 2 | `unique0 if(a == 0) b = 1;` (sv-tests/tests/chapter-12/12.4.2--unique0_if.sv) |
| 96 | `inside { ID` | 2 | `if (GetWhat inside {Get0, GetDefault}) begin : gen_zero` (Surelog/tests/InsideOp/dut.sv) |
| 97 | `tri ID ;` | 2 | `primitive udp_x (a_bad, b, c_bad);` (verilator/test_regress/t/t_udp_bad.v) |
| 98 | `{ ID ,` | 2 | `typedef enum [2:0] {` (verilator/test_regress/t/t_debug_emitv.v) |
| 99 | `!= NUM )` | 1 | `} != 1)` (verilator/test_regress/t/t_std_randomize_with.v) |
| 100 | `!= type (` | 1 | `if (type(value1) != type(value2)) $stop;` (verilator/test_regress/t/t_type_expression_compare.v) |
| 101 | `" " "` | 1 | `` (verilator/test_regress/t/t_parse_eof_qqq_bad.v) |
| 102 | `" ID "` | 1 | `$display("""First "quoted"\nsecond\` (verilator/test_regress/t/t_display_qqq.v) |
| 103 | `" ID -` | 1 | `import "DPI-BAD" task dpix_twice;` (verilator/test_regress/t/t_dpi_type_bad.v) |
| 104 | `" ID \` | 1 | `$display("broken \` (sv-tests/tests/chapter-5/5.9-string-broken-line.sv) |
| 105 | `' ( NUM` | 1 | `r[p+:2] = $bits(r[p+:2])'(1'sb1);` (sv2v/test/core/package_function_cast.sv) |
| 106 | `' ( {` | 1 | `parameter logic [2:0] PartInvDefault = 3'({` (Surelog/tests/ParamTypespec/dut.sv) |
| 107 | `' ID ID` | 1 | `parameter int foo = 1 'b x;` (sv-tests/tests/generic/number/number_test_11.sv) |
| 108 | `' ID ]` | 1 | `A['x]` (sv2v/test/error/enum_range_x.sv) |
| 109 | `( ) )` | 1 | `initial $display(nested_class0::handle1::handle2.nested_function());` (verible/verible/verilog/tools/kythe/testdata/nested_member_access.sv) |
| 110 | `( ID [` | 1 | `modport client_mp (output .client_req(req[i]));` (sv2v/test/core/interface_generate.sv) |
| 111 | `( NUM )` | 1 | `0 ? 1  :   1;` (verilator/test_regress/t/t_udp_bad_comb_trigger.v) |
| 112 | `( input logic` | 1 | `extern function myfunc (input logic val);` (verilator/test_regress/t/t_interface_modport_export.v) |
| 113 | `( ref bit` | 1 | `function void uvm_packer::get_packed_bits(ref bit unsigned stream[]);` (Surelog/tests/Assignments/dut.sv) |
| 114 | `) ( ID` | 1 | `module ModuleA #(P=1,) (inp,);` (sv2v/test/core/trailing_commas.sv) |
| 115 | `) ; endmodule` | 1 | `initial $display(C#()::X);` (sv2v/test/error/missing_class.sv) |
| 116 | `) ; integer` | 1 | `mreal #1.2 mr ();` (verilator/test_regress/t/t_param.v) |
| 117 | `) ID ++` | 1 | `foreach (queue[i]) foreach (queue[i][j]) exp_count_que++;` (verilator/test_regress/t/t_foreach_array.v) |
| 118 | `) NUM :` | 1 | `case (type(real))` (verilator/test_regress/t/t_type_compare_bad.v) |
| 119 | `) extends ID` | 1 | `class uvm_sequencer_analysis_fifo #(type RSP = uvm_sequence_item) extends uvm_tlm_fifo #(RSP);` (Surelog/tests/ExtendClassMember/dut.sv) |
| 120 | `) this .` | 1 | `foreach(this.foo.x[i][j])` (verilator/test_regress/t/t_foreach_nested.v) |
| 121 | `) type (` | 1 | `case (type(T))` (sv-tests/tests/chapter-6/6.23--type_op_compare.sv) |
| 122 | `) || ID` | 1 | `if (!bit'(std::randomize(x) with {unique {x};}) || x[0] == x[1]) $stop;` (verilator/test_regress/t/t_std_randomize_unsup_unq_arr.v) |
| 123 | `* ; ID` | 1 | `1 1 ?  :  ?: *;` (verilator/test_regress/t/t_udp_bad_illegal_output.v) |
| 124 | `+ ( (` | 1 | `localparam output_dirs_sparse_lp = (($bits(routing_matrix_p[0][i]) < 65) ? 1'b0 : ('X)) + ((((routing_matrix_p[0][i])>>(` (Surelog/tests/BlackParrotComplex/dut.sv) |
| 125 | `+ = NUM` | 1 | `for (x += 1; x < y; x++)` (sv2v/test/core/for_loop_inits.sv) |
| 126 | `+ NUM )` | 1 | `void'(1+2);` (iverilog/ivtest/ivltests/sv_void_cast_fail3.v) |
| 127 | `, ID ,` | 1 | `reg	     bit, rst, clk;` (Surelog/tests/1364_2005/dut.v) |
| 128 | `, ID =` | 1 | `parameter type CT1 = Ctype1, CT2 = Ctype2;` (Surelog/tests/ClassTypeParamAlias/dut.sv) |
| 129 | `, NUM )` | 1 | `specparam PATHPULSE$clk$q = (2, 9), PATHPULSE$clr$q = (0, 4),` (ispras-sv-tests/ieee-1800-2012/30/30.07.01_01.sv) |
| 130 | `, highz0 )` | 1 | `and (strong1, highz0) N1(a, b);` (ispras-sv-tests/ieee-1800-2012/28/28.12.02_01.sv) |
| 131 | `, highz1 )` | 1 | `wire (weak0, highz1) a = 1;` (verilator/test_regress/t/t_strength_highz.v) |
| 132 | `, parameter ID` | 1 | `, parameter lg_max_val_lp=($clog2(max_val_p+1))` (Surelog/tests/BlackParrotSkipParam/dut.sv) |
| 133 | `, parameter type` | 1 | `parameter type A = B,` (verilator/test_regress/t/t_recursive_typedef_bad.v) |
| 134 | `, strong1 )` | 1 | `assign (weak0, strong1) a = clk ? 'z : '0;` (verilator/test_regress/t/t_strength_2_uneq_assign.v) |
| 135 | `, type ID` | 1 | `virtual class uvm_sequence #(type REQ = uvm_sequence_item,` (Surelog/tests/ClassExtendParam/dut.sv) |
| 136 | `, weak0 )` | 1 | `assign (weak1, weak0) {rout_ca1, rout_ca2} = in; // Non-default strength so invalid` (iverilog/ivtest/ivltests/br_gh1222.v) |
| 137 | `. * )` | 1 | `sub sub (foo, .*);` (verilator/test_regress/t/t_inst_2star_bad.v) |
| 138 | `. ID <=` | 1 | `].s_dyn <= '1;  // OK: direct nonblocking assignment to dynamically-sized array` (verilator/test_regress/t/t_assign_automatic_bad.v) |
| 139 | `. ID [` | 1 | `localparam type p0_t = if0.rq_t[1:0];` (verilator/test_regress/t/t_lparam_assign_iface_typedef4.v) |
| 140 | `: ID /` | 1 | `endclass : ClassB` (ispras-sv-tests/ieee-1800-2012/08/08.26.04_01.sv) |
| 141 | `: ID always` | 1 | `begin : gen_block1` (verible/verible/verilog/tools/lint/testdata/generate_begin_module.sv) |
| 142 | `: ID default` | 1 | `begin : g` (verilator/test_regress/t/t_default_disable_iff_gen_multi_bad.v) |
| 143 | `: ID end` | 1 | `begin : block` (verilator/test_regress/t/t_gen_duplicated_blocks_bad.v) |
| 144 | `: ID int` | 1 | `begin : A` (sv2v/test/core/cast.sv) |
| 145 | `: ID localparam` | 1 | `begin : patterns` (sv2v/test/core/inside_expr.sv) |
| 146 | `: ID string` | 1 | `begin : DSM` (verilator/test_regress/t/t_dpi_string.v) |
| 147 | `: NUM :` | 1 | `$setuphold (posedge clk, negedge d, 0:0:0, 0:0:0);` (verilator/test_regress/t/t_setuphold.v) |
| 148 | `: extends int` | 1 | `function :extends int get_e; return 1; endfunction  // Bad` (verilator/test_regress/t/t_class_override_bad.v) |
| 149 | `: initial int` | 1 | `function :initial int get_i; return 1; endfunction` (verilator/test_regress/t/t_class_override.v) |
| 150 | `:: ID {` | 1 | `typedef enum pkg1::uint8_t {` (verilator/test_regress/t/t_typedef_package.v) |
| 151 | `; ID :` | 1 | `input clk;` (verilator/test_regress/t/t_assert_imm_nz_bad.v) |
| 152 | `; assign ID` | 1 | `wreal wr;` (verilator/test_regress/t/t_vams_basic.v) |
| 153 | `; end endgenerate` | 1 | `baz bam;` (verible/verible/verilog/tools/lint/testdata/generate_label_module.sv) |
| 154 | `; end endmodule` | 1 | `foreach (a[i]) ;` (iverilog/ivtest/ivltests/sf1289.v) |
| 155 | `; endclass ID` | 1 | `class bar extends pkg;` (verilator/test_regress/t/t_class_extends_pkg_bad.v) |
| 156 | `; endclass interface` | 1 | `class ClsBad1 implements NotIcls;` (verilator/test_regress/t/t_implements_noninterface_bad.v) |
| 157 | `; endclass module` | 1 | `class ClsI implements Inotfound;` (verilator/test_regress/t/t_implements_notfound_bad.v) |
| 158 | `; endmodule module` | 1 | `output bad_reout_port;` (verilator/test_regress/t/t_var_dup_bad.v) |
| 159 | `; endtask` | 1 | `@x[y];` (verible/verible/verilog/tools/kythe/testdata/event_control.sv) |
| 160 | `; endtask function` | 1 | `->write_events[index];` (verilator/test_regress/t/t_array_event.v) |
| 161 | `; if (` | 1 | `m_env.r_reg_model.p_s[p].n_doorbell.read(status, rdata, 0);` (verilator/test_regress/t/t_resize_lvalue.v) |
| 162 | `; initial ID` | 1 | `intf intf_i;` (verilator/test_regress/t/t_interface_paren_missing_bad.v) |
| 163 | `; initial bit` | 1 | `integer bit; initial bit = 1;` (verilator/test_regress/t/t_flag_language.v) |
| 164 | `; integer ID` | 1 | `dut inst[4];` (iverilog/ivtest/ivltests/br_gh553.v) |
| 165 | `; reg ID` | 1 | `input clk;` (verilator/test_regress/t/t_comb_do_not_convert_to.v) |
| 166 | `; supply1 ID` | 1 | `supply0       gnd;` (verilator/test_regress/t/t_gate_fdup.v) |
| 167 | `; typedef ID` | 1 | `tnoc_types  types;` (Surelog/tests/TNocBadType/dut.sv) |
| 168 | `; typedef struct` | 1 | `localparam lg_max_cfgs = ( ((max_cfgs)==1) ? 1 : $clog2((max_cfgs)));` (Surelog/tests/ConcatOrder/dut.sv) |
| 169 | `; virtual function` | 1 | `class Hello #(type T = ihello) implements T;` (sv-tests/tests/chapter-8/8.26.4--illegal_implements_parameter.sv) |
| 170 | `; } endgroup` | 1 | `bins auto[size_var];` (verilator/test_regress/t/t_covergroup_autobins_bad.v) |
| 171 | `<< << <<` | 1 | `module t;` (verilator/test_regress/t/t_lint_vcmarker_bad.v) |
| 172 | `<EOF>` | 1 | `(* attr` (verilator/test_regress/t/t_parse_eof_attr_bad.v) |
| 173 | `<NO-POSITION>` | 1 | `Error: failed to read input file 'stimuli/sv/subs/sv2v/test/lex/latin1.sv'

Caused by:
    stream did not contain valid ` (sv2v/test/lex/latin1.sv) |
| 174 | `= ( ID` | 1 | `assign active_command3[1:0][2:0][3:0] = (use_AnB) ?  command_A3[1:0][2:0][3:0] : command_B3[1:0][1:0][3:0];` (verilator/test_regress/t/t_mem_slice_bad.v) |
| 175 | `= ID (` | 1 | `input wire integer x = F(P); // defining a default here is non-standard` (sv2v/test/core/port_default.sv) |
| 176 | `= ID ,` | 1 | `d = d_d, // local input logic inferred, default actual` (ispras-sv-tests/ieee-1800-2012/16/16.08.02_01.sv) |
| 177 | `= ID -` | 1 | `j = i++ + (i = i - 1);` (ispras-sv-tests/ieee-1800-2012/11/11.04.02_01.sv) |
| 178 | `= ID [` | 1 | `output [2:1] z = r[2 :+ 1];` (verilator/test_regress/t/t_lint_colonplus_bad.v) |
| 179 | `= NUM '` | 1 | `output [3:0] i65 = 65'd3 ** a; // IWI` (verilator/test_regress/t/t_math_pow6.v) |
| 180 | `== type (` | 1 | `if (type(real) == type(logic[12:0])) $stop;` (verilator/test_regress/t/t_type_compare.v) |
| 181 | `> ! (` | 1 | `(logic'(x) === logic'(y)) |=> ! (logic'(x) === logic'(y));` (Surelog/tests/Assertions/dut.sv) |
| 182 | `> ( @` | 1 | `@(posedge clk) a ##1 (@(posedge clk1) s1) |=> (@(posedge clk2) s2);` (ispras-sv-tests/ieee-1800-2012/16/16.13.04_01.sv) |
| 183 | `? ; ID` | 1 | `?     :  ?;` (verilator/test_regress/t/t_timescale_udp.v) |
| 184 | `ID != =` | 1 | `if (T::VALUE !== 23) begin` (iverilog/ivtest/ivltests/sv_type_identifier_package_name.v) |
| 185 | `ID ( int` | 1 | `covergroup cg_bad with function foo(int x);` (verilator/test_regress/t/t_covergroup_with_function_foo_bad.v) |
| 186 | `ID , import` | 1 | `modport Tx(output sop, data, import unpack);` (verilator/test_regress/t/t_varref_scope_in_interface.v) |
| 187 | `ID / *` | 1 | `'b` (sv2v/test/lex/number_literal_whitespace.sv) |
| 188 | `ID ; ID` | 1 | `typedef my_iface#(T) self_t;` (verilator/test_regress/t/t_iface_self_ref_typedef.v) |
| 189 | `ID ; begin` | 1 | `logic [D-1:0][W-1:0] s;` (verilator/test_regress/t/t_cover_fsm_sel_assign.v) |
| 190 | `ID ; initial` | 1 | `environment env;` (verible/verible/verilog/tools/kythe/testdata/program.sv) |
| 191 | `ID ; logic` | 1 | `priv_lvl_e mpp;` (Surelog/tests/BadScope/dut.sv) |
| 192 | `ID ; module` | 1 | `typedef some_other_type myalias;` (sv-tests/tests/generic/typedef/typedef_test_8__bad.sv) |
| 193 | `ID ; output` | 1 | `output reg a;` (iverilog/ivtest/ivltests/module_output_port_sv_var2.v) |
| 194 | `ID ; }` | 1 | `uvm_reg_data_t data;` (Surelog/tests/IndexPartSelectBind/dut.sv) |
| 195 | `ID = new` | 1 | `static class2 instance1 = new();` (verible/verible/verilog/tools/kythe/testdata/reversed_definition.sv) |
| 196 | `ID = std` | 1 | `success = std::randomize(a, b) with {b - a > length;};` (ispras-sv-tests/ieee-1800-2012/18/18.12.01_01.sv) |
| 197 | `ID ? NUM` | 1 | `2'b ?1: out0 = 3'd0;` (verilator/test_regress/t/t_priority_case.v) |
| 198 | `ID NUM /` | 1 | `IntA = -'d 12 / 3; // The result is 1431655761` (ispras-sv-tests/ieee-1800-2012/11/11.03.03_01.sv) |
| 199 | `ID [ NUM` | 1 | `wire intf intf2  [2];` (Surelog/tests/ContAssign/dut.sv) |
| 200 | `ID eventually [` | 1 | `assert property (counter == 1 implies eventually[1: 2] counter == 3);` (verilator/test_regress/t/t_property_unsup.v) |
| 201 | `NUM ( NUM` | 1 | `assert #0_0 (1);` (sv2v/test/nosim/assert.sv) |
| 202 | `NUM ) :` | 1 | `(01) : ? : 0;` (verilator/test_regress/t/t_udp_sequential_x.v) |
| 203 | `NUM ) begin` | 1 | `if (1) begin` (verilator/test_regress/t/t_interface_gen6.v) |
| 204 | `NUM / *` | 1 | `/* comment */ 1          /* comment */ // comment` (iverilog/ivtest/ivltests/br_gh782b.v) |
| 205 | `NUM : NUM` | 1 | `table` (iverilog/ivtest/ivltests/sv_type_identifier_udp_ansi_name.v) |
| 206 | `NUM ; /` | 1 | `localparam type bad2 = 2;  // Bad` (verilator/test_regress/t/t_param_type_bad.v) |
| 207 | `NUM ; ID` | 1 | `a = 15'1;` (iverilog/ivtest/ivltests/pr3515542.v) |
| 208 | `[ ID ::` | 1 | `int [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH] var5 [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH];` (verible/verible/verilog/tools/kythe/testdata/primitive_type_with_dimensions.sv) |
| 209 | `\ : NUM` | 1 | `0   0  x \` (verilator/test_regress/t/t_udp_nonsequential.v) |
| 210 | `] ; ID` | 1 | `q = q[1:$];` (sv-tests/tests/chapter-7/queues/pop_front_assign.sv) |
| 211 | `] ID ,` | 1 | `logic [4] a, b;` (sv2v/test/core/delay.sv) |
| 212 | `] ID =` | 1 | `S [WIDTH] s = '{` (sv2v/test/core/struct_unit_array.sv) |
| 213 | `] } ;` | 1 | `q = { q[0:1], 10, q[2:$] }; // q.insert(2, 10)` (sv-tests/tests/chapter-7/queues/insert_assign.sv) |
| 214 | `` ID ;` | 1 | `localparam string FILENAME = `__FILE__;` (verilator/test_regress/t/t_stop_winos_bad.v) |
| 215 | `automatic = NUM` | 1 | `integer automatic = 2;` (sv2v/test/lex/begin_keywords.sv) |
| 216 | `begin : ID` | 1 | `end` (verilator/test_regress/t/t_gen_if.v) |
| 217 | `default ) ;` | 1 | `function new(default);` (verilator/test_regress/t/t_class_new_default.v) |
| 218 | `default : ID` | 1 | `casex (1'bx) inside` (verilator/test_regress/t/t_case_inside_bad.v) |
| 219 | `do ) ;` | 1 | `module t(input do);` (verilator/test_regress/t/t_langext_order.v) |
| 220 | `endproperty endmodule` | 1 | `endcase` (verilator/test_regress/t/t_property_case_bad.v) |
| 221 | `int ID ;` | 1 | `interface class Icls;` (verilator/test_regress/t/t_implements_contents_bad.v) |
| 222 | `interface class ID` | 1 | `interface class inte;` (verilator/test_regress/t/t_implements_nested_bad.v) |
| 223 | `logic ; /` | 1 | `reg [63:0] logic; // OK: 'logic' is not a keyword in 1364-2001` (ispras-sv-tests/ieee-1800-2012/22/22.14.01_01.sv) |
| 224 | `medium ( )` | 1 | `module medium();` (Surelog/tests/HierPathModule/dut.sv) |
| 225 | `module` | 1 | `endmodule` (sv-tests/tests/chapter-11/11.3.6--two_assign_in_expr.sv) |
| 226 | `new ; /` | 1 | `c = T::new; // This should fail, T is not a class` (iverilog/ivtest/ivltests/sv_class_new_typed_fail4.v) |
| 227 | `packed { ID` | 1 | `typedef union soft packed {` (verilator/test_regress/t/t_export_packed_struct.v) |
| 228 | `packed { logic` | 1 | `typedef union soft packed {` (verilator/test_regress/t/t_export_packed_struct2.v) |
| 229 | `reg ID ;` | 1 | `inout wire reg p;` (ispras-sv-tests/ieee-1800-2012/06/06.07.01_02.sv) |
| 230 | `return ID ;` | 1 | `return success;` (sv-tests/tests/chapter-18/18.12.1--adding-constraints-to-scope-variables_0.sv) |
| 231 | `tri0 ID ;` | 1 | `wire rst;` (verilator/test_regress/t/t_tri_clocking.v) |
| 232 | `tri1 ID )` | 1 | `inout tri1 z` (verilator/test_regress/t/t_tri_gen.v) |
| 233 | `weak0 , weak1` | 1 | `wire (weak0, weak1) [W-1:0] b = '1;` (verilator/test_regress/t/t_strength_equal_strength.v) |
| 234 | `with ( ID` | 1 | `bins apple = X with (a + b < 257) matches 127;` (ispras-sv-tests/ieee-1800-2012/19/19.06.01.02_01.sv) |
| 235 | `{ / /` | 1 | `enum logic [1:0][1:0] {  // BAD enum type` (verilator/test_regress/t/t_enum_base_bad.v) |
| 236 | `{ ID >` | 1 | `total = q.sum() with (item) { item > 0; };` (verilator/test_regress/t/t_randomize_with_idlist_method_bad.v) |
| 237 | `{ bit [` | 1 | `union soft {` (verilator/test_regress/t/t_union_soft.v) |
| 238 | `| ( NUM` | 1 | `| (32'(RV32E)        <<  4)` (Surelog/tests/CastShift/dut.sv) |
| 239 | `| -> NUM` | 1 | `assert property (@(posedge clk) a [*n] |-> 1);` (verilator/test_regress/t/t_assert_consec_rep_bad.v) |
| 240 | `| ID ;` | 1 | `assign y[0] = !|v;` (verilator/test_regress/t/t_lint_notredop_bad.v) |
| 241 | `|| ID inside` | 1 | `constraint B { w == 5; x inside {1,2} || x inside {4,5}; }` (verilator/test_regress/t/t_randomize_method_constraints.v) |
| 242 | `} ) ;` | 1 | `task tsk(int q[] = {});` (verilator/test_regress/t/t_queue_empty_pin.v) |
| 243 | `} + NUM` | 1 | `i = {} + 1;` (verilator/test_regress/t/t_queue_empty_bad.v) |
| 244 | `} ; if` | 1 | `words = {};` (iverilog/ivtest/ivltests/sv_queue3.v) |
| 245 | `} ID :` | 1 | `}` (verilator/test_regress/t/t_covergroup_cross_opt_unsup.v) |
| 246 | `} endgroup covergroup` | 1 | `option.weight = c;` (Surelog/tests/Covergroup/dut.sv) |
| 247 | `~ ID ==` | 1 | `constraint unary { !(-~c == 'h22); }` (verilator/test_regress/t/t_constraint_operators.v) |

