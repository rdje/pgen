# The sv_2023 × sv_2023_tolerant census of the preprocess→parse lane (SV-CORPUS-GRAD.43)

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.py --other sv_2023_tolerant`; `--check` re-derives and diffs.

Both readings run the SAME rows — every `.sv` / `.svh` / `.v` file under `stimuli/sv/subs` plus the folded `uvm-core` tree — through `parseability_probe --parse systemverilog <file> --profile <edition> --preprocess <the file's derived context> --lib-in <the reading's own library>`, with the same binary and the same context; only `--profile` differs (the library pass included). The tolerant profile is OPT-IN (`docs/decisions/project_sv_opt_in_tolerance_profile.md`): the primary reading stays `sv_2017`, the verdict-coverage tuple is defined over it, and this census moves no tuple.

## Instrument identity

> Re-hash the rows; if any differs, this census no longer describes your tree — re-run it, never quote it.

| input | repo-root-relative path | sha256 |
|---|---|---|
| parse binary | `rust/target/debug/parseability_probe` | `bdbff2e534f87c5a52cbb582d21929db020b818ac0c8c587ee71c86280a7f380` |
| grammar raw ast | `grammars/systemverilog.ebnf` | `e880d4d69928f48d2241c644b461285015741317a849009b39c92cd4da9779d6` |
| sv_2023 pipeline results | `stimuli/sv/characterization/pipeline_results_sv2023.tsv` | `b987f55646f24f63f5de40f4faecec6860dd9a0cafa9becc0214a4f34c9795bf` |
| sv_2023 library pass dump | `stimuli/sv/characterization/pipeline_library_sv2023.tsv` | `f3ba226fbd83dbd8456198c299ea8b8fa943e8b5ec4be2f08e1c0205e52624c9` |
| sv_2023_tolerant pipeline results | `stimuli/sv/characterization/pipeline_results_sv2023_tolerant.tsv` | `044ea4ba169dddafacf231ae91326204a8695f904a3c1d9d4d18ff343c01645d` |
| sv_2023_tolerant library pass dump | `stimuli/sv/characterization/pipeline_library_sv2023_tolerant.tsv` | `26cc1349d594cb7b3ae743e773673236e046225d1b17f691b150c3fdcd5220b0` |
| unit membership (both readings, byte-identical) | `stimuli/sv/characterization/pipeline_library_members_sv2023_tolerant.tsv` | `acac7a118b3cfe26a207b0d1bf64e5758093b730c825a75122d3c7fb1b6c3c6c` |
| sv_2017 re-adjudication table | `stimuli/sv/characterization/pipeline_readjudication.tsv` | `0c7b858d99b53133b756b1c508837e492cbf0db8d5b2fa4aafb572dec71edc3f` |
| sv_2023 answer key | `stimuli/sv/characterization/adjudication_manifest_sv2023.tsv` | `91d04e28dae5cd2b2772a924b5cf06eb1cd259d9f194b088ee35777e1876b7aa` |

The tolerant run's own parameters (its provenance sidecar): `timeout_s=600` `jobs=6` `files=16336`.

## The per-file matrix

| reading | `pass` | `fail` | `pp_fail` | `timeout` | `crash` |
|---|---:|---:|---:|---:|---:|
| `sv_2023` | 12751 | 3095 | 490 | 0 | 0 |
| `sv_2023_tolerant` | 12769 | 3077 | 490 | 0 | 0 |

Rows joined: **16336**; only in the sv_2023 reading: **0**; only in the sv_2023_tolerant reading: **0** (a non-zero here is a population breach, not a verdict).

| cell | rows | meaning |
|---|---:|---|
| `both_pass` | 12751 | parses under both profiles |
| `both_nonpass` | 3567 | parses under neither |
| `tolerant_only_pass` | 18 | text the standard does not derive and the opt-in tolerance profile accepts — the closed list of tool tolerances at work |
| `2023_only_pass` | 0 | the SUPERSET broken: `sv_2023` accepts it and the tolerant profile does not — must stay 0 |

### Status pairs (sv_2023, sv_2023_tolerant)

| sv_2023 | sv_2023_tolerant | rows |
|---|---|---:|
| `pass` | `pass` | 12751 |
| `fail` | `fail` | 3077 |
| `pp_fail` | `pp_fail` | 490 |
| `fail` | `pass` | 18 |

### Every row whose verdict differs

| cell | suite | rows |
|---|---|---:|
| `tolerant_only_pass` | `Surelog` | 2 |
| `tolerant_only_pass` | `opentitan` | 12 |
| `tolerant_only_pass` | `verilator` | 4 |

The stop is the REJECTING reading's own origin and the source line there; the 2017 re-adjudication verdict and the 2023 key's ruling ride beside it.

| cell | suite | relpath | stop (origin) | text at the stop | 2017 re-adjudication | 2023 key |
|---|---|---|---|---|---|---|
| `tolerant_only_pass` | `Surelog` | `tests/ForeachSquare/dut.sv` | `dut.sv:7:32` | `foreach(array[i][j])` | `must_reject:match_under_expansion` | `must_reject / match` |
| `tolerant_only_pass` | `Surelog` | `tests/SynthForeach/dut.sv` | `dut.sv:4:31` | `foreach(data[i][j])` | `must_reject:match_under_expansion` | `must_reject / match` |
| `tolerant_only_pass` | `opentitan` | `hw/dv/sv/sec_cm/sec_cm_pkg.sv` | `sec_cm_base_if_proxy.sv:21:46` | `pure virtual task automatic inject_fault();` | `chained:hollow` | `chained_must_accept / deferred:chained_only` |
| `tolerant_only_pass` | `opentitan` | `hw/dv/sv/tl_agent/tl_agent_pkg.sv` | `tl_seq_item.sv:190:18` | `extern function automatic logic [DataWidth-1:0] get_written_data();` | `chained:hollow` | `chained_must_accept / deferred:chained_only` |
| `tolerant_only_pass` | `opentitan` | `hw/ip/adc_ctrl/dv/env/adc_ctrl_env_pkg.sv` | `adc_ctrl_env_var_filter_cfg.sv:64:1` | `solve max_val, min_val, apply_max_v before filter_cfg[channel][filter].min_v;` | `chained:hollow` | `chained_must_accept / deferred:chained_only` |
| `tolerant_only_pass` | `opentitan` | `hw/ip/sysrst_ctrl/dv/env/sysrst_ctrl_env_pkg.sv` | `sysrst_ctrl_in_out_inverted_vseq.sv:49:39` | `key0_inXval: cross cp_key0_in, cfg.vif.key0_in;` | `chained:hollow` | `chained_must_accept / deferred:chained_only` |
| `tolerant_only_pass` | `opentitan` | `hw/ip_templates/flash_ctrl/dv/env/flash_ctrl_env_pkg.sv` | `flash_otf_read_entry.sv:129:30` | `foreach (prv_read[i][j]) begin` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_include` |
| `tolerant_only_pass` | `opentitan` | `hw/ip_templates/flash_ctrl/dv/env/flash_ctrl_scoreboard.sv` | `flash_ctrl_scoreboard.sv:106:49` | `foreach (cfg.flash_ctrl_vif.hazard[i][j]) begin` | `chained:unreached_in_unit` | `chained_must_accept / deferred:chained_only` |
| `tolerant_only_pass` | `opentitan` | `hw/top_earlgrey/ip_autogen/flash_ctrl/dv/env/flash_ctrl_env_pkg.sv` | `flash_otf_read_entry.sv:129:30` | `foreach (prv_read[i][j]) begin` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_include` |
| `tolerant_only_pass` | `opentitan` | `hw/top_earlgrey/ip_autogen/flash_ctrl/dv/env/flash_ctrl_scoreboard.sv` | `flash_ctrl_scoreboard.sv:106:49` | `foreach (cfg.flash_ctrl_vif.hazard[i][j]) begin` | `chained:unreached_in_unit` | `chained_must_accept / deferred:chained_only` |
| `tolerant_only_pass` | `opentitan` | `hw/top_englishbreakfast/ip_autogen/flash_ctrl/dv/env/flash_ctrl_env_pkg.sv` | `flash_otf_read_entry.sv:129:30` | `foreach (prv_read[i][j]) begin` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_include` |
| `tolerant_only_pass` | `opentitan` | `hw/top_englishbreakfast/ip_autogen/flash_ctrl/dv/env/flash_ctrl_scoreboard.sv` | `flash_ctrl_scoreboard.sv:106:49` | `foreach (cfg.flash_ctrl_vif.hazard[i][j]) begin` | `chained:unreached_in_unit` | `chained_must_accept / deferred:chained_only` |
| `tolerant_only_pass` | `opentitan` | `hw/vendor/lowrisc_ibex/dv/uvm/core_ibex/fcov/core_ibex_fcov_if.sv` | `core_ibex_fcov_if.sv:668:52` | `misaligned_insn_bus_err_cross: cross id_stage_i.instr_fetch_err_i,` | `chained:rejects_own_file` | `chained_must_accept / deferred:chained_only` |
| `tolerant_only_pass` | `opentitan` | `hw/vendor/lowrisc_ibex/dv/uvm/core_ibex/fcov/core_ibex_pmp_fcov_if.sv` | `core_ibex_pmp_fcov_if.sv:478:79` | `cross pmp_region_priv_bits, pmp_region_priv_bits_wr, csr_pmp_mseccfg.rlb iff` | `chained:rejects_own_file` | `chained_must_accept / deferred:chained_only` |
| `tolerant_only_pass` | `verilator` | `test_regress/t/t_foreach_array.v` | `t_foreach_array.v:64:44` | `foreach (queue[i]) foreach (queue[i][j]) exp_count_que++;` | `must_reject:match_under_expansion` | `must_reject / match` |
| `tolerant_only_pass` | `verilator` | `test_regress/t/t_foreach_nested.v` | `t_foreach_nested.v:14:31` | `foreach(this.foo.x[i][j])` | `must_reject:match_under_expansion` | `must_reject / match` |
| `tolerant_only_pass` | `verilator` | `test_regress/t/t_randomize_array.v` | `t_randomize_array.v:51:36` | `foreach (unpacked_array[i][j]) begin` | `must_reject:match_under_expansion` | `out_of_scope_with_cause / deferred:svpp_owned` |
| `tolerant_only_pass` | `verilator` | `test_regress/t/t_randomize_solve_before_foreach.v` | `t_randomize_solve_before_foreach.v:40:26` | `foreach (data[i]) {` | `must_reject:match_under_expansion` | `out_of_scope_with_cause / deferred:svpp_owned` |

