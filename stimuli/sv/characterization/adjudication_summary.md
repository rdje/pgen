# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (16336 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 578 | 4 | 46 | 200 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 799 | 5 | 16 | 446 |
| iverilog | 3799 | 1042 | 1 | 225 | 2531 |
| opentitan | 3983 | 0 | 0 | 0 | 3983 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 55 | 0 | 15 | 22 |
| sv-tests | 1028 | 809 | 1 | 123 | 95 |
| sv2v | 953 | 491 | 6 | 87 | 369 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 120 | 1 | 9 | 22 |
| verilator | 3263 | 2143 | 13 | 975 | 132 |
| **total** | **16336** | **6037** | **31** | **1496** | **8772** |

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
| divergence:explained_svpp_macro_use | 1098 |
| divergence:explained_svpp_protected_envelope | 4 |
| divergence:unexplained_accepts_invalid | 2 |
| divergence:unexplained_rejects_valid | 29 |
| match | 6037 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**31**: rejects-valid 29, accepts-invalid 2). Explained divergences are svpp/chaining/timeout-owned or store-gate-owned (`.13w`: the row parses once its undeclared type is declared) with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

