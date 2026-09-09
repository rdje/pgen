# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (16336 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 580 | 2 | 45 | 201 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 802 | 2 | 16 | 446 |
| iverilog | 3799 | 1043 | 0 | 225 | 2531 |
| opentitan | 3983 | 0 | 0 | 0 | 3983 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 55 | 0 | 8 | 29 |
| sv-tests | 1028 | 809 | 1 | 123 | 95 |
| sv2v | 953 | 499 | 2 | 79 | 373 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 120 | 1 | 7 | 24 |
| verilator | 3263 | 2156 | 1 | 922 | 184 |
| **total** | **16336** | **6064** | **9** | **1425** | **8838** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 4933 |
| deferred:impl_varying | 90 |
| deferred:ni_unimplemented | 6 |
| deferred:no_sv_key | 596 |
| deferred:not_source_text | 350 |
| deferred:svpp_owned | 232 |
| deferred:v2005_profile_lane | 2606 |
| deferred:verilog_ams_lane | 25 |
| divergence:explained_store_gate | 56 |
| divergence:explained_svpp_conditional | 195 |
| divergence:explained_svpp_include | 133 |
| divergence:explained_svpp_macro_use | 1037 |
| divergence:explained_svpp_protected_envelope | 4 |
| divergence:unexplained_accepts_invalid | 2 |
| divergence:unexplained_rejects_valid | 7 |
| match | 6064 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**9**: rejects-valid 7, accepts-invalid 2). Explained divergences are svpp/chaining/timeout-owned or store-gate-owned (`.13w`: the row parses once its undeclared type is declared) with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

