<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_predicate_parse_order_sota.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_predicate_parse_order_sota
description: "⚠️ RETIRED 2026-05-25 — this memory captured a user mandate (\"make predicate-vs-parse-order SOTA\") that was triggered by a MIS-FRAMING of the implicit-type port-list defect. Direct trace evidence later showed the semantic store works correctly for this defect class; the persistence framing was a phantom. Kept as a retirement notice so the lesson survives."
metadata: 
  node_type: memory
  type: feedback
  status: retired
  retiredOn: 2026-05-25
  retiredSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

# ⚠️ RETIRED — DO NOT USE AS ACTIVE GUIDANCE

This memory previously named "Architecture B (universal rule-commit fact promotion)" as the SOTA fix for an inferred "facts emitted in a committed rule get rolled back by enclosing try_parse failure" defect. **The defect this memory described was inferred from symptoms, never directly traced.** The Architecture B implementation (Slice-68, 2026-05-25) was decisively disproven by a corpus regression sweep (10/4/2 → 0/4/12) and reverted in-slice.

A subsequent tools-first re-analysis of the canonical failing input (`function bit f(string a, b="");`) showed:

- `has_fact(kind=type_name, name=Identifier("b")) → false` at the moment of the parse failure (correctly absent).
- Every predicate-GATED type-identifier alternative (`known_unscoped_class_scope_class_identifier`, `..._interface_class`, `..._type_parameter`, `checked_type_identifier`, `..._covergroup`) correctly **rejected** `b`.
- The only rule that succeeded on `b` was `provisional_unscoped_block_class_type` — alt 12 of `data_type` — **which has no predicate at all** (an ungated catch-all).

There is no persistence problem visible in this input. The semantic store works. The bug is purely structural: one alternative in `data_type` doesn't consult the fact store.

## Why this memory is retired, not deleted

- Preserves the lesson: a user mandate inherits the framing of the diagnosis that motivates it; if the diagnosis was wrong, the mandate is solving a phantom. Future sessions should be alert to this.
- The Architecture B disproof + decisive-baseline procedure (stash + rebuild + re-measure) remain useful patterns — they live in `[[feedback_prove_independence_with_decisive_baseline]]` and `[[feedback_tools_first_no_guessing]]` (2026-05-25 amendment).
- Outright deletion would lose the visible retirement signal and risk a future session re-deriving Architecture B from scratch.

## What this means for adjacent work

- **C3-B (Slice-67, committed 2026-05-25)** was designed as a per-branch tournament isolation fix for the same persistence framing. Its motivating input was never traced to demonstrate branch-loser fact leakage on real grammar. **C3-B's premise is suspect by the same evidence that retires this memory.** Whether to revert C3-B depends on running the trace on whatever input it was claimed to fix; if no such trace exists or none can be produced, C3-B is defensive engineering on top of an unverified hypothesis and should be revisited.
- **C3-A (`[[feedback_try_parse_must_snapshot_semantic_state]]`)** is a more defensible invariant in principle (speculation-only emissions shouldn't leak into committed state) but has the same documentation gap: no captured trace on a real failing input. Mark for re-verification, not retirement.
- **Task #70 (`.b.6.2.34`)** — "Architecture B′" — should also be retired. There is no demonstrated defect requiring targeted fact promotion. The targeted L1 grammar fix for the actual implicit-type port-list defect is: gate `provisional_unscoped_block_class_type` consistently with its gated siblings (the same intent as the original Slice-64).

## Original mandate (for historical reference)

> User-set 2026-05-25: "We need to make this 'how the predicate gate interacts with the parse-order' SOTA, signoff, top-notch!"

This was a real user statement, but it was triggered by my mis-framing of why Slice-64's predicate gate caused a uvm regression. The regression was attributed to "fact persistence" without a trace ever showing a committed fact being rolled back. The trace on the canonical minimal repro now shows the fact is correctly absent, the gates correctly fire, and one ungated alternative is the sole defect. The "predicate-vs-parse-order" problem this memory named may not exist at all for the inputs we have evidence for.

**Lesson durable enough to act on:** when a user mandate uses the framing words I just used to describe a diagnosis, the mandate is only as good as the diagnosis. If the diagnosis was speculative, the mandate inherits the speculation. Verify the diagnosis with tools BEFORE soliciting (or accepting) a mandate framed around it.

**Cross-references (still active):**
- `[[feedback_tools_first_no_guessing]]` — the discipline that would have prevented this entire detour
- `[[feedback_prove_independence_with_decisive_baseline]]` — the procedure that finally caught Architecture B
- `[[feedback_user_is_director_not_engineer]]` — my framing seeds the user's mandates; my responsibility to frame honestly
- `[[feedback_no_workarounds_fix_hierarchy]]` — L1 grammar gate was always the right answer for this defect
