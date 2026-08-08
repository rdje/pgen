# `vhdl` stuck-point clusters (SV-CORPUS-GRAD.3.0 engine; family lane CORPUS-GRAD-ALL.2.1)

9385 rows probed from the `fail` rows of `results.tsv`; 842 distinct 3-token stuck signatures. Keyed on `furthest_position` (the DEEP locus), family profile `vhdl`, case-folded.

> ⚠️ **RAW-FAIL lane — a cluster SIZES a candidate class, it does not adjudicate one.** Some corpus files are intentionally invalid, so a fail can be the CORRECT outcome. Expected verdicts come from the LRM / suite metadata, never from what the parser does today.

| # | signature | rows | example (stuck line) |
|---|---|---|---|
| 1 | `ID ; ID` | 431 | `x   :   bit_vector ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_055a.vhd) |
| 2 | `for ID :` | 359 | `for TestCtrl_1 : TestCtrl` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_AlertLogIDManager.vhd) |
| 3 | `range NUM to` | 357 | `type FREQ is range 0 to integer'high units` (stimuli/vhdl/subs/PoC/src/common/physical.vhdl) |
| 4 | `ID : ID` | 295 | `impure function to_string(variable value : value_mirror) return string ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_041.vhd) |
| 5 | `ID ( NUM` | 258 | `return new string(1 to 1000000) ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_030.vhd) |
| 6 | `ID ; begin` | 252 | `end proc;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/emptyquote.vhdl) |
| 7 | `. NUM ;` | 245 | `signal TxIdleProbability : real := 0.05 ; -- 5 %` (stimuli/vhdl/subs/OsvvmLibraries/UART/testbench_multiple_uarts/TbUart_MultipleProcess_1.vhd) |
| 8 | `file of ID` | 219 | `type T_PICFILE is file of character;` (stimuli/vhdl/subs/OsvvmLibraries/VideoBus_LouisAdriaens/src/bmp_pack.vhd) |
| 9 | `ID : in` | 179 | `constant Address         : In  std_logic_vector ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/common/src/Axi4ModelPkg.vhd) |
| 10 | `( ID )` | 177 | `alias  WriteAddressDelayCov  is ArrDelayCovID(WRITE_ADDRESS_ID) ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Axi4Manager_a.vhd) |
| 11 | `attribute ID :` | 174 | `attribute KEEP : boolean;` (stimuli/vhdl/subs/PoC/src/arith/arith_trng.vhdl) |
| 12 | `ID ( ID` | 168 | `function operation (value: integer) return integer);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_function_generic_in_package.vhd) |
| 13 | `ID ) ;` | 163 | `type my_type);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_type_generics_in_packages.vhd) |
| 14 | `access ID ;` | 149 | `type CALL_PATH_VECTOR_PTR is access CALL_PATH_VECTOR ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/deprecated/FileLinePathPkg_c.vhd) |
| 15 | `( NUM downto` | 133 | `variable slvBurstVector : slv_vector(1 to 5)(31 downto 0) ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/TestCases/TbStream_SendGetDemo1.vhd) |
| 16 | `, ID (` | 109 | `Address(AXI_ADDR_WIDTH-1 downto 0),` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/testbench/TbAxi4.vhd) |
| 17 | `ID ; function` | 105 | `type mytype;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_entity_generic_type.vhd) |
| 18 | `downto NUM =>` | 101 | `constant DATA_ZERO : std_logic_vector := (DATA_WIDTH - 1 downto 0 => '0') ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_DemoErrorMemoryReadWrite1.vhd) |
| 19 | `type ID is` | 100 | `type   tState is (Idle, Command);` (stimuli/vhdl/subs/PoC/src/comm/remote/remote_terminal_control.vhdl) |
| 20 | `ID ; end` | 98 | `end get_c;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0108/pkg.vhdl) |
| 21 | `. NUM )` | 91 | `signal b : b_t := (X"42", (others => '0'), (others => '1'), 4.0);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_075.vhd) |
| 22 | `constant ID :` | 90 | `constant AXI_ADDR_WIDTH      : integer := AxiBus.WriteAddress.Addr'length ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Vti/deprecated/Axi4ManagerVti_e.vhd) |
| 23 | `shared variable ID` | 86 | `shared variable OperationFifo  : osvvm.ScoreboardPkg_slv.ScoreboardPType ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_RandomReadWrite.vhd) |
| 24 | `ID , ID` | 84 | `set(ManagerRec(1).params, Axi4OptionsType'pos(WRITE_RESPONSE_READY_BEFORE_VALID),  BoolExpected) ;` (stimuli/vhdl/subs/OsvvmLibraries/Common/TbAddressBusArray/Axi/TbAxi4_SetModelOptions1.vhd) |
| 25 | `alias ID :` | 84 | `alias RxID    : std_logic_vector(ID_LEN-1 downto 0) is RxParam(PARAM_LEN-1 downto PARAM_LEN-ID_LEN) ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/TestCases/TbStream_AxiBurstAsyncNoLast1.vhd) |
| 26 | `ID : out` | 83 | `signal   pins   : out std_logic_vector;` (stimuli/vhdl/subs/UVVM/bitvis_vip_gpio/tb/maintenance_tb/gpio_vvc_tb.vhd) |
| 27 | `' ID '` | 78 | `type state2_t is ('a', 'b', 'c', 'd');` (stimuli/vhdl/subs/ghdl/testsuite/gna/fst01/enum1.vhdl) |
| 28 | `ID : inout` | 78 | `procedure write(variable l : inout line ; v : integer_vector) is` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_007.vhd) |
| 29 | `ID ; --` | 75 | `subtype Axi4OptionsType is resolved_max Axi4UnresolvedOptionsType ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/common/src/Axi4OptionsPkg.vhd) |
| 30 | `. NUM ,` | 70 | `v_rand_real.add_range_real(1.0, 5.5);` (stimuli/vhdl/subs/UVVM/uvvm_util/tb/maintenance_tb/questa_extension_tb.vhd) |
| 31 | `protected procedure ID` | 70 | `type test_t is protected` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_002.vhd) |
| 32 | `' NUM '` | 68 | `type tri_state_logic is ('0', '1', 'Z');` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_11_fg_11_01.vhd) |
| 33 | `, ID range` | 64 | `type real_natural2D_t is array (natural range <>, natural range <>) of real;` (stimuli/vhdl/subs/Compliance-Tests/cosim/dpi-ffi/ghdl-vffi/test/valuesFromVhdlToC.vhd) |
| 34 | `<NO-POSITION>` | 63 | `Error: failed to read input file 'stimuli/vhdl/subs/PoC/src/mem/lut/lut_Sine.vhdl'

Caused by:
    stream did not contai` (stimuli/vhdl/subs/PoC/src/mem/lut/lut_Sine.vhdl) |
