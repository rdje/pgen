# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (16336 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 557 | 49 | 22 | 200 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 769 | 35 | 16 | 446 |
| iverilog | 3799 | 1018 | 18 | 232 | 2531 |
| opentitan | 3983 | 0 | 0 | 4 | 3979 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 54 | 1 | 15 | 22 |
| sv-tests | 1028 | 795 | 11 | 121 | 101 |
| sv2v | 953 | 462 | 37 | 80 | 374 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 112 | 12 | 6 | 22 |
| verilator | 3263 | 2037 | 130 | 967 | 129 |
| **total** | **16336** | **5804** | **293** | **1463** | **8776** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 5272 |
| deferred:impl_varying | 90 |
| deferred:ni_unimplemented | 6 |
| deferred:no_sv_key | 743 |
| deferred:svpp_owned | 186 |
| deferred:v2005_profile_lane | 2459 |
| deferred:verilog_ams_lane | 20 |
| divergence:explained_svpp_conditional | 203 |
| divergence:explained_svpp_include | 141 |
| divergence:explained_svpp_macro_use | 1111 |
| divergence:explained_svpp_protected_envelope | 4 |
| divergence:explained_timeout | 4 |
| divergence:unexplained_accepts_invalid | 21 |
| divergence:unexplained_rejects_valid | 272 |
| match | 5804 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**293**: rejects-valid 272, accepts-invalid 21). Explained divergences are svpp/chaining/timeout-owned with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

