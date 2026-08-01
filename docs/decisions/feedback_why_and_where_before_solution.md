---
name: feedback-why-and-where-before-solution
description: "STANDING DISCIPLINE (user-set 2026-05-25, in this same session as `.b.6.2.35.1` / `.b.6.2.36.1` / `.b.6.2.36.2`): for ANY issue or unexpected behavior, we MUST know EXACTLY why it happens AND inside which function/rule it happened. Without that pair of facts (the WHY and the WHO), we cannot devise a stable solution. If the current tooling can't surface both, BUILD the tool first."
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
id: feedback-why-and-where-before-solution
title: Know the WHY and the WHERE before designing any fix — if the tooling cannot show both, BUILD the tool
date: 2026-05-25
answers:
  - "how much do I need to understand before proposing a fix"
  - "what are the two facts required before a solution is designed"
  - "the existing tools cannot show me the root cause — what now"
  - "why is my proposed fix being rejected as premature"
reverify: grep -n 'ROOT CAUSE (WHY + WHERE)' TOOLBOX.md | head -3
---

<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_why_and_where_before_solution.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->
**STANDING DISCIPLINE (user, 2026-05-25):** For any issue or unexpected behavior, we need to know EXACTLY **why** it happens and **inside which function/rule** it happened. Without such information we cannot devise a stable solution. If the current tooling can't produce both data points concretely, the next slice is a TOOL-BUILD slice — not a speculative fix attempt.

**Why:** speculation-based fixes have repeatedly led to disproven hypotheses (Slice-67's C3-B was built on a mis-framed persistence story; Slice-68's Architecture B was disproven on the corpus 10/4/2 → 0/4/12). Each speculation cost a full slice, an investigation, and (in Slice-68's case) a revert. The cost of building a TOOL to answer "why + where" concretely is much lower than the cost of one wrong-direction fix slice.

**How to apply:**

1. When a parse fails or behaves unexpectedly, BEFORE designing a fix, ask:
   - **WHY:** what's the concrete cause? (specific rule rejection, specific predicate eval, specific keyword mismatch, specific lookup miss)
   - **WHERE:** which function/rule fired the failure? (full rule_stack at the failure point; for state-store ops, which rule's transaction / try_parse triggered the rollback)

2. If the current tooling can produce both data points: proceed to fix design.

3. If the current tooling produces only one (or neither): BUILD THE TOOL FIRST. The tool-build is its own task-tree leaf, its own slice, its own commit. The fix slice is a separate, follow-up slice that uses the new tool.

4. Examples of "data point" the tools must produce on demand:
   - Which rule/function originated a specific `rollback_to` event in the semantic-runtime trace
   - Which `try_parse` boundary owns a specific position-rollback
   - Which predicate evaluation returned false (and against what state)
   - Which branch of which tournament won/lost

**Existing tooling (as of `.b.6.2.36.2`):**
- `rule_stack: [...]` on every parse event — answers WHERE for parse-time failures
- `--trace-rules <list>` filter — scopes trace to specific rules
- `furthest_position` engine (Slice-59) — surfaces the deepest position any speculation reached on parse failure
- `--dump-rule-call-counts` — per-rule frequency dashboard
- `emit_fact` / `has_fact` / `rollback_to` / `apply_delta` HIGH-level self-explaining events

**Known gap (motivating `.b.6.2.36.2`):**
- `rollback_to` events DO NOT include the `rule_name` of the failing transaction. When the IIFE-wrapped per-rule transaction (`.3.3.3`'s `with_semantic_runtime_rule_transaction` ) or the `SemanticRuntimeTransaction::drop()` Drop-rollback fires, the trace event has no caller identification. This makes class-of-defect diagnosis (e.g. the `.b.6.2.36` class-scoped type-parameter persistence) impossible to pin without speculation. `.b.6.2.36.2` builds this tool.

**Anti-pattern:**
- Designing a fix from "X is being rolled back, therefore add a `persistent: true` flag to X" — without knowing which rule's try_parse fires the rollback. The fix might be:
  - Wrong (the rollback might be legit and the actual defect is elsewhere)
  - Right but for the wrong reason (creating future drift)
  - Right but too narrow / too broad (missing structurally identical instances or affecting unrelated ones)

**Cross-references:** [[feedback_tools_first_no_guessing]] (this principle's general form); [[feedback_grammar_rules_must_consult_store]] (the principle that motivated `.b.6.2.35.x`); [[feedback_user_is_director_not_engineer]] (user makes principle decisions; we make implementation decisions WITHIN principles + tools).
