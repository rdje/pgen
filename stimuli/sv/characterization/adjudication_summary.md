# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)

Input: `results.tsv` (5128 rows); generator: `stimuli/sv/adjudicate_external_corpus.py` (deterministic).

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 54 | 1 | 15 | 22 |
| sv-tests | 1028 | 741 | 66 | 120 | 101 |
| verible | 152 | 111 | 13 | 6 | 22 |
| verilator | 3263 | 1933 | 241 | 961 | 128 |
| **total** | **5128** | **2839** | **321** | **1102** | **866** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 711 |
| deferred:svpp_owned | 155 |
| divergence:explained_svpp_conditional | 191 |
| divergence:explained_svpp_include | 124 |
| divergence:explained_svpp_macro_use | 786 |
| divergence:explained_timeout | 1 |
| divergence:unexplained_accepts_invalid | 6 |
| divergence:unexplained_rejects_valid | 315 |
| match | 2839 |

**The graduation burn-down baseline = the UNEXPLAINED divergence count** (**321**: rejects-valid 315, accepts-invalid 6). Explained divergences are svpp/chaining/timeout-owned with named causes; deferred rows adjudicate in their owning lanes (leaf .4 chaining, SVPP lane).

