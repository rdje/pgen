# rejects-valid construct families (SV-CORPUS-GRAD.3.2)

272 `divergence:unexplained_rejects_valid` rows classified into 7 construct families (priority-ordered structural bucketer over the stuck source line; the leaf-cutting map for the `.3` burn-down). Families ranked by cross-suite row count.

| # | family | rows | suite split |
|---|---|---|---|
| 1 | OTHER (per-row triage) | 214 | verilator:89, Surelog:41, ispras-sv-tests:26, sv2v:24, iverilog:12, sv-tests:11, verible:10, slang:1 |
| 2 | constraint/randomize (ch18) | 16 | verilator:15, ispras-sv-tests:1 |
| 3 | interface/modport (ch25) | 13 | verilator:9, sv2v:2, ispras-sv-tests:1, verible:1 |
| 4 | SVA implication/property (ch16) | 11 | ispras-sv-tests:5, verilator:4, Surelog:2 |
| 5 | foreach/array (ch7) | 9 | verilator:5, Surelog:2, iverilog:1, verible:1 |
| 6 | size/type cast N'(...) (ch6/11) | 6 | Surelog:4, iverilog:1, sv2v:1 |
| 7 | coverage bins/cross (ch19) | 3 | verilator:2, ispras-sv-tests:1 |

## Representative stuck lines per family

### OTHER (per-row triage)
- `Surelog`: `reg      bit, rst, clk;`  (tests/1364_2005/dut.v)
- `Surelog`: `let p2(x) = $past(x,,,@(posedge clock));`  (tests/Assertions/dut.sv)
- `Surelog`: `function void uvm_packer::get_packed_bits(ref bit unsigned stream[]);`  (tests/Assignments/dut.sv)
- `Surelog`: `$fatal(1, "slv_aw_select_i is %d: AW has selected a slave that is not defined.\`  (tests/AssumeProp/dut.sv)

### constraint/randomize (ch18)
- `ispras-sv-tests`: `success = std::randomize(a, b) with {b - a > length;};`  (ieee-1800-2012/18/18.12.01_01.sv)
- `verilator`: `constraint unary { !(-~c == 'h22); }`  (test_regress/t/t_constraint_operators.v)
- `verilator`: `if (!randomize() with { addr == a; data == d; }) begin`  (test_regress/t/t_constraint_unsat.v)
- `verilator`: `typedef union soft packed {`  (test_regress/t/t_export_packed_struct.v)

### interface/modport (ch25)
- `ispras-sv-tests`: `sim.queues[Active].push_back('{is_update: 1});`  (ieee-1800-2012/04/04.05_01.sv)
- `sv2v`: `modport client_mp (output .client_req(req[i]));`  (test/core/interface_generate.sv)
- `sv2v`: `$fatal(.x("x"));`  (test/error/severity_task_arg.sv)
- `verible`: `initial $display(nested_class0::handle1::handle2.nested_function());`  (verible/verilog/tools/kythe/testdata/nested_member_access.sv)

### SVA implication/property (ch16)
- `Surelog`: `@($global_clock) a[*1:$] ##1 b);`  (tests/CheckerInst/dut.sv)
- `Surelog`: `bins c1 = ! binsof(a) intersect {[100:200]};// 4 cross products`  (tests/CrossItem/dut.sv)
- `ispras-sv-tests`: `@($global_clock) a[*1:$] ##1 b);`  (ieee-1800-2012/14/14.14_05.sv)
- `ispras-sv-tests`: `trans ##1 start_trans ##1 s ##1 end_trans;`  (ieee-1800-2012/16/16.08_04.sv)

### foreach/array (ch7)
- `Surelog`: `foreach(array[i][j])`  (tests/ForeachSquare/dut.sv)
- `Surelog`: `foreach(data[i][j])`  (tests/SynthForeach/dut.sv)
- `iverilog`: `foreach (a[i]) ;`  (ivtest/ivltests/sf1289.v)
- `verible`: `foreach (y) begin`  (verible/verilog/tools/kythe/testdata/foreach.sv)

### size/type cast N'(...) (ch6/11)
- `Surelog`: `localparam host_base_addr_gp         = (dev_id_width_gp+dev_addr_width_gp)'('h00`  (tests/BlackBePipeInt/dut.sv)
- `Surelog`: `if ($clog2(RomSize)'(RomSize) == 5'd19) begin : gen_cast`  (tests/ClogCast/dut.sv)
- `Surelog`: `localparam logic [OtpByteAddrWidth:0] PartEnd = (OtpByteAddrWidth+1)'(PartInfo[k`  (tests/ParamArraySelect/dut.sv)
- `Surelog`: `parameter logic [2:0] PartInvDefault = 3'({`  (tests/ParamTypespec/dut.sv)

### coverage bins/cross (ch19)
- `ispras-sv-tests`: `bins apple = X with (a + b < 257) matches 127;`  (ieee-1800-2012/19/19.06.01.02_01.sv)
- `verilator`: `bins auto[size_var];`  (test_regress/t/t_covergroup_autobins_bad.v)
- `verilator`: `covergroup cg_bad with function foo(int x);`  (test_regress/t/t_covergroup_with_function_foo_bad.v)
