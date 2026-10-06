# The rows the parser rejects in their own file, classed by what it actually saw (SVPP-EXPANSION.6e)

Derived by `census.py` from the tracked re-adjudication table, the two lane dumps and a re-expansion of every row
under the lane's own derived context — **never edited by hand**. `--check` re-hashes the identity below.

## Identity

| input | path | sha256 |
|---|---|---|
| re-adjudication table | `stimuli/sv/characterization/pipeline_readjudication.tsv` | `7f8e422bfcc2fbae51d6ae5d173f730a948b01418265db30fd293f9c917cc409` |
| sv_2017 dump | `stimuli/sv/characterization/pipeline_results.tsv` | `6d0eabec327e9697ab814c1304a9c0d5bc295fc9eac34abf78eb8ea52c470097` |
| sv_2023 dump | `stimuli/sv/characterization/pipeline_results_sv2023.tsv` | `dc0f74836f6ceabfdd20584c12ca69d4f07c1a08660ad850842a26f900428ac5` |
| sv_2023_tolerant dump | `stimuli/sv/characterization/pipeline_results_sv2023_tolerant.tsv` | `7289e42f374d6c6e159e1090e4358a60699cf832c3e02ff8a07ad5f1801ee9da` |
| expansion binary | `rust/target/debug/ast_pipeline` | `03949beb82671cd85ad051f4a6734d97993a10e04e2a93893112d65398ad6caa` |

## The classes — 324 rows

| class | rows | what it means |
|---|---:|---|
| `raw_backtick` | 0 | the expanded text at the stop begins with a backtick: an undefined macro reached the parser — CONTEXT, not syntax |
| `base_class` | 222 | a class header with a base class the parser has not seen declared — the cross-file declaration barrier |
| `edition_2023` | 4 | the same row passes under the `sv_2023` reading — the 2017 rejection is the older text's |
| `tolerated_spelling` | 5 | the same row passes under `sv_2023_tolerant` only — a spelling of the opt-in profile's closed list |
| `edition_2023_moves` | 1 | the 2023 reading stops further: the construct at this stop is the 2023 text's, a later barrier remains |
| `tolerated_moves` | 0 | the tolerant reading stops further than the 2023 one: a tolerated spelling here, a later barrier remains |
| `error_test` | 6 | an intended-invalid test by name — the rejection is the expected verdict |
| `preprocess_only` | 5 | a verilator test its own driver runs with `-E` — the preprocessor's input, not SystemVerilog to parse |
| `declared_fragment` | 5 | the file declares itself a module body (`verilog_syntax: parse-as-…`) — the suite's own convention |
| `fragment` | 39 | the file is the `include` target of another corpus file — its standalone reading is not the parser's claim |
| `undeclared_type` | 11 | a declaration whose type is an identifier the parser has not seen declared — the cross-file barrier in data position |
| `residue` | 26 | none of the above — adjudicated one by one below |

## `raw_backtick` — 0 rows, by the macro the context did not define

| macro | rows | files |
|---|---:|---|

## `fragment` — 39 rows, by the token at the stop

| token at the stop | rows | one example |
|---|---:|---|
| `SATP_MODE` | 13 | `Cores-VeeR-EL2` `tools/riscv-dv/riscv_core_setting.sv` (6) |
| `xbar_devices` | 11 | `opentitan` `hw/top_darjeeling/dv/autogen/xbar_env_pkg__params.sv` (9) |
| `clk_rst_if_main` | 3 | `opentitan` `hw/top_darjeeling/dv/autogen/tb__xbar_connect.sv` (30) |
| `#` | 2 | `opentitan` `hw/ip/sram_ctrl/dv/env/sram_ctrl_env_cfg.sv` (6) |
| `assign` | 2 | `Cores-VeeR-EL2` `design/include/el2_dec_csr_equ_m.svh` (86) |
| `initial` | 2 | `opentitan` `hw/ip/prim/rtl/prim_util_memload.svh` (54) |
| `"` | 1 | `verilator` `test_regress/t/t_preproc_persist_inc.v` (10) |
| `,` | 1 | `opentitan` `hw/vendor/lowrisc_ibex/vendor/google_riscv-dv/src/isa/custom/riscv_custom_instr_enum.sv` (2) |
| `NUM_ALERTS` | 1 | `opentitan` `hw/top_englishbreakfast/dv/env/autogen/chip_env_pkg__params.sv` (7) |
| `num_dev` | 1 | `opentitan` `hw/ip/tlul/generic_dv/env/xbar_env_cov.sv` (6) |
| `state` | 1 | `black-parrot` `bp_me/src/include/bp_me_cce_pkgdef.svh` (19) |
| `type_id` | 1 | `opentitan` `hw/ip/tlul/generic_dv/env/xbar_scoreboard.sv` (15) |

