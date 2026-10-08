# SV external-corpus adjudication - the verilog_2005 profile lane (leaf SV-CORPUS-GRAD.8c)

Input: `results_v2005.tsv` (2943 rows parsed under `--profile verilog_2005`); expected verdicts per IEEE 1364-2005 answer keys (ispras TYPE headers + KNOWN_TEXT_BUGS deferral note, ivtest regress-vlg.list / plain-Verilog vvp_tests descriptors with the CE golden syntax-error split, sv2v conversion-golden contract).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| ispras-sv-tests | 356 | 317 | 0 | 10 | 29 |
| iverilog | 2246 | 2042 | 0 | 148 | 56 |
| sv2v | 341 | 294 | 0 | 47 | 0 |
| **total** | **2943** | **2653** | **0** | **205** | **85** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 2 |
| deferred:descriptor_conflict | 8 |
| deferred:impl_varying_v2005 | 27 |
| deferred:negative_stage_triage_v2005 | 40 |
| deferred:ni_unimplemented | 6 |
| deferred:svpp_owned_v2005 | 2 |
| divergence:explained_svpp_conditional | 80 |
| divergence:explained_svpp_include | 14 |
| divergence:explained_svpp_keywords_region | 2 |
| divergence:explained_svpp_macro_definition | 2 |
| divergence:explained_svpp_macro_use | 107 |
| match | 2653 |

**The v2005 arm's burn-down baseline = the UNEXPLAINED divergence count (0: rejects-valid 0, accepts-invalid 0, crash 0)** - a separate arm from the sv_2017 baseline; the `.5` graduation gate asserts both.

