# SV external-corpus adjudication - the verilog_2005 profile lane (leaf SV-CORPUS-GRAD.8c)

Input: `results_v2005.tsv` (2606 rows parsed under `--profile verilog_2005`); expected verdicts per IEEE 1364-2005 answer keys (ispras TYPE headers + KNOWN_TEXT_BUGS deferral note, ivtest regress-vlg.list / plain-Verilog vvp_tests descriptors with the CE golden syntax-error split, sv2v conversion-golden contract).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| ispras-sv-tests | 356 | 317 | 0 | 10 | 29 |
| iverilog | 1909 | 1734 | 0 | 133 | 42 |
| sv2v | 341 | 294 | 0 | 47 | 0 |
| **total** | **2606** | **2345** | **0** | **190** | **71** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 2 |
| deferred:descriptor_conflict | 8 |
| deferred:impl_varying_v2005 | 27 |
| deferred:negative_stage_triage_v2005 | 30 |
| deferred:ni_unimplemented | 2 |
| deferred:svpp_owned_v2005 | 2 |
| divergence:explained_svpp_conditional | 73 |
| divergence:explained_svpp_include | 14 |
| divergence:explained_svpp_keywords_region | 2 |
| divergence:explained_svpp_macro_definition | 2 |
| divergence:explained_svpp_macro_use | 99 |
| match | 2345 |

**The v2005 arm's burn-down baseline = the UNEXPLAINED divergence count (0: rejects-valid 0, accepts-invalid 0, crash 0)** - a separate arm from the sv_2017 baseline; the `.5` graduation gate asserts both.

