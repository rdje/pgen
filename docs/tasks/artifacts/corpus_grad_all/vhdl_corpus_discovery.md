# CORPUS-GRAD-ALL.1 — vhdl family: exhaustive external-corpus discovery (2026-07-22)

Research-grounded discovery (web agent, primary sources; ~48 searches/fetches;
file counts API-verified 2026-07-22). Baseline vendored: ghdl testsuite (incl.
VESTS 4,317 + gna 3,130), nvc, OsvvmLibraries, UVVM, VUnit, PoC,
Compliance-Tests, Interfaces, neorv32, Rudi-RV32I.

## Headline answers (the specific open questions)

1. **VESTS: NO maintained upstream newer than ghdl's vendored copy exists.**
   Original upstream dead; nickg/vests = a frozen 2002 CVS mirror; ghdl's
   vendored copy is 2 files AHEAD (fixes) — the de-facto maintained VESTS.
   Nothing to add.
2. **IEEE 1076 LRM example extraction: no public project exists** (IEEE
   copyright). The ONLY official machine-readable IEEE artifact =
   **VASG Packages** (opensource.ieee.org/vasg/Packages, Apache-2.0, 65
   files) — the canonical normative VHDL-2019 texts = the MUST-parse floor
   for 2019 mode.
3. **Sigasi: no public parser corpus** (closed-source; vhdl2008-tester = 12
   unlicensed frozen files, superseded by vendored Compliance-Tests). Reject.

## Ranked ADD recommendations

1. GRLIB GPL via TUT-ASI/leon3-grlib-gpl-mirror (1,168 files; auto-refreshed
   from official Gaisler releases): the most idiom-dense industrial VHDL
   (two-process/record-port/configurations/generate-heavy).
2. slaclab/surf (1,155; SLAC BSD-style; active): largest active permissive
   VHDL-2008 library — production-scale 2008 features.
3. vhdl-style-guide tests/ (2,692 tiny per-construct files, GPL-3; paired
   input/.fixed oracle; per-construct dir naming = free coverage bucketing;
   curate out its ieee dupes).
4. IEEE VASG Packages (65, Apache-2.0): normative 2019 must-parse floor —
   the closest legal thing to "LRM examples".
5. antonblanchard/microwatt (112, CC-BY-4.0): the reference open VHDL-2008 CPU.
6. CERN general-cores (269, LGPL/OHL/Solderpad per-file): third industrial dialect.
7. open-logic (211, permissive, active 2026): modern 2008 + VUnit testbench idioms.
8. vhdl-linter test/ (505, GPL-3): ⭐ the ONLY new corpus with NEGATIVE cases
   (deliberate-error files) — needs a one-time TS-spec -> valid/invalid
   manifest mapping.
9. bpadalino/vhdl-ideas (44, MIT; P1076 participant): aggressive VHDL-2019 stress.
10. cad-polito-it/I99T (22, EUPL-1.2): the recognized ITC'99 benchmark set.
11. (top-ups) hdlConvertor tests/vhdl (77, MIT) + vhdl-extras (79, MIT).

Conditional: fabriziotappero/ip-cores OpenCores mirror (~10k+ files, messy
real-world 87/93) — branch-per-core defeats a submodule pin; per-core license
audit needed; only as a scripted curated snapshot if maximal fuzz coverage is
later wanted.

## Rejects with cause

nickg/vests (strict subset of vendored — verified); ieee-sa GitHub packages
(frozen dup of VASG); FPHDL (unlicensed dup); rust_hdl (tests are inline
strings; libs are dups except ~10 VITAL2000/synopsys files); grammars-v4 vhdl
(IEEE dups, no tree/error keys); tree-sitter-vhdl ×2 (tiny; expectations bound
to a simplified non-conforming grammar); Sigasi; zamiaCAD (stale GRLIB dup);
Xilinx VHDL unisim + Intel sim libs (redistribution-restricted; open Xilinx
copies are Verilog-only); HuggingFace scraped sets (unvetted licenses, no
ground truth); Hamburg archive + FreeHDL (dead, not pinnable);
pyVHDLParser/pyVHDLModel/VHDL-Tool-Checks (negligible).

## ⚠️ Answer-key reality (roster design input)

Among ALL findable external VHDL corpora, only vendored VESTS
(compliant/non_compliant + .exp), vendored ghdl-gna/nvc regressions, and
vhdl-linter's spec-keyed fixtures carry machine-readable expected-fail
classification — every other suite is POSITIVE-ONLY (must-parse). VHDL
rejects-valid/over-accept coverage therefore rests on the vendored
non_compliant sets + VSG/vhdl-linter + generated mutations.

## Key primary sources

github.com/nickg/vests; ghdl testsuite/vests; ieee-p1076.gitlab.io;
opensource.ieee.org/vasg/Packages; github.com/TUT-ASI/leon3-grlib-gpl-mirror;
gaisler.com/grlib-ip-library; github.com/slaclab/surf;
github.com/jeremiah-c-leary/vhdl-style-guide; github.com/antonblanchard/microwatt;
gitlab.com/ohwr/project/general-cores; github.com/open-logic/open-logic;
github.com/vhdl-linter/vhdl-linter; github.com/bpadalino/vhdl-ideas;
github.com/cad-polito-it/I99T.

(Full agent report with the 28-candidate table preserved in the session task
output; this file is the durable roster-feeding extract.)
