---
name: reference-sv-external-corpus-and-oracle-repos
description: REFERENCE (director 2026-06-09) — canonical list of the official / recognized SystemVerilog external test-corpus + reference-tooling GitHub repos for PGEN's SV differential and external-corpus proof — chipsalliance/sv-tests (the SV external CORPUS), MikePopoloski/slang + chipsalliance/verible + verilator/verilator + chipsalliance/Surelog (parser/compiler reference ORACLES), chipsalliance/UHDM (reference elaborated-design data model). SUPERSEDED-IN-PART 2026-06-17: the CORPUS arm IS acquired — sv-tests/verible/slang/verilator vendored as pinned sparse submodules under stimuli/sv/subs/ (PROVENANCE.md; EXTERNAL-CORPUS.3.1, PGEN-EXTERNAL-CORPUS-0007) with runner stimuli/run_external_corpus.sh + characterization; the ORACLE arm (building slang/verible as reference binaries) remains future PARSE-COMPLETENESS work. 2026-07-22: corpus graduation is now a mandatory SV Done axis (tree SV-CORPUS-GRAD, [[project_sv_done_requires_external_corpus_graduation]]).
metadata:
  node_type: memory
  type: reference
  director_directive: true
  created: 2026-06-09
---

**THE SHARE (director, 2026-06-09).** The canonical list of official / recognized SystemVerilog external
test-corpus and reference-tooling GitHub repos, with the director's acquisition recipe (git submodules under
`third_party/sv/`). Recorded so the EXTERNAL-CORPUS / PARSE-COMPLETENESS / SVPP-EXPANSION work does not have
to re-derive it.

## The repos and their role for PGEN

| Repo | URL | Role for PGEN |
|---|---|---|
| `sv-tests` | https://github.com/chipsalliance/sv-tests | **The SV external CORPUS** — CHIPS Alliance SystemVerilog test suite. The SV analogue of `regex_corpus_bundle/` (PCRE2) and `json_corpus_bundle/` (JSONTestSuite) under the external-corpus doctrine: an independent, officially-recognized corpus to characterize the SV parser against (alongside the internal stimuli generator). |
| `slang` | https://github.com/MikePopoloski/slang | Reference **ORACLE** — a fast, standards-tracking SV parser/compiler. A differential oracle for PARSE-COMPLETENESS (`reference_passed ∧ rust_failed == 0`). Its preprocessor is a token-stream reference for SVPP-EXPANSION.1. |
| `verible` | https://github.com/chipsalliance/verible | Reference **ORACLE** — Google/CHIPS Alliance SV parser + linter + style tools. Differential oracle; token-stream preprocessor reference. |
| `verilator` | https://github.com/verilator/verilator | Reference **ORACLE** — the widely-used SV simulator/compiler. Differential oracle; token-stream preprocessor reference. |
| `Surelog` | https://github.com/chipsalliance/Surelog | Reference **ORACLE** — SV 2017 parser/elaborator (emits UHDM). Differential oracle. |
| `UHDM` | https://github.com/chipsalliance/UHDM | Reference **elaborated-design data model** (Universal Hardware Data Model) — relevant to the compiler/elaborator-enablement lane (a model of what a fully-elaborated SV design surface looks like). |

## Acquisition recipe (director-provided; NOT yet run)

```bash
#!/usr/bin/env bash
set -euo pipefail
BASE_DIR="third_party/sv"
mkdir -p "$BASE_DIR"; cd "$BASE_DIR"
git submodule add https://github.com/MikePopoloski/slang slang
git submodule add https://github.com/chipsalliance/sv-tests sv-tests
git submodule add https://github.com/chipsalliance/verible verible
git submodule add https://github.com/verilator/verilator verilator
git submodule add https://github.com/chipsalliance/Surelog surelog
git submodule add https://github.com/chipsalliance/UHDM uhdm
# git submodule update --init --recursive
```

## Where each slots into PGEN's tracked work

- **EXTERNAL-CORPUS** (`docs/tasks/EXTERNAL-CORPUS.md`): `sv-tests` is the SV external corpus — a `.3`-class
  item (other-grammar external corpora), currently not blocking the locked program (all-parsers-Done /
  cert-coverage clean + UNKNOWN=0). Characterize, don't game (the SV grammar is a real full-SV grammar, so
  divergences are genuine parser gaps, unlike the deliberately-simplified `json.ebnf`).
- **PARSE-COMPLETENESS** (`docs/tasks/PARSE-COMPLETENESS.md`): `slang` / `verible` / `verilator` / `Surelog`
  are the differential oracles for pillar A ("never wrongly REJECTS valid input"); the tree's `.2` already
  names them and is director-gated on having reference tooling available — this record is that tooling list.
- **SVPP-EXPANSION** (`docs/tasks/SVPP-EXPANSION.md`): `slang` / `verible` / `verilator` preprocessors are
  the token-stream reference implementations the `.1` SCOPING reads — now to CONFIRM/detail the already-decided
  parse-tree architecture (per [[project_svpp_expansion_stage_for_nexsim]]), not to re-litigate it.
- **COMPILER-ELABORATOR** (`docs/tasks/COMPILER-ELABORATOR.md`): `UHDM` is a reference elaborated-design model.

## Reproducibility caveat (decide at acquisition time)

PGEN's existing external-corpus bundles vendor an **immutable upstream snapshot** under `third_party/upstream/`
(separate from normalized corpus/oracle outputs) so the corpus + gate are reproducible. The recipe above uses
live git submodules (which track upstream HEAD). When this is acquired, reconcile the two: either **pin the
submodule commits** or vendor immutable snapshots per the `regex_corpus_bundle/` + `json_corpus_bundle/`
pattern, so the SV corpus/oracle results are deterministic and gate-stable. (A starting recipe, not a final
acquisition policy.)

## Status

**CORPORA ACQUIRED 2026-06-17** (`PGEN-EXTERNAL-CORPUS-0007`, EXTERNAL-CORPUS.3.1/.3.2) — director directive:
"maximum number of SV and VHDL stress test possible ... test corpora only, not the code." The **test CORPORA**
(not the oracle tools) were submoduled under the existing `stimuli/{sv,vhdl}/subs/` convention (NOT `third_party/`
— the repo already vendors corpus submodules there), shallow `--depth 1` + sparse-checkout to test dirs, pinned,
with per-family `stimuli/{sv,vhdl}/subs/PROVENANCE.md` (license flags). SV: sv-tests/verible/slang/verilator
(~5128 files). VHDL: ghdl/nvc/OsvvmLibraries/vunit/UVVM (+ pre-existing PoC/Compliance-Tests/… ⇒ 13,720 files).
Director confirmed **GPL-OK** (submodule = reference, not copied code; parser-test-input use ⇒ no copyleft impact
on PGEN). The **ORACLE tools** (slang/verible/verilator/Surelog as differential oracles, UHDM) were NOT
acquired/built — that stays the separate PARSE-COMPLETENESS lane. Characterization: `stimuli/{sv,vhdl}/characterization/`.

## Disciplines

[[feedback_corpus_expected_from_spec_not_fix]] (derive expecteds from the spec/oracle, not the fix),
[[feedback_report_expected_verify_against_oracle]] (run the authoritative executable oracle), and the
external-corpus doctrine [[project_external_corpus_doctrine]].
