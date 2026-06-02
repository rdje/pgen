<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_try_parse_must_snapshot_semantic_state.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_try_parse_must_snapshot_semantic_state
description: "Engine invariant — try_parse (the per-speculation wrapper used by branch alternatives, optional groups, quantifier iterations, lookahead) MUST snapshot+restore self.semantic_runtime_state alongside position; failure to do so leaks emit_fact/scope side effects of failed speculations and silently corrupts downstream predicate queries"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**The hazard (SV-EXH-PROOF.3.3.4.b.6.2.7 / C3-A, 2026-05-23).** The universal `try_parse` wrapper in the generated parser (`rust/src/ast_pipeline/ast_based_generator.rs`) is the single chokepoint for every PEG speculation: every `|` branch attempt in `generate_or_logic`, every `( X )?` optional group, every `*` / `+` / `{N,M}` iteration, every `&X` / `!X` lookahead funnels through it. Before the fix it snapshot only `self.position` + `self.recursion_guard.parse_stack.len()` and on failure rolled back only those — `self.semantic_runtime_state` was NEVER snapshotted at the speculative-attempt boundary.

So when a speculative branch entered a fact-emitting child rule (whose own `with_semantic_runtime_rule_transaction` *committed* the fact on the child's local success), and the OUTER speculation later failed (e.g. SV's `type_declaration_sv_2017` branch 6 matches `kw_typedef` + empty optional + `declared_type_identifier=TYPE` on `typedef TYPE T;`, emits `TYPE=typedef`, then fails on `semi` because the next token is ` T` not `;`), `try_parse` rolled back position but the leaked fact persisted permanently in the parent's committed state. Downstream `lacks_fact_attribute_equals(declaration_family,typedef)` predicates then incorrectly rejected `TYPE` (which actually has family `type_parameter`), corrupting the parse — the precise mechanism behind the `.b.6.2.6` minimal 7-line repro.

**The invariant.** Every per-speculation boundary in the engine must be transactional in ALL parser state, not just position. The fix is the irreducible primitive:
```rust
let saved_pos = self.position;
let saved_stack_len = self.recursion_guard.parse_stack.len();
let saved_semantic_checkpoint = self.semantic_runtime_state.checkpoint(); // ← THE FIX
match f(self) {
    Ok(result) => Some(result),
    Err(e) => {
        self.position = saved_pos;
        self.recursion_guard.parse_stack.truncate(saved_stack_len);
        self.semantic_runtime_state.rollback_to(saved_semantic_checkpoint); // ← THE FIX
        None
    }
}
```

**Why level 5** (parser-agnostic engine) [[feedback_no_workarounds_fix_hierarchy]]. Levels 1–4 are inadequate: the store IS transactional (per `.b.5.1.1` multi-index + rollback infrastructure), but the engine wasn't *using* the transaction at the speculative-attempt boundary. No annotation, no store query, no new annotation/store primitive can stop emit_fact from running during a speculation — only the engine's speculation wrapper can. Parser-AGNOSTIC: regex / json grammars without semantic predicates snapshot is trivially O(1) (`checkpoint()` captures three usize lengths + an empty `active_chain.clone()`).

**Scope limit — C3-B remains open.** This fix covers only the FAILED-speculation case. Under `longest_match` / `priority_first` branch policy the multi-branch codegen tries ALL branches and selects the best; loser-but-SUCCESSFUL branches' emissions still accumulate (the 8× duplication observed in the diagnostic dump). That's strictly broader engine work (commit-only-winner via delta-replay or winner-double-execute), deferred to its own slice.

**EMPIRICAL CORRECTION (2026-05-25, the original "does not corrupt has_fact" claim was wrong):** Slice-65 added fact-store-interaction trace and immediately observed the SAME `has_fact(type_name, T)` query returning `true` twice (right after `@emit_fact`) then `false` later, in the same parse. Re-applying Slice-64 (the predicate gate that depends on has_fact) reproduced this on T_repro: the gate's predicate fires AT a moment when speculative rollback has the fact rolled out, predicate returns false, branch rejected, parse fails — even though the type IS legitimately declared. **C3-B DOES corrupt boolean has_fact consumers** when those consumers fire on the parse path that committed AFTER a peer speculation rolled state out from under them. This made Slice-64 regress UVM (byte 828K → 162K furthest_position drop) — until C3-B is fixed, NO has_fact-based predicate added to a context where speculation has run is safe. The fact-store trace tool (Slice-65) is now the standard diagnostic for spotting this: any predicate site showing same-query-different-result is hitting C3-B. Promotes C3-B from "efficiency issue" to "correctness blocker for any future has_fact-based predicate added to existing speculation contexts."

