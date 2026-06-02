<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/project_sv_h2_builtin_classes.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: project-sv-h2-builtin-classes
description: SV-EXH-PROOF .b.6.2.37 H2 umbrella — built-in language types (IEEE 1800 §G.2 std::) are the uvm_pkg blocker; design locked at Level 2 (auto-load parser_libs/sv_*_std/ artifact)
metadata: 
  node_type: memory
  type: project
  originSessionId: 5737d722-67a3-4fc9-8c42-f79e2ff1db07
---

**Fact:** the SV external corpus residual 4-fail is two distinct defect classes, not one. **`.b.6.2.37` (H2) — built-in language types** targets uvm_pkg ×{2017,2023} (118 process/semaphore/mailbox uses each per LRM §G.2). The prior `.b.6.2.X (H1)` framing is **refined to "cross-package extends chain"** and now targets uvm_compat_pkg ×{2017,2023} specifically (zero process/semaphore/mailbox uses; fails at `class uvm_compat_packer extends uvm_pkg::uvm_packer;` line 3977).

**Why:** tools-first investigation in `.b.6.2.37.0` (`PGEN-SV-EXH-PROOF-0087`, 2026-05-25) ran minimal repros + grep counts on the actual preprocessed corpus files. uvm_pkg minimal repro `module m; function void f(); process p; endfunction endmodule` FAILS at byte 0 [furthest 42] but PASSES with `typedef class process;` preamble — pinning the built-in-class class. uvm_compat_pkg uses `extends uvm_pkg::uvm_packer` cross-package — entirely different mechanism. The prior framing conflated them.

**How to apply:**
- H2 (this umbrella) lands FIRST. uvm_pkg → PASS → 10/14 → 12/14 + library producible.
- H1 (refined) lands SECOND, dependent on H2 (needs uvm_pkg's library output as a bootstrap_file for uvm_compat_pkg). Mirrors veer_el2_lsu's existing `bootstrap_files` pattern. → 14/14.
- Design locked at **Level 2** (auto-load `parser_libs/sv_2017_std/` + `parser_libs/sv_2023_std/`, reuses `.3.3.4.a` infra). Per user direction "keep all 3 alternatives, pick one, roll with it": Level 1 (preprocessor preamble injection) and Level 3 (new `@bootstrap_facts` directive) are documented FALLBACKS in the umbrella's design-alternatives section — adopt only if Level 2 hits an unexpected snag.
- **Scope finding (verified `.37.0`)**: lib artifact only needs `type_name` facts — NOT method/enum-value enumerations. SV parser accepts `Class::ident(args?)` / `obj.method(args?)` universally for any known type. Initial artifact = 3 entries (process / semaphore / mailbox).
- Repros at `/tmp/pgen-bug/h2_*.sv` and `/tmp/pgen-bug/uvm_h2_method{1,2,3}.sv`.

Cross-refs: [[feedback_grammar_rules_must_consult_store]] (store IS the right mechanism; pre-seed it with what LRM says is always known), [[feedback_why_and_where_before_solution]] (the tools-first discipline that made the H1→H2 reframing decisive), [[feedback_no_workarounds_fix_hierarchy]] (Level 2 chosen because it reuses existing infra; Level 1 rejected as workaround).
