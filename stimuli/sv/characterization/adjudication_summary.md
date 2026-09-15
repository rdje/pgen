# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (16336 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 580 | 1 | 46 | 201 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 802 | 2 | 16 | 446 |
| iverilog | 3799 | 1043 | 0 | 225 | 2531 |
| opentitan | 3983 | 0 | 0 | 0 | 3983 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 55 | 0 | 8 | 29 |
| sv-tests | 1028 | 809 | 1 | 123 | 95 |
| sv2v | 953 | 500 | 1 | 79 | 373 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 120 | 0 | 8 | 24 |
| verilator | 3263 | 2156 | 1 | 910 | 196 |
| **total** | **16336** | **6065** | **6** | **1415** | **8850** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 4933 |
| deferred:impl_varying | 90 |
| deferred:ni_unimplemented | 6 |
| deferred:no_sv_key | 596 |
| deferred:not_source_text | 350 |
| deferred:svpp_owned | 244 |
| deferred:v2005_profile_lane | 2606 |
| deferred:verilog_ams_lane | 25 |
| divergence:explained_store_gate | 58 |
| divergence:explained_svpp_conditional | 195 |
| divergence:explained_svpp_include | 133 |
| divergence:explained_svpp_macro_use | 1025 |
| divergence:explained_svpp_protected_envelope | 4 |
| divergence:unexplained_accepts_invalid | 1 |
| divergence:unexplained_rejects_valid | 5 |
| match | 6065 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**6**: rejects-valid 5, accepts-invalid 1). Explained divergences are svpp/chaining/timeout-owned or store-gate-owned (`.13w`: the row parses once its undeclared type is declared) with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