## The library pass

Package files joined: **407**; only in one reading: **0**. Packages published: **278** under `sv_2023`, **283** under `sv_2023_tolerant`.

| cell | package files |
|---|---:|
| `both_pass` | 382 |
| `both_nonpass` | 18 |
| `tolerant_only_pass` | 7 |
| `2023_only_pass` | 0 |

Published under `sv_2023_tolerant` only: `adc_ctrl_env_pkg` `flash_ctrl_env_pkg` `sec_cm_pkg` `sysrst_ctrl_env_pkg` `tl_agent_pkg`.

Published under `sv_2023` only: (none).

| cell | suite | package file | packages | stop (origin) | text at the stop |
|---|---|---|---|---|---|
| `tolerant_only_pass` | `opentitan` | `hw/dv/sv/sec_cm/sec_cm_pkg.sv` | `sec_cm_pkg` | `sec_cm_base_if_proxy.sv:21:46` | `pure virtual task automatic inject_fault();` |
| `tolerant_only_pass` | `opentitan` | `hw/dv/sv/tl_agent/tl_agent_pkg.sv` | `tl_agent_pkg` | `tl_seq_item.sv:190:18` | `extern function automatic logic [DataWidth-1:0] get_written_data();` |
| `tolerant_only_pass` | `opentitan` | `hw/ip/adc_ctrl/dv/env/adc_ctrl_env_pkg.sv` | `adc_ctrl_env_pkg` | `adc_ctrl_env_var_filter_cfg.sv:64:1` | `solve max_val, min_val, apply_max_v before filter_cfg[channel][filter].min_v;` |
| `tolerant_only_pass` | `opentitan` | `hw/ip/sysrst_ctrl/dv/env/sysrst_ctrl_env_pkg.sv` | `sysrst_ctrl_env_pkg` | `sysrst_ctrl_in_out_inverted_vseq.sv:49:39` | `key0_inXval: cross cp_key0_in, cfg.vif.key0_in;` |
| `tolerant_only_pass` | `opentitan` | `hw/ip_templates/flash_ctrl/dv/env/flash_ctrl_env_pkg.sv` | `flash_ctrl_env_pkg` | `flash_otf_read_entry.sv:129:30` | `foreach (prv_read[i][j]) begin` |
| `tolerant_only_pass` | `opentitan` | `hw/top_earlgrey/ip_autogen/flash_ctrl/dv/env/flash_ctrl_env_pkg.sv` | `flash_ctrl_env_pkg` | `flash_otf_read_entry.sv:129:30` | `foreach (prv_read[i][j]) begin` |
| `tolerant_only_pass` | `opentitan` | `hw/top_englishbreakfast/ip_autogen/flash_ctrl/dv/env/flash_ctrl_env_pkg.sv` | `flash_ctrl_env_pkg` | `flash_otf_read_entry.sv:129:30` | `foreach (prv_read[i][j]) begin` |

