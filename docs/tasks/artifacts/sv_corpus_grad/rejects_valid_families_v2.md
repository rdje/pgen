# rejects-valid construct families (SV-CORPUS-GRAD.3.2)

543 `divergence:unexplained_rejects_valid` rows classified into 13 construct families (priority-ordered structural bucketer over the stuck source line; the leaf-cutting map for the `.3` burn-down). Families ranked by cross-suite row count.

| # | family | rows | suite split |
|---|---|---|---|
| 1 | OTHER (per-row triage) | 243 | verilator:104, Surelog:41, ispras-sv-tests:28, sv2v:26, iverilog:19, sv-tests:15, verible:10 |
| 2 | SVA implication/property (ch16) | 101 | ispras-sv-tests:57, verilator:31, Surelog:9, sv-tests:4 |
| 3 | interface/modport (ch25) | 50 | verilator:24, ispras-sv-tests:18, sv-tests:3, sv2v:3, Surelog:1, verible:1 |
| 4 | constraint/randomize (ch18) | 23 | verilator:19, ispras-sv-tests:2, sv-tests:2 |
| 5 | compiler directives (ch22) | 22 | iverilog:18, verilator:2, sv-tests:1, sv2v:1 |
| 6 | number literal spaced-based (ch5) | 22 | sv-tests:13, Surelog:5, ispras-sv-tests:2, verilator:2 |
| 7 | drive/charge strength (ch28) | 21 | verilator:10, ispras-sv-tests:5, iverilog:2, sv2v:2, Surelog:1, slang:1 |
| 8 | named block/label (ch9/27) | 20 | verilator:8, Surelog:6, sv2v:5, verible:1 |
| 9 | size/type cast N'(...) (ch6/11) | 13 | Surelog:5, sv2v:4, verilator:2, ispras-sv-tests:1, iverilog:1 |
| 10 | enum base range (ch6) | 9 | verilator:8, sv2v:1 |
| 11 | foreach/array (ch7) | 9 | verilator:5, Surelog:2, iverilog:1, verible:1 |
| 12 | unique0 (ch12) | 6 | verilator:3, ispras-sv-tests:1, sv-tests:1, sv2v:1 |
| 13 | coverage bins/cross (ch19) | 4 | ispras-sv-tests:2, verilator:2 |

## Representative stuck lines per family

### OTHER (per-row triage)
- `Surelog`: `reg	     bit, rst, clk;`  (tests/1364_2005/dut.v)
- `Surelog`: `function void uvm_packer::get_packed_bits(ref bit unsigned stream[]);`  (tests/Assignments/dut.sv)
- `Surelog`: `priv_lvl_e mpp;`  (tests/BadScope/dut.sv)
- `Surelog`: `extern function uvm_mem_mam_cfg reconfigure(uvm_mem_mam_cfg cfg = null);`  (tests/BitSelectHier/dut.sv)

### SVA implication/property (ch16)
- `Surelog`: `assert property ( @(posedge Clock)  strap_en_i |=> ##0 !strap_en_i [*]);`  (tests/Assert/dut.sv)
- `Surelog`: `(logic'(x) === logic'(y)) |=> ! (logic'(x) === logic'(y));`  (tests/Assertions/dut.sv)
- `Surelog`: `aw_select: assume property( @(posedge clk_i) (slv_req_i.aw_valid |->`  (tests/AssumeProp/dut.sv)
- `Surelog`: `@($global_clock) req |=> ack;`  (tests/CheckerInst/dut.sv)

### interface/modport (ch25)
- `Surelog`: `modport A (output .P(r[3:0]), input .Q(x), R);`  (tests/InterfaceModExp/dut.sv)
- `ispras-sv-tests`: `sim.queues[Active].push_back('{is_update: 1});`  (ieee-1800-2012/04/04.05_01.sv)
- `ispras-sv-tests`: `q = q[1:$];                      // void'(q.pop_front()) or q.delete(0)`  (ieee-1800-2012/07/07.10.04_01.sv)
- `ispras-sv-tests`: `modport slave(input req, addr, mode, start, clk,`  (ieee-1800-2012/25/25.05.01_01.sv)

### constraint/randomize (ch18)
- `ispras-sv-tests`: `x dist {100 := 1, 200 := 2, 300 := 5};`  (ieee-1800-2012/18/18.05.04_01.sv)
- `ispras-sv-tests`: `success = std::randomize(a, b) with {b - a > length;};`  (ieee-1800-2012/18/18.12.01_01.sv)
- `sv-tests`: `constraint c { b dist {3 := 1, 10 := 2}; }`  (tests/chapter-18/18.5.4--distribution_0.sv)
- `sv-tests`: `constraint c { b dist {3 := 0, 10 := 5}; }`  (tests/chapter-18/18.5.4--distribution_2.sv)

