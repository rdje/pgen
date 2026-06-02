<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_always_signoff_decisions.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_always_signoff_decisions
description: "STANDING, emphatic (user 2026-05-31, after a scare): ALWAYS take signoff-level decisions — never guess code, never over-claim; when tooling is degraded or a result can't be verified, the signoff decision is to STOP + checkpoint, not proceed."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 5737d722-67a3-4fc9-8c42-f79e2ff1db07
---

User, 2026-05-31 (emphatic, repeated — triggered by my careless phrase "writing generator code … by guessing", which alarmed them that I might have guessed code; I had NOT — git proved zero `.rs`/generator changes): **"Always, always take signoff decisions, always."**

**Why:** signoff-level quality is non-negotiable (README CODE QUALITY mandate). Guessing at code — especially `stimuli_generator.rs` / engine / grammar — would risk the hard-won 14/14 SV corpus and the parser's correctness, and violates the project's deepest disciplines. Over-claiming a result (e.g. saying a gate is "green / exit 0" from an incomplete log read) is the same class of failure: asserting beyond what a tool actually showed. The `.5.2.5` slice did exactly that (claimed family-status "exit 0 / closed" when the gate had failed its final live-tracker check); `.6` corrected it transparently.

**How to apply:**
- Every code change: tools-first, root cause shown DIRECTLY by a tool (per [[feedback_tools_first_no_guessing]] + [[feedback_why_and_where_before_solution]]); regen + verify (corpus/gates/lib) BEFORE claiming success.
- Report ONLY what a tool actually output. Read the FINAL gate exit/banner, not just a sub-step tally. If output is truncated/mangled, re-read from a file (the Read tool renders files reliably even when Bash echo is mangling) before concluding.
- When tooling is degraded (output mangling/duplication), or you cannot reliably verify, the correct signoff decision is to **STOP and checkpoint at a clean committed state** — never plow ahead by guessing. Stopping under those conditions is the RIGHT call, not a failure.
- Generator/engine/grammar changes must be GENERAL/parser-agnostic ([[feedback_ast_pipeline_parser_agnostic]]) and task-tree-owned ([[feedback_task_tree_workflow]]) — extra reason never to rush them.
- Phrase carefully: don't describe "what I refused to do" in words that read as "what I did."

Reinforces [[feedback_tools_first_no_guessing]], [[feedback_correctness_before_speed]], [[feedback_report_expected_verify_against_oracle]]. Push pacing unchanged: ~30 commits or explicit request ([[feedback_push_pacing]]).
