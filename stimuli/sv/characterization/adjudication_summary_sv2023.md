# SV external-corpus adjudication - the sv_2023 profile lane (leaf SV-CORPUS-GRAD.14b.2)

Input: `results_sv2023.tsv` (16336 rows - the SAME population as `results.tsv`, parsed under `--profile sv_2023`); expected verdicts = the sv_2017 key read EDITION-RELATIVELY: 20 rows the 2017 key rules edition contrasts are `must_accept` here (the 1800-2023 production is cited in their basis); 12 rows are `must_accept` by an explicit 1800-2023 reading (text 1800-2023 derives whose upstream failure is semantic); every other row inherits its 2017 ruling.

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 574 | 8 | 46 | 200 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 796 | 8 | 16 | 446 |
| iverilog | 3799 | 1026 | 9 | 233 | 2531 |
| opentitan | 3983 | 0 | 0 | 0 | 3983 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 54 | 1 | 15 | 22 |
| sv-tests | 1028 | 806 | 4 | 123 | 95 |
| sv2v | 953 | 484 | 13 | 87 | 369 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 115 | 6 | 9 | 22 |
| verilator | 3263 | 2124 | 33 | 974 | 132 |
| **total** | **16336** | **5979** | **82** | **1503** | **8772** |

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
| divergence:explained_svpp_macro_use | 1105 |
| divergence:explained_svpp_protected_envelope | 4 |
| divergence:unexplained_accepts_invalid | 2 |
| divergence:unexplained_rejects_valid | 80 |
| match | 5979 |

**The sv_2023 arm's burn-down baseline = the UNEXPLAINED divergence count (82: rejects-valid 80, accepts-invalid 2, crash 0)** - a separate arm from the sv_2017 baseline over the same files.

✅ **Re-read under 1800-2023 (.14b.3):** 48 inherited `must_reject` rulings carry over MECHANICALLY - every 1800-2017 production their 2017 basis cites reads identically in 1800-2023 Annex A (`production_parity.py`, its artifact `docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/parity.md`) - and 12 carry over BY HAND with the changed 2023 production cited in the basis.

⚠️ **What this lane has NOT judged for itself, stated rather than hidden:** 19 `must_reject` rows are INHERITED from a 1800-2017 production the 2017 key cites, and 201 from a suite's own 2017-era oracle; each carries the marker `its 1800-2023 re-read is owed (.14b.3)` in its basis. Their `match` here means the parser REJECTS them under sv_2023 exactly as under sv_2017 (the .14a census names every file whose verdict differs between the profiles, and these are not among them); whether 1800-2023 derives any of them is `.14b.3`'s per-row question, and a row it turns must_accept becomes an `unexplained_rejects_valid` of THIS lane.

