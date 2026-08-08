# `vhdl` stuck-point clusters (SV-CORPUS-GRAD.3.0 engine; family lane CORPUS-GRAD-ALL.2.1)

9689 rows probed from the `fail` rows of `results.tsv`; 861 distinct 3-token stuck signatures. Keyed on `furthest_position` (the DEEP locus), family profile `vhdl`, case-folded.

> ⚠️ **RAW-FAIL lane — a cluster SIZES a candidate class, it does not adjudicate one.** Some corpus files are intentionally invalid, so a fail can be the CORRECT outcome. Expected verdicts come from the LRM / suite metadata, never from what the parser does today.

| # | signature | rows | example (stuck line) |
|---|---|---|---|
| 1 | `ID ; ID` | 429 | `x   :   bit_vector ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_055a.vhd) |
| 2 | `until ID =` | 387 | `wait until nReset = '1' ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_AlertLogIDManager.vhd) |
| 3 | `range NUM to` | 357 | `type FREQ is range 0 to integer'high units` (stimuli/vhdl/subs/PoC/src/common/physical.vhdl) |
| 4 | `ID : ID` | 293 | `impure function to_string(variable value : value_mirror) return string ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_041.vhd) |
| 5 | `ID ( NUM` | 256 | `return new string(1 to 1000000) ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_030.vhd) |
| 6 | `after NUM ID` | 256 | `Clk <= not Clk after 10 ns ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/demo/AlertLog_Demo_Global.vhd) |
| 7 | `ID ; begin` | 252 | `end proc;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/emptyquote.vhdl) |
| 8 | `. NUM ;` | 244 | `signal TxIdleProbability : real := 0.05 ; -- 5 %` (stimuli/vhdl/subs/OsvvmLibraries/UART/testbench_multiple_uarts/TbUart_MultipleProcess_1.vhd) |
| 9 | `file of ID` | 219 | `type T_PICFILE is file of character;` (stimuli/vhdl/subs/OsvvmLibraries/VideoBus_LouisAdriaens/src/bmp_pack.vhd) |
| 10 | `ID : in` | 177 | `constant Address         : In  std_logic_vector ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/common/src/Axi4ModelPkg.vhd) |
| 11 | `attribute ID :` | 174 | `attribute KEEP : boolean;` (stimuli/vhdl/subs/PoC/src/arith/arith_trng.vhdl) |
| 12 | `ID ( ID` | 165 | `function operation (value: integer) return integer);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_function_generic_in_package.vhd) |
| 13 | `( ID )` | 164 | `alias  WriteAddressDelayCov  is ArrDelayCovID(WRITE_ADDRESS_ID) ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Axi4Manager_a.vhd) |
| 14 | `ID ) ;` | 162 | `type my_type);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_type_generics_in_packages.vhd) |
| 15 | `access ID ;` | 149 | `type CALL_PATH_VECTOR_PTR is access CALL_PATH_VECTOR ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/deprecated/FileLinePathPkg_c.vhd) |
| 16 | `( NUM downto` | 124 | `subtype T_SLVV_2            is T_SLVV(open)(1 downto 0);` (stimuli/vhdl/subs/PoC/src/common/vectors.vhdl) |
| 17 | `for ID :` | 110 | `for TestCtrl_1 : TestCtrl` (stimuli/vhdl/subs/OsvvmLibraries/Ethernet/TestStandAlone/Tb_xMii1.vhd) |
| 18 | `, ID (` | 108 | `Address(AXI_ADDR_WIDTH-1 downto 0),` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/testbench/TbAxi4.vhd) |
| 19 | `ID ; function` | 105 | `type mytype;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_entity_generic_type.vhd) |
| 20 | `type ID is` | 100 | `type   tState is (Idle, Command);` (stimuli/vhdl/subs/PoC/src/comm/remote/remote_terminal_control.vhdl) |
| 21 | `ID ; end` | 97 | `end get_c;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0108/pkg.vhdl) |
| 22 | `. NUM )` | 90 | `signal b : b_t := (X"42", (others => '0'), (others => '1'), 4.0);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_075.vhd) |
| 23 | `constant ID :` | 89 | `constant AXI_ADDR_WIDTH      : integer := AxiBus.WriteAddress.Addr'length ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Vti/deprecated/Axi4ManagerVti_e.vhd) |
| 24 | `shared variable ID` | 86 | `shared variable OperationFifo  : osvvm.ScoreboardPkg_slv.ScoreboardPType ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_RandomReadWrite.vhd) |
| 25 | `ID : out` | 82 | `signal   pins   : out std_logic_vector;` (stimuli/vhdl/subs/UVVM/bitvis_vip_gpio/tb/maintenance_tb/gpio_vvc_tb.vhd) |
| 26 | `ID , ID` | 81 | `procedure AlertIfFilesNotMatch (AlertLogID : AlertLogIDType ; file File1, File2 : text; Message : string := "" ; Level :` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/AlertLogPkg.vhd) |
| 27 | `' ID '` | 78 | `type state2_t is ('a', 'b', 'c', 'd');` (stimuli/vhdl/subs/ghdl/testsuite/gna/fst01/enum1.vhdl) |
| 28 | `ID : inout` | 78 | `procedure write(variable l : inout line ; v : integer_vector) is` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_007.vhd) |
| 29 | `ID ; --` | 75 | `subtype Axi4OptionsType is resolved_max Axi4UnresolvedOptionsType ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/common/src/Axi4OptionsPkg.vhd) |
| 30 | `alias ID :` | 71 | `alias Last  : std_logic is ResultParam(0) ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/src/AxiStreamTbPkg.vhd) |
| 31 | `protected procedure ID` | 70 | `type test_t is protected` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_002.vhd) |
| 32 | `' NUM '` | 67 | `type tri_state_logic is ('0', '1', 'Z');` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_11_fg_11_01.vhd) |
| 33 | `downto NUM =>` | 67 | `constant DATA_ZERO  : std_logic_vector := (DATA_WIDTH - 1 downto 0 => '0') ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_MemoryBurstAsyncPattern1.vhd) |
| 34 | `. NUM ,` | 65 | `v_rand_real.add_range_real(1.0, 5.5);` (stimuli/vhdl/subs/UVVM/uvvm_util/tb/maintenance_tb/questa_extension_tb.vhd) |
| 35 | `, ID range` | 64 | `type real_natural2D_t is array (natural range <>, natural range <>) of real;` (stimuli/vhdl/subs/Compliance-Tests/cosim/dpi-ffi/ghdl-vffi/test/valuesFromVhdlToC.vhd) |
| 36 | `<NO-POSITION>` | 63 | `Error: failed to read input file 'stimuli/vhdl/subs/PoC/src/mem/lut/lut_Sine.vhdl'

