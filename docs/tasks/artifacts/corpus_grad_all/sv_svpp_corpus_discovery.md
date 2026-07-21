# CORPUS-GRAD-ALL.1 — systemverilog + preprocessor + verilog_2005: exhaustive external-corpus discovery (2026-07-22)

Research-grounded discovery (web agent, primary sources; ~37 searches; sizes
API-verified 2026-07-22). Baseline vendored: sv-tests/slang/verible/verilator +
VeeR/scr1/friscv + uvm-core-2020.3.1 (the local sv-tests pin EXCLUDES its
third_party/ subtrees — no transitive coverage assumptions).

## Headline finds

1. ⭐ **ispras/sv-tests** (ISP RAS, BSD-3 — DISTINCT from chipsalliance/sv-tests,
   not a fork): 1,266 LRM-clause-keyed tests (filenames encode the clause;
   362 under ieee-1364-2005/, ~904 under ieee-1800-2012/) with per-file
   `// ! TYPE: POSITIVE|NEGATIVE|VARYING` answer keys + NEGATIVE_TEST ifdef
   variants. This IS the "IEEE LRM examples extraction" corpus — the exact
   instrument class VHDL lacks. Mind VARYING + its KNOWN_TEXT_BUGS file.
2. ⭐ **ivtest** (inside steveicarus/iverilog since 2023; the standalone repo is
   OBSOLETE): 3,799 .v with REAL answer keys — per-test JSON descriptors
   (`normal` / `CE` compile-error-expected / `EF`) + 759 gold outputs;
   `regress-sv.list` = 922 SV-2012 entries; `regress-vlg.list` = the natural
   IEEE 1364-2005 keyed corpus (as hypothesized). GPL-2.0 (fine as pinned
   test-stimuli submodule).
3. **No large dedicated open preprocessor torture suite exists** beyond
   Verilator's vendored t_preproc lineage — the real adds are hdlConvertor's
   sv_pp (IEEE 1800-2012 preprocessor-chapter examples page-by-page with
   golden expansions, MIT) + verilog-perl's preproc suite (the historical
   reference implementation's own goldens, Artistic-2.0) + Surelog's
   PreProc* dirs (Apache-2.0).

## Ranked ADDs — SV parser (1800-2017/2023)

1. ispras/sv-tests ieee-1800-2012/ (~904, clause-keyed, POSITIVE/NEGATIVE).
2. ivtest regress-sv.list subset (922, CE/EF/gold keys; vendor from
   steveicarus/iverilog master).
3. sv2v test/ (BSD-3): 253 core .sv + 287 golden .v pairs + 236 error/ (
   pre-triage: conversion-errors vs parse-errors) + 40 lex.
4. Surelog tests/ (839 files + 709 golden logs, Apache-2.0; extract
   accepts/reports-error as the key, don't diff tool logs).
5. Real-design bucket: OpenTitan (Apache-2.0, ≥1,264 .sv, transitively ibex +
   lowRISC prim/tlul — also the best implicit preprocessor stressor) +
   black-parrot (BSD-3, ~870, macro-heavy contrast); CVA6 optional.
6. CONDITIONAL (differential oracle only, do NOT vendor): gmlarumbe
   tree-sitter-systemverilog-test — 3,164 files WITH golden S-expression
   trees, but NO license on the test repo, ~1/3 overlap, tree shape != our AST.

## Ranked ADDs — verilog_2005 profile (1364-2005)

1. ivtest regress-vlg.list subset (keyed positives + CE negatives + gold).
2. ispras/sv-tests ieee-1364-2005/ (362 clause-keyed; VARYING = exactly the
   profile-boundary probes).
3. (free with sv2v) the 287 golden .v — Verilog-2005-compatible by contract.
4. CONDITIONAL: OSS-CVC tests_and_examples (≥414 simulator-grade .v; the
   OSS-CVC Modified Artistic license needs a read first); ispras/hdl-benchmarks
   (2,072 bulk netlists; mixed licenses, low diversity/byte).

## Ranked ADDs — systemverilog_preprocessor

1. hdlConvertor tests/sv_pp/ (MIT; 42 inputs / 23 golden expansions;
   2012_p641–p643 = the LRM preprocessor chapter page-by-page).
2. verilog-perl t/30_preproc* + t/80_vppreproc* (Artistic-2.0; 12 goldens;
   expansion-mode variants Verilator's copy does not replay).
3. Surelog PreProc*/Macro* dirs (free with SV add #4).
4. sv2v test/lex/ + test/define/ (free with SV add #3).

## Rejects with cause

yosys-tests (unlicensed, stale, generated); yosys tests/ (script-driven,
weak parse keys; asicworld subdir = unlicensed provenance); svlint/sv-parser/
grammars-v4/tree-sitter-verilog (small/low value); UHDM (no HDL); cocotb/mcy
(not corpora); VerilogEval (generated); skywater-pdk (generated netlists,
near-zero diversity); basejump_stl standalone (custom course license — its
idioms arrive via black-parrot); Surelog third_party/tests as-a-unit (prefer
true upstreams; avoids double-counting vendored copies); asic-world/
testbench.in (no license); verilog-mode tests (AUTO-centric); Verible
preprocessor tests (inline C++ strings, not extractable); OpenPiton (no
license + own PyHP preprocessing); ben-marshall/verilog-parser (archived,
asic-world provenance).

## Cross-cutting cautions

- License tiers: BSD/MIT/Apache/ISC clean; GPL-2.0 (ivtest) + Artistic-2.0
  (verilog-perl) fine as pinned submodules, keep out of redistributed bundles;
  NOASSERTION repos need a per-repo license read before roster entry.
- Dedupe by TRUE upstream, never via aggregates.
- 2017/2023-only constructs: ivtest/ispras stop at 2012 — vendored
  slang/sv-tests remain the ceiling there.

(Full agent report with the 33-candidate table preserved in the session task
output; this file is the durable roster-feeding extract.)
