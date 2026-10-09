# The sv_2017 × sv_2023 census of the preprocess→parse lane (SV-CORPUS-GRAD.38)

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.py`; `--check` re-derives and diffs.

Both readings run the SAME rows — every `.sv` / `.svh` / `.v` file under `stimuli/sv/subs` plus the folded `uvm-core` tree — through `parseability_probe --parse systemverilog <file> --profile <edition> --preprocess <the file's derived context> --lib-in <the reading's own library>`, with the same binary and the same context; only `--profile` differs (the library pass included). The PRIMARY reading is `sv_2017`: the verdict-coverage tuple under the pipeline is defined over it, and this census moves no tuple.

## Instrument identity

> Re-hash the rows; if any differs, this census no longer describes your tree — re-run it, never quote it.

| input | repo-root-relative path | sha256 |
|---|---|---|
| parse binary | `rust/target/debug/parseability_probe` | `5a60442e02c671cc2e44816c7cf808f5c7ffe016657d1540b60c927af911cd5d` |
| grammar raw ast | `grammars/systemverilog.ebnf` | `1aa05dcbdbb9a0077e7e98365a8d25a0ee47c315961f41f4c326b779205237f4` |
| sv_2017 pipeline results | `stimuli/sv/characterization/pipeline_results.tsv` | `608025918dc90715f1647be588dafa5d7a5f41a7f8a808f192f034677734d0a9` |
| sv_2017 library pass dump | `stimuli/sv/characterization/pipeline_library.tsv` | `517d140b1ef416a4ce8a4ae28941a9280c32cfdfd562b4d673a001b95a01b4b5` |
| sv_2023 pipeline results | `stimuli/sv/characterization/pipeline_results_sv2023.tsv` | `19f73459025860d7aceaca43e98a9a1bd9a76cd8a52dfcb8275d7338d13ccbce` |
| sv_2023 library pass dump | `stimuli/sv/characterization/pipeline_library_sv2023.tsv` | `9d6898c9f5a19013b5847b87bfcfcf64b2a9e758582d07bf803171d1a78e7bad` |
| unit membership (both readings, byte-identical) | `stimuli/sv/characterization/pipeline_library_members_sv2023.tsv` | `acac7a118b3cfe26a207b0d1bf64e5758093b730c825a75122d3c7fb1b6c3c6c` |
| sv_2017 re-adjudication table | `stimuli/sv/characterization/pipeline_readjudication.tsv` | `f7dcd0380671fed3d148e6812d15abf310a8984d1015dea4dc54e22e51bc10a3` |
| sv_2023 answer key | `stimuli/sv/characterization/adjudication_manifest_sv2023.tsv` | `cd2483d608c185a3b517010665dc10f9812605f5a7e608b395bcc01b3c44a861` |

The 2023 run's own parameters (its provenance sidecar): `timeout_s=600` `jobs=6` `files=16336`.

## The per-file matrix

| reading | `pass` | `fail` | `pp_fail` | `timeout` | `crash` |
|---|---:|---:|---:|---:|---:|
| `sv_2017` | 12704 | 2997 | 635 | 0 | 0 |
| `sv_2023` | 12745 | 2956 | 635 | 0 | 0 |

Rows joined: **16336**; only in the sv_2017 reading: **0**; only in the sv_2023 reading: **0** (a non-zero here is a population breach, not a verdict).

| cell | rows | meaning |
|---|---:|---|
| `both_pass` | 12704 | parses under both editions |
| `both_nonpass` | 3591 | parses under neither |
| `2023_only_pass` | 41 | text the 2017 profile rejects and the 2023 profile accepts — adjudicated where the standard admits it |
| `2017_only_pass` | 0 | a 2023-profile REGRESSION candidate — a parser question until adjudicated |

### Status pairs (sv_2017, sv_2023)

| sv_2017 | sv_2023 | rows |
|---|---|---:|
| `pass` | `pass` | 12704 |
| `fail` | `fail` | 2956 |
| `pp_fail` | `pp_fail` | 635 |
| `fail` | `pass` | 41 |

### Every row whose verdict differs

| cell | suite | rows |
|---|---|---:|
| `2023_only_pass` | `friscv` | 1 |
| `2023_only_pass` | `iverilog` | 20 |
| `2023_only_pass` | `opentitan` | 3 |
| `2023_only_pass` | `slang` | 1 |
| `2023_only_pass` | `verilator` | 16 |

The stop is the REJECTING reading's own origin and the source line there; the 2017 re-adjudication verdict and the 2023 key's ruling ride beside it.

| cell | suite | relpath | stop (origin) | text at the stop | 2017 re-adjudication | 2023 key |
|---|---|---|---|---|---|---|
| `2023_only_pass` | `friscv` | `rtl/friscv_alu.sv` | `friscv_alu.sv:195:43` | `assign _srli = (shamt == 5'd01) ? {01'b0, alu_rs1_val[XLEN-1:01]} :` | `chained:rejects_own_file` | `chained_only / deferred:chained_only` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/begin_keywords_1800_2023.v` | `begin_keywords_1800_2023.v:3:16` | `` `begin_keywords "1800-2023" `` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/event_array.v` | `event_array.v:23:19` | `#1 ->my_event[0];` | `-` | `out_of_scope_with_cause:v2005_profile_lane / deferred:v2005_profile_lane` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_soft_packed_union.v` | `sv_soft_packed_union.v:7:16` | `typedef union soft packed {` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_soft_packed_union_fail1.v` | `sv_soft_packed_union_fail1.v:5:16` | `typedef union soft {` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_class1.v` | `sv_type_param_restrict_class1.v:8:17` | `parameter type class T = C0` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_class2.v` | `sv_type_param_restrict_class2.v:12:17` | `parameter type class T = C0` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_class_fail1.v` | `sv_type_param_restrict_class_fail1.v:4:17` | `parameter type class T = int` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_class_fail2.v` | `sv_type_param_restrict_class_fail2.v:7:17` | `parameter type class T = class_t` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_enum1.v` | `sv_type_param_restrict_enum1.v:8:17` | `parameter type enum T = T0` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_enum2.v` | `sv_type_param_restrict_enum2.v:12:17` | `parameter type enum T = T0` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_enum_fail1.v` | `sv_type_param_restrict_enum_fail1.v:4:17` | `parameter type enum T = int` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_enum_fail2.v` | `sv_type_param_restrict_enum_fail2.v:8:17` | `parameter type enum T = enum_t` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_struct1.v` | `sv_type_param_restrict_struct1.v:8:17` | `parameter type struct T = T0` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_struct2.v` | `sv_type_param_restrict_struct2.v:12:17` | `parameter type struct T = T0` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_struct_fail1.v` | `sv_type_param_restrict_struct_fail1.v:9:17` | `parameter type struct T = union_t` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_struct_fail2.v` | `sv_type_param_restrict_struct_fail2.v:13:17` | `parameter type struct T = struct_t` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_union1.v` | `sv_type_param_restrict_union1.v:9:17` | `parameter type union T = T0` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_union2.v` | `sv_type_param_restrict_union2.v:14:17` | `parameter type union T = T0` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_union_fail1.v` | `sv_type_param_restrict_union_fail1.v:8:17` | `parameter type union T = struct_t` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_union_fail2.v` | `sv_type_param_restrict_union_fail2.v:13:17` | `parameter type union T = union_t` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `opentitan` | `hw/top_darjeeling/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | `rstmgr_env_pkg.sv:37:34` | `parameter string LIST_OF_LEAFS[] = {` | `chained:rejects_own_file` | `chained_only / deferred:chained_only` |
| `2023_only_pass` | `opentitan` | `hw/top_earlgrey/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | `rstmgr_env_pkg.sv:37:34` | `parameter string LIST_OF_LEAFS[] = {` | `chained:rejects_own_file` | `chained_only / deferred:chained_only` |
| `2023_only_pass` | `opentitan` | `hw/top_englishbreakfast/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | `rstmgr_env_pkg.sv:37:34` | `parameter string LIST_OF_LEAFS[] = {` | `chained:rejects_own_file` | `chained_only / deferred:chained_only` |
| `2023_only_pass` | `slang` | `tests/regression/all.sv` | `all.sv:255:48` | `default disable iff 1 dist { [1:2] :/ 3, 2, default :/ 4 };` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_array_event.v` | `t_array_event.v:15:26` | `->write_events[index];` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_extends_default.v` | `t_class_extends_default.v:14:26` | `class Cls1 extends Base1(default);` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_new_default.v` | `t_class_new_default.v:10:16` | `function new(default);` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_override.v` | `t_class_override.v:26:11` | `function :initial int get_i; return 1; endfunction` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_override_bad.v` | `t_class_override_bad.v:23:11` | `function :extends int get_e; return 1; endfunction // Bad` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_display_p_elab.v` | `t_display_p_elab.v:37:20` | `parameter int Q[$] = '{1, 2};` | `refuted:vendor_identity_branch` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_display_qqq.v` | `t_display_qqq.v:10:16` | `$display("""First "quoted"\nsecond\` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_event_array_fire.v` | `t_event_array_fire.v:39:36` | `->e_all_xfers_completed[id];` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_export_packed_struct.v` | `t_export_packed_struct.v:9:14` | `typedef union soft packed {` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_export_packed_struct2.v` | `t_export_packed_struct2.v:9:16` | `typedef union soft packed {` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_fork_join_none_wait_ev.v` | `t_fork_join_none_wait_ev.v:32:20` | `->ev[i];` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_inside_tolerance_unsup.v` | `t_inside_tolerance_unsup.v:18:30` | `` `checkh(r inside {[1492 +/- 2]}, 1'b1); `` | `must_reject:match_under_expansion` | `must_accept / divergence:explained_svpp_macro_use` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_preproc_preproczero_bad.v` | `t_preproc_preproczero_bad.v:8:16` | `` `begin_keywords "1800-2023" `` | `svpp_owned:refuted:key_question` | `out_of_scope_with_cause / deferred:svpp_owned` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_string_sel.v` | `t_string_sel.v:35:18` | `if (get_str()[1] != "a") $stop;` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_union_soft.v` | `t_union_soft.v:9:8` | `union soft {` | `must_reject:match_under_expansion` | `must_accept / match` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_var_ref_static.v` | `t_var_ref_static.v:12:30` | `function void crs(const ref static i);` | `must_reject:match_under_expansion` | `must_accept / match` |

## The library pass

Package files joined: **407**; only in one reading: **0**. Packages published: **277** under `sv_2017`, **278** under `sv_2023`.

| cell | package files |
|---|---:|
| `both_pass` | 379 |
| `both_nonpass` | 25 |
| `2023_only_pass` | 3 |
| `2017_only_pass` | 0 |

Published under `sv_2023` only: `rstmgr_env_pkg`.

Published under `sv_2017` only: (none).

| cell | suite | package file | packages | stop (origin) | text at the stop |
|---|---|---|---|---|---|
| `2023_only_pass` | `opentitan` | `hw/top_darjeeling/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | `rstmgr_env_pkg` | `rstmgr_env_pkg.sv:37:34` | `parameter string LIST_OF_LEAFS[] = {` |
| `2023_only_pass` | `opentitan` | `hw/top_earlgrey/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | `rstmgr_env_pkg` | `rstmgr_env_pkg.sv:37:34` | `parameter string LIST_OF_LEAFS[] = {` |
| `2023_only_pass` | `opentitan` | `hw/top_englishbreakfast/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | `rstmgr_env_pkg` | `rstmgr_env_pkg.sv:37:34` | `parameter string LIST_OF_LEAFS[] = {` |

## The unit reading

A member file's verdict under the pipeline is its package unit's (`SVPP-EXPANSION.6d.2`). Of the rows the tracked 2017 re-adjudication reads:

| 2017 reading | rows | under `sv_2023` |
|---|---:|---|
| `unreached_in_unit` | 337 | **51** are included by a unit that PASSES — reached |
| `accepted_in_unit` | 1501 | **0** have no passing unit — a regression candidate |

The members the 2023 reading reaches, by the passing unit that includes them (the per-member rows are in `census.tsv`, kind `member`):

| suite | unit (package file) | members reached |
|---|---|---:|
| `opentitan` | `hw/top_darjeeling/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | 17 |
| `opentitan` | `hw/top_earlgrey/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | 17 |
| `opentitan` | `hw/top_englishbreakfast/ip_autogen/rstmgr/dv/env/rstmgr_env_pkg.sv` | 17 |
