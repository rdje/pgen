# rejects-valid construct families (SV-CORPUS-GRAD.3.2)

426 `divergence:unexplained_rejects_valid` rows classified into 11 construct families (priority-ordered structural bucketer over the stuck source line; the leaf-cutting map for the `.3` burn-down). Families ranked by cross-suite row count.

| # | family | rows | suite split |
|---|---|---|---|
| 1 | OTHER (per-row triage) | 245 | verilator:103, Surelog:44, ispras-sv-tests:29, sv2v:25, iverilog:19, sv-tests:15, verible:10 |
| 2 | SVA implication/property (ch16) | 35 | ispras-sv-tests:22, verilator:10, Surelog:3 |
| 3 | interface/modport (ch25) | 25 | verilator:12, ispras-sv-tests:7, sv-tests:3, sv2v:2, verible:1 |
| 4 | constraint/randomize (ch18) | 23 | verilator:19, ispras-sv-tests:2, sv-tests:2 |
| 5 | compiler directives (ch22) | 22 | iverilog:18, verilator:2, sv-tests:1, sv2v:1 |
| 6 | drive/charge strength (ch28) | 21 | verilator:10, ispras-sv-tests:5, iverilog:2, sv2v:2, Surelog:1, slang:1 |
| 7 | named block/label (ch9/27) | 20 | verilator:8, Surelog:6, sv2v:5, verible:1 |
| 8 | size/type cast N'(...) (ch6/11) | 13 | Surelog:5, sv2v:4, verilator:2, ispras-sv-tests:1, iverilog:1 |
| 9 | enum base range (ch6) | 9 | verilator:8, sv2v:1 |
| 10 | foreach/array (ch7) | 9 | verilator:5, Surelog:2, iverilog:1, verible:1 |
| 11 | coverage bins/cross (ch19) | 4 | ispras-sv-tests:2, verilator:2 |

## Representative stuck lines per family

### OTHER (per-row triage)
- `Surelog`: `reg	     bit, rst, clk;`  (tests/1364_2005/dut.v)
- `Surelog`: `let p2(x) = $past(x,,,@(posedge clock));`  (tests/Assertions/dut.sv)
- `Surelog`: `function void uvm_packer::get_packed_bits(ref bit unsigned stream[]);`  (tests/Assignments/dut.sv)
- `Surelog`: `$fatal(1, "slv_aw_select_i is %d: AW has selected a slave that is not defined.\`  (tests/AssumeProp/dut.sv)

### SVA implication/property (ch16)
- `Surelog`: `bins c1 = ! binsof(a) intersect {[100:200]};// 4 cross products`  (tests/CrossItem/dut.sv)
- `Surelog`: `@(posedge clk) a ##[r1:r2] b ##2 1'b1 |=> d;`  (tests/PoundDelay/dut.sv)
- `Surelog`: `assert property (up ##[1:$] up |=> cnt == $past(cnt, 2) + 8'd 2);`  (tests/UnitSVA/dut.sv)
- `ispras-sv-tests`: `@(posedge clk) a ##[r1:r2] b ##1 c |=> d;`  (ieee-1800-2012/06/06.20.02.01_01.sv)

### interface/modport (ch25)
- `ispras-sv-tests`: `sim.queues[Active].push_back('{is_update: 1});`  (ieee-1800-2012/04/04.05_01.sv)
- `ispras-sv-tests`: `q = q[1:$];                      // void'(q.pop_front()) or q.delete(0)`  (ieee-1800-2012/07/07.10.04_01.sv)
- `ispras-sv-tests`: `instance top use #(.WIDTH(32));`  (ieee-1800-2012/33/33.04.03_02.sv)
- `ispras-sv-tests`: `instance top.a1 use #(.W(top.S));`  (ieee-1800-2012/33/33.04.03_03.sv)

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

### coverage bins/cross (ch19)
- `ispras-sv-tests`: `bins apple = X with (a + b < 257) matches 127;`  (ieee-1800-2012/19/19.06.01.02_01.sv)
- `ispras-sv-tests`: `bins one = '{'{1, 2}, '{3, 4}, '{5, 6}};`  (ieee-1800-2012/19/19.06.01.04_01.sv)
- `verilator`: `bins auto[size_var];`  (test_regress/t/t_covergroup_autobins_bad.v)
- `verilator`: `covergroup cg_bad with function foo(int x);`  (test_regress/t/t_covergroup_with_function_foo_bad.v)