## The unit reading

A member file's verdict under the pipeline is its package unit's (`SVPP-EXPANSION.6d.2`). Of the rows the tracked 2017 re-adjudication reads:

| 2017 reading | rows | under `sv_2023_tolerant` |
|---|---:|---|
| `unreached_in_unit` | 337 | **337** are included by a unit that PASSES — reached |
| `accepted_in_unit` | 1498 | **0** have no passing unit — a regression candidate |

The members the tolerant reading reaches, by the passing unit that includes them (the per-member rows are in `census.tsv`, kind `member`):

| suite | unit (package file) | members reached |
|---|---|---:|
| `opentitan` | `hw/ip_templates/flash_ctrl/dv/env/flash_ctrl_env_pkg.sv` | 76 |
| `opentitan` | `hw/top_earlgrey/ip_autogen/flash_ctrl/dv/env/flash_ctrl_env_pkg.sv` | 76 |
| `opentitan` | `hw/top_englishbreakfast/ip_autogen/flash_ctrl/dv/env/flash_ctrl_env_pkg.sv` | 76 |
| `opentitan` | `hw/ip/sysrst_ctrl/dv/env/sysrst_ctrl_env_pkg.sv` | 22 |
| `opentitan` | `hw/ip/adc_ctrl/dv/env/adc_ctrl_env_pkg.sv` | 21 |
| `opentitan` | `hw/top_darjeeling/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | 17 |
| `opentitan` | `hw/top_earlgrey/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | 17 |
| `opentitan` | `hw/top_englishbreakfast/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | 17 |
| `opentitan` | `hw/dv/sv/tl_agent/tl_agent_pkg.sv` | 15 |