Caused by:
    stream did not contai` (stimuli/vhdl/subs/PoC/src/mem/lut/lut_Sine.vhdl) |
| 37 | `new ID .` | 57 | `package my_fixed_pkg is new ieee.fixed_generic_pkg;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_fixed_generic_pkg.vhd) |
| 38 | `: for ID` | 55 | `loop_add_m: for I in 0 to A'length-1 loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1684/pkg.vhdl) |
| 39 | `) of ID` | 54 | `type arr is array (natural range 0 to 7) of bit;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug072/t.vhdl) |
| 40 | `ID ; type` | 54 | `end record rd_req_t ;` (stimuli/vhdl/subs/OsvvmLibraries/CoSimPCIe/testbench/TbPcie/Tb_Pcie.vhd) |
| 41 | `ID ; constant` | 53 | `end record ModelParametersIDType ;` (stimuli/vhdl/subs/OsvvmLibraries/Common/src/ModelParametersSingletonPkg.vhd) |
| 42 | `ID ; signal` | 53 | `constant CLK_FREQ : FREQ := 50 MHz;` (stimuli/vhdl/subs/PoC/tb/dstruct/dstruct_deque_tb.vhdl) |
| 43 | `; -- ID` | 51 | `)  ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/testbenchVti/TbStream.vhd) |
| 44 | `port ( ID` | 49 | `port (m : in matrix_t(0 to 1));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3046/b.vhdl) |
| 45 | `ID is ID` | 45 | `subtype T_slv_d is std_logic_vector(D_BITS - 1 downto 0);` (stimuli/vhdl/subs/PoC/src/fifo/fifo_stage.vhdl) |
| 46 | `ID ; procedure` | 44 | `end proc;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/proctarg.vhdl) |
| 47 | `; constant ID` | 43 | `constant AXI_ADDR_WIDTH : integer := ManagerRec.Address'length ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4Lite/testbench/TestCtrl_e.vhd) |
| 48 | `range - NUM` | 41 | `type frequency is range -2147483647 to 2147483647 units KHz;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug20255/test.vhd) |
| 49 | `. NUM to` | 40 | `valid_low_multiple_random_prob : real range 0.0 to 1.0; -- Probability of how often valid shall be deasserted when using` (stimuli/vhdl/subs/UVVM/bitvis_vip_avalon_st/src/avalon_st_bfm_pkg.vhd) |
| 50 | `ID ID :` | 39 | `constant DATA_BYTES : integer := DATA_WIDTH/8 ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/testbench/TestCtrl_e.vhd) |
| 51 | `alias ID is` | 38 | `alias ManagerRec     is <<signal ^.Manager_1.TransRec : AddressBusRecType>> ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/testbenchVti/TestCtrl_e.vhd) |
| 52 | `( ID -` | 37 | `ParIn                              : in  LinkType(0 to NUMOFLANES-1)(ENCODEDWIDTH-1 downto 0) ;` (stimuli/vhdl/subs/OsvvmLibraries/CoSimPCIe/src/PcieComponentPkg.vhd) |
| 53 | `, ' NUM` | 37 | `assert_value_ena <= '1', '0' after 256*C_CLK_PERIOD;  -- Enable assertion checking for 256 clock cycles` (stimuli/vhdl/subs/UVVM/uvvm_assertions/tb/uvvm_assertions_demo_tb.vhd) |
| 54 | `: ID is` | 37 | `alias THIS_VVCT       : t_vvc_target_record is AVALON_MM_VVCT;` (stimuli/vhdl/subs/UVVM/bitvis_vip_avalon_mm/src/vvc_methods_pkg.vhd) |
| 55 | `after ID ;` | 37 | `In1 <= I1 after I1toO;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug078/half_adder.vhdl) |
| 56 | `( ID (` | 36 | `signal ManagerRec, SubordinateRec : AddressBusRecArrayType(1 to 2)(` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/testbench_MultipleMemory/TbAxi4_MultipleMemory.vhd) |
| 57 | `; end package` | 36 | `alias Axi4Stream_SenderView is Axi4Stream_TransmitterView;` (stimuli/vhdl/subs/Interfaces/AMBA/AXI/v4/AXI4Stream.vhdl) |
| 58 | `( " NUM` | 35 | `check_equal(value, std_logic_vector'("001111"));` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_bit_string_literals.vhd) |
| 59 | `; alias ID` | 35 | `alias LVDS_Interface is Differatial_Interface;` (stimuli/vhdl/subs/Interfaces/IO/Common.vhdl) |
| 60 | `> ID ,` | 33 | `long_port(15 downto 0) => not_so_long_vector,` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_001.vhd) |
| 61 | `to NUM =>` | 33 | `adder  := ("0" & unsigned(not rqst)) + (1 to 1 => tin);` (stimuli/vhdl/subs/PoC/src/arith/arith_firstone.vhdl) |
| 62 | `( ID :` | 32 | `generic (c : natural);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug059/tb.vhdl) |
| 63 | `; type ID` | 31 | `type MessageStructType ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/MessageListPkg.vhd) |
| 64 | `ID port map` | 31 | `dut: component write` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue542/wrapper.vhd) |
| 65 | `begin end entity` | 31 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug18810/BENCH_OISC_SUBLEQ.vhd) |
| 66 | `until ID (` | 31 | `wait until rising_edge(clk);` (stimuli/vhdl/subs/PoC/tb/misc/misc_bit_lz_tb.vhdl) |
| 67 | `function ID (` | 30 | `function EQ_BUT_NOT_META(l, r : std_logic) return boolean is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug04/test.vhdl) |
| 68 | `( " ID` | 29 | `constant  COVERAGE_DEFAULT_WEIGHT_MODE  : string := IfElse(OSVVM_SETTINGS_REVISION >= "2024", string'("REMAIN"), "AT_LEA` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/OsvvmSettingsPkg_default.vhd) |
| 69 | `( NUM to` | 29 | `variable v_range_uns_vec : t_range_uns_vec(0 to 0)(0 to 1)(127 downto 0);` (stimuli/vhdl/subs/UVVM/uvvm_util/tb/maintenance_tb/rand_tb.vhd) |
| 70 | `<EOF>` | 28 | `--  'VAL parameter is any integer type (including universal integer)` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug09/univ2.vhdl) |
| 71 | `' subtype ;` | 27 | `signal gIntReqDelayed : gIntReq'subtype ;` (stimuli/vhdl/subs/OsvvmLibraries/CoSim/src/CoSimInterruptHandler.vhd) |
| 72 | `, NUM to` | 27 | `type rom_t is array(0 to 3, 0 to 3) of std_logic_vector(3 downto 0);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1220/top.vhdl) |
| 73 | `protected impure function` | 27 | `type test_t is protected` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_004.vhd) |
| 74 | `signal ID :` | 27 | `signal TransRec : StreamRecType(DataToModel(0 downto 0), DataFromModel(0 downto 0), ParamToModel(1 to 0), ParamFromModel` (stimuli/vhdl/subs/OsvvmLibraries/Common/src/InterruptGeneratorBitVti.vhd) |
| 75 | `" + "` | 26 | `function "+" (a, b: trit) return trit;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2536/bug.vhdl) |
| 76 | `; ID <=` | 26 | `cpu_rstb  <= (not rst) and (not int_write) and int_got when rising_edge(clk);` (stimuli/vhdl/subs/PoC/src/cache/cache_mem.vhdl) |
| 77 | `attribute ID of` | 26 | `attribute foreign of VInit : procedure is "VHPI VProc.so; VInit" ;` (stimuli/vhdl/subs/OsvvmLibraries/CoSim/src/OsvvmVprocAldecPkg.vhd) |
| 78 | `( NUM )` | 25 | `function f(arg : integer := integer'(1)) return boolean;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2164/e.vhdl) |
| 79 | `ID : process` | 25 | `P_CLKAGEN : process` (stimuli/vhdl/subs/OsvvmLibraries/CoSimPCIe/src/clkmux.vhd) |
| 80 | `range NUM .` | 25 | `type frequency is range 0.0 to real'high;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2156/timing_pkg.vhdl) |
| 81 | `( ID "` | 24 | `constant raw_format_slv_dec_result0		: string		:= raw_format_slv_dec(std_logic_vector'(x"12"));` (stimuli/vhdl/subs/PoC/tb/common/strings_tb.vhdl) |
| 82 | `> ID (` | 24 | `v := (v(v'left-1 downto 0) & '0') xor (to_stdlogicvector(GN) and (GN'range => v(v'left)));` (stimuli/vhdl/subs/PoC/src/comm/comm_scramble.vhdl) |
| 83 | `ID select ID` | 24 | `with fwd_r select q <=` (stimuli/vhdl/subs/PoC/src/mem/ocram/ocram_sdp_wf.vhdl) |
| 84 | `default ID is` | 24 | `default clock is rising_edge(clk);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_psl.vhd) |
| 85 | `view ID of` | 24 | `AxiBus      : view Axi4ManagerView of Axi4RecType ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Axi4ComponentPkg.vhd) |
| 86 | `( ' NUM` | 23 | `assert to_01(v0) = std_ulogic'('0') severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper03/tox01.vhdl) |
| 87 | `> ' NUM` | 23 | `data => (fmt.data'range => '0'));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1764/repro.vhdl) |
| 88 | `) ; begin` | 22 | `variable result : t_rec(addr(bits - 1 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue191/repro.vhdl) |
| 89 | `<> ) )` | 22 | `generic (package pkg is new work.gen1 generic map (<>));` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug055/gen2.vhdl) |
| 90 | `ID ID ,` | 22 | `terminal p_in1, p_in2, p_out : electrical;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/AMS_CS1_Mixed_Sig/tb_2in_switch.vhd) |
| 91 | `generic ( ID` | 22 | `generic (c : natural);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug057/tb2.vhdl) |
| 92 | `( ID '` | 21 | `StageOut_v(0)(inp'length-1 downto 0)	:= inp;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1051/psi_common_logic_pkg.vhd) |
| 93 | `( ID ,` | 21 | `p_clk : clock_generator(clk, GC_CLK_PERIOD);` (stimuli/vhdl/subs/UVVM/bitvis_vip_ethernet/tb/maintenance_tb/ethernet_gmii_mac_master_th.vhd) |
| 94 | `; variable ID` | 21 | `file f_in : text;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1145/tb.vhdl) |
| 95 | `ID ' ID` | 21 | `port(a:std'u);end ghdlcrash;architecture h of g is--` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/attr1.vhdl) |
| 96 | `ID ; variable` | 21 | `end pulse;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/forloop.vhdl) |
| 97 | `" = "` | 20 | `alias   Match is "=" [ActualType, ExpectedType return boolean] ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/deprecated/ScoreboardPkg_IntV_c.vhd) |
| 98 | `: ID (` | 20 | `alias DNA_VALUE : bit_vector(SIM_DNA_VALUE'length - 1 downto 0) is SIM_DNA_VALUE;` (stimuli/vhdl/subs/PoC/src/xil/xil_DNAPort.vhdl) |
| 99 | `; ID .` | 20 | `);` (stimuli/vhdl/subs/PoC/tb/arith/addw/arith_addw_TestHarness.vhdl) |
| 100 | `for all :` | 20 | `for all: ADDER` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue123/adder.vhdl) |
| 101 | `ID ; subtype` | 19 | `subtype  std_logic_max is resolved_max std_ulogic ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/ResolutionPkg.vhd) |
| 102 | `generic ( type` | 18 | `generic (` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_049.vhd) |
| 103 | `procedure ID (` | 18 | `PROCEDURE addup (i1,i2,i3:IN INTEGER;add:IN BOOLEAN;VARIABLE i4:OUT INTEGER) IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc149.vhd) |
| 104 | `range ID '` | 18 | `type ANGLE is range INTEGER'range units` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug036/phys.vhdl) |
| 105 | `; -- =` | 17 | `alias t_operation is work.transaction_pkg.t_operation;` (stimuli/vhdl/subs/UVVM/bitvis_vip_avalon_mm/src/vvc_cmd_pkg.vhd) |
| 106 | `ID ; assert` | 17 | `return_exp_check;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1630.vhd) |
| 107 | `on ID ;` | 16 | `wait on t;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug017/wait3.vhdl) |
| 108 | `pure function ID` | 16 | `pure function fOISC_SUBLEQ_DW_LATENCY (` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug18810/OISC_SUBLEQ.vhd) |
| 109 | `view ID ;` | 16 | `WriteAddress   : view Axi4_Address_ManagerView;` (stimuli/vhdl/subs/Interfaces/AMBA/AXI/v4/AXI4.vhdl) |
| 110 | `( ID =>` | 15 | `constant DEFAULT_CONFIG : config_t := config_t'(width => 10) ;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2463/test1.vhdl) |
| 111 | `. NUM /` | 15 | `constant T_RAS     : integer := integer(ceil(42.0/CLK_PERIOD));` (stimuli/vhdl/subs/PoC/src/mem/sdram/sdram_ctrl_de0.vhdl) |
| 112 | `; end ID` | 15 | `say_hello;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0114/hello.vhdl) |
| 113 | `ID ; component` | 15 | `end record stats_type;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_14_fg_14_09.vhd) |
| 114 | `ID ; view` | 15 | `end record axis_rec_type;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3172/axis_pkg.vhdl) |
| 115 | `package ID is` | 15 | `package pkg1 is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug052/tb.vhdl) |
| 116 | `' ID ;` | 14 | `signal thing : thing_t'base ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_018.vhd) |
| 117 | `, ID )` | 14 | `type branch is array (remains, bit) of remains;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug084/func_test1.vhdl) |
| 118 | `. NUM *` | 14 | `signal PRUEBA: std_logic_vector(SIZE-1 downto 0) := std_logic_vector(to_unsigned(integer(0.25*2**SIZE+0.5), SIZE));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1820/cordic.vhdl) |
| 119 | `<= ID +` | 14 | `clkgen: clk <= clk + 1 after 1 ns when clk < 7;` (stimuli/vhdl/subs/nvc/test/regress/psl13.vhd) |
| 120 | `ID ' (` | 14 | `l1 := new string'("Hello world");` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug017/call9.vhdl) |
| 121 | `ID generic map` | 14 | `mem : component altsyncram` (stimuli/vhdl/subs/PoC/src/mem/ocram/altera/ocram_sp_altera.vhdl) |
| 122 | `) ; signal` | 13 | `signal in_abc  : abc.Rec_t(Data(7 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2562/repro1.vhdl) |
| 123 | `<= ' NUM` | 13 | `reset : reset_s <= '0',` (stimuli/vhdl/subs/ghdl/testsuite/gna/deb585750/585750_deb.vhd) |
| 124 | `<= not ID` | 13 | `p_clk : clk <= not clk after C_CLK_PERIOD;` (stimuli/vhdl/subs/UVVM/external_vip_apb/tb/maintenance_tb/apb_bfm_tb.vhd) |
| 125 | `begin ID :` | 13 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1588/tent.vhdl) |
| 126 | `<< signal ID` | 12 | `<<signal i_test.sig : std_logic >> <= force '0';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2219/ent.vhdl) |
| 127 | `> ID )` | 12 | `return ((d and (q'range => en)) or (q and not (q'range => en))) and not (q'range => rst);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug019/PoC/src/common/components.vhdl) |
| 128 | `> NUM )` | 12 | `if(s_wrcnt = 3 and s_index => 5) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1664/ppm.vhdl) |
| 129 | `ID bus ;` | 12 | `signal interrupt_request : pulled_up bit bus;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_ch_16_01.vhd) |
| 130 | `[ ID ,` | 12 | `alias   Match is Match [ActualType, ExpectedType return boolean] ;` (stimuli/vhdl/subs/OsvvmLibraries/UART/src/deprecated/ScoreboardPkg_Uart_c.vhd) |
| 131 | `vunit ID (` | 12 | `vunit const_test_vunit (const_test(rtl))` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1724/const_test_vunit.vhdl) |
| 132 | `( NUM =>` | 11 | `stb => std_logic_vector'(0 => s_stb)   -- CRASH` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3199/test_qual.vhdl) |
| 133 | `; signal ID` | 11 | `end component;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1935/sm_tb.vhdl) |
| 134 | `<< signal .` | 11 | `tb_sig <= << signal .tb_external_names.dut_inst.local_sig : bit >>;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_external_names.vhd) |
| 135 | `ID ; package` | 11 | `function get return natural is begin return 0;end;end gen0;package n is generic(package g is new n generic map(<>));func` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/func1.vhdl) |
| 136 | `ID <= ID` | 11 | `data_o <= extend(data_i);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2652/func.vhdl) |
| 137 | `ID is array` | 11 | `generic (type VectorType is array (type is range <>) of type is private ) ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/DynamicVectorGenericPkg.vhd) |
| 138 | `" < "` | 10 | `function "<" (a, b : natural) return bit;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1979/repro.vhdl) |
| 139 | `. NUM -` | 10 | `function fract(F : real; maxDenominator : natural := 1000; maxError : real := 1.0E-6) return T_FRACTIONAL;` (stimuli/vhdl/subs/PoC/src/common/math.vhdl) |
| 140 | `; -- --` | 10 | `)  ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/testbench_GenericSignals/TbAxi4.vhd) |
| 141 | `; end architecture` | 10 | `(o, d) <= d & i when rising_edge(clk);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1914/delayline1d.vhdl) |
| 142 | `ID ; impure` | 10 | `type tyi;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2702/top.vhdl) |
| 143 | `[ ID return` | 10 | `alias to_Axi4RespType is from_Axi4RespEnumType[Axi4RespEnumType return Axi4RespType] ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/common/src/Axi4InterfaceCommonPkg.vhd) |
| 144 | `after ID /` | 10 | `SpiClk <= not SpiClk after OptSclkPeriod / 2;` (stimuli/vhdl/subs/OsvvmLibraries/SPI_GuyEschemann/src/SpiController.vhd) |
| 145 | `begin -- ID` | 10 | `begin` (stimuli/vhdl/subs/UVVM/bitvis_vip_avalon_mm/src/avalon_mm_vvc.vhd) |
| 146 | `protected end protected` | 10 | `type t_prot is protected` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1688/test1.vhdl) |
| 147 | `range ( NUM` | 10 | `type t1 is range (1+1) to (10+2);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc198.vhd) |
| 148 | `. ID (` | 9 | `DESC(i).List(j).Index			:= k;` (stimuli/vhdl/subs/PoC/src/misc/gearbox/gearbox_down_cc.vhdl) |
| 149 | `. ID :=` | 9 | `ranges(i).min := t_item_range_value(bins(i).min);` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/CoverageVendorApiPkg_NVC.vhd) |
| 150 | `. ID ;` | 9 | `stx.env(i).b;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2076/gcrash-6a.vhdl) |
| 151 | `ID . ID` | 9 | `Osvvm.ClockResetPkg.CreateReset(` (stimuli/vhdl/subs/PoC/tb/arith/scaler/arith_scaler_TestHarness.vhdl) |
| 152 | `ID := '` | 9 | `port (d : std_logic := '0';` (stimuli/vhdl/subs/ghdl/testsuite/gna/fst01/ex1.vhdl) |
| 153 | `ID ID is` | 9 | `nature electrical is real across real through ground reference;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug031/ams1.vhdl) |
| 154 | `ID generate when` | 9 | `g_case: case sel generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/fst01/casegen1.vhdl) |
| 155 | `ID range NUM` | 9 | `signal Sht : bit_vector(2 downtonatural range 0 to 7;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/badrng.vhdl) |
| 156 | `begin assert ID` | 9 | `begin` (stimuli/vhdl/subs/UVVM/bitvis_uart/src/uart.vhd) |
| 157 | `" NUM "` | 8 | `AffirmIfEqual(TbID, Get(Params, 5), 5X"11",   "Get(Params, 5) = 5X""11""") ;` (stimuli/vhdl/subs/OsvvmLibraries/Common/TbParams/TbModelParameters1.vhd) |
| 158 | `" and "` | 8 | `"and";` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue478/repro2.vhdl) |
| 159 | `( " "` | 8 | `write(line0, string'(" "));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2299/tb_dct8.vhdl) |
| 160 | `( ( ID` | 8 | `memory(to_integer(unsigned(addr)))((x+1)*8-1 downto x*8) := data_i((x+1)*8-1 downto x*8);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1780/imem.vhdl) |
| 161 | `( ID =` | 8 | `reg_read_selector : block (reg_sel = '1' and read = '1') is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_fg_16_07.vhd) |
| 162 | `( others =>` | 8 | `RETURN vector_t'(OTHERS => '0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue244/test.vhdl) |
| 163 | `) ; --` | 8 | `subtype wb_cpu_in_t  is wb_slave_out_t(dat(wb_cpu_data_bits-1 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1300/wishbone_types.vhdl) |
| 164 | `) ; end` | 8 | `s (7 downto 1) <= ();` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/noexpr.vhdl) |
| 165 | `. ID =` | 8 | `assert cst(0).nat = 1 severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2051/repro2.vhdl) |
| 166 | `. NUM **` | 8 | `constant frac			: INTEGER	:= integer(round((val - real(int)) * 10.0**precision));` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug019/PoC/src/common/strings.vhdl) |
| 167 | `? ID is` | 8 | `case? x is` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_case.vhd) |
| 168 | `ID := NUM` | 8 | `constant N : positive := 9;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/arith_addw_tb.vhdl) |
| 169 | `ID ; use` | 8 | `end c04s03b02x02p20n01i00163pkg;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc163.vhd) |
| 170 | `ID = NUM` | 8 | `gen: if False b = 0 generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/inst2.vhdl) |
| 171 | `NUM ' after` | 8 | `S1 <= transport '0' after 5  ns,` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1321.vhd) |
| 172 | `subtype ID is` | 8 | `subtype word is std_logic_vector(word_len - 1 downto 0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1704/orig.vhdl) |
| 173 | `until ( ID` | 8 | `wait until (Enable = '1');` (stimuli/vhdl/subs/PoC/src/bus/stream/stream_Source.vhdl) |
| 174 | `" " ;` | 7 | `constant s37: string:=""" ";   -- 37` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2737.vhd) |
| 175 | `( ' ID` | 7 | `o <= to_X01(std_ulogic' ('H'));` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1537/ent1.vhdl) |
| 176 | `( ID .` | 7 | `work.cpu_types.status_value'(work.cpu_types.fetch)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1640/test.vhdl) |
| 177 | `. ID .` | 7 | `assert path(idx).name.all               /= "" severity failure ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_015.vhd) |
| 178 | `: while ID` | 7 | `L1 :` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1509.vhd) |
| 179 | `; end ;` | 7 | `f ("0110");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1469/ent1.vhdl) |
| 180 | `<= ID (` | 7 | `COMPARATOR: op <= raw_op_sin(raw_op_sin'length -1); --Sign bit` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue44/costasloop.vhdl) |
| 181 | `<= ID ;` | 7 | `transfer: o <= x;                             -- Covered` (stimuli/vhdl/subs/nvc/test/cover/issue1431_sub.vhd) |
| 182 | `ID : entity` | 7 | `ram : entity work.ocram_sdp` (stimuli/vhdl/subs/PoC/src/mem/ocram/ocram_sdp_optimized.vhdl) |
| 183 | `ID ; alias` | 7 | `end view Axi4WriteAddressManagerView ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/src/Axi4InterfaceModeViewPkg.vhd) |
| 184 | `end ID ;` | 7 | `end behav;` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket41/bug1.vhdl) |
| 185 | `range ID to` | 7 | `type byte is range c07s01b00x00p08n01i01849arch to 3;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1849.vhd) |
| 186 | `to ID =>` | 7 | `return (bv => (1 to n => '0'));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1138/repro.vhdl) |
| 187 | `" - "` | 6 | `function "-" (a, b : t) return t is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug21513/pb.vhdl) |
| 188 | `" not "` | 6 | `use ieee.std_logic_1164."not";` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2396/frequency.vhdl) |
| 189 | `( ( NUM` | 6 | `alias ADreg1 is bufreg((2 * SIZE - 1) downto SIZE);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug21274/21274.vhd) |
| 190 | `, NUM downto` | 6 | `type A is array (1 to 4, 31 downto 0) of boolean;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_04_ch_04_04.vhd) |
| 191 | `. ID '` | 6 | `assert not APPEND_DEST_BITS or (Out_M2S.Dest'length = log2ceilnz(PORTS) + In_M2S(0).Dest'length)` (stimuli/vhdl/subs/PoC/src/bus/axi4/AXI4Stream/AXI4Stream_Mux.vhdl) |
| 192 | `. ID )` | 6 | `RETURN (ain(25).b2);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc973.vhd) |
| 193 | `; begin ID` | 6 | `file f : text ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_006a.vhd) |
| 194 | `; procedure ID` | 6 | `file my_file : text;` (stimuli/vhdl/subs/nvc/test/regress/issue284.vhd) |
| 195 | `<= ID and` | 6 | `z <= a and b and c;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_ch_05_24.vhd) |
| 196 | `after ID ,` | 6 | `clk <= '1' after T_pw, '0' after 2*T_pw;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_fg_05_03.vhd) |
| 197 | `assert ID report` | 6 | `assert FALSE` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1440.vhd) |
| 198 | `begin if ID` | 6 | `function begin if a s';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash42.vhdl) |
| 199 | `component ID end` | 6 | `component c05s02b02x00p02n01i03139ent_a` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc3139.vhd) |
| 200 | `for ID for` | 6 | `for instrumentation` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_14_fg_14_10.vhd) |
| 201 | `generate ID <=` | 6 | `else generate` (stimuli/vhdl/subs/OsvvmLibraries/DpRam/src/DpRam_Singleton.vhd) |
| 202 | `return ID ;` | 6 | `return true;      -- illegal in entity declaration region.` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1620.vhd) |
| 203 | `' ID )` | 5 | `bit'samples);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/name2.vhdl) |
| 204 | `' ID =` | 5 | `if (T2'BASE'LEFT = T1'BASE'LEFT) then   --- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc3161.vhd) |
| 205 | `( NUM ,` | 5 | `CONSTANT C_TEST_VECTOR : test_vector_t := (OTHERS => test_data_t'(0, 0, 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue242/test.vhdl) |
| 206 | `) := (` | 5 | `constant v : my_rec (addr(0 downto 0)) := (` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0117/repro7.vhdl) |
| 207 | `) ; type` | 5 | `subtype myboundedrecord is myrecord(a(1 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1287/myentity.vhdl) |
| 208 | `) ; variable` | 5 | `variable r1 : my_rec (v(0 to 3));` (stimuli/vhdl/subs/ghdl/testsuite/synth/conv01/convrec01.vhdl) |
| 209 | `. ID #` | 5 | `assert NOT(   16#F.FF#E+2 = 2#1.1111_1111_111#E11   and` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2713.vhd) |
| 210 | `: ID :=` | 5 | `constant AXI_DATA_WIDTH      : integer := AxiBus.WriteData.Data'length ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4Lite/src/Axi4LiteManager.vhd) |
| 211 | `<< constant .` | 5 | `<< constant .h.x : bit >>;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/extname1.vhdl) |
| 212 | `> NUM ,` | 5 | `MY_GENERIC(0)           => 1,` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3050/top.vhdl) |
| 213 | `?? ID )` | 5 | `if (?? tg) and falling_edge(clk) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc14.vhdl) |
| 214 | `ID := ID` | 5 | `port (d : bit_vector := x"01");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue610/repro1.vhdl) |
| 215 | `ID is generic` | 5 | `begin return 0;end;end;package gen2 is generic(package pkg is new work.gen0 generic map(<>));function get2 return natura` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/attr25.vhdl) |
| 216 | `ID return ID` | 5 | `function get_str` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug017/call7.vhdl) |
| 217 | `NUM to NUM` | 5 | `subtype small_int is range 0 to 7;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1385.vhd) |
| 218 | `[ return ID` | 5 | `alias IsVhdlAssertFailed is std.env.IsVhdlAssertFailed [return BOOLEAN] ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/AssertApiPkg.vhd) |
| 219 | `\ ID \` | 5 | `entity \foo\ is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug012/demo.vhd) |
| 220 | `component ID generic` | 5 | `component ic_socket` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc3128.vhd) |
| 221 | `entity ID is` | 5 | `ENTITY c07s03b04x00p03n01i02509ent IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2509.vhd) |
| 222 | `group ID is` | 5 | `group signal_pair is (signal, signal);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_20_ch_20_11.vhd) |
| 223 | `of ID is` | 5 | `function meaning_of_life return return_value of unsigned is` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_reading_function_return_value.vhd) |
| 224 | `on ID (` | 5 | `wait on A(k);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1202.vhd) |
| 225 | `process variable ID` | 5 | `process` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug017/call6.vhdl) |
| 226 | `protected -- --` | 5 | `type t_data_queue is protected` (stimuli/vhdl/subs/UVVM/uvvm_util/src/data_queue_pkg.vhd) |
| 227 | `protected -- ID` | 5 | `type ModelParametersPType is protected` (stimuli/vhdl/subs/OsvvmLibraries/Common/src/ModelParametersPtPkg.vhd) |
| 228 | `protected function ID` | 5 | `type p is protected` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue400/e.vhdl) |
| 229 | `range NUM downto` | 5 | `type beta is range 1000 downto 0` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc526.vhd) |
| 230 | `then null ;` | 5 | `if (cst = 3 then` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/paren.vhdl) |
| 231 | `use ID .` | 5 | `use work.c10s01b00x00p11n01i00886pkg_a.x;    -- creates a declarative item in this region` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc886.vhd) |
| 232 | `variable ID :` | 5 | `variable var : std_logic_vector(0 to 1);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2459/ent.vhdl) |
| 233 | `" ID "` | 4 | `"abc";` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/str.vhdl) |
| 234 | `' ' '` | 4 | `constant C_MSG_DELIMITER : character   := '''; -- Delimiter of the msg parameter in UVVM commands` (stimuli/vhdl/subs/UVVM/uvvm_util/src/adaptations_pkg.vhd) |
| 235 | `' ' ;` | 4 | `k := '';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2728.vhd) |
| 236 | `( - NUM` | 4 | `foobar <= to_signed(natural'(-1), foobar'length);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1240/issue1.vhdl) |
| 237 | `( ID downto` | 4 | `signal coeff: std_logic_vector_array (0 to 5) (high_c downto low_c) := (others => (others => '0'));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue887/test2.vhdl) |
| 238 | `+ ID ;` | 4 | `R := R * + R;  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2192.vhd) |
| 239 | `, NUM after` | 4 | `a <= 0, 5 after 10 ns, -5 after 20 ns, 8 after 30 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ap_a_ap_a_03.vhd) |
| 240 | `- ID ;` | 4 | `c := a/-b;   -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2188.vhd) |
| 241 | `. ID <=` | 4 | `port_req(i).stb <= req_i.stb and port_sel(i);` (stimuli/vhdl/subs/neorv32/rtl/core/neorv32_bus.vhd) |
| 242 | `. NUM +` | 4 | `variable w : real := 3.0 + 2.0;   -- No_failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2021.vhd) |
| 243 | `;` | 4 | `ENDc04s03b01x01p03n02i00046arch;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc46.vhd) |
| 244 | `; ID :` | 4 | `simInitialize;` (stimuli/vhdl/subs/PoC/tb/misc/gearbox/gearbox_down_dc_tb.vhdl) |
| 245 | `<= ' ID` | 4 | `synch_control_pull_up : synch_control <= 'H';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_11_fg_11_09.vhd) |
| 246 | `<= NUM after` | 4 | `sig : sma_int <= 6 after 5 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1852.vhd) |
| 247 | `?= ' NUM` | 4 | `check_equal('1' ?= '1', '1');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_eq.vhd) |
| 248 | `?= ID ;` | 4 | `slv_eq <= slv_a ?=  slv_b;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1109/ent-orig.vhdl) |
| 249 | `?? ID (` | 4 | `if ?? falling_edge(clk) and (?? tg) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc10.vhdl) |
| 250 | `ID " ID` | 4 | `subtype displacement is real tolerance "def_disp";` (stimuli/vhdl/subs/ghdl/testsuite/sanity/004all08/ams08.vhdl) |
| 251 | `ID ' ;` | 4 | `function is;n';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash25.vhdl) |
| 252 | `ID ( signal` | 4 | `procedure drive_sig (signal s : out bit)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2736/mwe.vhdl) |
| 253 | `ID , NUM` | 4 | `0 =>    300 Bd,   1 =>    600 Bd,   2 =>   1200 Bd,   3 =>   1800 Bd,   4 =>   2400 Bd,` (stimuli/vhdl/subs/PoC/src/io/uart/uart.pkg.vhdl) |
| 254 | `ID - NUM` | 4 | `other_ctrs : in T_ARRAY(0 to N_NODES-2)(COUNTER_BITS-1 downto 0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2631/node.vhd) |
| 255 | `ID : view` | 4 | `signal   TransactionRec : view (AddressBusTestCtrlView) of AddressBusRecArrayType ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/common/src/Axi4OptionsArrayPkg.vhd) |
| 256 | `ID ; process` | 4 | `Clk <= not Clk after 10 ns ;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue17/cond_assign_proc.vhdl) |
| 257 | `ID ID =` | 4 | `wait until nReset = '1' ;` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/Axi4/TestCases/TbAxi4_AxiIfOptionsManagerMemory.vhd) |
| 258 | `ID after ID` | 4 | `z <= transport a after Tpd_01;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_fg_05_12.vhd) |
| 259 | `ID after NUM` | 4 | `line_out <= transport line_in after 500 ps;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_ch_05_11.vhd) |
| 260 | `ID of ID` | 4 | `attribute foreign of c_getenv : function is "VHPIDIRECT c_getenv";` (stimuli/vhdl/subs/Compliance-Tests/cosim/dpi-ffi/ghdl-vffi/test/EnvironmentVariables.vhd) |
| 261 | `NUM ) ;` | 4 | `left <= wordrevorder(c, 1);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1257/direction_mismatch.vhd) |
| 262 | `` if ID` | 4 | ``if TOOL_TYPE = "SIMULATION" then` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_061.vhd) |
| 263 | `access ID (` | 4 | `type my_line is access string(1 to 80);` (stimuli/vhdl/subs/ghdl/testsuite/synth/types01/access01.vhdl) |
| 264 | `architecture ID of` | 4 | `ARCHITECTURE ch130301_p00201_06_arch OF ch130301_p00201_06_ent IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2586.vhd) |
| 265 | `process begin ID` | 4 | `process` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug7186/bug.vhdl) |
| 266 | `process is begin` | 4 | `postponed process is` (stimuli/vhdl/subs/nvc/test/parse/process.vhd) |
| 267 | `return ID (` | 4 | `return inner(0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2104/e.vhdl) |
| 268 | `until ID ;` | 4 | `wait until p_sync_test_enable;` (stimuli/vhdl/subs/UVVM/uvvm_util/tb/maintenance_tb/methods_tb_sync_arch.vhd) |
| 269 | `view ( ID` | 4 | `TX : view ( Differatial_OutView );` (stimuli/vhdl/subs/Interfaces/MIPI/M-PHY.vhdl) |
| 270 | `view ID )` | 4 | `port (source : view st_source_v);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue3267/top.vhdl) |
| 271 | `when ID <` | 4 | `cnt := cnt + 1 when cnt<g_NumberOfChannels-1 else 0;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2042/ent.vhdl) |
| 272 | `when ID =` | 4 | `return true when x = 1 else false ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_094a.vhd) |
| 273 | `when ID >` | 4 | `next when i > 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1573.vhd) |
| 274 | `" & "` | 3 | `function "&" ( a, b : stimulus_list ) return stimulus_list;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_20_ch_20_05.vhd) |
| 275 | `" > "` | 3 | `function ">" ( l,r : integer ) return bit;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1387/repro1.vhdl) |
| 276 | `" mod "` | 3 | `alias to_hash is "mod" [integer, integer return integer];` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug061/dictp.vhdl) |
| 277 | `% ID :` | 3 | `x%x : in bit; -- <==` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug24326/tb_thingy9.vhdl) |
| 278 | `' ' ID` | 3 | `architecture if''h';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash2.vhdl) |
| 279 | `( ID &` | 3 | `assert t_mem'(bv1 & bv2) =(0 to 1 => "00");` (stimuli/vhdl/subs/ghdl/testsuite/synth/err01/err_concat04.vhdl) |
| 280 | `( ID to` | 3 | `inst2: counter2(a to b)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1823/entity2.vhdl) |
| 281 | `( NUM **` | 3 | `(2**16-1 downto 0);` (stimuli/vhdl/subs/nvc/test/elab/signal34.vhd) |
| 282 | `( NUM .` | 3 | `x <= (complex'(0.0,0.0), complex'(1.0,1.0), complex'(2.0,2.0), complex'(3.0,3.0));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2572/tb.vhdl) |
| 283 | `( open )` | 3 | `subtype P2 is A(15 downto 0)(open);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue459/e3.vhdl) |
| 284 | `) ) ;` | 3 | `port (port1 : in OuterRecordType(inner1(inner2b(WIDTH-1 downto 0))));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1523/repro.vhdl) |
| 285 | `) ; subtype` | 3 | `subtype rec_4 is rec (s(1 to 4));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue238/pkg.vhdl) |
| 286 | `, ID ,` | 3 | `type word_vector is array(tArch_test, tSkip_test, boolean) of word;` (stimuli/vhdl/subs/PoC/tb/arith/addw/arith_addw_TestController_pkg.vhdl) |
| 287 | `. NUM report` | 3 | `assert mean > 0.0` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_19_random-b.vhd) |
| 288 | `: if ID` | 3 | `CLK_IF : if CLK = '1' then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3102/DAC_test.vhdl) |
| 289 | `: loop --` | 3 | `OperationLoop : loop` (stimuli/vhdl/subs/OsvvmLibraries/CoSim/testbench/TestCases_Ethernet/Tb_xMii1.vhd) |
| 290 | `: wait for` | 3 | `wait_1: wait for 1 ns;` (stimuli/vhdl/subs/nvc/test/lower/wait1.vhd) |
| 291 | `; -- default` | 3 | `constant is_original_pkg : boolean;` (stimuli/vhdl/subs/vunit/examples/vhdl/osvvm_log_integration/osvvm_integration/osvvm_to_vunit_common_log_pkg.vhd) |
| 292 | `; -- generate` | 3 | `simInitialize;` (stimuli/vhdl/subs/PoC/tb/io/ddrio/ddrio_inout_tb.vhdl) |
| 293 | `; ID '` | 3 | `architecture;b';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash21.vhdl) |
| 294 | `; ID (` | 3 | `watchdog_timer(watchdog_ctrl_terminate, C_GENERAL_WATCHDOG_TIMEOUT, ERROR, "Watchdog A");` (stimuli/vhdl/subs/UVVM/bitvis_vip_uart/tb/uvvm_demo_tb.vhd) |
| 295 | `; begin assert` | 3 | `constant x : sub_character;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/clifton-labs/compliant/functional/subtypes/modified-character-subtype.vhdl) |
| 296 | `; begin process` | 3 | `alias a_sig_1 is sig_1;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1751/ent.vhdl) |
| 297 | `; function ID` | 3 | `type		T_SLM								is array(NATURAL range <>, NATURAL range <>) of STD_LOGIC;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2/sortnet_BitonicSort.vhdl) |
| 298 | `> ) of` | 3 | `type er1 is array(range <>) of integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc321.vhd) |
| 299 | `> NUM to` | 3 | `type bit_vctor is array (integer => 1 to 8) of integer; --Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc324.vhd) |
| 300 | `? /= "` | 3 | `z <= a ?/= "1--0";` (stimuli/vhdl/subs/ghdl/testsuite/synth/match01/match02.vhdl) |
| 301 | `?= ID (` | 3 | `acc_en <= addr_i(abb_c'range) ?= IMEM_BASE(abb_c'range);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1781/imem.vhdl) |
| 302 | `ID " NUM` | 3 | `assert 5ub"11_000" = unsigned'(b"11_000");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1274/issue.vhdl) |
| 303 | `ID ( variable` | 3 | `procedure get_rec(variable rec : out rec_t) is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue397/e.vhdl) |
| 304 | `ID -- ID` | 3 | `gen_n_elementos : in integer -- quantidade de elementos no vetor de entradas` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2461/insere.vhdl) |
| 305 | `ID : block` | 3 | `blk : block` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue158/repro.vhdl) |
| 306 | `ID ;` | 3 | `assert false report "Hello world" severity note;with behav;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug090/hang3.vhdl) |
| 307 | `ID = (` | 3 | `assert NOT(2E6 = (2E3*1E3))` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2567.vhd) |
| 308 | `ID = ID` | 3 | `IDB <= TRB 	         when OP_SRC = x"0" else` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1243/DSPn.vhdl) |
| 309 | `ID ID ;` | 3 | `procedure initialize;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/sim_protected.v08.vhdl) |
| 310 | `NUM ' ;` | 3 | `glob_sig <= force '0';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2471/test.vhdl) |
| 311 | `NUM ID ;` | 3 | `permil	= 1000 ppm;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/sim_types.vhdl) |
| 312 | `NUM ID inertial` | 3 | `s <= reject 1 ns inertial 2 after 2 us;` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket11/tb3.vhdl) |
| 313 | `NUM after NUM` | 3 | `k <= transport 5 after 20 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1221.vhd) |
| 314 | `[ ID ]` | 3 | `mem[address]<=data_in;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue609/test.vhd) |
| 315 | `[ ] ;` | 3 | `alias prog_alias is proc[];` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket41/bug.vhdl) |
| 316 | `begin assert (` | 3 | `begin` (stimuli/vhdl/subs/UVVM/bitvis_vip_clock_generator/src/clock_generator_vvc.vhd) |
| 317 | `bus ; begin` | 3 | `signal s : std_logic bus;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2571/test1.vhdl) |
| 318 | `end case ;` | 3 | `end case;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/nochoice1.vhdl) |
| 319 | `generic ( constant` | 3 | `generic   (constant n : positive)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3110/issue.vhdl) |
| 320 | `generic map (` | 3 | `generic map (c => 5);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug057/tb.vhdl) |
| 321 | `null ; end` | 3 | `NULL;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1432.vhd) |
| 322 | `on ID ,` | 3 | `wait on PS1, PS2;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc884.vhd) |
| 323 | `procedure ID is` | 3 | `procedure foo is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue710/ent2.vhdl) |
| 324 | `protected -- open` | 3 | `type t_csv_file_reader is protected` (stimuli/vhdl/subs/UVVM/bitvis_vip_spec_cov/src/csv_file_reader_pkg.vhd) |
| 325 | `range " NUM` | 3 | `type I3 is range "0" to "9";  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc247.vhd) |
| 326 | `range ( (` | 3 | `type a is range (((((10-1)-1)-1)-1)-1) to (((((10+1)+1)+1)+1)+1);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc230.vhd) |
| 327 | `range - ID` | 3 | `type my_time is range -integer'low  to integer'high units` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue810/my_time_pkg.vhdl) |
| 328 | `ror NUM ;` | 3 | `outp <= inp ror 1; -- can also be 'rol'` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1909/reproducebug.vhdl) |
| 329 | `sra ID severity` | 3 | `assert rao = v sra n severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper02/tb_nushift01.vhdl) |
| 330 | `when ( ID` | 3 | `next when (a mod 2) = 0;` (stimuli/vhdl/subs/nvc/test/lower/loop1.vhd) |
| 331 | `when ID (` | 3 | `BitVal(BitIndex) := '1' when BitsToEncode(BitIndex) = 'U' else '0' ;` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/MemorySupportPkg.vhd) |
| 332 | `! ID :` | 2 | `variable k!k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2611.vhd) |
| 333 | `" -- ID` | 2 | `WB_ADDR          : std_logic_vector := "-" & X"--_----" ; -- Default 8M words divided between Register, Memory, DMA` (stimuli/vhdl/subs/OsvvmLibraries/Wishbone/src/WishboneComponentPkg.vhd) |
| 334 | `" ID .` | 2 | `file f : text is in "input.txt";` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug096/reader.vhdl) |
| 335 | `" ID :` | 2 | `variable "k : integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2580.vhd) |
| 336 | `" ID ID` | 2 | `report "***PASSED TEST: c08s02b00x00p05n01i01260 - This test needs manual check to make sure that default value for the ` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1260.vhd) |
| 337 | `" abs "` | 2 | `function "abs" (constant c1 : in integer) return newt is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2968.vhd) |
| 338 | `# ID :` | 2 | `variable k#k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2612.vhd) |
| 339 | `& ID :` | 2 | `variable k&k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2615.vhd) |
| 340 | `' ( ID` | 2 | `bit'(samples));` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/name1.vhdl) |
| 341 | `' ( others` | 2 | `cnt := cnt'last'(others => '1');` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/qual.vhdl) |
| 342 | `' ID (` | 2 | `constant time_image : time'image(now);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue387/bug.vhdl) |
| 343 | `' ID :` | 2 | `variable k'k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2629.vhd) |
| 344 | `' ID :=` | 2 | `constant c : x'element := (others => '0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3204/t3.vhdl) |
| 345 | `' ID >=` | 2 | `assert (not clk'event) or clk'delayed'last_event >= Tpw_clk` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_ch_05_07.vhd) |
| 346 | `' \ '` | 2 | `function ChangeSeparator (A : string ; FromSeparator : character := '\' ; ToSeparator : character := '/') return string ` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/FileUtilPkg.vhd) |
| 347 | `' range ;` | 2 | `subtype mypos is natural range my_bv4'element'range;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2323/st3.vhdl) |
| 348 | `' subtype )` | 2 | `function func (a : din.data'subtype) return std_logic_vector is` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2484/issue.vhdl) |
| 349 | `' subtype :=` | 2 | `signal Data : i_Data'subtype := (others=>'0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1857/subtype_test.vhdl) |
| 350 | `( " #` | 2 | `write(l, string'("# Encoding file for '" & Name & "'"));	writeline(tokenFile, l);` (stimuli/vhdl/subs/PoC/src/common/debug.vhdl) |
| 351 | `( " :` | 2 | `write(o,string'("::"));` (stimuli/vhdl/subs/nvc/test/regress/issue1079.vhd) |
| 352 | `( ( '` | 2 | `(V,W) <= AGGREGATE_ARRAY' (('d', 'x', others => 'a'),` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2460.vhd) |
| 353 | `( ID <` | 2 | `b1: block (value < 10) is` (stimuli/vhdl/subs/nvc/test/regress/guard1.vhd) |
| 354 | `( NUM +` | 2 | `tmp <= std_logic_vector'(1 + signed(tmp));` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1253/repro1.vhdl) |
| 355 | `( NUM ;` | 2 | `function(0is;r';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash4.vhdl) |
| 356 | `( NUM ID` | 2 | `insert(stimulus_list, stimulus_element'(0 ns, "0XXXXXXXXX"));` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/non_compliant/ch_17_fg_17_14.vhd) |
| 357 | `( constant ID` | 2 | `generic (( constant i : integer );                           -- extra parenthesis` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc722.vhd) |
| 358 | `( signal ID` | 2 | `port (( signal s : bit                -- extra parenthesis` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc727.vhd) |
| 359 | `) ; alias` | 2 | `variable v_dummy_hierarchy_node : t_hierarchy_node(name(1 to C_HIERARCHY_NODE_NAME_LENGTH));` (stimuli/vhdl/subs/UVVM/uvvm_util/tb/maintenance_tb/methods_tb_alert_arch.vhd) |
| 360 | `) ; function` | 2 | `subtype A1 is ABASE(ENUM1 range M1 to M5);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1139.vhd) |
| 361 | `) ID :` | 2 | `variable k)k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2618.vhd) |
| 362 | `) is begin` | 2 | `procedure activate_event(event_number : natural range 1 to 2; event_time : time; queue_idx : positive range request_queu` (stimuli/vhdl/subs/vunit/vunit/vhdl/data_types/test/tb_event_pkg.vhd) |
| 363 | `* ID :` | 2 | `variable k*k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2616.vhd) |
| 364 | `* ID ID` | 2 | `report "***FAILED TEST: c13s10b00x00p04n01i02858 - Only right hand side quotation mark ("") is replaced by percent chara` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2858.vhd) |
| 365 | `+ ID :` | 2 | `variable k+k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2619.vhd) |
| 366 | `- ID :` | 2 | `variable k-k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2621.vhd) |
| 367 | `-- ID ID` | 2 | `-- device under test` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2448/debouncer_no_vunit_not_ok_tb.vhdl) |
| 368 | `. ID *` | 2 | `constant C_DATA_VERSION         : bit_vector(C_SEGMENT_RECORDS(TYPE_2).word_length*8-1 downto 0) := C_DATA_INVALID_VERSI` (stimuli/vhdl/subs/nvc/test/bounds/issue477a.vhd) |
| 369 | `. ID ,` | 2 | `O(a) <= (RE => '0' & I(a mod I'length).RE, IM => '0' & I(a mod I'length).IM);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue961/tb.vhdl) |
| 370 | `. ID /=` | 2 | `check(new_actor("other actor").p_id_number /= new_actor("another actor").p_id_number, "Failed to create unique actors");` (stimuli/vhdl/subs/vunit/vunit/vhdl/com/test/tb_com.vhd) |
| 371 | `. ID :` | 2 | `variable k.k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2633.vhd) |
| 372 | `. NUM #` | 2 | `variable total_time : real := 5#1234.4321#E-10;   --No_failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2718.vhd) |
| 373 | `. NUM =` | 2 | `assert NOT(2.12E6 = (1.06E3*2.0E3))` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2570.vhd) |
| 374 | `. NUM then` | 2 | `if phase = 0.0 then` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug24064/pp_fir_filter.vhd) |
| 375 | `/ ID :` | 2 | `variable k/k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2634.vhd) |
| 376 | `: ID :` | 2 | `constant   one    : integer := 16:E:E1;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2854.vhd) |
| 377 | `: ID >` | 2 | `if gen_case : DIM > 1 generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2592/bug2.vhdl) |
| 378 | `: assert ID` | 2 | `erlab : assert FALSE;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1235.vhd) |
| 379 | `: if (` | 2 | `l_if_1: if (cnt = 0) then report "IF1: CNT = 0";` (stimuli/vhdl/subs/nvc/test/regress/cover3.vhd) |
| 380 | `: loop end` | 2 | `L1 : loop` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1513.vhd) |
| 381 | `: when NUM` | 2 | `when 2 => NULL:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1470.vhd) |
| 382 | `:= NUM ;` | 2 | `signal n : integer range v'range := 3;` (stimuli/vhdl/subs/nvc/test/regress/wait14.vhd) |
| 383 | `; -- -` | 2 | `function testp (I1:Bit) return;  --- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2871.vhd) |
| 384 | `; -- end` | 2 | `alias status_type is work.system_types.system_status;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_09_ch_09_03.vhd) |
| 385 | `; -- private` | 2 | `type bounded_buffer_object;  -- private` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_17_fg_17_08.vhd) |
| 386 | `; assert ID` | 2 | `k <= ;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1722.vhd) |
| 387 | `; process (` | 2 | `Delays					<= Delays(Delays'high - 1 downto 0) & DataIn when rising_edge(Clock);` (stimuli/vhdl/subs/PoC/src/misc/filter/filter_mean.vhdl) |
| 388 | `< ID :` | 2 | `variable k<k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2635.vhd) |
| 389 | `<= " NUM` | 2 | `as_q1: q1 <= "0000" & b;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue150/concat2.vhdl) |
| 390 | `<= ( others` | 2 | `proc: p <= (others => '1');` (stimuli/vhdl/subs/nvc/test/elab/clone2.vhd) |
| 391 | `<= ID .` | 2 | `update_a: a <= r.x after 1 ns;` (stimuli/vhdl/subs/nvc/test/regress/record11.vhd) |
| 392 | `<= ID or` | 2 | `res_or_1 <= lhs_or_1 or rhs_or_1;` (stimuli/vhdl/subs/nvc/test/regress/cover7.vhd) |
| 393 | `<= NUM ;` | 2 | `bad: sig <= 0;` (stimuli/vhdl/subs/nvc/test/elab/issue184.vhd) |
| 394 | `<= not (` | 2 | `func : y <= not ((a1 and a2) or (b1 and b2));` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_ch_05_23.vhd) |
| 395 | `<= transport ID` | 2 | `asym_delay : z <= transport a after Tpd_01 when a = '1' else` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_05_ch_05_16.vhd) |
| 396 | `= ID :` | 2 | `variable k=k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2622.vhd) |
| 397 | `= NUM ;` | 2 | `variable   s := 10;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1453.vhd) |
| 398 | `> ' ID` | 2 | `report (1 to v => 'X');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2148/e2.vhdl) |
| 399 | `> ( ID` | 2 | `w <= (byte'range => (b and mask), others => '1');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1493/test.vhdl) |
| 400 | `> ID :` | 2 | `variable k>k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2636.vhd) |
| 401 | `> open ,` | 2 | `test_out(15 ) => open,` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3109/file2.vhdl) |
| 402 | `? ( ID` | 2 | `case? (dec_vld) is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2620/casetb.vhdl) |
| 403 | `? ID :` | 2 | `variable k?k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2637.vhd) |
| 404 | `?= " NUM` | 2 | `r := l ?= "110";` (stimuli/vhdl/subs/ghdl/testsuite/synth/err01/err_slv2.vhdl) |
| 405 | `?> = '` | 2 | `check_equal('1' ?>= '1', '1');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_ge.vhd) |
| 406 | `?? ( ID` | 2 | `if ?? (tg) and falling_edge(clk) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc15.vhdl) |
| 407 | `?? ID and` | 2 | `if ?? tg and falling_edge(clk) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc13.vhdl) |
| 408 | `?? ID then` | 2 | `if ?? tg then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue818/tc1.vhdl) |
| 409 | `ID %` | 2 | `d%` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash1.vhdl) |
| 410 | `ID & ID` | 2 | `with a&b select` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2486/bug.vhdl) |
| 411 | `ID ( constant` | 2 | `procedure pkt_report(` (stimuli/vhdl/subs/nvc/test/regress/issue1186.vhd) |
| 412 | `ID , --` | 2 | `-- Use MODEL_ID_NAME Generic if set, otherwise,` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/src/AxiStreamReceiver.vhd) |
| 413 | `ID : --` | 2 | `H:--###############################################################################` (stimuli/vhdl/subs/Rudi-RV32I/c/template_program_memory.vhd) |
| 414 | `ID : for` | 2 | `nibble_array : for nibble_index in 0 to 7 generate` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_14_fg_14_04.vhd) |
| 415 | `ID : if` | 2 | `end_gen : if index = 7 generate` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_20_fg_20_07.vhd) |
| 416 | `ID ; )` | 2 | `generic ( g : in natural ; ) ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_071a.vhd) |
| 417 | `ID ; attribute` | 2 | `end rr1;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc955.vhd) |
| 418 | `ID ; pure` | 2 | `type type1;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2693/testbench.vhdl) |
| 419 | `ID ; wait` | 2 | `sig(C_VAL'range) <= force C_VAL;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2590/test2.vhdl) |
| 420 | `ID <= '` | 2 | `write <= '0';` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1155/tb_ent.vhdl) |
| 421 | `ID ID (` | 2 | `return ite ((s < 0.0), "-" & res, res);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/strings.vhdl) |
| 422 | `ID is --` | 2 | `case err is                           -- illegal location for case statement` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc821.vhd) |
| 423 | `ID is begin` | 2 | `ARCHITECTURE c04s01b00x00p08n01i00005arch OFc04s01b00x00p08n01i00005ent IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc5.vhd) |
| 424 | `ID is end` | 2 | `ENTITY c02s05b00x00p02n01i02985ent IS` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2985.vhd) |
| 425 | `ID is private` | 2 | `type element_type is private;                         -- any type` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_059.vhd) |
| 426 | `ID range -` | 2 | `port (a : integer range -3 to 2;` (stimuli/vhdl/subs/ghdl/testsuite/synth/case02/case05.vhdl) |
| 427 | `ID range <>` | 2 | `type t_slv32_x3 is array(1 tnatural range <>) of bit_vector(31 downto 0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/arr_err1.vhdl) |
| 428 | `ID register ;` | 2 | `signal source1, source2 : resolve_unique word register;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_fg_16_02.vhd) |
| 429 | `ID select --` | 2 | `with i select                        -- No_failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1766.vhd) |
| 430 | `ID then ID` | 2 | `if g_bug_mode then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2452/foo.vhdl) |
| 431 | `NUM . NUM` | 2 | `constant x : real := abs 10.5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2308.vhd) |
| 432 | `NUM => ID` | 2 | `1 => std_logic_vector'(x"01"));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2112/abc_tb2.vhdl) |
| 433 | `NUM ID ID` | 2 | `CBITS => 10                     -- 10 capacity bits for the quire` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue664/numeric_system_pkg-orig.vhdl) |
| 434 | `NUM downto NUM` | 2 | `Indices(abc_BUS_SETTINGS.PAIRS - 1 downto 0)(abc_BUS_SETTINGS.BW - 1 downto 0)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2112/abc_tb.vhdl) |
| 435 | `[ ID :` | 2 | `variable k[k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2626.vhd) |
| 436 | `[ ] (` | 2 | `if g[](0)then if 0='0'then` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/sign01.vhdl) |
| 437 | `\ ID :` | 2 | `variable k\k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2638.vhd) |
| 438 | `] ID :` | 2 | `variable k]k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2627.vhd) |
| 439 | `^ ID :` | 2 | `variable k^k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2614.vhd) |
| 440 | `` ID :` | 2 | `variable k`k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2623.vhd) |
| 441 | `assert ID =` | 2 | `postponed assert s = '0' severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug051/tb.vhdl) |
| 442 | `begin ID '` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash41.vhdl) |
| 443 | `begin end ;` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/sr2553/2553.vhd) |
| 444 | `begin end ID` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc791.vhd) |
| 445 | `begin postponed process` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_21_fg_21_05.vhd) |
| 446 | `begin return ;` | 2 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc803.vhd) |
| 447 | `bus ; ID` | 2 | `q : out resolved_byte bus;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_16_fg_16_05.vhd) |
| 448 | `context is` | 2 | `context is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash46.vhdl) |
| 449 | `end ;` | 2 | `end;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1514/ent.vhdl) |
| 450 | `end loop ID` | 2 | `end loop L;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1669.vhd) |
| 451 | `end package ;` | 2 | `end package;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2076/gcrash-9a.vhdl) |
| 452 | `for ID ,` | 2 | `for m1, m2 : nfet` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/components-and-configs/inline_02a.vhd) |
| 453 | `function " ID` | 2 | `function "an" & "d" return BOOLEAN;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2877.vhd) |
| 454 | `function ID return` | 2 | `function mytime return boolean is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2866.vhd) |
| 455 | `generate assert ID` | 2 | `else generate` (stimuli/vhdl/subs/PoC/src/fifo/fifo_cc_got.vhdl) |
| 456 | `generate end generate` | 2 | `else generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2982/nfmac.vhdl) |
| 457 | `guarded ' NUM` | 2 | `err   <= transport guarded '1';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1742.vhd) |
| 458 | `if ) ID` | 2 | `if)h';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash20.vhdl) |
| 459 | `impure function ID` | 2 | `impure function now return delay_length;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_07_ch_07_04.vhd) |
| 460 | `is constant ID` | 2 | `block is` (stimuli/vhdl/subs/nvc/test/regress/seqblock1.vhd) |
| 461 | `on ID until` | 2 | `wait on sig1 until x >= 12 for 20 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1214.vhd) |
| 462 | `out ID (` | 2 | `procedure to_t( signal sa : out bit_vector(31 downto 0))  is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue43/repro2.vhdl) |
| 463 | `package body ;` | 2 | `end package body;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue419/bug.vhdl) |
| 464 | `procedure ID ;` | 2 | `procedure proccall;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1718.vhd) |
| 465 | `process is --` | 2 | `p : postponed process is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_21_ch_21_02.vhd) |
| 466 | `pure function "` | 2 | `pure function "+" (a, b : fft_data_vector) return fft_data_vector;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2572/fft_types.vhdl) |
| 467 | `range ID "` | 2 | `type I5 is range B"000" to B"111";  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc249.vhd) |
| 468 | `record ; end` | 2 | `end record ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_082.vhd) |
| 469 | `sll ID ;` | 2 | `slv_sll <= slv sll int;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1520/test.vhdl) |
| 470 | `sll NUM )` | 2 | `assert (b sll 1) = "0110";` (stimuli/vhdl/subs/nvc/test/regress/issue1529.vhd) |
| 471 | `sll NUM =` | 2 | `assert B"10001010" sll 3  =  B"01010000";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_04_ch_04_07.vhd) |
| 472 | `srl NUM ;` | 2 | `ma_half_fdiv <= ma_fdiv srl 1;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1655/absenc_master.vhdl) |
| 473 | `to - NUM` | 2 | `when integer'Low to -11 | 6 to 100 =>   k := 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1488.vhd) |
| 474 | `to ID +` | 2 | `when C+2 to C+5 => -- . . .` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/non_compliant/ch_03_ch_03_09.vhd) |
| 475 | `to ID -` | 2 | `constant zeros : bit_vector(0 to width-1) := (0 to width-1 => '0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1387/repro5.vhdl) |
| 476 | `when NUM =>` | 2 | `when 1 => k := 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1464.vhd) |
| 477 | `{ ID :` | 2 | `variable k{k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2624.vhd) |
| 478 | `| ID '` | 2 | `CONSTANT C2_csa1_1 : t_csa1_1 := ( t_csa1_1'LEFT|t_csa1_1'RIGHT=>C2_scl1,` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2012.vhd) |
| 479 | `| ID :` | 2 | `variable k|k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2639.vhd) |
| 480 | `} ID :` | 2 | `variable k}k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2625.vhd) |
| 481 | `~ ID :` | 2 | `variable k~k : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2620.vhd) |
| 482 | ` restrict [` | 1 | `architecturerestrict[=to 0` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2110/psl2.vhdl) |
| 483 | `! : ID` | 1 | `variable k! : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2581.vhd) |
| 484 | `! ; end` | 1 | `assert 0!;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/psl04.vhdl) |
| 485 | `" " "` | 1 | `report """""";` (stimuli/vhdl/subs/nvc/test/parse/issue205.vhd) |
| 486 | `" " &` | 1 | `debug(memory_bfm_logger, "Writing x""" & to_hstring(data) & """ to address x""" & to_hstring(address) & """");` (stimuli/vhdl/subs/vunit/examples/vhdl/com/test/memory_bfm.vhd) |
| 487 | `" " begin` | 1 | `package function""begin r';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash44.vhdl) |
| 488 | `" " for` | 1 | `configuration"` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2110/conf1.vhdl) |
| 489 | `" ' NUM` | 1 | `variable bit_str : bit_vector (1 to 8) := b"'0''1''0''1''0''1''0''1'";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2757.vhd) |
| 490 | `" ) "` | 1 | `Std_logic'image(reentry_guard) ")"; -- MISSING AMPERSAND` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue478/repro.vhdl) |
| 491 | `" /= "` | 1 | `function "/=" (L, R: std_logic) return std_logic is` (stimuli/vhdl/subs/ghdl/testsuite/gna/perf02-long/fsm_163.vhd) |
| 492 | `" : ID` | 1 | `variable k" : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2601.vhd) |
| 493 | `" <= "` | 1 | `function "<=" (l, r : my_vec) return boolean;` (stimuli/vhdl/subs/nvc/test/parse/explicit.vhd) |
| 494 | `" >= "` | 1 | `function ">="(x, y : unsigned) return boolean;` (stimuli/vhdl/subs/nvc/test/parse/implicit.vhd) |
| 495 | `" ID ,` | 1 | `check_core_failure("Invalid ID name ""parent,child""");` (stimuli/vhdl/subs/vunit/vunit/vhdl/data_types/test/tb_id.vhd) |
| 496 | `" ID --` | 1 | `constant BASE_A : std_ulogic_vector(31 downto 0) := X"FFFF_02--";` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3056/top.vhdl) |
| 497 | `" NUM .` | 1 | `variable bit_str : bit_vector (1 to 8) := b"0101.0101";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2756.vhd) |
| 498 | `" NUM NUM` | 1 | `variable bit_str : bit_vector (1 to 8) := b"0101 0101";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2755.vhd) |
| 499 | `# : ID` | 1 | `variable k# : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2582.vhd) |
| 500 | `% % %` | 1 | `constant   a : string := %%%%;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2860.vhd) |
| 501 | `% % ID` | 1 | `%%d%` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash0.vhdl) |
| 502 | `% : ID` | 1 | `variable k% : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2583.vhd) |
| 503 | `% ID %` | 1 | `constant   two   : bit_vector := X%FF%;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2862.vhd) |
| 504 | `% ID =>` | 1 | `x%x => stimuli, -- <==` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug24326/tb_thingy7.vhdl) |
| 505 | `% ID ID` | 1 | `report %This string is illegal because of this character ".%` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2859.vhd) |
| 506 | `% NUM "` | 1 | `variable bit_str : bit_vector(0 to 7) := %01010101";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2857.vhd) |
| 507 | `% NUM %` | 1 | `CONSTANT i : bit_vector := B%1111_1111_1111%;` (stimuli/vhdl/subs/nvc/test/parse/bitstring.vhd) |
| 508 | `% NUM ;` | 1 | `constant c : natural := 5 % 4;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug24326/repro1.vhdl) |
| 509 | `& " NUM` | 1 | `"0101" &` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3002/top_concat_16k.vhdl) |
| 510 | `& : ID` | 1 | `variable k& : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2585.vhd) |
| 511 | `& ID <=` | 1 | `a & b <= "01";` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2613/mre.vhdl) |
| 512 | `' ' ,` | 1 | `' ', '!', '"', '#', '$', '%', '&', ''',` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2373.vhd) |
| 513 | `' ( '` | 1 | `(bit'('1'), y, z) <=  v;        -- Error` (stimuli/vhdl/subs/nvc/test/sem/signal.vhd) |
| 514 | `' , others` | 1 | `constant vec_msb       : std_logic_vector(tamanho - 1 downto 0) := (tamanho - 1 => '1', others => '0');` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2461/pacote_aux.vhdl) |
| 515 | `' : ID` | 1 | `variable k' : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2599.vhd) |
| 516 | `' ID <=` | 1 | `S1'DELAYED <= S2;  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc178.vhd) |
| 517 | `' ID is` | 1 | `function A(v : integer) return i000'er is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/func5.vhdl) |
| 518 | `' ID port` | 1 | `t:entity k't port map(0);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/aspect02.vhdl) |
| 519 | `' range '` | 1 | `subtype natural_range_record is natural'range'record ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_099.vhd) |
| 520 | `' range loop` | 1 | `for j in cst'element'range loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1772/repro2.vhdl) |
| 521 | `' subtype (` | 1 | `signal sig1 : sig0'subtype(0 to 3);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2267/ent.vhdl) |
| 522 | `( " -` | 1 | `result := std_ulogic_vector'("-000") ?/= std_ulogic_vector'("0000");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue262/repro.vhdl) |
| 523 | `( " |` | 1 | `WRITE(text_line, string'(" | "));` (stimuli/vhdl/subs/nvc/test/regress/issue351.vhd) |
| 524 | `( ( others` | 1 | `(W,V) <= (AGGREGATE_ARRAY'((others => '$'),( others => '$' )));` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2462.vhd) |
| 525 | `( ) --` | 1 | `lab : block ( )                   --guard condition must not be empty` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1657.vhd) |
| 526 | `( ) return` | 1 | `entity function()return n(of` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2768/missing_id.vhdl) |
| 527 | `( : ID` | 1 | `variable k( : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2587.vhd) |
| 528 | `( ID *` | 1 | `ram(to_integer(mem_addr))(i*8+7 downto i*8) <= (others => 'X');` (stimuli/vhdl/subs/PoC/tb/mem/mem_model.vhdl) |
| 529 | `( NUM *` | 1 | `mem(to_integer(unsigned(mem_write_address)))(8*b+7 downto 8*b) := avm_writedata_i(8*b+7 downto 8*b);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2185/avm_memory.vhd) |
| 530 | `( NUM while` | 1 | `function(0is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash6.vhdl) |
| 531 | `( NUM |` | 1 | `b1 := BYTE'(0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => '1');` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2853.vhd) |
| 532 | `) ) '` | 1 | `log(log_file, (integer'IMAGE(-(10 ** i))))'LENGTH;` (stimuli/vhdl/subs/ghdl/testsuite/gna/sr3060/integer_class.vhdl) |
| 533 | `) ) )` | 1 | `check_match(counter_out, ())))` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue563/repro.vhdl) |
| 534 | `) , NUM` | 1 | `constant c : Indices_t := (0 => bit_vector'(x"00"),` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2112/abc_tb3.vhdl) |
| 535 | `) : ID` | 1 | `variable k) : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2588.vhd) |
| 536 | `) := ID` | 1 | `) := Test(abc_bus);` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2112/abc.vhdl) |
| 537 | `) ; ID` | 1 | `bus_in : in bus_t(data(7 downto 0));` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1324/foo.vhdl) |
| 538 | `) ; constant` | 1 | `) ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_036a.vhd) |
| 539 | `) ; procedure` | 1 | `signal s : trans (bod(0 to 3));` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue788/repro.vhdl) |
| 540 | `) ; wait` | 1 | `xyzzy( std_ulogic_vector( v ) => unsigned( x ) );` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket97/bug.vhdl) |
| 541 | `) begin assert` | 1 | `TESTING: PROCESS(   )` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1682.vhd) |
| 542 | `) end architecture` | 1 | `process()` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/proc1.vhdl) |
| 543 | `) is --` | 1 | `) is                                -- Crash after error here` (stimuli/vhdl/subs/nvc/test/parse/issue942.vhd) |
| 544 | `) return ID` | 1 | `function trigger_rising() return std_logic_vector is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue563/tb_counter.vhdl) |
| 545 | `* ; --` | 1 | `use bar.*;                              -- Error` (stimuli/vhdl/subs/nvc/test/parse/visibility4.vhd) |
| 546 | `** NUM <=` | 1 | `S1**2   <= S1;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1277.vhd) |
| 547 | `+ : ID` | 1 | `variable k+ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2589.vhd) |
| 548 | `+ ID )` | 1 | `(a + b) := 10;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1353.vhd) |
| 549 | `+ NUM /=` | 1 | `assert A * +5 /= 0;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2466/t3.vhdl) |
| 550 | `, ( NUM` | 1 | `x <= (1, 1), (2, 2) after 2 ns, (3, 3) after 4 ns;` (stimuli/vhdl/subs/nvc/test/regress/implicit6.vhd) |
| 551 | `, -- ID` | 1 | `(natural range <>,  -- Address, specifies one word` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug18659/crash.vhd) |
| 552 | `, ID "` | 1 | `X"3f", X"1d", X"11", X"5f", X"6f", X"00", X"1a", X"0f", X"1e", X"a8", X"2e", X"61", X"32", X"bc", X"79",` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug040/tb.vhd) |
| 553 | `, ID +` | 1 | `x <= y + 1, y + 2 after 2 ns when z > 0 else 0;` (stimuli/vhdl/subs/nvc/test/regress/cond3.vhd) |
| 554 | `, ID :` | 1 | `a, x : in;                      -- Error` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_45a.vhd) |
| 555 | `, NUM )` | 1 | `type er1 is array(5,2) of integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc327.vhd) |
| 556 | `, NUM ;` | 1 | `constant v1: x1 := 0, 0;   -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2376.vhd) |
| 557 | `- : ID` | 1 | `variable k- : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2591.vhd) |
| 558 | `- ID (` | 1 | `A(1 to 2) := A(1) & - A(2);  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2191.vhd) |
| 559 | `- ID )` | 1 | `if ((x + -z) < (y + x)) then  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1802.vhd) |
| 560 | `- ID <=` | 1 | `TWO - ONE <= S1;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1280.vhd) |
| 561 | `- NUM /=` | 1 | `assert A * -5 /= 0;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2466/t1.vhdl) |
| 562 | `-- -- --` | 1 | `-------------------------------------------------------------------------` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket89/project/src93/adaptations_pkg.vhd) |
| 563 | `-- use ID` | 1 | `-- use model instance label (preferred if set as entityname_1)` (stimuli/vhdl/subs/OsvvmLibraries/AXI4/AxiStream/src/AxiStreamTransmitter.vhd) |
| 564 | `. . .` | 1 | `type t is . . .;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/subprograms/ent.vhd) |
| 565 | `. : ID` | 1 | `variable k. : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2603.vhd) |
| 566 | `. ID +` | 1 | `sum := sum + r(i).x + r(i).y;` (stimuli/vhdl/subs/nvc/test/regress/conv8.vhd) |
| 567 | `. ID is` | 1 | `configuration foo_cfg of foo.foo_e is` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_23.vhd) |
| 568 | `. ID report` | 1 | `assert OUTPUT = Vectors(i).OUTPUT_test` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug01/tb.vhdl) |
| 569 | `. NUM ID` | 1 | `constant d : time := 1.5e2 c'length;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue886/repro2.vhdl) |
| 570 | `. NUM loop` | 1 | `while v > 1.0 loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1051/psi_common_math_pkg.vhd) |
| 571 | `. all ;` | 1 | `return split_data_type(1).all;` (stimuli/vhdl/subs/vunit/vunit/vhdl/data_types/src/data_types_private_pkg.vhd) |
| 572 | `/ : ID` | 1 | `variable k/ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2604.vhd) |
| 573 | `: ; begin` | 1 | `variable based_int : integer := 3#12:;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2855.vhd) |
| 574 | `: = '` | 1 | `constant c1: character: ='A';   -- failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2577.vhd) |
| 575 | `: ID +` | 1 | `MEMORYV : MEMORYV + MEMORYV;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2044.vhd) |
| 576 | `: ID --` | 1 | `-- PROCESS: p_main` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket89/project/tb/partial_test_tb.vhd) |
| 577 | `: ID =` | 1 | `if l1: SPEED = "fast" generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0104/alt2.vhdl) |
| 578 | `: NUM #` | 1 | `variable based_int : integer := 3:12#;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2856.vhd) |
| 579 | `: NUM .` | 1 | `total_time := 5:1234.4321:E-10;      -- no_failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2861.vhd) |
| 580 | `: NUM :` | 1 | `CONSTANT g : INTEGER := 2:1_0:;` (stimuli/vhdl/subs/nvc/test/parse/based.vhd) |
| 581 | `: assert (` | 1 | `assert_a: assert ((a(3) or b(3)) = '1') report "a";` (stimuli/vhdl/subs/nvc/test/regress/ieee1.vhd) |
| 582 | `: block --` | 1 | `B:block  -- ERROR:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1697.vhd) |
| 583 | `: block is` | 1 | `b1: block is                    -- OK` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_07.vhd) |
| 584 | `: loop ID` | 1 | `L1 : Loop` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1522.vhd) |
| 585 | `: loop if` | 1 | `SYNC_LOOP: loop` (stimuli/vhdl/subs/nvc/test/regress/issue428.vhd) |
| 586 | `: null ;` | 1 | `v: null;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue140/repro.vhdl) |
| 587 | `: report "` | 1 | `rpt1: report "Report within GLOBAL procedure";` (stimuli/vhdl/subs/nvc/test/regress/cover1.vhd) |
| 588 | `: type ID` | 1 | `generic (period : time :type ns);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug090/crash14.vhdl) |
| 589 | `: wait ;` | 1 | `block_forever: wait;` (stimuli/vhdl/subs/nvc/test/parse/seq.vhd) |
| 590 | `: while (` | 1 | `L1: while  ( i < 10 )  loop       -- conditional loop` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1703.vhd) |
| 591 | `: while not` | 1 | `command_loop : while not endfile(control) loop` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_18_fg_18_10.vhd) |
| 592 | `:= ' NUM` | 1 | `V1(2 := '1' ;  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1036.vhd) |
| 593 | `; ( ID` | 1 | `all_possible_values ( test_vector, 10 ns );` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/digital-modeling/tb_full_adder.vhd) |
| 594 | `; -- alias` | 1 | `alias ground is electrical_ref;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/aliases/inline_01a.vhd) |
| 595 | `; -- all` | 1 | `proc;                           -- all unassociated` (stimuli/vhdl/subs/nvc/test/regress/issue227.vhd) |
| 596 | `; -- assert` | 1 | `check (foo);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1273/issue.vhdl) |
| 597 | `; : ID` | 1 | `variable k; : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2598.vhd) |
| 598 | `; ID :=` | 1 | `constant empty ; bit_vector := O"058";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2763.vhd) |
| 599 | `; ID ;` | 1 | `variable s_day ; day;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1506.vhd) |
| 600 | `; begin report` | 1 | `file f : text;` (stimuli/vhdl/subs/nvc/test/regress/stdenv2.vhd) |
| 601 | `; begin return` | 1 | `file fref, fref2: text;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1936/repro.vhdl) |
| 602 | `; end generate` | 1 | `assign_row(DataOut_i, Shifter_nxt(TAPS(i)), i);` (stimuli/vhdl/subs/PoC/src/misc/misc_Delay.vhdl) |
| 603 | `; end process` | 1 | `x := ;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/varcomp.vhdl) |
| 604 | `; file ID` | 1 | `file f0 : text;` (stimuli/vhdl/subs/vunit/vunit/vhdl/logging/src/file_pkg.vhd) |
| 605 | `; impure function` | 1 | `procedure Inc ;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue290/TbNamesPkg.vhd) |
| 606 | `; process constant` | 1 | `simInitialize;` (stimuli/vhdl/subs/PoC/tb/common/physical_tb.vhdl) |
| 607 | `; shared variable` | 1 | `alias T_SCOREBOARD is P_Scoreboard.ScoreBoardPType;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue202/repro.vhdl) |
| 608 | `< : ID` | 1 | `variable k< : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2605.vhd) |
| 609 | `<< << <<` | 1 | `<<<<<<< HEAD` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/merge1.vhdl) |
| 610 | `<< constant @` | 1 | `assert << constant @work.ename.c : integer >> = 42;` (stimuli/vhdl/subs/nvc/test/perf/ename.vhd) |
| 611 | `<< constant ID` | 1 | `assert << constant g(1).uut.k : integer >> = 1;` (stimuli/vhdl/subs/nvc/test/regress/ename3.vhd) |
| 612 | `<< constant ^` | 1 | `assert << constant ^.^.g(N).uut.k : integer >> = N;` (stimuli/vhdl/subs/nvc/test/regress/ename4.vhd) |
| 613 | `<< signal ^` | 1 | `assert <<signal ^.^.^.^.x : bit>> = '0';  -- Error` (stimuli/vhdl/subs/nvc/test/regress/ename17.vhd) |
| 614 | `<< variable ID` | 1 | `assert <<variable uut.x : integer>> = 0;  -- Error` (stimuli/vhdl/subs/nvc/test/regress/ename10.vhd) |
| 615 | `<= ( ID` | 1 | `res_assign: s <= (a xor b) xor cin;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1817/full_adder.vhdl) |
| 616 | `<= ( NUM` | 1 | `p1: s <= (1 to i => '1') & (i + 1 to 3 => '0');` (stimuli/vhdl/subs/nvc/test/regress/issue825.vhd) |
| 617 | `<= ID "` | 1 | `assign_p: vec <= X"52";` (stimuli/vhdl/subs/nvc/test/regress/signal9.vhd) |
| 618 | `<= ID '` | 1 | `delayed1: i2 <= i1'delayed(5 ns);` (stimuli/vhdl/subs/nvc/test/regress/implicit9.vhd) |
| 619 | `<= ID after` | 1 | `behavior : q <= d after Tpd_clk_q when clk = '1' and clk'event;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_12_fg_12_02.vhd) |
| 620 | `<= transport '` | 1 | `CONSIG: A <= transport '1' when B = '1' else` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1758.vhd) |
| 621 | `= ' NUM` | 1 | `k.x = '0';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc510.vhd) |
| 622 | `= : ID` | 1 | `variable k= : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2592.vhd) |
| 623 | `= ID ;` | 1 | `variable b1, b2 : boolean ;= true;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1492.vhd) |
| 624 | `=> null :` | 1 | `when   => NULL:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1469.vhd) |
| 625 | `> " ID` | 1 | `proc(s(4 to 15) => "Hello world!");` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket73/bug.vhdl) |
| 626 | `> " NUM` | 1 | `test(1) => "00",` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2529/outer.vhdl) |
| 627 | `> : ID` | 1 | `variable k> : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2606.vhd) |
| 628 | `> ID &` | 1 | `x(3 downto 2) => testab & '0',` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3201/tb.vhdl) |
| 629 | `> ID <=` | 1 | `S1 > S2   <= S3;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1275.vhd) |
| 630 | `> not ID` | 1 | `assert always (not rst and wr -> not is_x(d))@rising_edge(clkw)` (stimuli/vhdl/subs/vunit/examples/vhdl/array_axis_vcs/src/fifo.vhd) |
| 631 | `> open )` | 1 | `output2_o(0) => open` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2764/tb.vhdl) |
| 632 | `? /= '` | 1 | `check_equal('1' ?/= '1', '0');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_ne.vhd) |
| 633 | `? /= ID` | 1 | `z <= "1--0" ?/= a;` (stimuli/vhdl/subs/ghdl/testsuite/synth/match01/match07.vhdl) |
| 634 | `? : ID` | 1 | `variable k? : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2607.vhd) |
| 635 | `?< ' NUM` | 1 | `check_equal('1' ?< '1', '0');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_lt.vhd) |
| 636 | `?< = '` | 1 | `check_equal('1' ?<= '1', '1');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_le.vhd) |
| 637 | `?= " "` | 1 | `subtype bv4 is bit_vector(1 to 4);type vec0 is array(natural range<>)of bv4;constant s:vec0:=(x"0",""?="");procedure pri` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2116/eval1.vhdl) |
| 638 | `?= " -` | 1 | `z <= a ?= "-1--0";` (stimuli/vhdl/subs/ghdl/testsuite/synth/match01/match04.vhdl) |
| 639 | `?= " ID` | 1 | `r := z ?= "LL";` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue14/repro.vhdl) |
| 640 | `?= ID "` | 1 | `z <= a ?= b"-10-_1111_1110_1101_1100_1011_1010_1001_1000_0--1";` (stimuli/vhdl/subs/ghdl/testsuite/synth/match01/match03.vhdl) |
| 641 | `?= ID )` | 1 | `check_relation(sl_1 ?= sl_0);` (stimuli/vhdl/subs/vunit/vunit/vhdl/check/test/tb_check_relation_2008p.vhd) |
| 642 | `?= ID or` | 1 | `if ( wr and ( d?=dx or d?=du ) ) then` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1731/fifo.vhdl) |
| 643 | `?> ' NUM` | 1 | `check_equal('1' ?> '1', '0');` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_matching_operator_gt.vhd) |
| 644 | `?> = NUM` | 1 | `assert (u5 ?>= 4) = '1' severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper03/match01.vhdl) |
| 645 | `?? ID '` | 1 | `check_equal(?? std_logic'('1'), true);` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2008/tb_condition_operator.vhd) |
| 646 | `@ ID :` | 1 | `variable @k : integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2665.vhd) |
| 647 | `@ for (` | 1 | `architecture@for(""x""4000000000x"` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash15.vhdl) |
| 648 | `ID " -` | 1 | `function "-" (R : AlertCountType) return AlertCountType ;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1980/AlertLogPkg.vhd) |
| 649 | `ID " default` | 1 | `type array3 is array (10 downto 1) of real tolerance "default";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/composite-data/inline_14a.vhd) |
| 650 | `ID ( "` | 1 | `echo("1");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue394/bug.vhdl) |
| 651 | `ID ) mod` | 1 | `RETVAL := ( ( abs ARG ) mod 10 ) + 1980;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc879.vhd) |
| 652 | `ID ) report` | 1 | `assert (xo = xor xi) report "mixmatch at i=" & integer'image(i) severity ERROR;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2614/recursive_xor_tb.vhdl) |
| 653 | `ID ) return` | 1 | `function F1 ( A : linkage integer ) return boolean is -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2894.vhd) |
| 654 | `ID ) when` | 1 | `iIntReq   <= TO_X01(or IntReq) when TransactionPending(InterruptRec.Rdy, InterruptRec.Ack)` (stimuli/vhdl/subs/OsvvmLibraries/Common/src/deprecated/InterruptHandler_c.vhd) |
| 655 | `ID - ID` | 1 | `ghdl -r -gG_RST=1 toto` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue3011/toto_vec.vhdl) |
| 656 | `ID -- !` | 1 | `type Data_Type_t                  --! The data type of the input and output.` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2544/generic_fixed_delay.vhdl) |
| 657 | `ID -- =` | 1 | `-- Type declarations` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/utils.vhdl) |
| 658 | `ID . --` | 1 | `error.` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2770.vhd) |
| 659 | `ID . NUM` | 1 | `constant R : real := 10_000.0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/analog-modeling/inline_10a.vhd) |
| 660 | `ID . all` | 1 | `use work.c10s05b00x00p03n02i00941pkg1.all, work c10s05b00x00p03n02i00941pkg2.all;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc941.vhd) |
| 661 | `ID /= NUM` | 1 | `assert B ** abs A /= 0;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2466/t4.vhdl) |
| 662 | `ID : buffer` | 1 | `component A2 generic (constant G2 : buffer BOOLEAN);  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc191.vhd) |
| 663 | `ID : generate` | 1 | `else g2: generate` (stimuli/vhdl/subs/nvc/test/regress/elab34.vhd) |
| 664 | `ID : linkage` | 1 | `component A2 generic (constant G2 : linkage BOOLEAN);  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc192.vhd) |
| 665 | `ID := "` | 1 | `port(	S 	: string 	:= "abcdef");` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug018/repro.vhdl) |
| 666 | `ID ; entity` | 1 | `end pkg;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue321/repro.vhdl) |
| 667 | `ID ; library` | 1 | `end test_pkg;` (stimuli/vhdl/subs/nvc/test/regress/issue327.vhd) |
| 668 | `ID <= NUM` | 1 | `x <= 5;` (stimuli/vhdl/subs/nvc/test/sem/generate.vhd) |
| 669 | `ID <= not` | 1 | `osc  clk <= not clk after period/2;    -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1741.vhd) |
| 670 | `ID = '` | 1 | `assert and v = '0';` (stimuli/vhdl/subs/nvc/test/regress/logical3.vhd) |
| 671 | `ID ID :=` | 1 | `procedure tbFail(msg : in string := "") is` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket32/simulation.vhdl) |
| 672 | `ID ID <=` | 1 | `with B` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1764.vhd) |
| 673 | `ID ID =>` | 1 | `Rmd <= (Rmd'left downto N => '0') & unsigned(arg);` (stimuli/vhdl/subs/PoC/src/arith/arith_sqrt.vhdl) |
| 674 | `ID ID signal` | 1 | `-- 1.	you MUST initialize your matrix signal with 'Z' to get correct simulation results (iSIM, vSIM, ghdl/gtkwave)` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/vectors.vhdl) |
| 675 | `ID NUM )` | 1 | `, sel( 3 downto 0)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2549/pkg_types.vhd) |
| 676 | `ID NUM ID` | 1 | `k <= '1' aftre 10 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1324.vhd) |
| 677 | `ID downto ID` | 1 | `variable init_array      : t_sf_array(0 to samples - 1)(ret_type'left downto ret_type'right) := (others => (others => '0` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue339/test_pkg.vhdl) |
| 678 | `ID function ID` | 1 | `pure function get_wb_rst( wb_i : wb_M2S_t)` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1346/wb_rst_pkg.vhd) |
| 679 | `ID inertial '` | 1 | `x <= reject v inertial '1' after 2 ns;` (stimuli/vhdl/subs/nvc/test/regress/delay3.vhd) |
| 680 | `ID is (` | 1 | `type T_Command is (<>); --> Not yet supported by Riviera-PRO 2020.04` (stimuli/vhdl/subs/Interfaces/PoC/CSE.vhdl) |
| 681 | `ID is <>` | 1 | `type t1 is <>;` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_59.vhd) |
| 682 | `ID is protected` | 1 | `type AlertLogStructPType is protected` (stimuli/vhdl/subs/nvc/test/sem/osvvm1.vhd) |
| 683 | `ID is record` | 1 | `type evs_type is record` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2713/reprod.vhdl) |
| 684 | `ID null ;` | 1 | `if  (I /= 47) the` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1434.vhd) |
| 685 | `ID package ;` | 1 | `end package;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2/sortnet_OddEvenSort.vhdl) |
| 686 | `ID process variable` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue20/fp_write_read_issue.vhdl) |
| 687 | `ID range '` | 1 | `subtype GROUND BIT range '0' to '0';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc11.vhd) |
| 688 | `ID select ?` | 1 | `with x select? y <=` (stimuli/vhdl/subs/nvc/test/regress/issue705.vhd) |
| 689 | `ID { ID` | 1 | `FELL_0_a : assert always {a; not a} |-> fell(a);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue662/psl_fell_err1.vhdl) |
| 690 | `NUM ' '` | 1 | `variable k : bit_vector(0 to 7) := b'0''1''0''1''0''1''0''1';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2746.vhd) |
| 691 | `NUM ( "` | 1 | `architecture 0for(""x""4000000000x"` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash16.vhdl) |
| 692 | `NUM ( NUM` | 1 | `architecture 0for(4000000000x"` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash11.vhdl) |
| 693 | `NUM , NUM` | 1 | `(0, 0, 0) := (0, 0, 0);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1352.vhd) |
| 694 | `NUM /= NUM` | 1 | `assert A * abs 5 /= 0;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2466/t2.vhdl) |
| 695 | `NUM : ID` | 1 | `variable 01k : integer;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2609.vhd) |
| 696 | `NUM ; assert` | 1 | `INTV2 := abs 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2324.vhd) |
| 697 | `NUM ; begin` | 1 | `constant x : integer := abs 10;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2307.vhd) |
| 698 | `NUM ; wait` | 1 | `k <= transport 5;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1323.vhd) |
| 699 | `NUM => NUM` | 1 | `constant C1 : T1 := (1 2 => 0, others => 4) ; -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2381.vhd) |
| 700 | `NUM => null` | 1 | `1 => NULL;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1468.vhd) |
| 701 | `NUM inertial NUM` | 1 | `x <= reject 5 inertial 7;           -- Reject not time` (stimuli/vhdl/subs/nvc/test/sem/conc.vhd) |
| 702 | `NUM select ID` | 1 | `with 2 select b <=` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue9/repro2.vhdl) |
| 703 | `NUM when NUM` | 1 | `j <= transport 1 when 1,` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1772.vhd) |
| 704 | `[ * NUM` | 1 | `architecture restrict[*9000000000` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash12.vhdl) |
| 705 | `[ : ID` | 1 | `variable k[ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2596.vhd) |
| 706 | `\ : ID` | 1 | `variable k\ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2608.vhd) |
| 707 | `\ : \` | 1 | `signal \:\ : xyz := (others => (others => '1'));` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue3232/tb.vhdl) |
| 708 | `\ ID .` | 1 | `signal \extend.id\ : std_ulogic := 'H';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1228/test_load.vhdl) |
| 709 | `\ ID <` | 1 | `port ( \a<0>\, \a<1>\, \a<2>\, \a<3>\ : in std_ulogic;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ap_a_fg_a_11.vhd) |
| 710 | `\ ID ID` | 1 | `signal \foo bar\ : integer;` (stimuli/vhdl/subs/nvc/test/parse/extended.vhd) |
| 711 | `] : ID` | 1 | `variable k] : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2597.vhd) |
| 712 | `^ : ID` | 1 | `variable k^ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2584.vhd) |
| 713 | `` : ID` | 1 | `variable k` : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2593.vhd) |
| 714 | `` ID ID` | 1 | ``protect begin_protected` (stimuli/vhdl/subs/nvc/test/parse/protect1.vhd) |
| 715 | `` entity ID` | 1 | ``entity fum is` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug099/repro.vhdl) |
| 716 | `access ; --` | 1 | `type ADDRESS is access ; --- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc514.vhd) |
| 717 | `after ( ID` | 1 | `S <= '0' after (t1 - t2);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1333.vhd) |
| 718 | `after - ID` | 1 | `S <= 1 ns after - S;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/simulator_failure/tc1336.vhd) |
| 719 | `after - NUM` | 1 | `S <= '0' after -5 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1334.vhd) |
| 720 | `after ID (` | 1 | `s <= not s after random(1 ns, 2 ns);` (stimuli/vhdl/subs/nvc/test/regress/issue527.vhd) |
| 721 | `after ID *` | 1 | `sbi_rdata <= sbi_rdata_i after GC_FIXED_WAIT_CYCLES_READ * GC_CLK_PERIOD;` (stimuli/vhdl/subs/UVVM/bitvis_vip_sbi/tb/maintenance_tb/sbi_slave.vhd) |
| 722 | `after ID when` | 1 | `q <= d after Tpd when To_bit(latch_en) = '1';` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_15_latch-b.vhd) |
| 723 | `alias variable ID` | 1 | `alias` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/alias1.vhdl) |
| 724 | `and ID <=` | 1 | `S1 and S2   <= S3;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1276.vhd) |
| 725 | `architecture ID ;` | 1 | `end architecture tb;` (stimuli/vhdl/subs/nvc/test/regress/issue1472.vhd) |
| 726 | `assert ID '` | 1 | `assert CHARACTER'('1')/=BIT'('1') report "oops"; -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2928.vhd) |
| 727 | `assert ID <` | 1 | `postponed assert x < 10;` (stimuli/vhdl/subs/nvc/test/regress/assert5.vhd) |
| 728 | `assert ID >=` | 1 | `assert V2 >= 10;  -- failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2994.vhd) |
| 729 | `assert ID severity` | 1 | `assert false severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2209/e.vhdl) |
| 730 | `assert ID {` | 1 | `postponed assert always {s = '0'; s = '1'} severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug051/tb2.vhdl) |
| 731 | `assert not (` | 1 | `assert NOT( gimme_value = 10 )` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc893.vhd) |
| 732 | `attribute ID ID` | 1 | `attribute p  POSITIVE;         --Failure Here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc181.vhd) |
| 733 | `begin   ID` | 1 | `package function begin n';` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash24.vhdl) |
| 734 | `begin ; architecture` | 1 | `BEGIN` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc714.vhd) |
| 735 | `begin ID (` | 1 | `begin` (stimuli/vhdl/subs/nvc/test/regress/proc12.vhd) |
| 736 | `begin NUM` | 1 | `package body function begin 0package` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash28.vhdl) |
| 737 | `begin end block` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1656.vhd) |
| 738 | `begin exit ;` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc802.vhd) |
| 739 | `begin generic (` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc795.vhd) |
| 740 | `begin if (` | 1 | `begin` (stimuli/vhdl/subs/PoC/src/arith/arith_shifter_barrel.vhdl) |
| 741 | `begin next ;` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc801.vhd) |
| 742 | `begin port (` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc794.vhd) |
| 743 | `begin process begin` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc793.vhd) |
| 744 | `begin process variable` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/synth/attr02/name02.vhdl) |
| 745 | `begin return ID` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1621.vhd) |
| 746 | `begin wait ;` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1696.vhd) |
| 747 | `begin wait NUM` | 1 | `begin` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc796.vhd) |
| 748 | `block generate signal` | 1 | `signal g : std_logic_vector(K-1 downto 1);  -- Block Generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug037/arith_addw.vhdl) |
| 749 | `bus := '` | 1 | `signal s : std_logic bus := 'H';` (stimuli/vhdl/subs/nvc/test/regress/guard3.vhd) |
| 750 | `bus ; --` | 1 | `signal s1 : bit bus; -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc74.vhd) |
| 751 | `bus ; disconnect` | 1 | `signal a : std_logic bus;` (stimuli/vhdl/subs/nvc/test/regress/issue829.vhd) |
| 752 | `component ; end` | 1 | `end component;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc786.vhd) |
| 753 | `component ID --` | 1 | `component C  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2926.vhd) |
| 754 | `configuration ID of` | 1 | `configuration C of E is               -- component illegal here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc785.vhd) |
| 755 | `constant ID ,` | 1 | `CONSTANT a,b    : INTEGER := 2;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc138.vhd) |
| 756 | `disconnect all :` | 1 | `disconnect all : BIT after 0 ns;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1695.vhd) |
| 757 | `disconnect architecture ID` | 1 | `disconnect` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug090/hang2.vhdl) |
| 758 | `downto ID +` | 1 | `SLV1 <= (SLV1'left downto Index+1 => '0') & '1' & (Index-1 downto 0 => '0');` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2581/aggr.vhdl) |
| 759 | `downto ID =>` | 1 | `x <= (31 downto WIDTH => '0') & y1;` (stimuli/vhdl/subs/nvc/test/regress/elab20.vhd) |
| 760 | `downto NUM )` | 1 | `alias  address_ms       : std_logic_vector(3 downto 0)   is   processor_address(15 downto 12);` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug18280/alias_bug.vhd) |
| 761 | `end ; elsif` | 1 | `end;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue106/ent2.vhdl) |
| 762 | `end ; entity` | 1 | `end;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2984.vhd) |
| 763 | `end block ;` | 1 | `end block;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2448/debouncer_no_vunit_tb.vhdl) |
| 764 | `end block ID` | 1 | `end block BL;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1663.vhd) |
| 765 | `end if ;` | 1 | `end if;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/proc4.vhdl) |
| 766 | `end loop ;` | 1 | `end loop;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1556.vhd) |
| 767 | `entity ID (` | 1 | `entity E (PT:BIT) is   -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2924.vhd) |
| 768 | `file ID ;` | 1 | `type A1 is file ARR;     -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc543.vhd) |
| 769 | `for ID (` | 1 | `for bit_array ( 0 to n - 1 )` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-ams/ashenden/compliant/generators/last_pass_spice.vhd) |
| 770 | `for ID --` | 1 | `for BLOCK_LABEL1   -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2995.vhd) |
| 771 | `for ID use` | 1 | `for c01s03b00x00p02n01i00829_arch` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc829.vhd) |
| 772 | `function " +` | 1 | `function "+" (I1:Bit) return bit;  --- No_Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc2874.vhd) |
| 773 | `function " abs` | 1 | `function "abs " return real is` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2875.vhd) |
| 774 | `function ; function` | 1 | `end function;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2/sortnet_OddEvenSort_tb.vhdl) |
| 775 | `generate -- use` | 1 | `elsif (DEV_INFO.Vendor = VENDOR_XILINX) generate` (stimuli/vhdl/subs/PoC/src/sync/sync_Bits.vhdl) |
| 776 | `generate ID :` | 1 | `else generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2564/mwe.vhdl) |
| 777 | `generate signal ID` | 1 | `else generate` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2614/recursive_xor.vhdl) |
| 778 | `generic ( procedure` | 1 | `generic (procedure preal(value : out real);` (stimuli/vhdl/subs/nvc/test/regress/gensub6.vhd) |
| 779 | `generic ( pure` | 1 | `generic ( pure function f1 (x, y : integer) return integer );  -- OK` (stimuli/vhdl/subs/nvc/test/parse/issue917.vhd) |
| 780 | `if ; assert` | 1 | `en if;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1439.vhd) |
| 781 | `in " ID` | 1 | `report "***FAILED TEST: c08s04b00x00p04n01i01290 - A port whose mode is "IN" can not be on the left-hand side of a signa` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1290.vhd) |
| 782 | `in " or` | 1 | `report "***FAILED TEST: c08s04b00x00p04n01i01284 - A port whose mode is "IN" or "LINKAGE" can not be on the left-hand si` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1284.vhd) |
| 783 | `is ; NUM` | 1 | `architecture function is;0package` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash18.vhdl) |
| 784 | `is NUM` | 1 | `architecture function is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash17.vhdl) |
| 785 | `is begin report` | 1 | `process (vec(0)) is` (stimuli/vhdl/subs/nvc/test/regress/attr7.vhd) |
| 786 | `is end ;` | 1 | `context b is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash23_1.vhdl) |
| 787 | `is private )` | 1 | `generic ( g1 : type is private );   -- OK` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_16.vhd) |
| 788 | `library ID ;` | 1 | `library work;      -- ERROR:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc3015.vhd) |
| 789 | `library use ID` | 1 | `library use T.context is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash23.vhdl) |
| 790 | `linkage " ID` | 1 | `report "***FAILED TEST: c08s04b00x00p04n01i01285 - A port whose mode is "LINKAGE" can not be on the left-hand side of a ` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1285.vhd) |
| 791 | `loop -- ID` | 1 | `for i in 4 loop                 -- Error` (stimuli/vhdl/subs/nvc/test/parse/range1.vhd) |
| 792 | `loop ; assert` | 1 | `end loop;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1584.vhd) |
| 793 | `loop ID (` | 1 | `package function is loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash34.vhdl) |
| 794 | `loop ID :=` | 1 | `for foo in integer range MY_WORD'range loop` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1516.vhd) |
| 795 | `loop for ID` | 1 | `for a in std_ulogic range '0' to '1' loop` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2486/bug3.vhdl) |
| 796 | `null : assert` | 1 | `NULL:` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1441.vhd) |
| 797 | `of ID ;` | 1 | `function resize(x : std_logic_vector) return rv_t of std_logic_vector ;` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_072b.vhd) |
| 798 | `on ID '` | 1 | `WAIT ON cmd'TRANSACTION;` (stimuli/vhdl/subs/nvc/test/regress/issue640.vhd) |
| 799 | `on ID .` | 1 | `wait on s.x;` (stimuli/vhdl/subs/nvc/test/regress/attr14.vhd) |
| 800 | `on ID for` | 1 | `wait on sig for 1000 ms;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2832/ent.vhdl) |
| 801 | `others => ID` | 1 | `arr <= (B"11111111" others => B"00000000");` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1500/test.vhdl) |
| 802 | `out NUM ;` | 1 | `s <= force out 1;               -- OK` (stimuli/vhdl/subs/nvc/test/sem/force.vhd) |
| 803 | `procedure " and` | 1 | `procedure "and";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2873.vhd) |
| 804 | `process ( ID` | 1 | `process(clk)` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1081/test.vhdl) |
| 805 | `process ; end` | 1 | `end process;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1438.vhd) |
| 806 | `process ID ;` | 1 | `END PROCESS TESTING;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1616.vhd) |
| 807 | `process begin wait` | 1 | `process` (stimuli/vhdl/subs/nvc/test/regress/cover28.vhd) |
| 808 | `protected -- -` | 1 | `type RandomPType is protected` (stimuli/vhdl/subs/OsvvmLibraries/osvvm/RandomPkg.vhd) |
| 809 | `protected generic (` | 1 | `type test_t is protected` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_034.vhd) |
| 810 | `protected private variable` | 1 | `type t_test1 is protected` (stimuli/vhdl/subs/nvc/test/sem/lcs2016_33.vhd) |
| 811 | `protected type ID` | 1 | `type p is protected` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue470/e.vhdl) |
| 812 | `range ( ID` | 1 | `type t3 is range (ns/fs) downto (fs/fs);` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/disputed/tc238.vhd) |
| 813 | `range - (` | 1 | `type my_empty_range is range -(-8) to 7;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue810/nullrng2.vhdl) |
| 814 | `range <> )` | 1 | `a:c;n:c;s:s;t:std_logic;end record;type fa_array is array(0 range<>)of rc_data;constant e:fa_array:=(('0','0','0','%'),(` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash45.vhdl) |
| 815 | `range NUM #` | 1 | `type   INIT_1 is range 16#1# to 16#FF#;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1282.vhd) |
| 816 | `record ; --` | 1 | `end record;  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc502.vhd) |
| 817 | `record ; begin` | 1 | `end record;` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug0100/emptyrec.vhdl) |
| 818 | `record ; constant` | 1 | `type t_data_segment is record end record;` (stimuli/vhdl/subs/nvc/test/parse/aggregate2.vhd) |
| 819 | `record is ID` | 1 | `type t_frame_x record is` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1196/my_pkg.vhdl) |
| 820 | `register ; --` | 1 | `signal s1 : bit register; -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc73.vhd) |
| 821 | `report " ID` | 1 | `report "This should not get through";` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc1241.vhd) |
| 822 | `restrict [ *` | 1 | `entity begin restrict[*to 0` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2110/psl1.vhdl) |
| 823 | `return ID .` | 1 | `package function return g.b of` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2110/retid.vhdl) |
| 824 | `return of` | 1 | `package function return of` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue2070/crash31.vhdl) |
| 825 | `rol ID (` | 1 | `lo <= v rol to_integer(q);` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper02/urot02.vhdl) |
| 826 | `ror ID *` | 1 | `dst <= src ror step*STEP_SIZE;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2159/bug.vhdl) |
| 827 | `ror ID ;` | 1 | `ror_val  <= input ror i;` (stimuli/vhdl/subs/ghdl/testsuite/gna/sr2737/testit.vhdl) |
| 828 | `select ID <=` | 1 | `with (cnt) select res_2 <=` (stimuli/vhdl/subs/nvc/test/regress/cover4.vhd) |
| 829 | `signal ID ,` | 1 | `signal b1,b2,b3,b4,b5,b6,b7 : boolean;` (stimuli/vhdl/subs/nvc/test/bounds/bounds2.vhd) |
| 830 | `sla NUM =` | 1 | `assert l sla 1 = x"8" severity failure;` (stimuli/vhdl/subs/ghdl/testsuite/synth/oper02/shift02.vhdl) |
| 831 | `sll ID (` | 1 | `temp_result := bv_s1 sll bv_to_natural(bv_s2(27 to 31));` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/ashenden/compliant/ch_15_alu-b.vhd) |
| 832 | `sll NUM ;` | 1 | `uns_sll <= resize (uns_inp, sgn_srl'length) sll 1; -- work` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1264/issue2.vhdl) |
| 833 | `srl ( ID` | 1 | `dout <= din srl (ch + 1);` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1218/top.vhdl) |
| 834 | `then -- ID` | 1 | `if TRUE then                           -- illegal location for if statement` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc822.vhd) |
| 835 | `to ID '` | 1 | `to sub_int'high => assert (false)` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1499.vhd) |
| 836 | `to ID (` | 1 | `when integer(REGS.C) to integer(REGS.D) => sig <= 1;` (stimuli/vhdl/subs/nvc/test/simp/issue1353.vhd) |
| 837 | `to ID )` | 1 | `subtype char128 is character range (NUL to DEL);  -- Error` (stimuli/vhdl/subs/nvc/test/sem/error2.vhd) |
| 838 | `to NUM |` | 1 | `v17 := (1 to 0 | p => 17);  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2449.vhd) |
| 839 | `type ID ;` | 1 | `type treeNode;` (stimuli/vhdl/subs/nvc/test/perf/binarytrees.vhd) |
| 840 | `type is private` | 1 | `A : type is private ;   -- any type` (stimuli/vhdl/subs/Compliance-Tests/vhdl_2019/tb_016.vhd) |
| 841 | `units -- ID` | 1 | `units  -- Failure_here` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc266.vhd) |
| 842 | `until ( and` | 1 | `wait until (and foo);` (stimuli/vhdl/subs/vunit/examples/vhdl/run/tb_with_watchdog.vhd) |
| 843 | `until ID '` | 1 | `wait until clk'stable;` (stimuli/vhdl/subs/ghdl/testsuite/gna/issue1715/mwe-repro.vhdl) |
| 844 | `until ID .` | 1 | `wait until r.x = '1' and r.x'event;` (stimuli/vhdl/subs/nvc/test/lower/wait2.vhd) |
| 845 | `until ID and` | 1 | `wait until idle and is_empty(message_queue) and rising_edge(aclk);` (stimuli/vhdl/subs/vunit/vunit/vhdl/verification_components/src/axi_lite_master.vhd) |
| 846 | `until ID for` | 1 | `wait until hello_world_received for 1 ns;` (stimuli/vhdl/subs/vunit/vunit/vhdl/com/test/tb_com_deprecated.vhd) |
| 847 | `vunit ID {` | 1 | `vunit issue_1899_vu0 {` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue1899/issue1899.vhdl) |
| 848 | `when ID '` | 1 | `constant c0 : integer := 10 when cond5'path_name = ":cond5:" else 5;` (stimuli/vhdl/subs/nvc/test/regress/cond5.vhd) |
| 849 | `when ID /=` | 1 | `p_actor := actor when actor /= null_actor else new_actor;` (stimuli/vhdl/subs/vunit/vunit/vhdl/verification_components/src/bus_master_pkg-body.vhd) |
| 850 | `when ID ;` | 1 | `next when FALSE;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/compliant/tc1576.vhd) |
| 851 | `when ID <=` | 1 | `m := a when a <= b else b;` (stimuli/vhdl/subs/ghdl/testsuite/synth/issue2045/ghdlcrash.vhdl) |
| 852 | `when ID else` | 1 | `r := t when c else f;` (stimuli/vhdl/subs/nvc/test/parse/issue1188.vhd) |
| 853 | `when ID rem` | 1 | `next when tmp1 rem 2 = 0;` (stimuli/vhdl/subs/nvc/test/regress/issue676.vhd) |
| 854 | `when NUM >` | 1 | `constant c1 : integer := 1 when 1 > 2 else 5;` (stimuli/vhdl/subs/nvc/test/simp/condexpr.vhd) |
| 855 | `{ : ID` | 1 | `variable k{ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2594.vhd) |
| 856 | `| : ID` | 1 | `variable k| : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2610.vhd) |
| 857 | `} : ID` | 1 | `variable k} : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2595.vhd) |
| 858 | `~ : ID` | 1 | `variable k~ : integer := 0;` (stimuli/vhdl/subs/ghdl/testsuite/vests/vhdl-93/billowitch/non_compliant/analyzer_failure/tc2590.vhd) |
| 859 | `Å ID ß` | 1 | `constant Åxyzß : bit := '1';          -- Warning` (stimuli/vhdl/subs/nvc/test/charset/utf8.vhd) |
| 860 | `‘ NUM ’` | 1 | `if (CLEAR = ‘1’) then` (stimuli/vhdl/subs/ghdl/testsuite/gna/bug063/dff.vhdl) |
| 861 | `﻿ -- ID` | 1 | `﻿-- This is utf-8 encoding, with a BOM.` (stimuli/vhdl/subs/ghdl/testsuite/gna/ticket35/utf8.vhdl) |

