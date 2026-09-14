# The sv_2017 × sv_2023 verdict census (SV-CORPUS-GRAD.14a)

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/census.py`; `--check` re-derives and diffs.

Both lanes parse the SAME population — every `.sv` / `.svh` / `.v` file under `stimuli/sv/subs` plus the folded `uvm-core` tree — with the same binary; only `--profile` differs. A row is `pass` or not (`fail` / `timeout` / `crash` are all not-pass; the status pairs are below).

## Instrument identity

> Re-hash the rows; if any differs, this census no longer describes your tree — re-run it, never quote it.

| input | repo-root-relative path | sha256 |
|---|---|---|
| parse binary | `rust/target/release/parseability_probe` | `bcd1193a41755e65c9817213e5704ef79e9f894537c015daef9026f8382724c3` |
| grammar raw ast | `grammars/systemverilog.ebnf` | `fe2a293d88a0f00cac5de8bc9a21ccc1719ce182fefa0655a6a727b06727ec32` |
| sv_2017 results (output) | `stimuli/sv/characterization/results.tsv` | `9b86536adaf192ccb0eeb02af281aee1b6ee7a8475fd82e80787b986eb693355` |
| sv_2023 results (output) | `stimuli/sv/characterization/results_sv2023.tsv` | `aa3cbc42852ae8da43be306f2e451395bc85f1a1a52c4c4ae950ae495550b8e3` |
| sv_2017 manifest | `stimuli/sv/characterization/adjudication_manifest.tsv` | `75fa5b1c6706eef59967f8edc6af905a8a4f7999e60353c5d73366a4de4a4deb` |

## The matrix

Rows joined: **16336**; only in the sv_2017 lane: **0**; only in the sv_2023 lane: **0** (a non-zero here is a population breach, not a verdict).

| cell | rows | meaning |
|---|---|---|
| `both_pass` | 9844 | parses under both editions |
| `both_nonpass` | 6467 | parses under neither |
| `2023_only_pass` | 25 | the 1800-2023 widening the corpus exercises — `must_accept` on a 2023 lane (`.14b`) |
| `2017_only_pass` | 0 | a 2023-profile REGRESSION candidate — a parser question until adjudicated |

### Status pairs (sv_2017, sv_2023)

| sv_2017 | sv_2023 | rows |
|---|---|---|
| `pass` | `pass` | 9844 |
| `fail` | `fail` | 6467 |
| `fail` | `pass` | 25 |

## Every row whose verdict differs

The 2017-lane adjudication class is carried beside each row so the reader sees whether the row is already ruled (an edition contrast keyed `must_reject` on the 2017 lane is exactly a `2023_only_pass` row).

| cell | 2017-lane adjudication | rows |
|---|---|---|
| `2023_only_pass` | `deferred:svpp_owned` | 3 |
| `2023_only_pass` | `deferred:v2005_profile_lane` | 1 |
| `2023_only_pass` | `match` | 21 |

| cell | suite | relpath | sv_2017 (furthest) | sv_2023 (furthest) | 2017-lane adjudication | expected |
|---|---|---|---|---|---|---|
| `2023_only_pass` | `iverilog` | `ivtest/ivltests/begin_keywords_1800_2023.v` | fail (81) | pass (-) | `match` | `must_reject` |
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
| `2023_only_pass` | `verilator` | `test_regress/t/t_display_qqq.v` | fail (245) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_event_array_fire.v` | fail (1625) | pass (-) | `deferred:svpp_owned` | `out_of_scope_with_cause` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_export_packed_struct.v` | fail (243) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_export_packed_struct2.v` | fail (262) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_preproc_ifexpr_bad.v` | fail (241) | pass (-) | `deferred:svpp_owned` | `out_of_scope_with_cause` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_preproc_preproczero_bad.v` | fail (241) | pass (-) | `deferred:svpp_owned` | `out_of_scope_with_cause` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_string_sel.v` | fail (651) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_union_soft.v` | fail (221) | pass (-) | `match` | `must_reject` |
| `2023_only_pass` | `verilator` | `test_regress/t/t_var_ref_static.v` | fail (350) | pass (-) | `match` | `must_reject` |
