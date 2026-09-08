# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (16336 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 574 | 8 | 46 | 200 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 796 | 8 | 16 | 446 |
| iverilog | 3799 | 1035 | 9 | 224 | 2531 |
| opentitan | 3983 | 0 | 0 | 0 | 3983 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 54 | 1 | 15 | 22 |
| sv-tests | 1028 | 806 | 4 | 123 | 95 |
| sv2v | 953 | 484 | 13 | 87 | 369 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 115 | 6 | 9 | 22 |
| verilator | 3263 | 2123 | 33 | 975 | 132 |
| **total** | **16336** | **5987** | **82** | **1495** | **8772** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 4926 |
| deferred:impl_varying | 90 |
| deferred:ni_unimplemented | 6 |
| deferred:no_sv_key | 596 |
| deferred:not_source_text | 350 |
| deferred:svpp_owned | 173 |
| deferred:v2005_profile_lane | 2606 |
| deferred:verilog_ams_lane | 25 |
| divergence:explained_store_gate | 56 |
| divergence:explained_svpp_conditional | 198 |
| divergence:explained_svpp_include | 140 |
| divergence:explained_svpp_macro_use | 1097 |
| divergence:explained_svpp_protected_envelope | 4 |
| divergence:unexplained_accepts_invalid | 2 |
| divergence:unexplained_rejects_valid | 80 |
| match | 5987 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**82**: rejects-valid 80, accepts-invalid 2). Explained divergences are svpp/chaining/timeout-owned or store-gate-owned (`.13w`: the row parses once its undeclared type is declared) with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

