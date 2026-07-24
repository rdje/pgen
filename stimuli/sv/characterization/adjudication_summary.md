# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (16336 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 545 | 61 | 22 | 200 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 735 | 69 | 16 | 446 |
| iverilog | 3799 | 1015 | 45 | 208 | 2531 |
| opentitan | 3983 | 0 | 0 | 8 | 3975 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 54 | 1 | 15 | 22 |
| sv-tests | 1028 | 786 | 21 | 120 | 101 |
| sv2v | 953 | 450 | 50 | 79 | 374 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 111 | 13 | 6 | 22 |
| verilator | 3263 | 1987 | 187 | 961 | 128 |
| **total** | **16336** | **5683** | **447** | **1435** | **8771** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 5267 |
| deferred:impl_varying | 90 |
| deferred:ni_unimplemented | 6 |
| deferred:no_sv_key | 743 |
| deferred:svpp_owned | 186 |
| deferred:v2005_profile_lane | 2459 |
| deferred:verilog_ams_lane | 20 |
| divergence:explained_svpp_conditional | 210 |
| divergence:explained_svpp_include | 142 |
| divergence:explained_svpp_macro_use | 1074 |
| divergence:explained_timeout | 9 |
| divergence:unexplained_accepts_invalid | 21 |
| divergence:unexplained_rejects_valid | 426 |
| match | 5683 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**447**: rejects-valid 426, accepts-invalid 21). Explained divergences are svpp/chaining/timeout-owned with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

