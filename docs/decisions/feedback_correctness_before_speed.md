<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_correctness_before_speed.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_correctness_before_speed
description: STANDING POLICY — correctness comes BEFORE speed; only after a parser fully and accurately parses its corpus without errors do we optimize; applies to every parser the pgen engine builds
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User-set policy (2026-05-24, emphatic + repeated):** correctness first, speed second. Universal. Applies to every parser pgen generates.

**Frame of reference:**
- Commercial SV compilers parse `uvm_pkg.sv` (3MB, ~90K preprocessed lines) in **<1 second**.
- A chip design contains **hundreds to thousands** of SV/VHDL/Verilog files; full compilation must still be fast (commercial tools do whole-design elaboration in seconds-to-minutes).
- Our current state: uvm_pkg takes 99s and rejects with a real grammar error. That is **NOT "fast"** — it's "not catastrophic-backtracking anymore."

**The discipline:**
1. **Get to fully-clean parse FIRST.** Every file in the corpus (uvm, uvm_compat, veer, friscv, scr1, …) accepted without errors. Surface-position triage at PASS=14/14, not 10/14.
2. **Only THEN look at speed.** Profile, identify hot paths, optimize.
3. **Don't conflate "no longer hanging" with "fast".** A 99s parse that completes is a correctness step (no more catastrophic backtracking), NOT a speed achievement.

**Why this order:**
- Optimizing a buggy parser bakes in the bugs.
- A wrong-fast parser is worse than a slow-correct one (users get wrong AST silently).
- Speed-first work on grammars/engines that aren't yet correct introduces premature couplings that block later correctness fixes.

**How to apply:**
- When reporting progress, distinguish "advanced N more lines" (correctness progress) from "got faster" (speed progress).
- When proposing engine fixes, document whether they target CORRECTNESS (e.g. fixing a parse failure) or PERFORMANCE (e.g. better caching). Correctness fixes are always-on; perf fixes wait until the campaign clears.
- Universal Packrat memoization (`.b.6.2.15`) is unusual — it was BOTH a correctness fix (eliminated catastrophic backtracking that masquerades as failure) AND a perf fix (true O(N)). Both motivations justified.
- Future tooling proposals (`--trace-rules`, AST-aware bisection, etc.) are CORRECTNESS-enabling — they help find the next bug. They're in-scope during the correctness phase.

**Cross-references:** strengthens [[feedback_no_workarounds_fix_hierarchy]] (no quick-and-dirty fixes that trade correctness for any reason); pairs with [[feedback_post_campaign_audit.md]] (defer shape-correctness audits until the campaign closes; same shape — defer perf until correctness closes).

**Concrete current state (as of 2026-05-24):**
- SV corpus: 10/14 pass; uvm_pkg ×{2017,2023} + uvm_compat_pkg ×{2017,2023} fail.
- Cumulative uvm_pkg deep parse advance this session: 5521 → 19378 (~15% of 90K).
- Speed is NOT YET a concern. Get the remaining ~85% of uvm_pkg parsed correctly first.
