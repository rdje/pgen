# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (16336 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 584 | 0 | 43 | 201 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 802 | 0 | 17 | 447 |
| iverilog | 3799 | 1240 | 0 | 227 | 2332 |
| opentitan | 3983 | 0 | 0 | 0 | 3983 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 55 | 0 | 8 | 29 |
| sv-tests | 1028 | 809 | 0 | 124 | 95 |
| sv2v | 953 | 500 | 0 | 80 | 373 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 120 | 0 | 8 | 24 |
| verilator | 3263 | 2156 | 0 | 911 | 196 |
| **total** | **16336** | **6266** | **0** | **1418** | **8652** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 4937 |
| deferred:impl_varying | 90 |
| deferred:ni_unimplemented | 14 |
| deferred:no_sv_key | 48 |
| deferred:not_source_text | 350 |
| deferred:svpp_owned | 245 |
| deferred:v2005_profile_lane | 2943 |
| deferred:verilog_ams_lane | 25 |
| divergence:explained_store_gate | 54 |
| divergence:explained_svpp_conditional | 196 |
| divergence:explained_svpp_include | 133 |
| divergence:explained_svpp_keywords_region | 4 |
| divergence:explained_svpp_macro_use | 1027 |
| divergence:explained_svpp_protected_envelope | 4 |
| match | 6266 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**0**: rejects-valid 0, accepts-invalid 0). Explained divergences are svpp/chaining/timeout-owned or store-gate-owned (`.13w`: the row parses once its undeclared type is declared) with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