| 35 | `new ID .` | 57 | `package my_fixed_pkg is new ieee.fixed_generic_pkg;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_fixed_generic_pkg.vhd) |
| 36 | `: for ID` | 56 | `loop_add_m: for I in 0 to A'length-1 loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1684/pkg.vhdl) |
| 37 | `) of ID` | 54 | `type arr is array (natural range 0 to 7) of bit;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug072/t.vhdl) |
| 38 | `ID ; type` | 54 | `end record rd_req_t ;` (stimuli/vhdl/subs/OsvvmLibraries/CoSimPCIe/testbench/TbPcie/Tb_Pcie.vhd) |
| 39 | `; -- ID` | 53 | `)  ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/testbenchVti/TbStream.vhd) |
| 40 | `ID ; constant` | 53 | `end record ModelParametersIDType ;` (stimuli/vhdl/subs/OsvvmLibraries/Common/src/ModelParametersSingletonPkg.vhd) |
| 41 | `ID ; signal` | 53 | `constant CLK_FREQ : FREQ := 50 MHz;` (stimuli/vhdl/subs/PoC/tb/dstruct/dstruct_deque_tb.vhdl) |
| 42 | `port ( ID` | 52 | `port (m : in matrix_t(0 to 1));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3046/b.vhdl) |
| 43 | `ID is ID` | 48 | `subtype T_slv_d is std_logic_vector(D_BITS - 1 downto 0);` (stimuli/vhdl/subs/PoC/src/fifo/fifo_stage.vhdl) |
| 44 | `ID ; procedure` | 44 | `end proc;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/proctarg.vhdl) |
| 45 | `; constant ID` | 43 | `constant AXI_ADDR_WIDTH : integer := ManagerRec.Address'length ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4Lite/testbench/TestCtrl_e.vhd) |
| 46 | `> ID ,` | 41 | `long_port(15 downto 0) => not_so_long_vector,` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_001.vhd) |
| 47 | `alias ID is` | 41 | `alias ManagerRec     is <<signal ^.Manager_1.TransRec : AddressBusRecType>> ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/testbenchVti/TestCtrl_e.vhd) |
| 48 | `range - NUM` | 41 | `type frequency is range -2147483647 to 2147483647 units KHz;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug20255/test.vhd) |
| 49 | `. NUM to` | 40 | `valid_low_multiple_random_prob : real range 0.0 to 1.0; -- Probability of how often valid shall be deasserted when using` (stimuli/vhdl/subs/UVVM/bitvis_vip_avalon_st/src/avalon_st_bfm_pkg.vhd) |
| 50 | `ID ID :` | 39 | `constant DATA_BYTES : integer := DATA_WIDTH/8 ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/testbench/TestCtrl_e.vhd) |
| 51 | `( ID (` | 37 | `signal ManagerRec, SubordinateRec : AddressBusRecArrayType(1 to 2)(` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/testbench_MultipleMemory/TbAxi4_MultipleMemory.vhd) |
| 52 | `( ID -` | 37 | `ParIn                              : in  LinkType(0 to NUMOFLANES-1)(ENCODEDWIDTH-1 downto 0) ;` (stimuli/vhdl/subs/OsvvmLibraries/CoSimPCIe/src/PcieComponentPkg.vhd) |
| 53 | `: ID is` | 37 | `alias THIS_VVCT       : t_vvc_target_record is AVALON_MM_VVCT;` (stimuli/vhdl/subs/UVVM/bitvis_vip_avalon_mm/src/vvc_methods_pkg.vhd) |
| 54 | `( " NUM` | 36 | `check_equal(value, std_logic_vector'("001111"));` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_bit_string_literals.vhd) |
| 55 | `; end package` | 36 | `alias Axi4Stream_SenderView is Axi4Stream_TransmitterView;` (stimuli/vhdl/subs/Interfaces/AMBA/AXI/v4/AXI4Stream.vhdl) |
| 56 | `; alias ID` | 35 | `alias LVDS_Interface is Differatial_Interface;` (stimuli/vhdl/subs/Interfaces/IO/Common.vhdl) |
| 57 | `ID port map` | 34 | `dut: component write` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue542/wrapper.vhd) |
| 58 | `to NUM =>` | 34 | `adder  := ("0" & unsigned(not rqst)) + (1 to 1 => tin);` (stimuli/vhdl/subs/PoC/src/arith/arith_firstone.vhdl) |
| 59 | `( ID :` | 33 | `generic (c : natural);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug059/tb.vhdl) |
| 60 | `( " ID` | 32 | `constant  COVERAGE_DEFAULT_WEIGHT_MODE  : string := IfElse(OSVVM_SETTINGS_REVISION >= "2024", string'("REMAIN"), "AT_LEA` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/OsvvmSettingsPkg_default.vhd) |
| 61 | `( NUM to` | 32 | `variable v_range_uns_vec : t_range_uns_vec(0 to 0)(0 to 1)(127 downto 0);` (stimuli/vhdl/subs/UVVM/uvvm_util/tb/maintenance_tb/rand_tb.vhd) |
| 62 | `; type ID` | 31 | `type MessageStructType ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/MessageListPkg.vhd) |
| 63 | `<EOF>` | 31 | `--  'VAL parameter is any integer type (including universal integer)` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug09/univ2.vhdl) |
| 64 | `begin end entity` | 31 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug18810/BENCH_OISC_SUBLEQ.vhd) |
| 65 | `( ID ,` | 30 | `log(TbID, "Received: " & to_string(UartStimType'(Data, Error))) ;` (stimuli/vhdl/subs/OsvvmLibraries/UART/GHDL_Debug/TbUart_GhdlDebug_3.vhd) |
| 66 | `( ID =>` | 30 | `AffirmIf(GetAlertCount = AlertCountType'(FAILURE => 1, ERROR => 0, WARNING => 0), "Expecting: (FAILURE => 1, ERROR => 0,` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_MultipleDriversManager.vhd) |
| 67 | `function ID (` | 30 | `function EQ_BUT_NOT_META(l, r : std_logic) return boolean is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug04/test.vhdl) |
| 68 | `( NUM )` | 29 | `wait on TxStim(1)'transaction, TestActive ;` (stimuli/vhdl/subs/OsvvmLibraries/UART/testbench/TbUart_UartX1_1.vhd) |
| 69 | `for all :` | 29 | `for all: ADDER` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue123/adder.vhdl) |
| 70 | `; ID <=` | 28 | `cpu_rstb  <= (not rst) and (not int_write) and int_got when rising_edge(clk);` (stimuli/vhdl/subs/PoC/src/cache/cache_mem.vhdl) |
| 71 | `> ' NUM` | 28 | `x <= (x'range => '0');` (stimuli/vhdl/subs/PoC/tb/arith/same/arith_same_Simple.vhdl) |
| 72 | `' subtype ;` | 27 | `signal gIntReqDelayed : gIntReq'subtype ;` (stimuli/vhdl/subs/OsvvmLibraries/CoSim/src/CoSimInterruptHandler.vhd) |
| 73 | `, NUM to` | 27 | `type rom_t is array(0 to 3, 0 to 3) of std_logic_vector(3 downto 0);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1220/top.vhdl) |
| 74 | `protected impure function` | 27 | `type test_t is protected` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_004.vhd) |
| 75 | `signal ID :` | 27 | `signal TransRec : StreamRecType(DataToModel(0 downto 0), DataFromModel(0 downto 0), ParamToModel(1 to 0), ParamFromModel` (stimuli/vhdl/subs/OsvvmLibraries/Common/src/InterruptGeneratorBitVti.vhd) |
| 76 | `" + "` | 26 | `function "+" (a, b: trit) return trit;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2536/bug.vhdl) |
| 77 | `ID : process` | 26 | `P_CLKAGEN : process` (stimuli/vhdl/subs/OsvvmLibraries/CoSimPCIe/src/clkmux.vhd) |
| 78 | `ID select ID` | 26 | `with fwd_r select q <=` (stimuli/vhdl/subs/PoC/src/mem/ocram/ocram_sdp_wf.vhdl) |
| 79 | `attribute ID of` | 26 | `attribute foreign of VInit : procedure is "VHPI VProc.so; VInit" ;` (stimuli/vhdl/subs/OsvvmLibraries/CoSim/src/OsvvmVprocAldecPkg.vhd) |
| 80 | `( ID "` | 25 | `Push(UartScoreboard, UartStimType'(X"50", UARTTB_NO_ERROR) ) ;` (stimuli/vhdl/subs/OsvvmLibraries/UART/testbench/TbUart_Scoreboard1.vhd) |
| 81 | `range NUM .` | 25 | `type frequency is range 0.0 to real'high;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2156/timing_pkg.vhdl) |
| 82 | `( ID '` | 24 | `variable QQ : T_SLVV(Z'range)(A'range);` (stimuli/vhdl/subs/PoC/tb/arith/div/arith_div_Simple.vhdl) |
| 83 | `> ID (` | 24 | `v := (v(v'left-1 downto 0) & '0') xor (to_stdlogicvector(GN) and (GN'range => v(v'left)));` (stimuli/vhdl/subs/PoC/src/comm/comm_scramble.vhdl) |
| 84 | `default ID is` | 24 | `default clock is rising_edge(clk);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_psl.vhd) |
| 85 | `view ID of` | 24 | `AxiBus      : view Axi4ManagerView of Axi4RecType ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Axi4ComponentPkg.vhd) |
| 86 | `( ' NUM` | 23 | `assert to_01(v0) = std_ulogic'('0') severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper03/tox01.vhdl) |
| 87 | `) ; begin` | 22 | `variable result : t_rec(addr(bits - 1 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue191/repro.vhdl) |
| 88 | `<> ) )` | 22 | `generic (package pkg is new work.gen1 generic map (<>));` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug055/gen2.vhdl) |
| 89 | `ID ID ,` | 22 | `terminal p_in1, p_in2, p_out : electrical;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/AMS_CS1_Mixed_Sig/tb_2in_switch.vhd) |
| 90 | `generic ( ID` | 22 | `generic (c : natural);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug057/tb2.vhdl) |
| 91 | `; variable ID` | 21 | `file f_in : text;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1145/tb.vhdl) |
| 92 | `ID ' ID` | 21 | `port(a:std'u);end ghdlcrash;architecture h of g is--` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/attr1.vhdl) |
| 93 | `ID ; variable` | 21 | `end pulse;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/forloop.vhdl) |
| 94 | `" = "` | 20 | `alias   Match is "=" [ActualType, ExpectedType return boolean] ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/deprecated/ScoreboardPkg_IntV_c.vhd) |
| 95 | `: ID (` | 20 | `alias DNA_VALUE : bit_vector(SIM_DNA_VALUE'length - 1 downto 0) is SIM_DNA_VALUE;` (stimuli/vhdl/subs/PoC/src/xil/xil_DNAPort.vhdl) |
| 96 | `; ID .` | 20 | `);` (stimuli/vhdl/subs/PoC/tb/arith/addw/arith_addw_TestHarness.vhdl) |
| 97 | `ID ; subtype` | 19 | `subtype  std_logic_max is resolved_max std_ulogic ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/ResolutionPkg.vhd) |
| 98 | `generic ( type` | 19 | `generic (` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_049.vhd) |
| 99 | `' ID ;` | 18 | `signal thing : thing_t'base ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_018.vhd) |
| 100 | `procedure ID (` | 18 | `PROCEDURE addup (i1,i2,i3:IN INTEGER;add:IN BOOLEAN;VARIABLE i4:OUT INTEGER) IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc149.vhd) |
| 101 | `range ID '` | 18 | `type ANGLE is range INTEGER'range units` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug036/phys.vhdl) |
| 102 | `; -- =` | 17 | `alias t_operation is work.transaction_pkg.t_operation;` (stimuli/vhdl/subs/UVVM/bitvis_vip_avalon_mm/src/vvc_cmd_pkg.vhd) |
| 103 | `ID ; assert` | 17 | `return_exp_check;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1630.vhd) |
| 104 | `" NUM "` | 16 | `Push(WriteBurstFifo, X"00FF_UUUU") ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_AxSizeManagerMemory2.vhd) |
| 105 | `. ID ,` | 16 | `AddBins(DelayCovID(WRITE_ADDRESS_ID).BurstLengthCov, GenBin(7)) ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_AxiManagerRandomTiming1.vhd) |
| 106 | `pure function ID` | 16 | `pure function fOISC_SUBLEQ_DW_LATENCY (` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug18810/OISC_SUBLEQ.vhd) |
| 107 | `view ID ;` | 16 | `WriteAddress   : view Axi4_Address_ManagerView;` (stimuli/vhdl/subs/Interfaces/AMBA/AXI/v4/AXI4.vhdl) |
| 108 | `. NUM /` | 15 | `constant T_RAS     : integer := integer(ceil(42.0/CLK_PERIOD));` (stimuli/vhdl/subs/PoC/src/mem/sdram/sdram_ctrl_de0.vhdl) |
| 109 | `; end ID` | 15 | `say_hello;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0114/hello.vhdl) |
| 110 | `ID ; component` | 15 | `end record stats_type;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_14_fg_14_09.vhd) |
| 111 | `ID ; view` | 15 | `end record axis_rec_type;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3172/axis_pkg.vhdl) |
| 112 | `ID generic map` | 15 | `mem : component altsyncram` (stimuli/vhdl/subs/PoC/src/mem/ocram/altera/ocram_sp_altera.vhdl) |
| 113 | `package ID is` | 15 | `package pkg1 is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug052/tb.vhdl) |
| 114 | `, ID )` | 14 | `type branch is array (remains, bit) of remains;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug084/func_test1.vhdl) |
| 115 | `. NUM *` | 14 | `signal PRUEBA: std_logic_vector(SIZE-1 downto 0) := std_logic_vector(to_unsigned(integer(0.25*2**SIZE+0.5), SIZE));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1820/cordic.vhdl) |
| 116 | `: loop --` | 14 | `OperationLoop : loop` (stimuli/vhdl/subs/OsvvmLibraries/CoSim/testbench/TestCases/TbAb_CoSim.vhd) |
| 117 | `<= ID +` | 14 | `clkgen: clk <= clk + 1 after 1 ns when clk < 7;` (stimuli/vhdl/subs/nvc/test/regress/psl13.vhd) |
| 118 | `ID ' (` | 14 | `l1 := new string'("Hello world");` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug017/call9.vhdl) |
| 119 | `) ; signal` | 13 | `signal in_abc  : abc.Rec_t(Data(7 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2562/repro1.vhdl) |
| 120 | `<< signal ID` | 13 | `<<signal i_test.sig : std_logic >> <= force '0';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2219/ent.vhdl) |
| 121 | `<= ' NUM` | 13 | `reset : reset_s <= '0',` (stimuli/vhdl/subs/ghdl/testsuite/gna/deb585750/585750_deb.vhd) |
| 122 | `<= not ID` | 13 | `p_clk : clk <= not clk after C_CLK_PERIOD;` (stimuli/vhdl/subs/UVVM/external_vip_apb/tb/maintenance_tb/apb_bfm_tb.vhd) |
| 123 | `> ID )` | 13 | `return ((d and (q'range => en)) or (q and not (q'range => en))) and not (q'range => rst);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug019/PoC/src/common/components.vhdl) |
| 124 | `begin ID :` | 13 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1588/tent.vhdl) |
| 125 | `; end architecture` | 12 | `(o, d) <= d & i when rising_edge(clk);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1914/delayline1d.vhdl) |
| 126 | `> NUM )` | 12 | `if(s_wrcnt = 3 and s_index => 5) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1664/ppm.vhdl) |
| 127 | `ID bus ;` | 12 | `signal interrupt_request : pulled_up bit bus;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_ch_16_01.vhd) |
| 128 | `[ ID ,` | 12 | `alias   Match is Match [ActualType, ExpectedType return boolean] ;` (stimuli/vhdl/subs/OsvvmLibraries/UART/src/deprecated/ScoreboardPkg_Uart_c.vhd) |
| 129 | `vunit ID (` | 12 | `vunit const_test_vunit (const_test(rtl))` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1724/const_test_vunit.vhdl) |
| 130 | `( NUM =>` | 11 | `stb => std_logic_vector'(0 => s_stb)   -- CRASH` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3199/test_qual.vhdl) |
| 131 | `; signal ID` | 11 | `end component;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1935/sm_tb.vhdl) |
| 132 | `<< signal .` | 11 | `tb_sig <= << signal .tb_external_names.dut_inst.local_sig : bit >>;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_external_names.vhd) |
| 133 | `ID ; package` | 11 | `function get return natural is begin return 0;end;end gen0;package n is generic(package g is new n generic map(<>));func` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/func1.vhdl) |
| 134 | `ID <= ID` | 11 | `data_o <= extend(data_i);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2652/func.vhdl) |
| 135 | `ID is array` | 11 | `generic (type VectorType is array (type is range <>) of type is private ) ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/DynamicVectorGenericPkg.vhd) |
| 136 | `" < "` | 10 | `function "<" (a, b : natural) return bit;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1979/repro.vhdl) |
| 137 | `( ID =` | 10 | `reg_read_selector : block (reg_sel = '1' and read = '1') is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_fg_16_07.vhd) |
| 138 | `. ID (` | 10 | `Data(15 downto 0) := SafeResize(ManagerRec(1).DataFromModel(15 downto 0), 16) ;` (stimuli/vhdl/subs/OsvvmLibraries/Common/TbAddressBusArray/AddressBusMitTestCases/TbAxi4_ReadPoll1.vhd) |
| 139 | `. NUM -` | 10 | `function fract(F : real; maxDenominator : natural := 1000; maxError : real := 1.0E-6) return T_FRACTIONAL;` (stimuli/vhdl/subs/PoC/src/common/math.vhdl) |
| 140 | `; -- --` | 10 | `)  ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/testbench_GenericSignals/TbAxi4.vhd) |
| 141 | `ID ; impure` | 10 | `type tyi;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2702/top.vhdl) |
| 142 | `[ ID return` | 10 | `alias to_Axi4RespType is from_Axi4RespEnumType[Axi4RespEnumType return Axi4RespType] ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/common/src/Axi4InterfaceCommonPkg.vhd) |
| 143 | `begin -- ID` | 10 | `begin` (stimuli/vhdl/subs/UVVM/bitvis_vip_avalon_mm/src/avalon_mm_vvc.vhd) |
| 144 | `protected end protected` | 10 | `type t_prot is protected` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1688/test1.vhdl) |
| 145 | `range ( NUM` | 10 | `type t1 is range (1+1) to (10+2);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc198.vhd) |
| 146 | `( ( ID` | 9 | `memory(to_integer(unsigned(addr)))((x+1)*8-1 downto x*8) := data_i((x+1)*8-1 downto x*8);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1780/imem.vhdl) |
| 147 | `. ID :=` | 9 | `ranges(i).min := t_item_range_value(bins(i).min);` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/CoverageVendorApiPkg_NVC.vhd) |
| 148 | `. ID ;` | 9 | `stx.env(i).b;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2076/gcrash-6a.vhdl) |
| 149 | `ID . ID` | 9 | `Osvvm.ClockResetPkg.CreateReset(` (stimuli/vhdl/subs/PoC/tb/arith/scaler/arith_scaler_TestHarness.vhdl) |
| 150 | `ID := '` | 9 | `port (d : std_logic := '0';` (stimuli/vhdl/subs/ghdl/testsuite/gna/fst01/ex1.vhdl) |
| 151 | `ID ID is` | 9 | `nature electrical is real across real through ground reference;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug031/ams1.vhdl) |
| 152 | `ID generate when` | 9 | `g_case: case sel generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/fst01/casegen1.vhdl) |
| 153 | `ID range NUM` | 9 | `signal Sht : bit_vector(2 downtonatural range 0 to 7;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/badrng.vhdl) |
| 154 | `begin assert ID` | 9 | `begin` (stimuli/vhdl/subs/UVVM/bitvis_uart/src/uart.vhd) |
| 155 | `" ID "` | 8 | `Push(WriteBurstFifo, X"UUUU_2211") ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_AxSizeManagerMemory1.vhd) |
| 156 | `" and "` | 8 | `"and";` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue478/repro2.vhdl) |
| 157 | `( " "` | 8 | `write(line0, string'(" "));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2299/tb_dct8.vhdl) |
| 158 | `( others =>` | 8 | `RETURN vector_t'(OTHERS => '0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue244/test.vhdl) |
| 159 | `) ; --` | 8 | `subtype wb_cpu_in_t  is wb_slave_out_t(dat(wb_cpu_data_bits-1 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1300/wishbone_types.vhdl) |
| 160 | `) ; end` | 8 | `s (7 downto 1) <= ();` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/noexpr.vhdl) |
| 161 | `. ID =` | 8 | `assert cst(0).nat = 1 severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2051/repro2.vhdl) |
| 162 | `. NUM **` | 8 | `constant frac			: INTEGER	:= integer(round((val - real(int)) * 10.0**precision));` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug019/PoC/src/common/strings.vhdl) |
| 163 | `; end ;` | 8 | `f ("0110");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1469/ent1.vhdl) |
| 164 | `? ID is` | 8 | `case? x is` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_case.vhd) |
| 165 | `ID := NUM` | 8 | `constant N : positive := 9;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/arith_addw_tb.vhdl) |
| 166 | `ID ; use` | 8 | `end c04s03b02x02p20n01i00163pkg;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc163.vhd) |
| 167 | `ID = NUM` | 8 | `gen: if False b = 0 generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/inst2.vhdl) |
| 168 | `NUM ' after` | 8 | `S1 <= transport '0' after 5  ns,` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1321.vhd) |
| 169 | `subtype ID is` | 8 | `subtype word is std_logic_vector(word_len - 1 downto 0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1704/orig.vhdl) |
| 170 | `" " ;` | 7 | `constant s37: string:=""" ";   -- 37` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2737.vhd) |
| 171 | `( ' ID` | 7 | `o <= to_X01(std_ulogic' ('H'));` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1537/ent1.vhdl) |
| 172 | `( ID .` | 7 | `work.cpu_types.status_value'(work.cpu_types.fetch)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1640/test.vhdl) |
| 173 | `. ID .` | 7 | `assert path(idx).name.all               /= "" severity failure ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_015.vhd) |
| 174 | `: while ID` | 7 | `L1 :` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1509.vhd) |
| 175 | `<= ID (` | 7 | `COMPARATOR: op <= raw_op_sin(raw_op_sin'length -1); --Sign bit` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue44/costasloop.vhdl) |
| 176 | `<= ID ;` | 7 | `transfer: o <= x;                             -- Covered` (stimuli/vhdl/subs/nvc/test/cover/issue1431_sub.vhd) |
| 177 | `ID : entity` | 7 | `ram : entity work.ocram_sdp` (stimuli/vhdl/subs/PoC/src/mem/ocram/ocram_sdp_optimized.vhdl) |
| 178 | `ID ; alias` | 7 | `end view Axi4WriteAddressManagerView ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Axi4InterfaceModeViewPkg.vhd) |
| 179 | `ID after NUM` | 7 | `S2_inertial  <= inertial S1 after 100 ns;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue458/e.vhdl) |
| 180 | `component ID end` | 7 | `component c05s02b02x00p02n01i03139ent_a` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc3139.vhd) |
| 181 | `end ID ;` | 7 | `end behav;` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket41/bug1.vhdl) |
| 182 | `range ID to` | 7 | `type byte is range c07s01b00x00p08n01i01849arch to 3;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1849.vhd) |
| 183 | `to ID =>` | 7 | `return (bv => (1 to n => '0'));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1138/repro.vhdl) |
| 184 | `" - "` | 6 | `function "-" (a, b : t) return t is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug21513/pb.vhdl) |
| 185 | `" not "` | 6 | `use ieee.std_logic_1164."not";` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2396/frequency.vhdl) |
| 186 | `( ( NUM` | 6 | `alias ADreg1 is bufreg((2 * SIZE - 1) downto SIZE);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug21274/21274.vhd) |
| 187 | `( NUM ,` | 6 | `CONSTANT C_TEST_VECTOR : test_vector_t := (OTHERS => test_data_t'(0, 0, 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue242/test.vhdl) |
| 188 | `, NUM downto` | 6 | `type A is array (1 to 4, 31 downto 0) of boolean;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_04_ch_04_04.vhd) |
| 189 | `. ID '` | 6 | `assert not APPEND_DEST_BITS or (Out_M2S.Dest'length = log2ceilnz(PORTS) + In_M2S(0).Dest'length)` (stimuli/vhdl/subs/PoC/src/bus/axi4/AXI4Stream/AXI4Stream_Mux.vhdl) |
| 190 | `. ID )` | 6 | `RETURN (ain(25).b2);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc973.vhd) |
| 191 | `; ID :` | 6 | `simInitialize;` (stimuli/vhdl/subs/PoC/tb/misc/gearbox/gearbox_down_dc_tb.vhdl) |
| 192 | `; begin ID` | 6 | `file f : text ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_006a.vhd) |
| 193 | `; procedure ID` | 6 | `file my_file : text;` (stimuli/vhdl/subs/nvc/test/regress/issue284.vhd) |
| 194 | `<= ID and` | 6 | `z <= a and b and c;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_ch_05_24.vhd) |
| 195 | `assert ID report` | 6 | `assert FALSE` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1440.vhd) |
| 196 | `begin if ID` | 6 | `function begin if a s';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash42.vhdl) |
| 197 | `for ID for` | 6 | `for instrumentation` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_14_fg_14_10.vhd) |
| 198 | `generate ID <=` | 6 | `else generate` (stimuli/vhdl/subs/OsvvmLibraries/DpRam/src/DpRam_Singleton.vhd) |
| 199 | `process is begin` | 6 | `postponed process is` (stimuli/vhdl/subs/nvc/test/parse/process.vhd) |
| 200 | `return ID ;` | 6 | `return true;      -- illegal in entity declaration region.` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1620.vhd) |
| 201 | `' ID )` | 5 | `bit'samples);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/name2.vhdl) |
| 202 | `' ID =` | 5 | `if (T2'BASE'LEFT = T1'BASE'LEFT) then   --- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc3161.vhd) |
| 203 | `) := (` | 5 | `constant v : my_rec (addr(0 downto 0)) := (` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0117/repro7.vhdl) |
| 204 | `) ; type` | 5 | `subtype myboundedrecord is myrecord(a(1 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1287/myentity.vhdl) |
| 205 | `) ; variable` | 5 | `variable r1 : my_rec (v(0 to 3));` (stimuli/vhdl/subs/ghdl/testsuite/synth/conv01/convrec01.vhdl) |
| 206 | `. ID #` | 5 | `assert NOT(   16#F.FF#E+2 = 2#1.1111_1111_111#E11   and` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2713.vhd) |
| 207 | `. ID <=` | 5 | `port_req(i).stb <= req_i.stb and port_sel(i);` (stimuli/vhdl/subs/neorv32/rtl/core/neorv32_bus.vhd) |
| 208 | `: ID :=` | 5 | `constant AXI_DATA_WIDTH      : integer := AxiBus.WriteData.Data'length ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4Lite/src/Axi4LiteManager.vhd) |
| 209 | `<< constant .` | 5 | `<< constant .h.x : bit >>;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/extname1.vhdl) |
| 210 | `> NUM ,` | 5 | `MY_GENERIC(0)           => 1,` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3050/top.vhdl) |
| 211 | `?? ID )` | 5 | `if (?? tg) and falling_edge(clk) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc14.vhdl) |
| 212 | `ID := ID` | 5 | `port (d : bit_vector := x"01");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue610/repro1.vhdl) |
| 213 | `ID is generic` | 5 | `begin return 0;end;end;package gen2 is generic(package pkg is new work.gen0 generic map(<>));function get2 return natura` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/attr25.vhdl) |
| 214 | `ID return ID` | 5 | `function get_str` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug017/call7.vhdl) |
| 215 | `NUM ID inertial` | 5 | `s <= reject 1 ns inertial 2 after 2 us;` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket11/tb3.vhdl) |
| 216 | `NUM to NUM` | 5 | `subtype small_int is range 0 to 7;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1385.vhd) |
| 217 | `[ return ID` | 5 | `alias IsVhdlAssertFailed is std.env.IsVhdlAssertFailed [return BOOLEAN] ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/AssertApiPkg.vhd) |
| 218 | `\ ID \` | 5 | `entity \foo\ is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug012/demo.vhd) |
| 219 | `component ID generic` | 5 | `component ic_socket` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc3128.vhd) |
| 220 | `entity ID is` | 5 | `ENTITY c07s03b04x00p03n01i02509ent IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2509.vhd) |
| 221 | `group ID is` | 5 | `group signal_pair is (signal, signal);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_20_ch_20_11.vhd) |
| 222 | `of ID is` | 5 | `function meaning_of_life return return_value of unsigned is` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_reading_function_return_value.vhd) |
| 223 | `process variable ID` | 5 | `process` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug017/call6.vhdl) |
| 224 | `protected -- --` | 5 | `type t_data_queue is protected` (stimuli/vhdl/subs/UVVM/uvvm_util/src/data_queue_pkg.vhd) |
| 225 | `protected -- ID` | 5 | `type ModelParametersPType is protected` (stimuli/vhdl/subs/OsvvmLibraries/Common/src/ModelParametersPtPkg.vhd) |
| 226 | `protected function ID` | 5 | `type p is protected` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue400/e.vhdl) |
| 227 | `range NUM downto` | 5 | `type beta is range 1000 downto 0` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc526.vhd) |
| 228 | `return ID (` | 5 | `return inner(0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2104/e.vhdl) |
| 229 | `then null ;` | 5 | `if (cst = 3 then` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/paren.vhdl) |
| 230 | `use ID .` | 5 | `use work.c10s01b00x00p11n01i00886pkg_a.x;    -- creates a declarative item in this region` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc886.vhd) |
| 231 | `variable ID :` | 5 | `variable var : std_logic_vector(0 to 1);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2459/ent.vhdl) |
| 232 | `when ID =` | 5 | `return true when x = 1 else false ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_094a.vhd) |
| 233 | `' ' '` | 4 | `constant C_MSG_DELIMITER : character   := '''; -- Delimiter of the msg parameter in UVVM commands` (stimuli/vhdl/subs/UVVM/uvvm_util/src/adaptations_pkg.vhd) |
| 234 | `' ' ;` | 4 | `k := '';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2728.vhd) |
| 235 | `( - NUM` | 4 | `foobar <= to_signed(natural'(-1), foobar'length);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1240/issue1.vhdl) |
| 236 | `( ID downto` | 4 | `signal coeff: std_logic_vector_array (0 to 5) (high_c downto low_c) := (others => (others => '0'));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue887/test2.vhdl) |
| 237 | `+ ID ;` | 4 | `R := R * + R;  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2192.vhd) |
| 238 | `- ID ;` | 4 | `c := a/-b;   -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2188.vhd) |
| 239 | `. NUM +` | 4 | `variable w : real := 3.0 + 2.0;   -- No_failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2021.vhd) |
| 240 | `;` | 4 | `ENDc04s03b01x01p03n02i00046arch;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc46.vhd) |
| 241 | `<= ' ID` | 4 | `synch_control_pull_up : synch_control <= 'H';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_11_fg_11_09.vhd) |
| 242 | `<= NUM after` | 4 | `sig : sma_int <= 6 after 5 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1852.vhd) |
| 243 | `?= ' NUM` | 4 | `check_equal('1' ?= '1', '1');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_eq.vhd) |
| 244 | `?= ID ;` | 4 | `slv_eq <= slv_a ?=  slv_b;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1109/ent-orig.vhdl) |
| 245 | `?? ID (` | 4 | `if ?? falling_edge(clk) and (?? tg) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc10.vhdl) |
| 246 | `ID " ID` | 4 | `subtype displacement is real tolerance "def_disp";` (stimuli/vhdl/subs/ghdl/testsuite/sanity/004all08/ams08.vhdl) |
| 247 | `ID ' ;` | 4 | `function is;n';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash25.vhdl) |
| 248 | `ID ( signal` | 4 | `procedure drive_sig (signal s : out bit)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2736/mwe.vhdl) |
| 249 | `ID , NUM` | 4 | `0 =>    300 Bd,   1 =>    600 Bd,   2 =>   1200 Bd,   3 =>   1800 Bd,   4 =>   2400 Bd,` (stimuli/vhdl/subs/PoC/src/io/uart/uart.pkg.vhdl) |
| 250 | `ID - NUM` | 4 | `other_ctrs : in T_ARRAY(0 to N_NODES-2)(COUNTER_BITS-1 downto 0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2631/node.vhd) |
| 251 | `ID : view` | 4 | `signal   TransactionRec : view (AddressBusTestCtrlView) of AddressBusRecArrayType ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/common/src/Axi4OptionsArrayPkg.vhd) |
| 252 | `ID ID ;` | 4 | `procedure initialize;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/sim_protected.v08.vhdl) |
| 253 | `ID after ID` | 4 | `z <= transport a after Tpd_01;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_fg_05_12.vhd) |
| 254 | `ID of ID` | 4 | `attribute foreign of c_getenv : function is "VHPIDIRECT c_getenv";` (stimuli/vhdl/subs/Compliance-Tests/cosim/dpi-ffi/ghdl-vffi/test/EnvironmentVariables.vhd) |
| 255 | `NUM ) ;` | 4 | `left <= wordrevorder(c, 1);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1257/direction_mismatch.vhd) |
| 256 | `NUM after NUM` | 4 | `k <= transport 5 after 20 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1221.vhd) |
| 257 | `` if ID` | 4 | ``if TOOL_TYPE = "SIMULATION" then` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_061.vhd) |
| 258 | `access ID (` | 4 | `type my_line is access string(1 to 80);` (stimuli/vhdl/subs/ghdl/testsuite/synth/types01/access01.vhdl) |
| 259 | `architecture ID of` | 4 | `ARCHITECTURE ch130301_p00201_06_arch OF ch130301_p00201_06_ent IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2586.vhd) |
| 260 | `process begin ID` | 4 | `process` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug7186/bug.vhdl) |
| 261 | `view ( ID` | 4 | `TX : view ( Differatial_OutView );` (stimuli/vhdl/subs/Interfaces/MIPI/M-PHY.vhdl) |
| 262 | `view ID )` | 4 | `port (source : view st_source_v);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue3267/top.vhdl) |
| 263 | `when ID <` | 4 | `cnt := cnt + 1 when cnt<g_NumberOfChannels-1 else 0;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2042/ent.vhdl) |
| 264 | `when ID >` | 4 | `next when i > 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1573.vhd) |
| 265 | `" & "` | 3 | `function "&" ( a, b : stimulus_list ) return stimulus_list;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_20_ch_20_05.vhd) |
| 266 | `" > "` | 3 | `function ">" ( l,r : integer ) return bit;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1387/repro1.vhdl) |
| 267 | `" ID ID` | 3 | `report "***PASSED TEST: c08s02b00x00p05n01i01260 - This test needs manual check to make sure that default value for the ` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1260.vhd) |
| 268 | `" NUM --` | 3 | `AffirmIfEqual(Data, X"0403_----", "Subordinate Write Data: ") ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_BasicBurst.vhd) |
| 269 | `" mod "` | 3 | `alias to_hash is "mod" [integer, integer return integer];` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug061/dictp.vhdl) |
| 270 | `% ID :` | 3 | `x%x : in bit; -- <==` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug24326/tb_thingy9.vhdl) |
| 271 | `' ' ID` | 3 | `architecture if''h';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash2.vhdl) |
| 272 | `( ID &` | 3 | `assert t_mem'(bv1 & bv2) =(0 to 1 => "00");` (stimuli/vhdl/subs/ghdl/testsuite/synth/err01/err_concat04.vhdl) |
| 273 | `( ID to` | 3 | `inst2: counter2(a to b)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1823/entity2.vhdl) |
| 274 | `( NUM **` | 3 | `(2**16-1 downto 0);` (stimuli/vhdl/subs/nvc/test/elab/signal34.vhd) |
| 275 | `( NUM .` | 3 | `x <= (complex'(0.0,0.0), complex'(1.0,1.0), complex'(2.0,2.0), complex'(3.0,3.0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2572/tb.vhdl) |
| 276 | `( open )` | 3 | `subtype P2 is A(15 downto 0)(open);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue459/e3.vhdl) |
| 277 | `) ) ;` | 3 | `port (port1 : in OuterRecordType(inner1(inner2b(WIDTH-1 downto 0))));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1523/repro.vhdl) |
| 278 | `) ; subtype` | 3 | `subtype rec_4 is rec (s(1 to 4));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue238/pkg.vhdl) |
| 279 | `, ID ,` | 3 | `type word_vector is array(tArch_test, tSkip_test, boolean) of word;` (stimuli/vhdl/subs/PoC/tb/arith/addw/arith_addw_TestController_pkg.vhdl) |
| 280 | `. NUM report` | 3 | `assert mean > 0.0` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_19_random-b.vhd) |
| 281 | `: if ID` | 3 | `CLK_IF : if CLK = '1' then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3102/DAC_test.vhdl) |
| 282 | `: wait for` | 3 | `wait_1: wait for 1 ns;` (stimuli/vhdl/subs/nvc/test/lower/wait1.vhd) |
| 283 | `; -- default` | 3 | `constant is_original_pkg : boolean;` (stimuli/vhdl/subs/vunit/examples/vhdl/osvvm_log_integration/osvvm_integration/osvvm_to_vunit_common_log_pkg.vhd) |
| 284 | `; -- generate` | 3 | `simInitialize;` (stimuli/vhdl/subs/PoC/tb/io/ddrio/ddrio_inout_tb.vhdl) |
| 285 | `; ID '` | 3 | `architecture;b';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash21.vhdl) |
| 286 | `; ID (` | 3 | `watchdog_timer(watchdog_ctrl_terminate, C_GENERAL_WATCHDOG_TIMEOUT, ERROR, "Watchdog A");` (stimuli/vhdl/subs/UVVM/bitvis_vip_uart/tb/uvvm_demo_tb.vhd) |
| 287 | `; assert ID` | 3 | `wait on ;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1188.vhd) |
| 288 | `; begin assert` | 3 | `constant x : sub_character;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/clifton-labs/compliant/functional/subtypes/modified-character-subtype.vhdl) |
| 289 | `; begin process` | 3 | `alias a_sig_1 is sig_1;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1751/ent.vhdl) |
| 290 | `; function ID` | 3 | `type		T_SLM								is array(NATURAL range <>, NATURAL range <>) of STD_LOGIC;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2/sortnet_BitonicSort.vhdl) |
| 291 | `> ' ID` | 3 | `report (1 to v => 'X');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2148/e2.vhdl) |
| 292 | `> ) of` | 3 | `type er1 is array(range <>) of integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc321.vhd) |
| 293 | `> NUM to` | 3 | `type bit_vctor is array (integer => 1 to 8) of integer; --Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc324.vhd) |
| 294 | `? /= "` | 3 | `z <= a ?/= "1--0";` (stimuli/vhdl/subs/ghdl/testsuite/synth/match01/match02.vhdl) |
| 295 | `?= ID (` | 3 | `acc_en <= addr_i(abb_c'range) ?= IMEM_BASE(abb_c'range);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1781/imem.vhdl) |
| 296 | `ID " NUM` | 3 | `assert 5ub"11_000" = unsigned'(b"11_000");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1274/issue.vhdl) |
| 297 | `ID ( variable` | 3 | `procedure get_rec(variable rec : out rec_t) is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue397/e.vhdl) |
| 298 | `ID -- ID` | 3 | `gen_n_elementos : in integer -- quantidade de elementos no vetor de entradas` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2461/insere.vhdl) |
| 299 | `ID : block` | 3 | `blk : block` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue158/repro.vhdl) |
| 300 | `ID ;` | 3 | `assert false report "Hello world" severity note;with behav;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug090/hang3.vhdl) |
| 301 | `ID = (` | 3 | `assert NOT(2E6 = (2E3*1E3))` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2567.vhd) |
| 302 | `ID = ID` | 3 | `IDB <= TRB 	         when OP_SRC = x"0" else` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1243/DSPn.vhdl) |
| 303 | `NUM ' ;` | 3 | `glob_sig <= force '0';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2471/test.vhdl) |
| 304 | `NUM ID ;` | 3 | `permil	= 1000 ppm;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/sim_types.vhdl) |
| 305 | `[ ID ]` | 3 | `mem[address]<=data_in;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue609/test.vhd) |
| 306 | `[ ] ;` | 3 | `alias prog_alias is proc[];` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket41/bug.vhdl) |
| 307 | `begin assert (` | 3 | `begin` (stimuli/vhdl/subs/UVVM/bitvis_vip_clock_generator/src/clock_generator_vvc.vhd) |
| 308 | `bus ; begin` | 3 | `signal s : std_logic bus;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2571/test1.vhdl) |
| 309 | `end case ;` | 3 | `end case;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/nochoice1.vhdl) |
| 310 | `for ID --` | 3 | `for functionality    -- configure the block` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_fg_16_16.vhd) |
| 311 | `generic ( constant` | 3 | `generic   (constant n : positive)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3110/issue.vhdl) |
| 312 | `generic map (` | 3 | `generic map (c => 5);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug057/tb.vhdl) |
| 313 | `null ; end` | 3 | `NULL;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1432.vhd) |
| 314 | `procedure ID is` | 3 | `procedure foo is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue710/ent2.vhdl) |
| 315 | `protected -- open` | 3 | `type t_csv_file_reader is protected` (stimuli/vhdl/subs/UVVM/bitvis_vip_spec_cov/src/csv_file_reader_pkg.vhd) |
| 316 | `range " NUM` | 3 | `type I3 is range "0" to "9";  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc247.vhd) |
| 317 | `range ( (` | 3 | `type a is range (((((10-1)-1)-1)-1)-1) to (((((10+1)+1)+1)+1)+1);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc230.vhd) |
| 318 | `range - ID` | 3 | `type my_time is range -integer'low  to integer'high units` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue810/my_time_pkg.vhdl) |
| 319 | `ror NUM ;` | 3 | `outp <= inp ror 1; -- can also be 'rol'` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1909/reproducebug.vhdl) |
| 320 | `sra ID severity` | 3 | `assert rao = v sra n severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper02/tb_nushift01.vhdl) |
| 321 | `when ( ID` | 3 | `next when (a mod 2) = 0;` (stimuli/vhdl/subs/nvc/test/lower/loop1.vhd) |
| 322 | `when ID (` | 3 | `BitVal(BitIndex) := '1' when BitsToEncode(BitIndex) = 'U' else '0' ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/MemorySupportPkg.vhd) |
| 323 | `! ID :` | 2 | `variable k!k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2611.vhd) |
| 324 | `" -- ID` | 2 | `WB_ADDR          : std_logic_vector := "-" & X"--_----" ; -- Default 8M words divided between Register, Memory, DMA` (stimuli/vhdl/subs/OsvvmLibraries/Wishbone/src/WishboneComponentPkg.vhd) |
| 325 | `" ID --` | 2 | `AffirmIfEqual(TbManagerID, ReadData, X"BAD0_00--", "AXI Manager Read Data: ") ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_TimeOutManager.vhd) |
| 326 | `" ID .` | 2 | `file f : text is in "input.txt";` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug096/reader.vhdl) |
| 327 | `" ID :` | 2 | `variable "k : integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2580.vhd) |
| 328 | `" abs "` | 2 | `function "abs" (constant c1 : in integer) return newt is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2968.vhd) |
| 329 | `# ID :` | 2 | `variable k#k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2612.vhd) |
| 330 | `& ID :` | 2 | `variable k&k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2615.vhd) |
| 331 | `' ( ID` | 2 | `bit'(samples));` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/name1.vhdl) |
| 332 | `' ( others` | 2 | `cnt := cnt'last'(others => '1');` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/qual.vhdl) |
| 333 | `' ID (` | 2 | `constant time_image : time'image(now);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue387/bug.vhdl) |
| 334 | `' ID :` | 2 | `variable k'k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2629.vhd) |
| 335 | `' ID :=` | 2 | `constant c : x'element := (others => '0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3204/t3.vhdl) |
| 336 | `' ID >=` | 2 | `assert (not clk'event) or clk'delayed'last_event >= Tpw_clk` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_ch_05_07.vhd) |
| 337 | `' \ '` | 2 | `function ChangeSeparator (A : string ; FromSeparator : character := '\' ; ToSeparator : character := '/') return string ` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/FileUtilPkg.vhd) |
| 338 | `' range ;` | 2 | `subtype mypos is natural range my_bv4'element'range;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2323/st3.vhdl) |
| 339 | `' subtype )` | 2 | `function func (a : din.data'subtype) return std_logic_vector is` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2484/issue.vhdl) |
| 340 | `' subtype :=` | 2 | `signal Data : i_Data'subtype := (others=>'0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1857/subtype_test.vhdl) |
| 341 | `( " #` | 2 | `write(l, string'("# Encoding file for '" & Name & "'"));	writeline(tokenFile, l);` (stimuli/vhdl/subs/PoC/src/common/debug.vhdl) |
| 342 | `( " :` | 2 | `write(o,string'("::"));` (stimuli/vhdl/subs/nvc/test/regress/issue1079.vhd) |
| 343 | `( ( '` | 2 | `(V,W) <= AGGREGATE_ARRAY' (('d', 'x', others => 'a'),` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2460.vhd) |
| 344 | `( ID <` | 2 | `b1: block (value < 10) is` (stimuli/vhdl/subs/nvc/test/regress/guard1.vhd) |
| 345 | `( NUM +` | 2 | `tmp <= std_logic_vector'(1 + signed(tmp));` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1253/repro1.vhdl) |
| 346 | `( NUM ;` | 2 | `function(0is;r';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash4.vhdl) |
| 347 | `( NUM ID` | 2 | `insert(stimulus_list, stimulus_element'(0 ns, "0XXXXXXXXX"));` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/non_compliant/ch_17_fg_17_14.vhd) |
| 348 | `( constant ID` | 2 | `generic (( constant i : integer );                           -- extra parenthesis` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc722.vhd) |
| 349 | `( signal ID` | 2 | `port (( signal s : bit                -- extra parenthesis` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc727.vhd) |
| 350 | `) ; alias` | 2 | `variable v_dummy_hierarchy_node : t_hierarchy_node(name(1 to C_HIERARCHY_NODE_NAME_LENGTH));` (stimuli/vhdl/subs/UVVM/uvvm_util/tb/maintenance_tb/methods_tb_alert_arch.vhd) |
| 351 | `) ; function` | 2 | `subtype A1 is ABASE(ENUM1 range M1 to M5);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1139.vhd) |
| 352 | `) ID :` | 2 | `variable k)k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2618.vhd) |
| 353 | `) is begin` | 2 | `procedure activate_event(event_number : natural range 1 to 2; event_time : time; queue_idx : positive range request_queu` (stimuli/vhdl/subs/vunit/vunit/vhdl/data_types/test/tb_event_pkg.vhd) |
| 354 | `* ID :` | 2 | `variable k*k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2616.vhd) |
| 355 | `* ID ID` | 2 | `report "***FAILED TEST: c13s10b00x00p04n01i02858 - Only right hand side quotation mark ("") is replaced by percent chara` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2858.vhd) |
| 356 | `+ ID :` | 2 | `variable k+k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2619.vhd) |
| 357 | `- ID :` | 2 | `variable k-k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2621.vhd) |
| 358 | `-- ID ID` | 2 | `-- device under test` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2448/debouncer_no_vunit_not_ok_tb.vhdl) |
| 359 | `. ID *` | 2 | `constant C_DATA_VERSION         : bit_vector(C_SEGMENT_RECORDS(TYPE_2).word_length*8-1 downto 0) := C_DATA_INVALID_VERSI` (stimuli/vhdl/subs/nvc/test/bounds/issue477a.vhd) |
| 360 | `. ID /=` | 2 | `check(new_actor("other actor").p_id_number /= new_actor("another actor").p_id_number, "Failed to create unique actors");` (stimuli/vhdl/subs/vunit/vunit/vhdl/com/test/tb_com.vhd) |
| 361 | `. ID :` | 2 | `variable k.k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2633.vhd) |
| 362 | `. NUM #` | 2 | `variable total_time : real := 5#1234.4321#E-10;   --No_failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2718.vhd) |
| 363 | `. NUM =` | 2 | `assert NOT(2.12E6 = (1.06E3*2.0E3))` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2570.vhd) |
| 364 | `. NUM then` | 2 | `if phase = 0.0 then` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug24064/pp_fir_filter.vhd) |
| 365 | `/ ID :` | 2 | `variable k/k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2634.vhd) |
| 366 | `: ID :` | 2 | `constant   one    : integer := 16:E:E1;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2854.vhd) |
| 367 | `: ID >` | 2 | `if gen_case : DIM > 1 generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2592/bug2.vhdl) |
| 368 | `: assert ID` | 2 | `erlab : assert FALSE;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1235.vhd) |
| 369 | `: if (` | 2 | `l_if_1: if (cnt = 0) then report "IF1: CNT = 0";` (stimuli/vhdl/subs/nvc/test/regress/cover3.vhd) |
| 370 | `: loop ID` | 2 | `OperationLoop: loop` (stimuli/vhdl/subs/OsvvmLibraries/CoSim/testbench/TestCases_Uart/TbUart_SendGet1.vhd) |
| 371 | `: loop end` | 2 | `L1 : loop` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1513.vhd) |
| 372 | `: when NUM` | 2 | `when 2 => NULL:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1470.vhd) |
| 373 | `:= NUM ;` | 2 | `signal n : integer range v'range := 3;` (stimuli/vhdl/subs/nvc/test/regress/wait14.vhd) |
| 374 | `; -- -` | 2 | `function testp (I1:Bit) return;  --- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2871.vhd) |
| 375 | `; -- end` | 2 | `alias status_type is work.system_types.system_status;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_09_ch_09_03.vhd) |
| 376 | `; -- private` | 2 | `type bounded_buffer_object;  -- private` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_17_fg_17_08.vhd) |
| 377 | `; process (` | 2 | `Delays					<= Delays(Delays'high - 1 downto 0) & DataIn when rising_edge(Clock);` (stimuli/vhdl/subs/PoC/src/misc/filter/filter_mean.vhdl) |
| 378 | `< ID :` | 2 | `variable k<k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2635.vhd) |
| 379 | `<= " NUM` | 2 | `as_q1: q1 <= "0000" & b;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue150/concat2.vhdl) |
| 380 | `<= ( others` | 2 | `proc: p <= (others => '1');` (stimuli/vhdl/subs/nvc/test/elab/clone2.vhd) |
| 381 | `<= ID .` | 2 | `update_a: a <= r.x after 1 ns;` (stimuli/vhdl/subs/nvc/test/regress/record11.vhd) |
| 382 | `<= ID or` | 2 | `res_or_1 <= lhs_or_1 or rhs_or_1;` (stimuli/vhdl/subs/nvc/test/regress/cover7.vhd) |
| 383 | `<= NUM ;` | 2 | `bad: sig <= 0;` (stimuli/vhdl/subs/nvc/test/elab/issue184.vhd) |
| 384 | `<= not (` | 2 | `func : y <= not ((a1 and a2) or (b1 and b2));` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_ch_05_23.vhd) |
| 385 | `<= transport ID` | 2 | `asym_delay : z <= transport a after Tpd_01 when a = '1' else` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_ch_05_16.vhd) |
| 386 | `= ID :` | 2 | `variable k=k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2622.vhd) |
| 387 | `= NUM ;` | 2 | `variable   s := 10;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1453.vhd) |
| 388 | `> ( ID` | 2 | `w <= (byte'range => (b and mask), others => '1');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1493/test.vhdl) |
| 389 | `> ID :` | 2 | `variable k>k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2636.vhd) |
| 390 | `> open ,` | 2 | `test_out(15 ) => open,` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3109/file2.vhdl) |
| 391 | `? ( ID` | 2 | `case? (dec_vld) is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2620/casetb.vhdl) |
| 392 | `? ID :` | 2 | `variable k?k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2637.vhd) |
| 393 | `?= " NUM` | 2 | `r := l ?= "110";` (stimuli/vhdl/subs/ghdl/testsuite/synth/err01/err_slv2.vhdl) |
| 394 | `?> = '` | 2 | `check_equal('1' ?>= '1', '1');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_ge.vhd) |
| 395 | `?? ( ID` | 2 | `if ?? (tg) and falling_edge(clk) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc15.vhdl) |
| 396 | `?? ID and` | 2 | `if ?? tg and falling_edge(clk) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc13.vhdl) |
| 397 | `?? ID then` | 2 | `if ?? tg then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc1.vhdl) |
| 398 | `ID %` | 2 | `d%` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash1.vhdl) |
| 399 | `ID & ID` | 2 | `with a&b select` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2486/bug.vhdl) |
| 400 | `ID ( constant` | 2 | `procedure pkt_report(` (stimuli/vhdl/subs/nvc/test/regress/issue1186.vhd) |
| 401 | `ID , --` | 2 | `-- Use MODEL_ID_NAME Generic if set, otherwise,` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/src/AxiStreamReceiver.vhd) |
| 402 | `ID : --` | 2 | `H:--###############################################################################` (stimuli/vhdl/subs/Rudi-RV32I/c/template_program_memory.vhd) |
| 403 | `ID : for` | 2 | `nibble_array : for nibble_index in 0 to 7 generate` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_14_fg_14_04.vhd) |
| 404 | `ID : if` | 2 | `end_gen : if index = 7 generate` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_20_fg_20_07.vhd) |
| 405 | `ID ; )` | 2 | `generic ( g : in natural ; ) ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_071a.vhd) |
| 406 | `ID ; attribute` | 2 | `end rr1;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc955.vhd) |
| 407 | `ID ; pure` | 2 | `type type1;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2693/testbench.vhdl) |
| 408 | `ID ; wait` | 2 | `sig(C_VAL'range) <= force C_VAL;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2590/test2.vhdl) |
| 409 | `ID <= '` | 2 | `write <= '0';` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1155/tb_ent.vhdl) |
| 410 | `ID ID (` | 2 | `return ite ((s < 0.0), "-" & res, res);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/strings.vhdl) |
| 411 | `ID NUM ID` | 2 | `wait on k until (k = 5) rof 60 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1195.vhd) |
| 412 | `ID is --` | 2 | `case err is                           -- illegal location for case statement` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc821.vhd) |
| 413 | `ID is begin` | 2 | `ARCHITECTURE c04s01b00x00p08n01i00005arch OFc04s01b00x00p08n01i00005ent IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc5.vhd) |
| 414 | `ID is end` | 2 | `ENTITY c02s05b00x00p02n01i02985ent IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2985.vhd) |
| 415 | `ID is private` | 2 | `type element_type is private;                         -- any type` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_059.vhd) |
| 416 | `ID range -` | 2 | `port (a : integer range -3 to 2;` (stimuli/vhdl/subs/ghdl/testsuite/synth/case02/case05.vhdl) |
| 417 | `ID range <>` | 2 | `type t_slv32_x3 is array(1 tnatural range <>) of bit_vector(31 downto 0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/arr_err1.vhdl) |
| 418 | `ID register ;` | 2 | `signal source1, source2 : resolve_unique word register;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_fg_16_02.vhd) |
| 419 | `ID select --` | 2 | `with i select                        -- No_failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1766.vhd) |
| 420 | `ID then ID` | 2 | `if g_bug_mode then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2452/foo.vhdl) |
| 421 | `NUM . NUM` | 2 | `constant x : real := abs 10.5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2308.vhd) |
| 422 | `NUM => ID` | 2 | `1 => std_logic_vector'(x"01"));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2112/abc_tb2.vhdl) |
| 423 | `NUM ID ID` | 2 | `CBITS => 10                     -- 10 capacity bits for the quire` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue664/numeric_system_pkg-orig.vhdl) |
| 424 | `NUM downto NUM` | 2 | `Indices(abc_BUS_SETTINGS.PAIRS - 1 downto 0)(abc_BUS_SETTINGS.BW - 1 downto 0)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2112/abc_tb.vhdl) |
| 425 | `[ ID :` | 2 | `variable k[k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2626.vhd) |
| 426 | `[ ] (` | 2 | `if g[](0)then if 0='0'then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/sign01.vhdl) |
| 427 | `\ ID :` | 2 | `variable k\k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2638.vhd) |
| 428 | `] ID :` | 2 | `variable k]k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2627.vhd) |
| 429 | `^ ID :` | 2 | `variable k^k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2614.vhd) |
| 430 | `` ID :` | 2 | `variable k`k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2623.vhd) |
| 431 | `assert ID =` | 2 | `postponed assert s = '0' severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug051/tb.vhdl) |
| 432 | `begin ID '` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash41.vhdl) |
| 433 | `begin end ;` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/sr2553/2553.vhd) |
| 434 | `begin end ID` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc791.vhd) |
| 435 | `begin postponed process` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_21_fg_21_05.vhd) |
| 436 | `begin return ;` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc803.vhd) |
| 437 | `bus ; ID` | 2 | `q : out resolved_byte bus;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_fg_16_05.vhd) |
| 438 | `context is` | 2 | `context is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash46.vhdl) |
| 439 | `else ' ID` | 2 | `A := 'H' when Clk = '1' else 'L' ;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue17/cond_assign_proc.vhdl) |
| 440 | `end ;` | 2 | `end;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1514/ent.vhdl) |
| 441 | `end loop ID` | 2 | `end loop L;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1669.vhd) |
| 442 | `end package ;` | 2 | `end package;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2076/gcrash-9a.vhdl) |
| 443 | `for ID ,` | 2 | `for m1, m2 : nfet` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/components-and-configs/inline_02a.vhd) |
| 444 | `function " ID` | 2 | `function "an" & "d" return BOOLEAN;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2877.vhd) |
| 445 | `function ID return` | 2 | `function mytime return boolean is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2866.vhd) |
| 446 | `generate assert ID` | 2 | `else generate` (stimuli/vhdl/subs/PoC/src/fifo/fifo_cc_got.vhdl) |
| 447 | `generate end generate` | 2 | `else generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2982/nfmac.vhdl) |
| 448 | `guarded ' NUM` | 2 | `err   <= transport guarded '1';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1742.vhd) |
| 449 | `if ) ID` | 2 | `if)h';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash20.vhdl) |
| 450 | `impure function ID` | 2 | `impure function now return delay_length;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_07_ch_07_04.vhd) |
| 451 | `is constant ID` | 2 | `block is` (stimuli/vhdl/subs/nvc/test/regress/seqblock1.vhd) |
| 452 | `on ID ;` | 2 | `wait until (j = 1) on i;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1184.vhd) |
| 453 | `out ID (` | 2 | `procedure to_t( signal sa : out bit_vector(31 downto 0))  is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue43/repro2.vhdl) |
| 454 | `package body ;` | 2 | `end package body;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue419/bug.vhdl) |
| 455 | `procedure ID ;` | 2 | `procedure proccall;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1718.vhd) |
| 456 | `process ( ID` | 2 | `process(clk)` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1081/test.vhdl) |
| 457 | `process is --` | 2 | `p : postponed process is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_21_ch_21_02.vhd) |
| 458 | `pure function "` | 2 | `pure function "+" (a, b : fft_data_vector) return fft_data_vector;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2572/fft_types.vhdl) |
| 459 | `range ID "` | 2 | `type I5 is range B"000" to B"111";  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc249.vhd) |
| 460 | `record ; end` | 2 | `end record ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_082.vhd) |
| 461 | `sll ID ;` | 2 | `slv_sll <= slv sll int;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1520/test.vhdl) |
| 462 | `sll NUM )` | 2 | `assert (b sll 1) = "0110";` (stimuli/vhdl/subs/nvc/test/regress/issue1529.vhd) |
| 463 | `sll NUM =` | 2 | `assert B"10001010" sll 3  =  B"01010000";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_04_ch_04_07.vhd) |
| 464 | `srl NUM ;` | 2 | `ma_half_fdiv <= ma_fdiv srl 1;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1655/absenc_master.vhdl) |
| 465 | `to - NUM` | 2 | `when integer'Low to -11 | 6 to 100 =>   k := 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1488.vhd) |
| 466 | `to ID +` | 2 | `when C+2 to C+5 => -- . . .` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/non_compliant/ch_03_ch_03_09.vhd) |
| 467 | `to ID -` | 2 | `constant zeros : bit_vector(0 to width-1) := (0 to width-1 => '0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1387/repro5.vhdl) |
| 468 | `when NUM =>` | 2 | `when 1 => k := 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1464.vhd) |
| 469 | `{ ID :` | 2 | `variable k{k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2624.vhd) |
| 470 | `| ID '` | 2 | `CONSTANT C2_csa1_1 : t_csa1_1 := ( t_csa1_1'LEFT|t_csa1_1'RIGHT=>C2_scl1,` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2012.vhd) |
| 471 | `| ID :` | 2 | `variable k|k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2639.vhd) |
| 472 | `} ID :` | 2 | `variable k}k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2625.vhd) |
| 473 | `~ ID :` | 2 | `variable k~k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2620.vhd) |
| 474 | ` restrict [` | 1 | `architecturerestrict[=to 0` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2110/psl2.vhdl) |
| 475 | `! : ID` | 1 | `variable k! : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2581.vhd) |
| 476 | `! ; end` | 1 | `assert 0!;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/psl04.vhdl) |
| 477 | `" " "` | 1 | `report """""";` (stimuli/vhdl/subs/nvc/test/parse/issue205.vhd) |
| 478 | `" " &` | 1 | `debug(memory_bfm_logger, "Writing x""" & to_hstring(data) & """ to address x""" & to_hstring(address) & """");` (stimuli/vhdl/subs/vunit/examples/vhdl/com/test/memory_bfm.vhd) |
| 479 | `" " begin` | 1 | `package function""begin r';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash44.vhdl) |
| 480 | `" " for` | 1 | `configuration"` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2110/conf1.vhdl) |
| 481 | `" ' NUM` | 1 | `variable bit_str : bit_vector (1 to 8) := b"'0''1''0''1''0''1''0''1'";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2757.vhd) |
| 482 | `" ) "` | 1 | `Std_logic'image(reentry_guard) ")"; -- MISSING AMPERSAND` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue478/repro.vhdl) |
| 483 | `" /= "` | 1 | `function "/=" (L, R: std_logic) return std_logic is` (stimuli/vhdl/subs/ghdl/testsuite/gna/perf02-long/fsm_163.vhd) |
| 484 | `" : ID` | 1 | `variable k" : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2601.vhd) |
| 485 | `" <= "` | 1 | `function "<=" (l, r : my_vec) return boolean;` (stimuli/vhdl/subs/nvc/test/parse/explicit.vhd) |
| 486 | `" >= "` | 1 | `function ">="(x, y : unsigned) return boolean;` (stimuli/vhdl/subs/nvc/test/parse/implicit.vhd) |
| 487 | `" ID ,` | 1 | `check_core_failure("Invalid ID name ""parent,child""");` (stimuli/vhdl/subs/vunit/vunit/vhdl/data_types/test/tb_id.vhd) |
| 488 | `" NUM .` | 1 | `variable bit_str : bit_vector (1 to 8) := b"0101.0101";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2756.vhd) |
| 489 | `" NUM NUM` | 1 | `variable bit_str : bit_vector (1 to 8) := b"0101 0101";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2755.vhd) |
| 490 | `# : ID` | 1 | `variable k# : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2582.vhd) |
| 491 | `% % %` | 1 | `constant   a : string := %%%%;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2860.vhd) |
| 492 | `% % ID` | 1 | `%%d%` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash0.vhdl) |
| 493 | `% : ID` | 1 | `variable k% : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2583.vhd) |
| 494 | `% ID %` | 1 | `constant   two   : bit_vector := X%FF%;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2862.vhd) |
| 495 | `% ID =>` | 1 | `x%x => stimuli, -- <==` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug24326/tb_thingy7.vhdl) |
| 496 | `% ID ID` | 1 | `report %This string is illegal because of this character ".%` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2859.vhd) |
| 497 | `% NUM "` | 1 | `variable bit_str : bit_vector(0 to 7) := %01010101";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2857.vhd) |
| 498 | `% NUM %` | 1 | `CONSTANT i : bit_vector := B%1111_1111_1111%;` (stimuli/vhdl/subs/nvc/test/parse/bitstring.vhd) |
| 499 | `% NUM ;` | 1 | `constant c : natural := 5 % 4;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug24326/repro1.vhdl) |
| 500 | `& " NUM` | 1 | `"0101" &` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3002/top_concat_16k.vhdl) |
| 501 | `& : ID` | 1 | `variable k& : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2585.vhd) |
| 502 | `& ID <=` | 1 | `a & b <= "01";` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2613/mre.vhdl) |
| 503 | `' ' ,` | 1 | `' ', '!', '"', '#', '$', '%', '&', ''',` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2373.vhd) |
| 504 | `' ( '` | 1 | `(bit'('1'), y, z) <=  v;        -- Error` (stimuli/vhdl/subs/nvc/test/sem/signal.vhd) |
| 505 | `' , others` | 1 | `constant vec_msb       : std_logic_vector(tamanho - 1 downto 0) := (tamanho - 1 => '1', others => '0');` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2461/pacote_aux.vhdl) |
| 506 | `' : ID` | 1 | `variable k' : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2599.vhd) |
| 507 | `' ID <=` | 1 | `S1'DELAYED <= S2;  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc178.vhd) |
| 508 | `' ID is` | 1 | `function A(v : integer) return i000'er is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/func5.vhdl) |
| 509 | `' ID port` | 1 | `t:entity k't port map(0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/aspect02.vhdl) |
| 510 | `' range '` | 1 | `subtype natural_range_record is natural'range'record ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_099.vhd) |
| 511 | `' range loop` | 1 | `for j in cst'element'range loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1772/repro2.vhdl) |
| 512 | `' subtype (` | 1 | `signal sig1 : sig0'subtype(0 to 3);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2267/ent.vhdl) |
| 513 | `( " -` | 1 | `result := std_ulogic_vector'("-000") ?/= std_ulogic_vector'("0000");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue262/repro.vhdl) |
| 514 | `( " |` | 1 | `WRITE(text_line, string'(" | "));` (stimuli/vhdl/subs/nvc/test/regress/issue351.vhd) |
| 515 | `( ( others` | 1 | `(W,V) <= (AGGREGATE_ARRAY'((others => '$'),( others => '$' )));` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2462.vhd) |
| 516 | `( ) --` | 1 | `lab : block ( )                   --guard condition must not be empty` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1657.vhd) |
| 517 | `( ) return` | 1 | `entity function()return n(of` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2768/missing_id.vhdl) |
| 518 | `( : ID` | 1 | `variable k( : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2587.vhd) |
| 519 | `( ID *` | 1 | `ram(to_integer(mem_addr))(i*8+7 downto i*8) <= (others => 'X');` (stimuli/vhdl/subs/PoC/tb/mem/mem_model.vhdl) |
| 520 | `( NUM *` | 1 | `mem(to_integer(unsigned(mem_write_address)))(8*b+7 downto 8*b) := avm_writedata_i(8*b+7 downto 8*b);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2185/avm_memory.vhd) |
| 521 | `( NUM while` | 1 | `function(0is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash6.vhdl) |
| 522 | `( NUM |` | 1 | `b1 := BYTE'(0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => '1');` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2853.vhd) |
| 523 | `) ) '` | 1 | `log(log_file, (integer'IMAGE(-(10 ** i))))'LENGTH;` (stimuli/vhdl/subs/ghdl/testsuite/gna/sr3060/integer_class.vhdl) |
| 524 | `) ) )` | 1 | `check_match(counter_out, ())))` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue563/repro.vhdl) |
| 525 | `) , NUM` | 1 | `constant c : Indices_t := (0 => bit_vector'(x"00"),` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2112/abc_tb3.vhdl) |
| 526 | `) : ID` | 1 | `variable k) : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2588.vhd) |
| 527 | `) := ID` | 1 | `) := Test(abc_bus);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2112/abc.vhdl) |
| 528 | `) ; ID` | 1 | `bus_in : in bus_t(data(7 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1324/foo.vhdl) |
| 529 | `) ; constant` | 1 | `) ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_036a.vhd) |
| 530 | `) ; procedure` | 1 | `signal s : trans (bod(0 to 3));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue788/repro.vhdl) |
| 531 | `) ; wait` | 1 | `xyzzy( std_ulogic_vector( v ) => unsigned( x ) );` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket97/bug.vhdl) |
| 532 | `) begin assert` | 1 | `TESTING: PROCESS(   )` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1682.vhd) |
| 533 | `) end architecture` | 1 | `process()` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/proc1.vhdl) |
| 534 | `) is --` | 1 | `) is                                -- Crash after error here` (stimuli/vhdl/subs/nvc/test/parse/issue942.vhd) |
| 535 | `) return ID` | 1 | `function trigger_rising() return std_logic_vector is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue563/tb_counter.vhdl) |
| 536 | `* ; --` | 1 | `use bar.*;                              -- Error` (stimuli/vhdl/subs/nvc/test/parse/visibility4.vhd) |
| 537 | `** NUM <=` | 1 | `S1**2   <= S1;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1277.vhd) |
| 538 | `+ : ID` | 1 | `variable k+ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2589.vhd) |
| 539 | `+ ID )` | 1 | `(a + b) := 10;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1353.vhd) |
| 540 | `+ NUM /=` | 1 | `assert A * +5 /= 0;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2466/t3.vhdl) |
| 541 | `, -- ID` | 1 | `(natural range <>,  -- Address, specifies one word` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug18659/crash.vhd) |
| 542 | `, ID "` | 1 | `X"3f", X"1d", X"11", X"5f", X"6f", X"00", X"1a", X"0f", X"1e", X"a8", X"2e", X"61", X"32", X"bc", X"79",` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug040/tb.vhd) |
| 543 | `, ID :` | 1 | `a, x : in;                      -- Error` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_45a.vhd) |
| 544 | `, NUM )` | 1 | `type er1 is array(5,2) of integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc327.vhd) |
| 545 | `, NUM ;` | 1 | `constant v1: x1 := 0, 0;   -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2376.vhd) |
| 546 | `- : ID` | 1 | `variable k- : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2591.vhd) |
| 547 | `- ID (` | 1 | `A(1 to 2) := A(1) & - A(2);  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2191.vhd) |
| 548 | `- ID )` | 1 | `if ((x + -z) < (y + x)) then  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1802.vhd) |
| 549 | `- ID <=` | 1 | `TWO - ONE <= S1;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1280.vhd) |
| 550 | `- NUM /=` | 1 | `assert A * -5 /= 0;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2466/t1.vhdl) |
| 551 | `-- -- --` | 1 | `-------------------------------------------------------------------------` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket89/project/src93/adaptations_pkg.vhd) |
| 552 | `-- use ID` | 1 | `-- use model instance label (preferred if set as entityname_1)` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/src/AxiStreamTransmitter.vhd) |
| 553 | `. . .` | 1 | `type t is . . .;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/subprograms/ent.vhd) |
| 554 | `. : ID` | 1 | `variable k. : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2603.vhd) |
| 555 | `. ID +` | 1 | `sum := sum + r(i).x + r(i).y;` (stimuli/vhdl/subs/nvc/test/regress/conv8.vhd) |
| 556 | `. ID is` | 1 | `configuration foo_cfg of foo.foo_e is` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_23.vhd) |
| 557 | `. ID report` | 1 | `assert OUTPUT = Vectors(i).OUTPUT_test` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug01/tb.vhdl) |
| 558 | `. NUM ID` | 1 | `constant d : time := 1.5e2 c'length;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue886/repro2.vhdl) |
| 559 | `. NUM after` | 1 | `sr <= 3.2    after 1 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc3047.vhd) |
| 560 | `. NUM loop` | 1 | `while v > 1.0 loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1051/psi_common_math_pkg.vhd) |
| 561 | `. all ;` | 1 | `return split_data_type(1).all;` (stimuli/vhdl/subs/vunit/vunit/vhdl/data_types/src/data_types_private_pkg.vhd) |
| 562 | `/ : ID` | 1 | `variable k/ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2604.vhd) |
| 563 | `: ; begin` | 1 | `variable based_int : integer := 3#12:;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2855.vhd) |
| 564 | `: = '` | 1 | `constant c1: character: ='A';   -- failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2577.vhd) |
| 565 | `: ID +` | 1 | `MEMORYV : MEMORYV + MEMORYV;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2044.vhd) |
| 566 | `: ID --` | 1 | `-- PROCESS: p_main` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket89/project/tb/partial_test_tb.vhd) |
| 567 | `: ID =` | 1 | `if l1: SPEED = "fast" generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0104/alt2.vhdl) |
| 568 | `: NUM #` | 1 | `variable based_int : integer := 3:12#;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2856.vhd) |
| 569 | `: NUM .` | 1 | `total_time := 5:1234.4321:E-10;      -- no_failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2861.vhd) |
| 570 | `: NUM :` | 1 | `CONSTANT g : INTEGER := 2:1_0:;` (stimuli/vhdl/subs/nvc/test/parse/based.vhd) |
| 571 | `: assert (` | 1 | `assert_a: assert ((a(3) or b(3)) = '1') report "a";` (stimuli/vhdl/subs/nvc/test/regress/ieee1.vhd) |
| 572 | `: block --` | 1 | `B:block  -- ERROR:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1697.vhd) |
| 573 | `: block is` | 1 | `b1: block is                    -- OK` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_07.vhd) |
| 574 | `: loop if` | 1 | `SYNC_LOOP: loop` (stimuli/vhdl/subs/nvc/test/regress/issue428.vhd) |
| 575 | `: null ;` | 1 | `v: null;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue140/repro.vhdl) |
| 576 | `: report "` | 1 | `rpt1: report "Report within GLOBAL procedure";` (stimuli/vhdl/subs/nvc/test/regress/cover1.vhd) |
| 577 | `: type ID` | 1 | `generic (period : time :type ns);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug090/crash14.vhdl) |
| 578 | `: wait ;` | 1 | `block_forever: wait;` (stimuli/vhdl/subs/nvc/test/parse/seq.vhd) |
| 579 | `: while (` | 1 | `L1: while  ( i < 10 )  loop       -- conditional loop` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1703.vhd) |
| 580 | `: while not` | 1 | `command_loop : while not endfile(control) loop` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_18_fg_18_10.vhd) |
| 581 | `:= ' NUM` | 1 | `V1(2 := '1' ;  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1036.vhd) |
| 582 | `; ( ID` | 1 | `all_possible_values ( test_vector, 10 ns );` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/digital-modeling/tb_full_adder.vhd) |
| 583 | `; -- alias` | 1 | `alias ground is electrical_ref;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/aliases/inline_01a.vhd) |
| 584 | `; -- all` | 1 | `proc;                           -- all unassociated` (stimuli/vhdl/subs/nvc/test/regress/issue227.vhd) |
| 585 | `; -- assert` | 1 | `check (foo);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1273/issue.vhdl) |
| 586 | `; : ID` | 1 | `variable k; : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2598.vhd) |
| 587 | `; ID :=` | 1 | `constant empty ; bit_vector := O"058";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2763.vhd) |
| 588 | `; ID ;` | 1 | `variable s_day ; day;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1506.vhd) |
| 589 | `; begin report` | 1 | `file f : text;` (stimuli/vhdl/subs/nvc/test/regress/stdenv2.vhd) |
| 590 | `; begin return` | 1 | `file fref, fref2: text;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1936/repro.vhdl) |
| 591 | `; end block` | 1 | `q_zd <= next_count when clk_ipd = '1' and clk_ipd'event;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_fg_16_13.vhd) |
| 592 | `; end generate` | 1 | `assign_row(DataOut_i, Shifter_nxt(TAPS(i)), i);` (stimuli/vhdl/subs/PoC/src/misc/misc_Delay.vhdl) |
| 593 | `; end process` | 1 | `x := ;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/varcomp.vhdl) |
| 594 | `; file ID` | 1 | `file f0 : text;` (stimuli/vhdl/subs/vunit/vunit/vhdl/logging/src/file_pkg.vhd) |
| 595 | `; impure function` | 1 | `procedure Inc ;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue290/TbNamesPkg.vhd) |
| 596 | `; process begin` | 1 | `clk <= not clk after 10 ns when now < 50 ns;` (stimuli/vhdl/subs/nvc/test/regress/elab26.vhd) |
| 597 | `; process constant` | 1 | `simInitialize;` (stimuli/vhdl/subs/PoC/tb/common/physical_tb.vhdl) |
| 598 | `; shared variable` | 1 | `alias T_SCOREBOARD is P_Scoreboard.ScoreBoardPType;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue202/repro.vhdl) |
| 599 | `< : ID` | 1 | `variable k< : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2605.vhd) |
| 600 | `<< << <<` | 1 | `<<<<<<< HEAD` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/merge1.vhdl) |
| 601 | `<< constant @` | 1 | `assert << constant @work.ename.c : integer >> = 42;` (stimuli/vhdl/subs/nvc/test/perf/ename.vhd) |
| 602 | `<< constant ID` | 1 | `assert << constant g(1).uut.k : integer >> = 1;` (stimuli/vhdl/subs/nvc/test/regress/ename3.vhd) |
| 603 | `<< constant ^` | 1 | `assert << constant ^.^.g(N).uut.k : integer >> = N;` (stimuli/vhdl/subs/nvc/test/regress/ename4.vhd) |
| 604 | `<< signal ^` | 1 | `assert <<signal ^.^.^.^.x : bit>> = '0';  -- Error` (stimuli/vhdl/subs/nvc/test/regress/ename17.vhd) |
| 605 | `<< variable ID` | 1 | `assert <<variable uut.x : integer>> = 0;  -- Error` (stimuli/vhdl/subs/nvc/test/regress/ename10.vhd) |
| 606 | `<= ( ID` | 1 | `res_assign: s <= (a xor b) xor cin;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1817/full_adder.vhdl) |
| 607 | `<= ( NUM` | 1 | `p1: s <= (1 to i => '1') & (i + 1 to 3 => '0');` (stimuli/vhdl/subs/nvc/test/regress/issue825.vhd) |
| 608 | `<= ID "` | 1 | `assign_p: vec <= X"52";` (stimuli/vhdl/subs/nvc/test/regress/signal9.vhd) |
| 609 | `<= ID '` | 1 | `delayed1: i2 <= i1'delayed(5 ns);` (stimuli/vhdl/subs/nvc/test/regress/implicit9.vhd) |
| 610 | `<= ID after` | 1 | `behavior : q <= d after Tpd_clk_q when clk = '1' and clk'event;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_12_fg_12_02.vhd) |
| 611 | `<= transport '` | 1 | `CONSIG: A <= transport '1' when B = '1' else` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1758.vhd) |
| 612 | `= ' NUM` | 1 | `k.x = '0';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc510.vhd) |
| 613 | `= : ID` | 1 | `variable k= : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2592.vhd) |
| 614 | `= ID ;` | 1 | `variable b1, b2 : boolean ;= true;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1492.vhd) |
| 615 | `=> null :` | 1 | `when   => NULL:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1469.vhd) |
| 616 | `> " ID` | 1 | `proc(s(4 to 15) => "Hello world!");` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket73/bug.vhdl) |
| 617 | `> " NUM` | 1 | `test(1) => "00",` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2529/outer.vhdl) |
| 618 | `> : ID` | 1 | `variable k> : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2606.vhd) |
| 619 | `> ID &` | 1 | `x(3 downto 2) => testab & '0',` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3201/tb.vhdl) |
| 620 | `> ID <=` | 1 | `S1 > S2   <= S3;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1275.vhd) |
| 621 | `> not ID` | 1 | `assert always (not rst and wr -> not is_x(d))@rising_edge(clkw)` (stimuli/vhdl/subs/vunit/examples/vhdl/array_axis_vcs/src/fifo.vhd) |
| 622 | `> open )` | 1 | `output2_o(0) => open` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2764/tb.vhdl) |
| 623 | `? /= '` | 1 | `check_equal('1' ?/= '1', '0');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_ne.vhd) |
| 624 | `? /= ID` | 1 | `z <= "1--0" ?/= a;` (stimuli/vhdl/subs/ghdl/testsuite/synth/match01/match07.vhdl) |
| 625 | `? : ID` | 1 | `variable k? : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2607.vhd) |
| 626 | `?< ' NUM` | 1 | `check_equal('1' ?< '1', '0');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_lt.vhd) |
| 627 | `?< = '` | 1 | `check_equal('1' ?<= '1', '1');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_le.vhd) |
| 628 | `?= " "` | 1 | `subtype bv4 is bit_vector(1 to 4);type vec0 is array(natural range<>)of bv4;constant s:vec0:=(x"0",""?="");procedure pri` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/eval1.vhdl) |
| 629 | `?= " -` | 1 | `z <= a ?= "-1--0";` (stimuli/vhdl/subs/ghdl/testsuite/synth/match01/match04.vhdl) |
| 630 | `?= " ID` | 1 | `r := z ?= "LL";` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue14/repro.vhdl) |
| 631 | `?= ID "` | 1 | `z <= a ?= b"-10-_1111_1110_1101_1100_1011_1010_1001_1000_0--1";` (stimuli/vhdl/subs/ghdl/testsuite/synth/match01/match03.vhdl) |
| 632 | `?= ID )` | 1 | `check_relation(sl_1 ?= sl_0);` (stimuli/vhdl/subs/vunit/vunit/vhdl/check/test/tb_check_relation_2008p.vhd) |
| 633 | `?= ID or` | 1 | `if ( wr and ( d?=dx or d?=du ) ) then` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1731/fifo.vhdl) |
| 634 | `?> ' NUM` | 1 | `check_equal('1' ?> '1', '0');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_gt.vhd) |
| 635 | `?> = NUM` | 1 | `assert (u5 ?>= 4) = '1' severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper03/match01.vhdl) |
| 636 | `?? ID '` | 1 | `check_equal(?? std_logic'('1'), true);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_condition_operator.vhd) |
| 637 | `@ ID :` | 1 | `variable @k : integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2665.vhd) |
| 638 | `@ for (` | 1 | `architecture@for(""x""4000000000x"` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash15.vhdl) |
| 639 | `ID " -` | 1 | `function "-" (R : AlertCountType) return AlertCountType ;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1980/AlertLogPkg.vhd) |
| 640 | `ID " default` | 1 | `type array3 is array (10 downto 1) of real tolerance "default";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/composite-data/inline_14a.vhd) |
| 641 | `ID ( "` | 1 | `echo("1");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue394/bug.vhdl) |
| 642 | `ID ) mod` | 1 | `RETVAL := ( ( abs ARG ) mod 10 ) + 1980;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc879.vhd) |
| 643 | `ID ) report` | 1 | `assert (xo = xor xi) report "mixmatch at i=" & integer'image(i) severity ERROR;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2614/recursive_xor_tb.vhdl) |
| 644 | `ID ) return` | 1 | `function F1 ( A : linkage integer ) return boolean is -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2894.vhd) |
| 645 | `ID ) when` | 1 | `iIntReq   <= TO_X01(or IntReq) when TransactionPending(InterruptRec.Rdy, InterruptRec.Ack)` (stimuli/vhdl/subs/OsvvmLibraries/Common/src/deprecated/InterruptHandler_c.vhd) |
| 646 | `ID - ID` | 1 | `ghdl -r -gG_RST=1 toto` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3011/toto_vec.vhdl) |
| 647 | `ID -- !` | 1 | `type Data_Type_t                  --! The data type of the input and output.` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2544/generic_fixed_delay.vhdl) |
| 648 | `ID -- =` | 1 | `-- Type declarations` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/utils.vhdl) |
| 649 | `ID . --` | 1 | `error.` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2770.vhd) |
| 650 | `ID . NUM` | 1 | `constant R : real := 10_000.0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/analog-modeling/inline_10a.vhd) |
| 651 | `ID . all` | 1 | `use work.c10s05b00x00p03n02i00941pkg1.all, work c10s05b00x00p03n02i00941pkg2.all;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc941.vhd) |
| 652 | `ID /= NUM` | 1 | `assert B ** abs A /= 0;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2466/t4.vhdl) |
| 653 | `ID : buffer` | 1 | `component A2 generic (constant G2 : buffer BOOLEAN);  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc191.vhd) |
| 654 | `ID : generate` | 1 | `else g2: generate` (stimuli/vhdl/subs/nvc/test/regress/elab34.vhd) |
| 655 | `ID : linkage` | 1 | `component A2 generic (constant G2 : linkage BOOLEAN);  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc192.vhd) |
| 656 | `ID := "` | 1 | `port(	S 	: string 	:= "abcdef");` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug018/repro.vhdl) |
| 657 | `ID ; entity` | 1 | `end pkg;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue321/repro.vhdl) |
| 658 | `ID ; library` | 1 | `end test_pkg;` (stimuli/vhdl/subs/nvc/test/regress/issue327.vhd) |
| 659 | `ID <= <<` | 1 | `test_signal <= <<signal .issue1062.i_nvc_crash.test_signal : natural>>;` (stimuli/vhdl/subs/nvc/test/regress/issue1062.vhd) |
| 660 | `ID <= NUM` | 1 | `x <= 5;` (stimuli/vhdl/subs/nvc/test/sem/generate.vhd) |
| 661 | `ID <= not` | 1 | `osc  clk <= not clk after period/2;    -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1741.vhd) |
| 662 | `ID = '` | 1 | `assert and v = '0';` (stimuli/vhdl/subs/nvc/test/regress/logical3.vhd) |
| 663 | `ID ID :=` | 1 | `procedure tbFail(msg : in string := "") is` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket32/simulation.vhdl) |
| 664 | `ID ID <=` | 1 | `with B` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1764.vhd) |
| 665 | `ID ID =>` | 1 | `Rmd <= (Rmd'left downto N => '0') & unsigned(arg);` (stimuli/vhdl/subs/PoC/src/arith/arith_sqrt.vhdl) |
| 666 | `ID ID signal` | 1 | `-- 1.	you MUST initialize your matrix signal with 'Z' to get correct simulation results (iSIM, vSIM, ghdl/gtkwave)` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/vectors.vhdl) |
| 667 | `ID NUM )` | 1 | `, sel( 3 downto 0)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2549/pkg_types.vhd) |
| 668 | `ID downto ID` | 1 | `variable init_array      : t_sf_array(0 to samples - 1)(ret_type'left downto ret_type'right) := (others => (others => '0` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue339/test_pkg.vhdl) |
| 669 | `ID function ID` | 1 | `pure function get_wb_rst( wb_i : wb_M2S_t)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1346/wb_rst_pkg.vhd) |
| 670 | `ID inertial '` | 1 | `x <= reject v inertial '1' after 2 ns;` (stimuli/vhdl/subs/nvc/test/regress/delay3.vhd) |
| 671 | `ID is (` | 1 | `type T_Command is (<>); --> Not yet supported by Riviera-PRO 2020.04` (stimuli/vhdl/subs/Interfaces/PoC/CSE.vhdl) |
| 672 | `ID is <>` | 1 | `type t1 is <>;` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_59.vhd) |
| 673 | `ID is protected` | 1 | `type AlertLogStructPType is protected` (stimuli/vhdl/subs/nvc/test/sem/osvvm1.vhd) |
| 674 | `ID is record` | 1 | `type evs_type is record` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2713/reprod.vhdl) |
| 675 | `ID null ;` | 1 | `if  (I /= 47) the` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1434.vhd) |
| 676 | `ID package ;` | 1 | `end package;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2/sortnet_OddEvenSort.vhdl) |
| 677 | `ID process variable` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue20/fp_write_read_issue.vhdl) |
| 678 | `ID range '` | 1 | `subtype GROUND BIT range '0' to '0';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc11.vhd) |
| 679 | `ID select ?` | 1 | `with x select? y <=` (stimuli/vhdl/subs/nvc/test/regress/issue705.vhd) |
| 680 | `ID { ID` | 1 | `FELL_0_a : assert always {a; not a} |-> fell(a);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue662/psl_fell_err1.vhdl) |
| 681 | `NUM ' '` | 1 | `variable k : bit_vector(0 to 7) := b'0''1''0''1''0''1''0''1';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2746.vhd) |
| 682 | `NUM ( "` | 1 | `architecture 0for(""x""4000000000x"` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash16.vhdl) |
| 683 | `NUM ( NUM` | 1 | `architecture 0for(4000000000x"` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash11.vhdl) |
| 684 | `NUM , NUM` | 1 | `(0, 0, 0) := (0, 0, 0);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1352.vhd) |
| 685 | `NUM /= NUM` | 1 | `assert A * abs 5 /= 0;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2466/t2.vhdl) |
| 686 | `NUM : ID` | 1 | `variable 01k : integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2609.vhd) |
| 687 | `NUM ; assert` | 1 | `INTV2 := abs 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2324.vhd) |
| 688 | `NUM ; begin` | 1 | `constant x : integer := abs 10;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2307.vhd) |
| 689 | `NUM ; wait` | 1 | `k <= transport 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1323.vhd) |
| 690 | `NUM => NUM` | 1 | `constant C1 : T1 := (1 2 => 0, others => 4) ; -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2381.vhd) |
| 691 | `NUM => null` | 1 | `1 => NULL;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1468.vhd) |
| 692 | `NUM inertial NUM` | 1 | `x <= reject 5 inertial 7;           -- Reject not time` (stimuli/vhdl/subs/nvc/test/sem/conc.vhd) |
| 693 | `NUM select ID` | 1 | `with 2 select b <=` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue9/repro2.vhdl) |
| 694 | `NUM when NUM` | 1 | `j <= transport 1 when 1,` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1772.vhd) |
| 695 | `[ * NUM` | 1 | `architecture restrict[*9000000000` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash12.vhdl) |
| 696 | `[ : ID` | 1 | `variable k[ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2596.vhd) |
| 697 | `\ : ID` | 1 | `variable k\ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2608.vhd) |
| 698 | `\ : \` | 1 | `signal \:\ : xyz := (others => (others => '1'));` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue3232/tb.vhdl) |
| 699 | `\ ID .` | 1 | `signal \extend.id\ : std_ulogic := 'H';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1228/test_load.vhdl) |
| 700 | `\ ID <` | 1 | `port ( \a<0>\, \a<1>\, \a<2>\, \a<3>\ : in std_ulogic;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ap_a_fg_a_11.vhd) |
| 701 | `\ ID ID` | 1 | `signal \foo bar\ : integer;` (stimuli/vhdl/subs/nvc/test/parse/extended.vhd) |
| 702 | `] : ID` | 1 | `variable k] : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2597.vhd) |
| 703 | `^ : ID` | 1 | `variable k^ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2584.vhd) |
| 704 | `` : ID` | 1 | `variable k` : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2593.vhd) |
| 705 | `` ID ID` | 1 | ``protect begin_protected` (stimuli/vhdl/subs/nvc/test/parse/protect1.vhd) |
| 706 | `` entity ID` | 1 | ``entity fum is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug099/repro.vhdl) |
| 707 | `access ; --` | 1 | `type ADDRESS is access ; --- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc514.vhd) |
| 708 | `alias variable ID` | 1 | `alias` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/alias1.vhdl) |
| 709 | `and ID <=` | 1 | `S1 and S2   <= S3;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1276.vhd) |
| 710 | `architecture ID ;` | 1 | `end architecture tb;` (stimuli/vhdl/subs/nvc/test/regress/issue1472.vhd) |
| 711 | `assert ID '` | 1 | `assert CHARACTER'('1')/=BIT'('1') report "oops"; -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2928.vhd) |
| 712 | `assert ID <` | 1 | `postponed assert x < 10;` (stimuli/vhdl/subs/nvc/test/regress/assert5.vhd) |
| 713 | `assert ID >=` | 1 | `assert V2 >= 10;  -- failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2994.vhd) |
| 714 | `assert ID severity` | 1 | `assert false severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2209/e.vhdl) |
| 715 | `assert ID {` | 1 | `postponed assert always {s = '0'; s = '1'} severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug051/tb2.vhdl) |
| 716 | `assert not (` | 1 | `assert NOT( gimme_value = 10 )` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc893.vhd) |
| 717 | `attribute ID ID` | 1 | `attribute p  POSITIVE;         --Failure Here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc181.vhd) |
| 718 | `begin   ID` | 1 | `package function begin n';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash24.vhdl) |
| 719 | `begin ; architecture` | 1 | `BEGIN` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc714.vhd) |
| 720 | `begin ID (` | 1 | `begin` (stimuli/vhdl/subs/nvc/test/regress/proc12.vhd) |
| 721 | `begin NUM` | 1 | `package body function begin 0package` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash28.vhdl) |
| 722 | `begin end block` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1656.vhd) |
| 723 | `begin exit ;` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc802.vhd) |
| 724 | `begin generic (` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc795.vhd) |
| 725 | `begin if (` | 1 | `begin` (stimuli/vhdl/subs/PoC/src/arith/arith_shifter_barrel.vhdl) |
| 726 | `begin next ;` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc801.vhd) |
| 727 | `begin port (` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc794.vhd) |
| 728 | `begin process begin` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc793.vhd) |
| 729 | `begin process variable` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/synth/attr02/name02.vhdl) |
| 730 | `begin return ID` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1621.vhd) |
| 731 | `begin wait ;` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1696.vhd) |
| 732 | `begin wait NUM` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc796.vhd) |
| 733 | `block generate signal` | 1 | `signal g : std_logic_vector(K-1 downto 1);  -- Block Generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/arith_addw.vhdl) |
| 734 | `bus := '` | 1 | `signal s : std_logic bus := 'H';` (stimuli/vhdl/subs/nvc/test/regress/guard3.vhd) |
| 735 | `bus ; --` | 1 | `signal s1 : bit bus; -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc74.vhd) |
| 736 | `bus ; disconnect` | 1 | `signal a : std_logic bus;` (stimuli/vhdl/subs/nvc/test/regress/issue829.vhd) |
| 737 | `component ; end` | 1 | `end component;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc786.vhd) |
| 738 | `component ID --` | 1 | `component C  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2926.vhd) |
| 739 | `configuration ID of` | 1 | `configuration C of E is               -- component illegal here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc785.vhd) |
| 740 | `constant ID ,` | 1 | `CONSTANT a,b    : INTEGER := 2;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc138.vhd) |
| 741 | `disconnect all :` | 1 | `disconnect all : BIT after 0 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1695.vhd) |
| 742 | `disconnect architecture ID` | 1 | `disconnect` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug090/hang2.vhdl) |
| 743 | `downto ID +` | 1 | `SLV1 <= (SLV1'left downto Index+1 => '0') & '1' & (Index-1 downto 0 => '0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2581/aggr.vhdl) |
| 744 | `downto ID =>` | 1 | `x <= (31 downto WIDTH => '0') & y1;` (stimuli/vhdl/subs/nvc/test/regress/elab20.vhd) |
| 745 | `downto NUM )` | 1 | `alias  address_ms       : std_logic_vector(3 downto 0)   is   processor_address(15 downto 12);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug18280/alias_bug.vhd) |
| 746 | `end ; elsif` | 1 | `end;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue106/ent2.vhdl) |
| 747 | `end ; entity` | 1 | `end;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2984.vhd) |
| 748 | `end block ;` | 1 | `end block;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2448/debouncer_no_vunit_tb.vhdl) |
| 749 | `end block ID` | 1 | `end block BL;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1663.vhd) |
| 750 | `end if ;` | 1 | `end if;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/proc4.vhdl) |
| 751 | `end loop ;` | 1 | `end loop;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1556.vhd) |
| 752 | `entity ID (` | 1 | `entity E (PT:BIT) is   -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2924.vhd) |
| 753 | `file ID ;` | 1 | `type A1 is file ARR;     -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc543.vhd) |
| 754 | `for ID (` | 1 | `for bit_array ( 0 to n - 1 )` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/generators/last_pass_spice.vhd) |
| 755 | `for ID use` | 1 | `for c01s03b00x00p02n01i00829_arch` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc829.vhd) |
| 756 | `function " +` | 1 | `function "+" (I1:Bit) return bit;  --- No_Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2874.vhd) |
| 757 | `function " abs` | 1 | `function "abs " return real is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2875.vhd) |
| 758 | `function ; function` | 1 | `end function;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2/sortnet_OddEvenSort_tb.vhdl) |
| 759 | `generate -- use` | 1 | `elsif (DEV_INFO.Vendor = VENDOR_XILINX) generate` (stimuli/vhdl/subs/PoC/src/sync/sync_Bits.vhdl) |
| 760 | `generate ID :` | 1 | `else generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2564/mwe.vhdl) |
| 761 | `generate signal ID` | 1 | `else generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2614/recursive_xor.vhdl) |
| 762 | `generic ( procedure` | 1 | `generic (procedure preal(value : out real);` (stimuli/vhdl/subs/nvc/test/regress/gensub6.vhd) |
| 763 | `generic ( pure` | 1 | `generic ( pure function f1 (x, y : integer) return integer );  -- OK` (stimuli/vhdl/subs/nvc/test/parse/issue917.vhd) |
| 764 | `if ; assert` | 1 | `en if;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1439.vhd) |
| 765 | `in " ID` | 1 | `report "***FAILED TEST: c08s04b00x00p04n01i01290 - A port whose mode is "IN" can not be on the left-hand side of a signa` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1290.vhd) |
| 766 | `in " or` | 1 | `report "***FAILED TEST: c08s04b00x00p04n01i01284 - A port whose mode is "IN" or "LINKAGE" can not be on the left-hand si` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1284.vhd) |
| 767 | `is ; NUM` | 1 | `architecture function is;0package` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash18.vhdl) |
| 768 | `is NUM` | 1 | `architecture function is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash17.vhdl) |
| 769 | `is begin report` | 1 | `process (vec(0)) is` (stimuli/vhdl/subs/nvc/test/regress/attr7.vhd) |
| 770 | `is end ;` | 1 | `context b is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash23_1.vhdl) |
| 771 | `is private )` | 1 | `generic ( g1 : type is private );   -- OK` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_16.vhd) |
| 772 | `library ID ;` | 1 | `library work;      -- ERROR:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc3015.vhd) |
| 773 | `library use ID` | 1 | `library use T.context is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash23.vhdl) |
| 774 | `linkage " ID` | 1 | `report "***FAILED TEST: c08s04b00x00p04n01i01285 - A port whose mode is "LINKAGE" can not be on the left-hand side of a ` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1285.vhd) |
| 775 | `loop -- ID` | 1 | `for i in 4 loop                 -- Error` (stimuli/vhdl/subs/nvc/test/parse/range1.vhd) |
| 776 | `loop ; assert` | 1 | `end loop;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1584.vhd) |
| 777 | `loop ID (` | 1 | `package function is loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash34.vhdl) |
| 778 | `loop ID :=` | 1 | `for foo in integer range MY_WORD'range loop` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1516.vhd) |
| 779 | `loop ID <=` | 1 | `for i in natural range 0 to 2**N-1 loop` (stimuli/vhdl/subs/PoC/tb/arith/firstone/arith_firstone_Simple.vhdl) |
| 780 | `loop for ID` | 1 | `for a in std_ulogic range '0' to '1' loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2486/bug3.vhdl) |
| 781 | `null : assert` | 1 | `NULL:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1441.vhd) |
| 782 | `of ID ;` | 1 | `function resize(x : std_logic_vector) return rv_t of std_logic_vector ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_072b.vhd) |
| 783 | `or ID :` | 1 | `for TestCtrl_1 : TestCtrl` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/TestCases/TbStream_AxiSendGetRandom1.vhd) |
| 784 | `others => ID` | 1 | `arr <= (B"11111111" others => B"00000000");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1500/test.vhdl) |
| 785 | `out NUM ;` | 1 | `s <= force out 1;               -- OK` (stimuli/vhdl/subs/nvc/test/sem/force.vhd) |
| 786 | `procedure " and` | 1 | `procedure "and";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2873.vhd) |
| 787 | `process ; end` | 1 | `end process;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1438.vhd) |
| 788 | `process ID ;` | 1 | `END PROCESS TESTING;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1616.vhd) |
| 789 | `process begin wait` | 1 | `process` (stimuli/vhdl/subs/nvc/test/regress/cover28.vhd) |
| 790 | `protected -- -` | 1 | `type RandomPType is protected` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/RandomPkg.vhd) |
| 791 | `protected generic (` | 1 | `type test_t is protected` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_034.vhd) |
| 792 | `protected private variable` | 1 | `type t_test1 is protected` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_33.vhd) |
| 793 | `protected type ID` | 1 | `type p is protected` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue470/e.vhdl) |
| 794 | `range ( ID` | 1 | `type t3 is range (ns/fs) downto (fs/fs);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/disputed/tc238.vhd) |
| 795 | `range - (` | 1 | `type my_empty_range is range -(-8) to 7;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue810/nullrng2.vhdl) |
| 796 | `range <> )` | 1 | `a:c;n:c;s:s;t:std_logic;end record;type fa_array is array(0 range<>)of rc_data;constant e:fa_array:=(('0','0','0','%'),(` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash45.vhdl) |
| 797 | `range NUM #` | 1 | `type   INIT_1 is range 16#1# to 16#FF#;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1282.vhd) |
| 798 | `record ; --` | 1 | `end record;  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc502.vhd) |
| 799 | `record ; begin` | 1 | `end record;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/emptyrec.vhdl) |
| 800 | `record ; constant` | 1 | `type t_data_segment is record end record;` (stimuli/vhdl/subs/nvc/test/parse/aggregate2.vhd) |
| 801 | `record is ID` | 1 | `type t_frame_x record is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1196/my_pkg.vhdl) |
| 802 | `register ; --` | 1 | `signal s1 : bit register; -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc73.vhd) |
| 803 | `report " ID` | 1 | `report "This should not get through";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1241.vhd) |
| 804 | `restrict [ *` | 1 | `entity begin restrict[*to 0` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2110/psl1.vhdl) |
| 805 | `return ID .` | 1 | `package function return g.b of` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2110/retid.vhdl) |
| 806 | `return of` | 1 | `package function return of` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash31.vhdl) |
| 807 | `rol ID (` | 1 | `lo <= v rol to_integer(q);` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper02/urot02.vhdl) |
| 808 | `ror ID *` | 1 | `dst <= src ror step*STEP_SIZE;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2159/bug.vhdl) |
| 809 | `ror ID ;` | 1 | `ror_val  <= input ror i;` (stimuli/vhdl/subs/ghdl/testsuite/gna/sr2737/testit.vhdl) |
| 810 | `select ID <=` | 1 | `with (cnt) select res_2 <=` (stimuli/vhdl/subs/nvc/test/regress/cover4.vhd) |
| 811 | `signal ID ,` | 1 | `signal b1,b2,b3,b4,b5,b6,b7 : boolean;` (stimuli/vhdl/subs/nvc/test/bounds/bounds2.vhd) |
| 812 | `sla NUM =` | 1 | `assert l sla 1 = x"8" severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper02/shift02.vhdl) |
| 813 | `sll ID (` | 1 | `temp_result := bv_s1 sll bv_to_natural(bv_s2(27 to 31));` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_15_alu-b.vhd) |
| 814 | `sll NUM ;` | 1 | `uns_sll <= resize (uns_inp, sgn_srl'length) sll 1; -- work` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1264/issue2.vhdl) |
| 815 | `srl ( ID` | 1 | `dout <= din srl (ch + 1);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1218/top.vhdl) |
| 816 | `then -- ID` | 1 | `if TRUE then                           -- illegal location for if statement` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc822.vhd) |
| 817 | `to ID '` | 1 | `to sub_int'high => assert (false)` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1499.vhd) |
| 818 | `to ID (` | 1 | `when integer(REGS.C) to integer(REGS.D) => sig <= 1;` (stimuli/vhdl/subs/nvc/test/simp/issue1353.vhd) |
| 819 | `to ID )` | 1 | `subtype char128 is character range (NUL to DEL);  -- Error` (stimuli/vhdl/subs/nvc/test/sem/error2.vhd) |
| 820 | `to NUM |` | 1 | `v17 := (1 to 0 | p => 17);  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2449.vhd) |
| 821 | `transport ; assert` | 1 | `T1   <=   1 after 10 ns transport;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1272.vhd) |
| 822 | `type ID ;` | 1 | `type treeNode;` (stimuli/vhdl/subs/nvc/test/perf/binarytrees.vhd) |
| 823 | `type is private` | 1 | `A : type is private ;   -- any type` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_016.vhd) |
| 824 | `units -- ID` | 1 | `units  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc266.vhd) |
| 825 | `until ( ID` | 1 | `wait for 60 ns until (k = 1);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1186.vhd) |
| 826 | `vunit ID {` | 1 | `vunit issue_1899_vu0 {` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1899/issue1899.vhdl) |
| 827 | `wait on ID` | 1 | `wait on value;` (stimuli/vhdl/subs/vunit/vunit/vhdl/verification_components/src/std_logic_checker.vhd) |
| 828 | `when ID '` | 1 | `constant c0 : integer := 10 when cond5'path_name = ":cond5:" else 5;` (stimuli/vhdl/subs/nvc/test/regress/cond5.vhd) |
| 829 | `when ID .` | 1 | `next when CurFG.Active = FALSE;` (stimuli/vhdl/subs/PoC/src/bus/stream/stream_Source.vhdl) |
| 830 | `when ID /=` | 1 | `p_actor := actor when actor /= null_actor else new_actor;` (stimuli/vhdl/subs/vunit/vunit/vhdl/verification_components/src/bus_master_pkg-body.vhd) |
| 831 | `when ID ;` | 1 | `next when FALSE;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1576.vhd) |
| 832 | `when ID <=` | 1 | `m := a when a <= b else b;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2045/ghdlcrash.vhdl) |
| 833 | `when ID else` | 1 | `r := t when c else f;` (stimuli/vhdl/subs/nvc/test/parse/issue1188.vhd) |
| 834 | `when ID rem` | 1 | `next when tmp1 rem 2 = 0;` (stimuli/vhdl/subs/nvc/test/regress/issue676.vhd) |
| 835 | `when NUM >` | 1 | `constant c1 : integer := 1 when 1 > 2 else 5;` (stimuli/vhdl/subs/nvc/test/simp/condexpr.vhd) |
| 836 | `{ : ID` | 1 | `variable k{ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2594.vhd) |
| 837 | `| : ID` | 1 | `variable k| : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2610.vhd) |
| 838 | `} : ID` | 1 | `variable k} : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2595.vhd) |
| 839 | `~ : ID` | 1 | `variable k~ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2590.vhd) |
| 840 | `Å ID ß` | 1 | `constant Åxyzß : bit := '1';          -- Warning` (stimuli/vhdl/subs/nvc/test/charset/utf8.vhd) |
| 841 | `‘ NUM ’` | 1 | `if (CLEAR = ‘1’) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug063/dff.vhdl) |
| 842 | `﻿ -- ID` | 1 | `﻿-- This is utf-8 encoding, with a BOM.` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket35/utf8.vhdl) |