## `undeclared_type` — 11 rows, by the type the parser never saw

| type | rows | files |
|---|---:|---|
| `DD` | 1 | Surelog:top.v |
| `Driver` | 1 | Surelog:Environemnt.sv |
| `REQ` | 1 | Surelog:top.v |
| `aw_id` | 1 | Surelog:top.v |
| `satp_mode_t` | 1 | opentitan:riscv_core_setting.tpl.sv |
| `t_MainMode` | 1 | opentitan:spec_api.sv |
| `t_padtype` | 1 | verilator:pads.sv |
| `t_pinid` | 1 | verilator:pinout_h.sv |
| `type_scr1_mem_resp_e` | 1 | Surelog:dut.sv |
| `uvm_component` | 1 | Surelog:top.v |
| `uvm_packer` | 1 | Surelog:top_1.sv |

## `base_class` — 222 rows, by suite

| suite | rows |
|---|---:|
| `opentitan` | 218 |
| `Surelog` | 4 |

## `edition_2023` — 4 rows, by suite

| suite | rows |
|---|---:|
| `opentitan` | 3 |
| `verilator` | 1 |

## `tolerated_spelling` — 5 rows, by suite

| suite | rows |
|---|---:|
| `opentitan` | 5 |

## `edition_2023_moves` — 1 rows, by suite

| suite | rows |
|---|---:|
| `verilator` | 1 |

## `tolerated_moves` — 0 rows, by suite

| suite | rows |
|---|---:|

## `error_test` — 6 rows, by suite

| suite | rows |
|---|---:|
| `verilator` | 6 |

## `preprocess_only` — 5 rows, by suite

| suite | rows |
|---|---:|
| `verilator` | 5 |

## `declared_fragment` — 5 rows, by suite

| suite | rows |
|---|---:|
| `verible` | 5 |

## `residue` — 26 rows, one by one

