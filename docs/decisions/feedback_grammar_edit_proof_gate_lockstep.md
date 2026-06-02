<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_grammar_edit_proof_gate_lockstep.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback-grammar-edit-proof-gate-lockstep
description: A grammar (*.ebnf) edit owns ALL its downstream proof surfaces — re-run+re-baseline the syntax-closure / zero-plausible-gap / reachability gates in the SAME slice, not only the shape-contract manifest + book.
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

When editing any `grammars/*.ebnf`, the slice that owns the edit must
also re-run and (if needed) re-baseline **every** downstream proof
gate whose checked-in contract depends on the grammar's static
structure — at minimum `sv_syntax_closure_gate` /
`sv_preprocessor_syntax_closure_gate` and the
`*_zero_plausible_gap_proof` / `*_reachability_closure` contracts —
in the **same commit**, not just the per-grammar AST shape-contract
manifest + the live-book.

**Why:** discovered 2026-05-17 by `SV-EXH-PROOF.1`'s measured
baseline. `PGEN-POST-SV-AUDIT-0002` (Cat-A `macro_formals` factoring)
and `PGEN-INLINE-ALT-FIX-0001` (SVPP-0001 `pp_if_branch` inline-alt
lift) edited `systemverilog_preprocessor.ebnf` and correctly
lockstepped the shape-contract manifest + book + bug-ledger, but did
**not** re-run `sv_preprocessor_syntax_closure_gate`. Factoring a
named record rule / lifting an inline alternation changes the static
reach-set; `systemverilog_preprocessor_syntax_closure_contract.json`'s
`max_unreachable_branches: 3` silently went stale and the gate now
fails on `main` (`unreachable_branches=13 > 3`). Those campaigns were
committed as "Done" while leaving a real regression that blocks the
SV family-status / formal-exhaustive umbrella. Now owned by
`SV-EXH-PROOF.2`.

**How to apply:**
- A Code-Change-Doctrine leaf that touches a grammar enumerates *all*
  proof surfaces that key off that grammar (shape-contract manifest +
  book + ledger **and** syntax-closure / reachability /
  zero-plausible-gap gates + their contracts) and verifies/rebaselines
  each in the same slice.
- Re-baselining a closure contract after a legitimate structural
  change (e.g. bumping `max_unreachable_branches` + classifying the
  newly-unreachable branches as an allowed benign pocket — the proven
  preprocessor zero-plausible-gap pattern) is itself a leaf-owned
  code change with its own honest justification, never silent.
- Treat closure-gate staleness like book↔code drift: a tracked
  correctness defect, surfaced and fixed, not deferred.

Related: [[feedback_task_tree_workflow]] (Code-Change Doctrine),
[[feedback_regex_book_live]] (lockstep is non-negotiable),
[[feedback_quantified_group_extraction]] (the Cat-A/B/C edits that
triggered this), [[project_all_task_trees_complete]] (SV-EXH-PROOF
state).
