# SV external-corpus adjudication - the sv_2023 profile lane (leaf SV-CORPUS-GRAD.14b.2)

Input: `results_sv2023.tsv` (16336 rows - the SAME population as `results.tsv`, parsed under `--profile sv_2023`); expected verdicts = the sv_2017 key read EDITION-RELATIVELY: 25 rows the 2017 key rules edition contrasts are `must_accept` here (the 1800-2023 production is cited in their basis); 12 rows are `must_accept` by an explicit 1800-2023 reading (text 1800-2023 derives whose upstream failure is semantic); every other row inherits its 2017 ruling.

| suite | rows | match | UNEXPLAINED div | explained div | deferred |
|---|---|---|---|---|---|
| Cores-VeeR-EL2 | 102 | 0 | 0 | 0 | 102 |
| Surelog | 828 | 584 | 0 | 43 | 201 |
| black-parrot | 205 | 0 | 0 | 0 | 205 |
| friscv | 441 | 0 | 0 | 0 | 441 |
| ispras-sv-tests | 1266 | 802 | 0 | 17 | 447 |
| iverilog | 3799 | 1034 | 0 | 234 | 2531 |
| opentitan | 3983 | 0 | 0 | 0 | 3983 |
| scr1 | 50 | 0 | 0 | 0 | 50 |
| slang | 92 | 55 | 0 | 8 | 29 |
| sv-tests | 1028 | 809 | 0 | 124 | 95 |
| sv2v | 953 | 500 | 0 | 80 | 373 |
| uvm-core | 174 | 0 | 0 | 0 | 174 |
| verible | 152 | 120 | 0 | 8 | 24 |
| verilator | 3263 | 2157 | 0 | 914 | 192 |
| **total** | **16336** | **6061** | **0** | **1428** | **8847** |

## Verdict-class detail

| class | count |
|---|---|
| deferred:chained_only | 4933 |
| deferred:impl_varying | 90 |
| deferred:ni_unimplemented | 6 |
| deferred:no_sv_key | 596 |
| deferred:not_source_text | 350 |
| deferred:svpp_owned | 241 |
| deferred:v2005_profile_lane | 2606 |
| deferred:verilog_ams_lane | 25 |
| divergence:explained_store_gate | 54 |
| divergence:explained_svpp_conditional | 195 |
| divergence:explained_svpp_include | 133 |
| divergence:explained_svpp_keywords_region | 4 |
| divergence:explained_svpp_macro_use | 1038 |
| divergence:explained_svpp_protected_envelope | 4 |
| match | 6061 |

**The sv_2023 arm's burn-down baseline = the UNEXPLAINED divergence count (0: rejects-valid 0, accepts-invalid 0, crash 0)** - a separate arm from the sv_2017 baseline over the same files.

✅ **Re-read under 1800-2023 (.14b.3):** 49 inherited `must_reject` rulings carry over MECHANICALLY - every 1800-2017 production their 2017 basis cites reads identically in 1800-2023 Annex A (`production_parity.py`, its artifact `docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/parity.md`) - and 72 carry over BY HAND with the changed 2023 production cited in the basis.

⚠️ **What this lane has NOT judged for itself, stated rather than hidden:** 54 `must_reject` rows are INHERITED from a 1800-2017 production the 2017 key cites, and 240 from a suite's own 2017-era oracle; each carries the marker `its 1800-2023 re-read is owed (.14b.3)` in its basis. Their `match` here means the parser REJECTS them under sv_2023 exactly as under sv_2017 (the .14a census names every file whose verdict differs between the profiles, and these are not among them); whether 1800-2023 derives any of them is `.14b.3`'s per-row question, and a row it turns must_accept becomes an `unexplained_rejects_valid` of THIS lane.