| suite | file | origin | token at the stop | source construct | sv_2023 | tolerant |
|---|---|---|---|---|---|---|
| `Surelog` | `tests/ElabSysCall/dut.sv` | `Surelog/tests/ElabSysCall/dut.sv:3:10` | `"` | other | `fail` | `fail` |
| `Surelog` | `tests/GenerateAssigns/top.v` | `Surelog/tests/GenerateAssigns/top.v:25:21` | `~` | other | `fail` | `fail` |
| `Surelog` | `tests/SimpleInterface/simple_if.sv` | `Surelog/tests/SimpleInterface/simple_if.sv:15:46` | `modport` | other | `fail` | `fail` |
| `Surelog` | `tests/TestMacros/macros.v` | `Surelog/tests/TestMacros/macros.v:6:1` | `begin` | other | `fail` | `fail` |
| `Surelog` | `tests/UnitForeach/top2.v` | `Surelog/tests/UnitForeach/top2.v:5:45` | `(` | a `::`-scoped name | `fail` | `fail` |
| `Surelog` | `tests/UnitTest/top.v` | `Surelog/tests/UnitTest/top.v:34:22` | `~` | other | `fail` | `fail` |
| `friscv` | `rtl/friscv_alu.sv` | `friscv/rtl/friscv_alu.sv:195:43` | `b0` | other | `fail` | `fail` |
| `opentitan` | `hw/ip/kmac/fpv/tb/keccak_2share_fpv.sv` | `opentitan/hw/ip/kmac/fpv/tb/keccak_2share_fpv.sv:46:18` | `;` | other | `fail` | `fail` |
| `opentitan` | `hw/ip/kmac/pre_dv/kmac_reduced_tb/rtl/kmac_reduced_tb.sv` | `opentitan/hw/ip/kmac/pre_dv/kmac_reduced_tb/rtl/kmac_reduced_tb.sv:52:33` | `)` | other | `fail` | `fail` |
| `opentitan` | `hw/ip/otbn/dv/verilator/otbn_top_sim.sv` | `opentitan/hw/ip/otbn/dv/verilator/otbn_top_sim.sv:168:48` | `,` | a parameter port list `#(` | `fail` | `fail` |
| `opentitan` | `hw/ip/otbn/pre_sca/rtl/otbn_mask_accelerator_sca_wrapper.sv` | `opentitan/hw/ip/otbn/pre_sca/rtl/otbn_mask_accelerator_sca_wrapper.sv:50:24` | `else` | other | `fail` | `fail` |
| `opentitan` | `hw/ip/spi_device/pre_dv/program/prog_passthrough_host.sv` | `opentitan/hw/ip/spi_device/pre_dv/program/prog_passthrough_host.sv:117:9` | `task` | a function / task header | `fail` | `fail` |
| `opentitan` | `hw/ip/spi_device/pre_dv/program/prog_passthrough_sw.sv` | `opentitan/hw/ip/spi_device/pre_dv/program/prog_passthrough_sw.sv:229:9` | `task` | a function / task header | `fail` | `fail` |
| `opentitan` | `hw/ip/spi_device/pre_dv/program/spiflash.sv` | `opentitan/hw/ip/spi_device/pre_dv/program/spiflash.sv:192:9` | `task` | a function / task header | `fail` | `fail` |
| `opentitan` | `hw/ip/spi_device/pre_dv/tb/spid_jedec_tb.sv` | `opentitan/hw/ip/spi_device/pre_dv/tb/spid_jedec_tb.sv:97:9` | `task` | a function / task header | `fail` | `fail` |
| `opentitan` | `hw/ip/spi_device/pre_dv/tb/spid_readcmd_tb.sv` | `opentitan/hw/ip/spi_device/pre_dv/tb/spid_readcmd_tb.sv:180:9` | `task` | a function / task header | `fail` | `fail` |
| `opentitan` | `hw/ip/spi_device/pre_dv/tb/spid_upload_tb.sv` | `opentitan/hw/ip/spi_device/pre_dv/tb/spid_upload_tb.sv:186:9` | `task` | a function / task header | `fail` | `fail` |
| `opentitan` | `hw/vendor/lowrisc_ibex/dv/verilator/simple_system_cosim/ibex_simple_system_cosim_checker_bind.sv` | `opentitan/hw/vendor/lowrisc_ibex/dv/verilator/simple_system_cosim/ibex_simple_system_cosim_checker_bind.sv:7:18` | `,` | other | `fail` | `fail` |
| `opentitan` | `hw/vendor/lowrisc_ibex/rtl/ibex_top_tracing.sv` | `opentitan/hw/vendor/lowrisc_ibex/rtl/ibex_top_tracing.sv:123:12` | `"` | other | `fail` | `fail` |
| `opentitan` | `hw/vendor/lowrisc_ibex/syn/rtl/latch_map.v` | `opentitan/hw/vendor/lowrisc_ibex/syn/rtl/latch_map.v:6:7` | `$_DLATCH_P_` | other | `fail` | `fail` |
| `slang` | `tests/unittests/data/file_uses_defn.svh` | `slang/tests/unittests/data/file_uses_defn.svh:2:1` | `"` | other | `fail` | `fail` |
| `verilator` | `test_regress/t/t_langext_order_sub.v` | `verilator/test_regress/t/t_langext_order_sub.v:11:33` | `do` | other | `fail` | `fail` |
| `verilator` | `test_regress/t/t_preproc_persist2.v` | `verilator/test_regress/t/t_preproc_persist2.v:8:7` | `"` | other | `fail` | `fail` |
| `verilator` | `test_regress/t/t_sv_cpu_code/cpu.sv` | `verilator/test_regress/t/t_sv_cpu_code/cpu.sv:225:33` | `;` | other | `fail` | `fail` |
| `verilator` | `test_regress/t/uvm/uvm_pkg_all_v2017_1_0_dpi.svh` | `uvm_pkg_all_v2017_1_0_dpi.svh:18004:19` | `.` | other | `fail` | `fail` |
| `verilator` | `test_regress/t/uvm/uvm_pkg_all_v2017_1_0_nodpi.svh` | `uvm_pkg_all_v2017_1_0_nodpi.svh:18053:19` | `.` | other | `fail` | `fail` |
