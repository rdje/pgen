# The rows the parser rejects in their own file, classed by what it actually saw (SVPP-EXPANSION.6e)

Derived by `census.py` from the tracked re-adjudication table, the two lane dumps and a re-expansion of every row
under the lane's own derived context — **never edited by hand**. `--check` re-hashes the identity below.

## Identity

| input | path | sha256 |
|---|---|---|
| re-adjudication table | `stimuli/sv/characterization/pipeline_readjudication.tsv` | `a8beb4071e8e164e2689e925c6740cbaeec1022a6e2a664cc326f415e0b5946a` |
| sv_2017 dump | `stimuli/sv/characterization/pipeline_results.tsv` | `0f05e65c9e8be4e9953be35ed189aa267abc5832af7727ed1a4add15aa16261e` |
| sv_2023 dump | `stimuli/sv/characterization/pipeline_results_sv2023.tsv` | `a5bfe12d3ef1b56767fe95e52326091251138893736028ccacf6dbc67d4287ba` |
| sv_2023_tolerant dump | `stimuli/sv/characterization/pipeline_results_sv2023_tolerant.tsv` | `b6869df90588e34acfa408a54ec263234cc96aec64add1a81397aa78ad88476c` |
| expansion binary | `rust/target/debug/ast_pipeline` | `9382dff7f4f2a60f0e0f74aa9b61d563fce05064f7becf9fffed6a6e93cd3614` |

## The classes — 306 rows

| class | rows | what it means |
|---|---:|---|
| `raw_backtick` | 0 | the expanded text at the stop begins with a backtick: an undefined macro reached the parser — CONTEXT, not syntax |
| `base_class` | 222 | a class header with a base class the parser has not seen declared — the cross-file declaration barrier |
| `edition_2023` | 4 | the same row passes under the `sv_2023` reading — the 2017 rejection is the older text's |
| `tolerated_spelling` | 5 | the same row passes under `sv_2023_tolerant` only — a spelling of the opt-in profile's closed list |
| `edition_2023_moves` | 0 | the 2023 reading stops further: the construct at this stop is the 2023 text's, a later barrier remains |
| `tolerated_moves` | 0 | the tolerant reading stops further than the 2023 one: a tolerated spelling here, a later barrier remains |
| `error_test` | 0 | an intended-invalid test by name — the rejection is the expected verdict |
| `preprocess_only` | 0 | a verilator test its own driver runs with `-E` — the preprocessor's input, not SystemVerilog to parse |
| `declared_fragment` | 5 | the file declares itself a module body (`verilog_syntax: parse-as-…`) — the suite's own convention |
| `fragment` | 40 | the file is the `include` target of another corpus file — its standalone reading is not the parser's claim |
| `undeclared_type` | 13 | a declaration whose type is an identifier the parser has not seen declared — the cross-file barrier in data position |
| `residue` | 17 | none of the above — adjudicated one by one below |

## `raw_backtick` — 0 rows, by the macro the context did not define

| macro | rows | files |
|---|---:|---|

## `fragment` — 40 rows, by the token at the stop

| token at the stop | rows | one example |
|---|---:|---|
| `SATP_MODE` | 13 | `Cores-VeeR-EL2` `tools/riscv-dv/riscv_core_setting.sv` (6) |
| `xbar_devices` | 11 | `opentitan` `hw/top_darjeeling/dv/autogen/xbar_env_pkg__params.sv` (9) |
| `clk_rst_if_main` | 3 | `opentitan` `hw/top_darjeeling/dv/autogen/tb__xbar_connect.sv` (30) |
| `#` | 2 | `opentitan` `hw/ip/sram_ctrl/dv/env/sram_ctrl_env_cfg.sv` (6) |
| `assign` | 2 | `Cores-VeeR-EL2` `design/include/el2_dec_csr_equ_m.svh` (86) |
| `initial` | 2 | `opentitan` `hw/ip/prim/rtl/prim_util_memload.svh` (54) |
| `"` | 1 | `verilator` `test_regress/t/t_preproc_persist_inc.v` (10) |
| `$display` | 1 | `iverilog` `ivtest/ivltests/line_directive_inc.v` (1) |
| `,` | 1 | `opentitan` `hw/vendor/lowrisc_ibex/vendor/google_riscv-dv/src/isa/custom/riscv_custom_instr_enum.sv` (2) |
| `NUM_ALERTS` | 1 | `opentitan` `hw/top_englishbreakfast/dv/env/autogen/chip_env_pkg__params.sv` (7) |
| `num_dev` | 1 | `opentitan` `hw/ip/tlul/generic_dv/env/xbar_env_cov.sv` (6) |
| `state` | 1 | `black-parrot` `bp_me/src/include/bp_me_cce_pkgdef.svh` (19) |
| `type_id` | 1 | `opentitan` `hw/ip/tlul/generic_dv/env/xbar_scoreboard.sv` (15) |

## `undeclared_type` — 13 rows, by the type the parser never saw

| type | rows | files |
|---|---:|---|
| `DD` | 1 | Surelog:top.v |
| `Driver` | 1 | Surelog:Environemnt.sv |
| `REQ` | 1 | Surelog:top.v |
| `aw_id` | 1 | Surelog:top.v |
| `busproperty` | 1 | verilator:cpu.sv |
| `satp_mode_t` | 1 | opentitan:riscv_core_setting.tpl.sv |
| `sel_mux` | 1 | opentitan:keccak_2share_fpv.sv |
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
| `friscv` | 1 |

## `tolerated_spelling` — 5 rows, by suite

| suite | rows |
|---|---:|
| `opentitan` | 5 |

## `edition_2023_moves` — 0 rows, by suite

| suite | rows |
|---|---:|

## `tolerated_moves` — 0 rows, by suite

| suite | rows |
|---|---:|

## `error_test` — 0 rows, by suite

| suite | rows |
|---|---:|

## `preprocess_only` — 0 rows, by suite

| suite | rows |
|---|---:|

## `declared_fragment` — 5 rows, by suite

| suite | rows |
|---|---:|
| `verible` | 5 |

## `residue` — 17 rows, one by one

| suite | file | origin | token at the stop | source construct | sv_2023 | tolerant |
|---|---|---|---|---|---|---|
| `Surelog` | `tests/ElabSysCall/dut.sv` | `Surelog/tests/ElabSysCall/dut.sv:3:10` | `"` | other | `fail` | `fail` |
| `Surelog` | `tests/TestMacros/macros.v` | `Surelog/tests/TestMacros/macros.v:6:1` | `begin` | other | `fail` | `fail` |
| `Surelog` | `tests/UnitForeach/top2.v` | `Surelog/tests/UnitForeach/top2.v:5:45` | `(` | a `::`-scoped name | `fail` | `fail` |
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
| `verilator` | `test_regress/t/t_preproc_persist2.v` | `verilator/test_regress/t/t_preproc_persist2.v:8:7` | `"` | other | `fail` | `fail` |
