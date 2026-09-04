# The sv_2017 × sv_2023 verdict census (SV-CORPUS-GRAD.14a)

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/census.py`; `--check` re-derives and diffs.

Both lanes parse the SAME population — every `.sv` / `.svh` / `.v` file under `stimuli/sv/subs` plus the folded `uvm-core` tree — with the same binary; only `--profile` differs. A row is `pass` or not (`fail` / `timeout` / `crash` are all not-pass; the status pairs are below).

## Instrument identity

> Re-hash the rows; if any differs, this census no longer describes your tree — re-run it, never quote it.

| input | repo-root-relative path | sha256 |
|---|---|---|
| parse binary | `rust/target/release/parseability_probe` | `8f7a9dea8127eafcae924f4990d9a24c789788d5a243f63f651c1b1497a1f9eb` |
| grammar raw ast | `grammars/systemverilog.ebnf` | `1e2ad24a6550f631b0a712fb5f40d5eca70d0920915653cb868af1dca722ccef` |
| sv_2017 results (output) | `stimuli/sv/characterization/results.tsv` | `a36818d015123d07d5b675125a972c3f63a3ce1495c753d3ccd26199902456ad` |
| sv_2023 results (output) | `stimuli/sv/characterization/results_sv2023.tsv` | `6c57b4dc928e6641d75c96566972173ca207544929a18be1cb93c4bfbbac1277` |
| sv_2017 manifest | `stimuli/sv/characterization/adjudication_manifest.tsv` | `02b98e38c03bb70902c6b9062cefb61d276b88d8d34e41667a4d6074e70d2247` |

## The matrix

Rows joined: **16336**; only in the sv_2017 lane: **0**; only in the sv_2023 lane: **0** (a non-zero here is a population breach, not a verdict).

| cell | rows | meaning |
|---|---|---|
| `both_pass` | 9782 | parses under both editions |
| `both_nonpass` | 6533 | parses under neither |
| `2023_only_pass` | 21 | the 1800-2023 widening the corpus exercises — `must_accept` on a 2023 lane (`.14b`) |
| `2017_only_pass` | 0 | a 2023-profile REGRESSION candidate — a parser question until adjudicated |

### Status pairs (sv_2017, sv_2023)

| sv_2017 | sv_2023 | rows |
|---|---|---|
| `pass` | `pass` | 9782 |
| `fail` | `fail` | 6533 |
| `fail` | `pass` | 21 |

## Every row whose verdict differs

The 2017-lane adjudication class is carried beside each row so the reader sees whether the row is already ruled (an edition contrast keyed `must_reject` on the 2017 lane is exactly a `2023_only_pass` row).

| cell | 2017-lane adjudication | rows |
|---|---|---|
| `2023_only_pass` | `deferred:v2005_profile_lane` | 1 |
| `2023_only_pass` | `divergence:explained_svpp_macro_use` | 1 |
| `2023_only_pass` | `match` | 19 |

| cell | suite | relpath | sv_2017 (furthest) | sv_2023 (furthest) | 2017-lane adjudication | expected |
|---|---|---|---|---|---|---|
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/event_array.v` | fail (394) | pass (-) | `deferred:v2005_profile_lane` | `out_of_scope_with_cause:v2005_profile_lane` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_soft_packed_union_fail1.v` | fail (99) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_class_fail1.v` | fail (108) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_class_fail2.v` | fail (131) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_enum_fail1.v` | fail (107) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_enum_fail2.v` | fail (148) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_struct_fail1.v` | fail (187) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_struct_fail2.v` | fail (243) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_union_fail1.v` | fail (166) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/sv_type_param_restrict_union_fail2.v` | fail (241) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_array_event.v` | fail (393) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_extends_default.v` | fail (320) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_new_default.v` | fail (249) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_override.v` | fail (978) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_class_override_bad.v` | fail (819) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_event_array_fire.v` | fail (1625) | pass (-) | `divergence:explained_svpp_macro_use` | `must_accept` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_export_packed_struct.v` | fail (243) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_export_packed_struct2.v` | fail (262) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_string_sel.v` | fail (651) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_union_soft.v` | fail (221) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_var_ref_static.v` | fail (350) | pass (-) | `match` | `must_reject` |
