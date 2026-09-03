# The sv_2017 × sv_2023 verdict census (SV-CORPUS-GRAD.14a)

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/census.py`; `--check` re-derives and diffs.

Both lanes parse the SAME population — every `.sv` / `.svh` / `.v` file under `stimuli/sv/subs` plus the folded `uvm-core` tree — with the same binary; only `--profile` differs. A row is `pass` or not (`fail` / `timeout` / `crash` are all not-pass; the status pairs are below).

## Instrument identity

> Re-hash the rows; if any differs, this census no longer describes your tree — re-run it, never quote it.

| input | repo-root-relative path | sha256 |
|---|---|---|
| parse binary | `rust/target/release/parseability_probe` | `d31dc9c3b2a8500912f1e41b502f574abf189daa2a10ccd3d51f890c9e575bda` |
| grammar raw ast | `grammars/systemverilog.ebnf` | `c43ae6d66d4a880522b70647e726ce17a17c6ce283d92d50d68676117a085603` |
| sv_2017 results (output) | `stimuli/sv/characterization/results.tsv` | `1be32296850992d0c6af870c75658532374a1f9cad6b87526313c853d800639f` |
| sv_2023 results (output) | `stimuli/sv/characterization/results_sv2023.tsv` | `b3c3ef13ea8cb3a29c21fd2df713acb914cede1f514cb46f538371ca8fafc8a4` |
| sv_2017 manifest | `stimuli/sv/characterization/adjudication_manifest.tsv` | `e628a65a15f4227b7777f274f03fe6dd9bcc6a815954002bb5136f11c3350033` |

## The matrix

Rows joined: **16336**; only in the sv_2017 lane: **0**; only in the sv_2023 lane: **0** (a non-zero here is a population breach, not a verdict).

| cell | rows | meaning |
|---|---|---|
| `both_pass` | 9831 | parses under both editions |
| `both_nonpass` | 6477 | parses under neither |
| `2023_only_pass` | 28 | the 1800-2023 widening the corpus exercises — `must_accept` on a 2023 lane (`.14b`) |
| `2017_only_pass` | 0 | a 2023-profile REGRESSION candidate — a parser question until adjudicated |

### Status pairs (sv_2017, sv_2023)

| sv_2017 | sv_2023 | rows |
|---|---|---|
| `pass` | `pass` | 9831 |
| `fail` | `fail` | 6477 |
| `fail` | `pass` | 28 |

## Every row whose verdict differs

The 2017-lane adjudication class is carried beside each row so the reader sees whether the row is already ruled (an edition contrast keyed `must_reject` on the 2017 lane is exactly a `2023_only_pass` row).

| cell | 2017-lane adjudication | rows |
|---|---|---|
| `2023_only_pass` | `deferred:v2005_profile_lane` | 1 |
| `2023_only_pass` | `divergence:explained_svpp_macro_use` | 1 |
| `2023_only_pass` | `divergence:unexplained_rejects_valid` | 15 |
| `2023_only_pass` | `match` | 11 |

| cell | suite | relpath | sv_2017 (furthest) | sv_2023 (furthest) | 2017-lane adjudication | expected |
|---|---|---|---|---|---|---|
| `2023_only_pass` | `ispras-sv-tests` | `ieee-1800-2012/12/12.06.01_05.sv` | fail (606) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `ispras-sv-tests` | `ieee-1800-2012/12/12.06.01_06.sv` | fail (628) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `ispras-sv-tests` | `ieee-1800-2012/12/12.06.02_01.sv` | fail (454) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `ispras-sv-tests` | `ieee-1800-2012/12/12.06.02_02.sv` | fail (433) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `ispras-sv-tests` | `ieee-1800-2012/12/12.06.02_03.sv` | fail (456) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/event_array.v` | fail (394) | pass (-) | `deferred:v2005_profile_lane` | `out_of_scope_with_cause:v2005_profile_lane` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_soft_packed_union_fail1.v` | fail (104) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_class_fail1.v` | fail (108) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_class_fail2.v` | fail (131) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_enum_fail1.v` | fail (107) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_enum_fail2.v` | fail (148) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_struct_fail1.v` | fail (187) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_struct_fail2.v` | fail (243) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_union_fail1.v` | fail (166) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_union_fail2.v` | fail (241) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_array_event.v` | fail (393) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_extends_default.v` | fail (320) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_new_default.v` | fail (249) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_override.v` | fail (978) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_override_bad.v` | fail (819) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_event_array_fire.v` | fail (1625) | pass (-) | `divergence:explained_svpp_macro_use` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_export_packed_struct.v` | fail (248) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_export_packed_struct2.v` | fail (267) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_param_type_fwd.v` | fail (378) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_param_type_fwd_bad.v` | fail (251) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_string_sel.v` | fail (651) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_union_soft.v` | fail (221) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_var_ref_static.v` | fail (357) | pass (-) | `divergence:unexplained_rejects_valid` | `must_accept` |
