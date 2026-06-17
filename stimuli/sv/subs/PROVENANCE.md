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

**Pre-existing real-design SV corpora** (under `stimuli/sv/subs/`, added earlier, feed the curated
`sv_external_corpus_triage_gate` at 14/14): `Cores-VeeR-EL2`, `scr1`, `friscv`. These are
multi-file designs meant to be parsed as *preprocessed units* (with include/lib chaining), not
file-by-file — so the bulk `run_external_corpus.sh` runner under-reports them (it parses each
file in isolation). See `stimuli/sv/characterization/characterization.md`.

Re-acquire: `git submodule update --init --depth 1 stimuli/sv/subs/<name>` (sparse paths are
recorded in each submodule's `.git/info/sparse-checkout`).
