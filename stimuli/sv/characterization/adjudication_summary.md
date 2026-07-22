# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (16336 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 536 | 70 | 22 | 200 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 687 | 117 | 16 | 446 |
| iverilog | 3799 | 739 | 38 | 208 | 2814 |
| opentitan | 3983 | 0 | 0 | 8 | 3975 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 54 | 1 | 15 | 22 |
| sv-tests | 1028 | 768 | 39 | 120 | 101 |
| sv2v | 953 | 447 | 53 | 79 | 374 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 111 | 13 | 6 | 22 |
| verilator | 3263 | 1948 | 226 | 961 | 128 |
| **total** | **16336** | **5290** | **557** | **1435** | **9054** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 5267 |
| deferred:impl_varying | 90 |
| deferred:negative_stage_triage | 283 |
| deferred:ni_unimplemented | 6 |
| deferred:no_sv_key | 743 |
| deferred:svpp_owned | 186 |
| deferred:v2005_profile_lane | 2459 |
| deferred:verilog_ams_lane | 20 |
| divergence:explained_svpp_conditional | 210 |
| divergence:explained_svpp_include | 142 |
| divergence:explained_svpp_macro_use | 1074 |
| divergence:explained_timeout | 9 |
| divergence:unexplained_accepts_invalid | 17 |
| divergence:unexplained_rejects_valid | 540 |
| match | 5290 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**557**: rejects-valid 540, accepts-invalid 17). Explained divergences are svpp/chaining/timeout-owned with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

