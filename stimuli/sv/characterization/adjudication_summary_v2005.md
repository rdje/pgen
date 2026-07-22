# SV external-corpus adjudication - the verilog_2005 profile lane (leaf SV-CORPUS-GRAD.8c)

Input: `results_v2005.tsv` (2459 rows parsed under `--profile verilog_2005`); expected verdicts per IEEE 1364-2005 answer keys (ispras TYPE headers + KNOWN_TEXT_BUGS deferral note, ivtest regress-vlg.list / plain-Verilog vvp_tests descriptors with the CE golden syntax-error split, sv2v conversion-golden contract).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| ispras-sv-tests | 356 | 301 | 17 | 9 | 29 |
| iverilog | 1762 | 1365 | 107 | 114 | 176 |
| sv2v | 341 | 281 | 11 | 49 | 0 |
| **total** | **2459** | **1947** | **135** | **172** | **205** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 2 |
| deferred:impl_varying_v2005 | 27 |
| deferred:negative_stage_triage_v2005 | 176 |
| divergence:explained_svpp_conditional | 73 |
| divergence:explained_svpp_include | 15 |
| divergence:explained_svpp_macro_use | 84 |
| divergence:unexplained_accepts_invalid | 1 |
| divergence:unexplained_crash | 1 |
| divergence:unexplained_rejects_valid | 133 |
| match | 1947 |

**The v2005 arm's burn-down baseline = the UNEXPLAINED divergence count (135: rejects-valid 133, accepts-invalid 1, crash 1)** - a separate arm from the sv_2017 baseline; the `.5` graduation gate asserts both.