### compiler directives (ch22)
- `iverilog`: `$display("FAILED(%0d). Expected 1, got %b", `__LINE__, a);`  (ivtest/ivltests/br_gh1321.v)
- `iverilog`: `$display(`__FILE__);`  (ivtest/ivltests/fileline.v)
- `iverilog`: `check_size(0, `__FILE__, `__LINE__);`  (ivtest/ivltests/sv_queue_parray.v)
- `iverilog`: `check_size(0, `__FILE__, `__LINE__);`  (ivtest/ivltests/sv_queue_parray_bounded.v)

### number literal spaced-based (ch5)
- `Surelog`: `parameter logic [31:0]     IdcodeValue = 32'h 0000_0001`  (tests/BindStmt2/dut.sv)
- `Surelog`: `parameter logic [13:0] OTP_CTRL_INTR_STATE_OFFSET = 14'h 0;`  (tests/BlackParrotMuteErrors/dut.sv)
- `Surelog`: `Prefix: 96'(96'h 4c52_5443_5f4d_4f52_4001_0001)`  (tests/LargeHexCast/dut.sv)
- `Surelog`: `AccessAck     = 3'h 0,`  (tests/PackStructVar/dut.sv)

### drive/charge strength (ch28)
- `Surelog`: `assign (highz0, weak1)    inout_i1 = pu;`  (tests/MultContAssign/dut.sv)
- `ispras-sv-tests`: `wire (strong1, pull0) mynet = enable;`  (ieee-1800-2012/10/10.03.01_01.sv)
- `ispras-sv-tests`: `assign (strong1, pull0) mynet = enable;`  (ieee-1800-2012/10/10.03.02_01.sv)
- `ispras-sv-tests`: `nor (highz1, strong0) n1(out1, in1, in2);`  (ieee-1800-2012/28/28.03.02_01.sv)

### named block/label (ch9/27)
- `Surelog`: `generate begin:foo`  (tests/DefParamIndex/dut.sv)
- `Surelog`: `begin : A`  (tests/GenerateBlock/dut.sv)
- `Surelog`: `begin : A`  (tests/GenerateRegion/dut.sv)
- `Surelog`: `begin : foo`  (tests/ImplicitGenBlock/dut.sv)

### size/type cast N'(...) (ch6/11)
- `Surelog`: `localparam host_base_addr_gp         = (dev_id_width_gp+dev_addr_width_gp)'('h00`  (tests/BlackBePipeInt/dut.sv)
- `Surelog`: `| (32'(RV32E)        <<  4)`  (tests/CastShift/dut.sv)
- `Surelog`: `if (addr_q < $clog2(RomSize)'(RomSize)) begin`  (tests/ClogCast/dut.sv)
- `Surelog`: `localparam logic [OtpByteAddrWidth:0] PartEnd = (OtpByteAddrWidth+1)'(PartInfo[k`  (tests/ParamArraySelect/dut.sv)

### enum base range (ch6)
- `sv2v`: `typedef enum [5:0] {`  (test/core/enum_scope.sv)
- `verilator`: `typedef enum [15:0] {`  (test_regress/t/t_cast.v)
- `verilator`: `typedef enum [2:0] {`  (test_regress/t/t_debug_emitv.v)
- `verilator`: `typedef enum [2:0] { ONES=~0 } three_t;`  (test_regress/t/t_enum.v)

### foreach/array (ch7)
- `Surelog`: `foreach(array[i][j])`  (tests/ForeachSquare/dut.sv)
- `Surelog`: `foreach(data[i][j])`  (tests/SynthForeach/dut.sv)
- `iverilog`: `foreach (a[i]) ;`  (ivtest/ivltests/sf1289.v)
- `verible`: `foreach (y) begin`  (verible/verilog/tools/kythe/testdata/foreach.sv)

### unique0 (ch12)
- `ispras-sv-tests`: `unique0 case(a) // values 3, 5, 6, 7 do not cause a violation report`  (ieee-1800-2012/12/12.05.03_01.sv)
- `sv-tests`: `unique0 if(a == 0) b = 1;`  (tests/chapter-12/12.4.2--unique0_if.sv)
- `sv2v`: `unique0 case (select)`  (test/core/case_violation_checks.sv)
- `verilator`: `unique0 case (i)`  (test_regress/t/t_langext_2.v)

### coverage bins/cross (ch19)
- `ispras-sv-tests`: `bins apple = X with (a + b < 257) matches 127;`  (ieee-1800-2012/19/19.06.01.02_01.sv)
- `ispras-sv-tests`: `bins one = '{'{1, 2}, '{3, 4}, '{5, 6}};`  (ieee-1800-2012/19/19.06.01.04_01.sv)
- `verilator`: `bins auto[size_var];`  (test_regress/t/t_covergroup_autobins_bad.v)
- `verilator`: `covergroup cg_bad with function foo(int x);`  (test_regress/t/t_covergroup_with_function_foo_bad.v)
