---
name: project_sv_speed_on_par_with_slang_and_verilator
description: "DIRECTOR DIRECTIVE (2026-10-18) — once the SystemVerilog parser is released to Nexsim, the next SV campaign is SPEED: the preprocess → parse pipeline runs as fast as it possibly can, on par with — or beating — slang and Verilator, then beyond (no commercial tool is accessible, so these two are the reference). Sequenced after the release; correctness is the floor. Owning tree SV-SPEED."
metadata:
  node_type: memory
  type: project
---

# SystemVerilog speed: on par with — or beating — slang and Verilator, then beyond (director, 2026-10-18)

**Status:** ACTIVE directive, SEQUENCED — the campaign starts when the SystemVerilog parser is released to Nexsim.
Owning tree: `SV-SPEED` ([docs/tasks/SV-SPEED.md](../tasks/SV-SPEED.md)).

## Context

The directive arrived during release `1.0.8` of the `systemverilog_preprocessor` parser (`SV-CORPUS-GRAD.13c.2v.2`),
right after the director had confirmed that `uvm_pkg.sv` is parsed end to end by the preprocess → parse pipeline.

## Decision (the director's words, 2026-10-18)

> After the SV parser is released to NEXSIM we will focus on speed and the SystemVerilog run as fast as it possibly
> can, as we need to be on par with commercial parsers and be lighting fast.

> I can't access commercial tool, so we will have to be on par or even beat slang and Verilator and go beyond.

## Consequences

- **Sequence.** The SV lane lock holds ([[project_nexsim_sv_signoff_delivery_focus]]): no speed lever lands before the
  Nexsim release. Until then the campaign exists as this record and its tree, `proposed` and sequenced after the release.
- **The reference tools.** Commercial tools are out of reach, so the bar is set by the two leading open-source front
  ends:
  - **slang** at the revision this repository already vendors, `stimuli/sv/subs/slang` @ `4106501b`. The submodule is a
    SPARSE checkout (`tests/` only), but its object store holds the whole tree at that revision (733 objects, 0
    missing, measured 2026-10-18); the benchmark leaf exports it read-only (`git archive`, lazy fetch disabled) to a
    directory under `rust/target/` and builds it there — no byte is
    written inside the submodule, which stays a read-only black box like every repository other than this one
    ([[reference-sv-external-corpus-and-oracle-repos]]). Its CMake build fetches
    `fmt` (and, by option, `mimalloc`) through `FetchContent` into that build directory. Verified in its driver source
    (`tools/driver/slang_main.cpp`): `-E,--preprocess` (*"Only run the preprocessor"*) and `--parse-only` (*"Stop after
    parsing input files, don't perform elaboration or type checking"*).
  - **Verilator 5.050** (host install, `verilator --version` → `5.050 2026-07-01`; the vendored
    `stimuli/sv/subs/verilator` @ `a534a1d1` is a complete store too, so the benchmark leaf may pin that revision
    instead). It has `-E` (*"Preprocess, but do
    not compile"*) and `--lint-only`; it has no parse-only mode, so its numbers are recorded per mode and never
    passed off as a parse-only time.
- **The bar.** For the same work on the same host — each tool's narrowest mode that does at least what PGEN's pipeline
  does — PGEN is at or below the faster reference's time, per file and in aggregate, and then below it. Measured as a
  RATIO on one host (wall time and peak RSS per file), never as an absolute second count, which would go green on a
  faster machine without the parser moving ([[derive-the-bar-from-the-subject-when-no-authority-owns-the-number]]).
  The first leaf fixes the protocol — the modes, the population, how include directories and defines reach each tool,
  allocator parity — and every published number names its mode and its binary.
- **"Go beyond."** Parity is a milestone, not the end: once the ratio is at or below 1, the leaf that reaches it opens
  the next bar.
- **The floor.** [[feedback_correctness_before_speed]], hardened for a speed phase: a lever that moves any verdict, any
  tree, any certificate, any manifest or any pin does not land. The instruments are the ones the SV identity re-key
  already uses — `REKEY-MOVERS`, `AST-IDENTITY-SWEEP`, the adjudication manifests byte-identical, the certificate
  sweep — run before and after each lever.
- **Root cause by profiler.** A slowness is diagnosed with self-time / call-graph attribution
  ([[project-speed-phase-profiler-root-cause-signature]]); `RGX-0078` (the regex speed campaign) is the template.
