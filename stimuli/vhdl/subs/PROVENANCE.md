# VHDL external test-corpus provenance (EXTERNAL-CORPUS.3.2)

External VHDL **test corpora** vendored as **git submodules** (pinned commits) and
sparse-checked-out to their test directories only ("test corpora, not the tool code").
Used **only as parser test inputs** by `stimuli/run_external_corpus.sh vhdl` — never compiled,
linked, modified, or redistributed.

> **License note (director decision 2026-06-17):** a git submodule stores only a *pointer*
> (upstream URL + pinned commit SHA); the upstream code is **not copied into PGEN** and does
> **not** change PGEN's own license. Using these files as parser test inputs does not create a
> derivative work, so no copyleft (GPL/LGPL) obligation attaches to PGEN. The director
> explicitly accepted GPL corpora to maximise the VHDL stress-test count. Copyleft repos are
> flagged ⚠️ below. (Not legal advice; standard understanding for test-input data + submodule
> references.)

## Big test-suites added 2026-06-17 (EXTERNAL-CORPUS.3.2)

| Submodule | Upstream | Pinned commit | License | Sparse path | ~VHDL files | Role |
|---|---|---|---|---|---|---|
| `ghdl` | github.com/ghdl/ghdl | `8bc05db0` | ⚠️ **GPL-2.0** | `testsuite/` | 9758 | The single largest/most authoritative open VHDL regression corpus (incl. **VESTS** 4317 + `gna` regressions 3130 + `synth` 2198) |
| `nvc` | github.com/nickg/nvc | `adc497f4` | ⚠️ **GPL-3.0** | `test/` | 2074 | Modern VHDL-2008/2019 sim; dedicated `parse`/`sem` dirs (excellent parser stress) |
| `OsvvmLibraries` | github.com/OSVVM/OsvvmLibraries | `24949594` | **Apache-2.0** | (recursive nested submodules) | 715 | OSVVM verification methodology libraries |
| `vunit` | github.com/VUnit/vunit | `edddc0fd` | **MPL-2.0** | `vunit/vhdl/`, `examples/vhdl/`, `tests/acceptance/` | 294 | VUnit framework VHDL test/example sources |
| `UVVM` | github.com/UVVM/UVVM | `90d56e93` | **Apache-2.0** | (whole, shallow) | 302 | UVVM (Bitvis) verification methodology VHDL |

## Pre-existing real-design VHDL corpora (added earlier; feed the curated vhdl corpus)

`PoC` (VHDL/PoC, Apache-2.0, 371) · `Compliance-Tests` (VHDL/Compliance-Tests, Apache-2.0, 78 —
VHDL-2008/2019 conformance) · `Interfaces` (VHDL/Interfaces, 30) · `neorv32` (stnolting/neorv32, 69) ·
`Rudi-RV32I` (hamsternz/Rudi-RV32I, 29).

**Total VHDL stress-test files ≈ 13,720.** Characterization (raw parse-pass/fail per sub-corpus):
`stimuli/vhdl/characterization/characterization.md`. Note: GHDL `gna`/VESTS `non_compliant` files are
INTENTIONALLY invalid, so a parse-FAIL there is frequently the correct outcome — characterize, don't game.

Re-acquire: `git submodule update --init --depth 1 [--recursive] stimuli/vhdl/subs/<name>`.