**Audit pattern.** Any future engine work that adds a new per-speculation primitive (a new lookahead form, a new tournament structure, an alternative iteration) MUST mirror the three-snapshot-three-restore pattern. Adding a new `mem::take`-on-state inside a fallible body is the C2 sibling hazard ([[feedback_question_bypasses_manual_cleanup]]); not snapshotting state across a try_parse boundary is C3-A. Both can corrupt the fact store silently — there is no parse-failure that surfaces them; they only manifest as wrong predicate results downstream.

---

## ⚠️ CAUTIONARY NOTE 2026-05-25 — the "EMPIRICAL CORRECTION" section above and C3-B's framing are suspect

The 2026-05-25 "EMPIRICAL CORRECTION" paragraph claims that re-applying Slice-64 reproduced a same-query-different-result `has_fact` event on T_repro, attributing Slice-64's UVM regression to C3-B-mediated fact corruption. **A later tools-first re-analysis (same date, after Architecture B was disproven) of the canonical implicit-type port-list failing input (`function bit f(string a, b="");`) showed the semantic store working correctly:**

- `has_fact(kind=type_name, name=Identifier("b")) → false` at the moment of the failure (correctly absent).
- Every predicate-gated type-identifier alternative (`known_unscoped_class_scope_class_identifier`, `..._interface_class`, `..._type_parameter`, `checked_type_identifier`, `..._covergroup`) correctly rejected `b`.
- The actual mechanism was that `provisional_unscoped_block_class_type` (alt 12 of `data_type`) has **no predicate** and matched `b` as a catch-all — a pure structural/grammar defect, not a fact-persistence defect.

What this means for THIS memory:
- **C3-A (the original invariant, lines 10-30 above) still holds in principle** — speculation-only emissions leaking into committed state is a real category of bug. The mechanism described is still defensible engine hygiene.
- **C3-B's framing ("loser-but-successful branch accumulation corrupts has_fact") is unverified** for the inputs actually under examination. The minimised repro that supposedly demonstrated it was never traced to show the specific same-query-different-result mechanism on a real failing input. The persistence framing as a whole (`[[feedback_predicate_parse_order_sota]]`, RETIRED) was downstream of mis-framing the implicit-type port-list defect.
- **C3-B-the-code (Slice-67) IS retained** because it empirically passes one specific test (`systemverilog_context_gated_method_chain_handles_negated_and_uvm_shape`, verified passing on HEAD 2026-05-25). Whether it passes for the reasons its commit message claims is undetermined; reverting it would re-introduce that test failure for no current benefit.

**How to use C3-B going forward:** treat it as defensive correctness with unverified scope. Do NOT cite "C3-B's framing" as justification for further engine work (additional fact-promotion mechanisms, new per-speculation snapshot extensions, expanded delta-replay logic). Any new engine slice that wants to invoke C3-B as a precedent needs FRESH trace evidence — on a real failing input — that demonstrates the specific persistence mechanism the new work intends to address.

**How to use this memory's "EMPIRICAL CORRECTION" paragraph:** read it as a historical record of what was believed on 2026-05-25 morning, not as a current statement about engine behavior. The implicit-type port-list defect, which that paragraph cited as the mechanism's cleanest demonstration, has since been shown to have a different root cause entirely.

**Cross-references:** [[feedback_question_bypasses_manual_cleanup]] (sibling C2, per-rule boundary); [[feedback_universal_semantic_store]] (the rollback primitive used); [[feedback_no_workarounds_fix_hierarchy]] (C3-A is the canonical level-5 example; C3-B's level-5 status is suspect); [[feedback_predicate_parse_order_sota]] (RETIRED, related framing); [[feedback_tools_first_no_guessing]] (the discipline that surfaced the mis-framing); `docs/reference/SV_EXH_PROOF_DEFECT_TAXONOMY.md` C3-A (taxonomy entry); `rust/src/ast_shape_contract.rs::systemverilog_context_gated_method_chain_handles_negated_and_uvm_shape` (the test that empirically depends on C3-B code).
