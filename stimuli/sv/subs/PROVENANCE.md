# SystemVerilog external test-corpus provenance (EXTERNAL-CORPUS.3.1)

External SystemVerilog **test corpora** vendored as **git submodules** (pinned commits) and
sparse-checked-out to their test directories only ("test corpora, not the tool code").
Used **only as parser test inputs** (read → parse → characterize) by
`stimuli/run_external_corpus.sh sv` — never compiled, linked, modified, or redistributed.

> **License note (director decision 2026-06-17):** a git submodule stores only a *pointer*
> (upstream URL + pinned commit SHA); the upstream code is **not copied into PGEN** and does
> **not** change PGEN's own license. Using these files as parser test inputs does not create a
> derivative work, so no copyleft (GPL/LGPL) obligation attaches to PGEN. Copyleft repos are
> flagged ⚠️ below for transparency. (Not legal advice; standard understanding for test-input
> data + submodule references.)

| Submodule | Upstream | Pinned commit | License | Sparse path | ~SV files | Role |
|---|---|---|---|---|---|---|
| `sv-tests` | github.com/chipsalliance/sv-tests | `25e4d275` | **ISC** (permissive) | `tests/` | 1028 | THE canonical SV conformance corpus (per-file `:should_fail_because:`/`:tags:` metadata) |
| `verible` | github.com/chipsalliance/verible | `a0a8d8eb` | **Apache-2.0** | `verible/verilog/` | 152 | Google/CHIPS SV linter/formatter test fixtures |
| `slang` | github.com/MikePopoloski/slang | `4106501b` | **MIT** | `tests/` | 92 | Leading FOSS SV front-end standalone test files |
| `verilator` | github.com/verilator/verilator | `a534a1d1` | ⚠️ **LGPL-3.0-only OR Artistic-2.0** (copyleft; Artistic arm elected) | `test_regress/` | ~3263 | De-facto FOSS SV simulator regression suite (largest practical SV corpus; SV mostly in `.v` files) |
| `ispras-sv-tests` | github.com/ispras/sv-tests | `f9062e68` | **BSD-3-Clause** | `ieee-1364-2005/` + `ieee-1800-2012/` | 1266 | ISP RAS LRM-clause-keyed conformance corpus (DISTINCT from chipsalliance/sv-tests, not a fork): filenames encode the clause; per-file `// ! TYPE: POSITIVE\|NEGATIVE\|VARYING` answer keys; 362 IEEE 1364-2005 + ~904 IEEE 1800-2012; mind `VARYING` + the `KNOWN_TEXT_BUGS` list (`SV-CORPUS-GRAD.8` ADD-v1) |
| `iverilog` | github.com/steveicarus/iverilog | `a4989d02` | ⚠️ **GPL-2.0** (copyleft; test-input use only) | `ivtest/` | 3799 | Icarus Verilog regression suite (ivtest lives inside the iverilog repo since 2023; the standalone ivtest repo is obsolete): per-test JSON descriptors + `regress-sv.list` (992 SV entries, CE/EF/normal keys) + `regress-vlg.list` (the natural keyed IEEE 1364-2005 corpus) + `gold/` outputs (`SV-CORPUS-GRAD.8` ADD-v1) |
| `sv2v` | github.com/zachjs/sv2v | `6662fa5d` | **BSD-3-Clause** | `test/` | 953 | sv2v conversion test suite: core `.sv` + paired golden `.v` (Verilog-2005-compatible by contract) + `error/` negatives (pre-triage: conversion- vs parse-errors) + `lex/` (`SV-CORPUS-GRAD.8` ADD-v1) |
| `Surelog` | github.com/chipsalliance/Surelog | `d21c1c70` | **Apache-2.0** | `tests/` | 828 | Surelog (UHDM front-end) test corpus with golden logs (accept/error key extraction — never diff tool logs) (`SV-CORPUS-GRAD.8` ADD-v1) |
| `black-parrot` | github.com/black-parrot/black-parrot | `f91010f6` | **BSD-3-Clause** | `bp_be/ bp_common/ bp_fe/ bp_me/ bp_top/` | 205 | Real-design RISC-V multicore — macro-heavy idiom stress (bsg macro style). `external/` submodules (basejump_stl etc.) deliberately NOT initialized: separate upstreams (dedupe-by-true-upstream) + basejump's custom license (`SV-CORPUS-GRAD.8` ADD-v1) |
| `opentitan` | github.com/lowRISC/opentitan | `720d7242` | **Apache-2.0** | `hw/` | 3983 | OpenTitan silicon RoT — UVM-scale verification + design breadth (incl. vendored ibex, prim/tlul libraries); the best implicit preprocessor stressor (`SV-CORPUS-GRAD.8` ADD-v1) |

**ADD-v1 acquisition (`SV-CORPUS-GRAD.8`, 2026-07-22, director-ordered — [[project_sv_corpus_100pct_lrm_coverage_mandate]]):** the six rows above
were vendored with `git clone --depth 1 --filter=blob:none --sparse <url>` + `git sparse-checkout set <dirs>` +
`git submodule add` + `git submodule absorbgitdirs` (blob-filtered partial clones keep the heavy repos small on disk —
opentitan `hw/` = 237 MB instead of multi-GB). The already-vendored **uvm-core-2020.3.1** (`stimuli/sv/uvm/`, plain tracked
files, pre-submodule-pattern vintage) is folded into the bulk runner universe by `run_external_corpus.sh` as sub-corpus
`uvm-core` rather than re-vendored. Roster source: `docs/tasks/artifacts/corpus_grad_all/frozen_rosters_v1.md` (FROZEN v1).

**Pre-existing real-design SV corpora** (under `stimuli/sv/subs/`, added earlier, feed the curated
`sv_external_corpus_triage_gate` at 14/14): `Cores-VeeR-EL2`, `scr1`, `friscv`. These are
multi-file designs meant to be parsed as *preprocessed units* (with include/lib chaining), not
file-by-file — so the bulk `run_external_corpus.sh` runner under-reports them (it parses each
file in isolation). See `stimuli/sv/characterization/characterization.md`.

Re-acquire: `git submodule update --init --depth 1 stimuli/sv/subs/<name>` (sparse paths are
recorded in each submodule's `.git/info/sparse-checkout`).
