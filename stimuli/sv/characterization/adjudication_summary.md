# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (16336 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 0 | 0 | 0 | 828 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 669 | 114 | 16 | 467 |
| iverilog | 3799 | 604 | 21 | 103 | 3071 |
| opentitan | 3983 | 0 | 0 | 8 | 3975 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 54 | 1 | 15 | 22 |
| sv-tests | 1028 | 768 | 39 | 120 | 101 |
| sv2v | 953 | 265 | 31 | 79 | 578 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 111 | 13 | 6 | 22 |
| verilator | 3263 | 1948 | 226 | 961 | 128 |
| **total** | **16336** | **4419** | **445** | **1308** | **10164** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 5896 |
| deferred:error_pretriage | 234 |
| deferred:impl_varying | 90 |
| deferred:negative_stage_triage | 201 |
| deferred:no_sv_key | 1130 |
| deferred:svpp_owned | 155 |
| deferred:v2005_profile_lane | 2458 |
| divergence:explained_svpp_conditional | 207 |
| divergence:explained_svpp_include | 140 |
| divergence:explained_svpp_macro_use | 952 |
| divergence:explained_timeout | 9 |
| divergence:unexplained_accepts_invalid | 6 |
| divergence:unexplained_rejects_valid | 439 |
| match | 4419 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**445**: rejects-valid 439, accepts-invalid 6). Explained divergences are svpp/chaining/timeout-owned with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

