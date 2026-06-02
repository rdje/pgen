<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_uvm_is_valid_sv.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_uvm_is_valid_sv
description: UVM packages are valid IEEE 1800 SystemVerilog; every commercial SV simulator parses them cleanly. A pgen parser failure on UVM is a parser defect, not an input defect; "would-be-fail-eventually" / "deep recursion exploring paths before failing" are misleading frames
metadata:
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User-set correction (2026-05-24, in response to my misleading framing of uvm_pkg timeouts as "fail-eventually"):** UVM is an Accellera-standardized open-source SystemVerilog library used by every commercial SV simulator (VCS, Xcelium, Questa, Verilator, Slang, …). The UVM source code IS valid IEEE 1800 SystemVerilog by construction.

**The discipline this enforces:**
- A pgen parser failure on UVM means **our parser has a defect**. The input is correct.
- A pgen parser TIMEOUT on UVM means **our parser has a defect that the engine's recursion-exploration is masking from us**. Worse than a fast-fail-with-wrong-error.
- Never report uvm-corpus outcomes as "deep-recursion exploring all paths before failure" — that frames the parser as correct-but-slow. The truthful frame: "parser rejects/hangs on known-valid input; root cause unknown."
- Conflating "parser is fundamentally working" with "parser hangs on the canonical SV stress test" is the kind of soft-framing the standing correctness-before-speed policy ([[feedback_correctness_before_speed]]) is meant to prevent.

**Cross-references:**
- Strengthens [[feedback_correctness_before_speed]] — corpus tally must report PASS / parser-bug-FAIL / parser-bug-TIMEOUT (not PASS/FAIL/timeout). The denominator is "valid SV files in the corpus"; everything not-PASS is a parser defect.
- Strengthens [[feedback_no_workarounds_fix_hierarchy]] — adding a `timeout 60` per-file bound to make the sweep finish is a workaround, not a fix. Track timeout as a defect category separate from but morally equivalent to FAIL.
- Sharpens [[feedback_recursion_ceiling_must_bound_real_stack]] — recursion-bounding mechanisms (universal memoization, ceiling guards) must NOT silently convert a parser defect into a clean timeout; they must surface that the parser is stuck and where.

**Concrete current corpus (as of 2026-05-24, post-Slice-57):**
- 16-file SV corpus, 60s per-file debug-build timeout
- 10/16 PASS — friscv ×4, scr1 ×4, veer_bootstrap_1 ×2
- 4/16 parser-bug FAIL — uvm_compat ×2, veer_el2_lsu_real ×2
- 2/16 parser-bug TIMEOUT — uvm_pkg ×{2017,2023}
- TARGET: 16/16 PASS. None of the not-PASS files are "expected to fail" — every one is a defect to track and close.
