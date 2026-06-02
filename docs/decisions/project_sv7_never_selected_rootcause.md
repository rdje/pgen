<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/project_sv7_never_selected_rootcause.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: project-sv7-never-selected-rootcause
description: "SV-EXH-PROOF.7.1 finding (2026-05-31, tools-first source read): the 358 never_selected replay-gap branches are a REACHABILITY problem (structural gates fire before weighting), NOT a weak-weight problem — the generator already boosts uncovered branches ×24/×2. Fix direction = deterministic reach/force of a chosen target branch, not more weighting."
metadata: 
  node_type: memory
  type: project
  originSessionId: 5737d722-67a3-4fc9-8c42-f79e2ff1db07
---

**Context:** `SV-EXH-PROOF.7` closes `focused_replay_target_debt_zero` (the ONE criterion keeping the SV main parser at `Mostly Done`). Fresh family-status baseline: `focused_replay_target_count = 696` (566 branch + 130 rule; 298 unique rules; reasons 358 never_selected + 208 selected_but_failed + 130 never_hit; top rule property_expr_sv_2017 @29). `.7.1` = pure-docs investigation of WHY the 358 `never_selected`.

**Verified from source (`rust/src/ast_pipeline/stimuli_generator.rs`, read 2026-05-31):**
- `never_selected` = `selected_counts[branch_idx]==0` (classifier line 1739). `selected_counts` only increments (line 324) when the branch's PARENT OR-node is actually entered + that branch chosen.
- The generator ALREADY biases hard toward uncovered branches: `coverage_guidance_multiplier` (line 4883) gives ×24 for `success_hits==0`, extra ×2 for `selected_hits==0`, ×(1+min(uncovered_refs,4)). So weighting is NOT the bottleneck.
- The weighted choice (lines 3780-3821, `LongestMatch` policy) only runs over `candidate_indices` AFTER several structural filters that can EXCLUDE a branch before any weighting:
  1. **Depth-floor pruning (3638-3648):** at `depth >= max_depth-1` (default `max_depth=24`, struct line 96/114), `candidate_indices` retained to ONLY min-recursion branches → recursion-heavy branches reachable only deep get pruned.
  2. **Missing-rule pruning (3650-3673):** branches referencing a rule absent from the active grammar/profile are dropped (unless they carry an explicit probability).
  3. **Recursion-pressure penalty (3791-3796; def 4824-4839):** as `remaining_depth` shrinks ≤8/≤4/≤2, penalty ×4/×6/×8 → deep-but-rare branches crushed.
  4. **DOMINANT — parent never entered:** target branches live under parent OR-nodes that are themselves low-probability/deep, so the parent is rarely entered → the child group is never instantiated → `selected_hits=0`. (e.g. highest-priority target `block_data_type` branch_idx 10 depends_on `scoped_block_class_type`; these deep type-system branches sit under rarely-entered parents.)

**Fix direction (NOT yet designed — needs a real design pass, NOT a guess):** the signoff fix is NOT "increase weights" (already ×24). It is to give the target-driven generator a way to **deterministically REACH and FORCE a chosen uncovered branch** — e.g. a per-target depth budget + path-forcing that steers parent OR decisions along the path to the target branch, so a specifically-targeted branch is reached even when its natural probability is ~0. This directly serves [[project_stimuli_generator_signoff_vision]] (guaranteed/exhaustive coverage, not best-effort). MUST be a GENERAL grammar-structure property per [[feedback_ast_pipeline_parser_agnostic]] (never hardcode rule names) and task-tree-owned + signoff-verified per [[feedback_always_signoff_decisions]]; `forced_or_branch_for_site` (line 3699) already exists as a path-forcing primitive worth studying as the building block.

**`.7.2` design fork (from reading `forced_or_branch_for_site` @3307, 2026-05-31):** the existing path-forcing primitive is wired ONLY to grammar-MUTATION-replay (`self.mutation_replay` / `GrammarMutationSelection::Or{site_key, forced_branch}` — it replays a single OR mutation for differential testing), NOT to coverage-driving. So a force-this-OR-branch capability EXISTS but is single-site + mutation-purpose. The `.7.2` design question: generalize mutation-replay forcing into a TARGET-REACH mechanism (force the chain of parent OR decisions along the path from entry to a chosen uncovered target branch, with a per-target depth budget so depth-floor pruning @3638 + recursion-pressure penalty @4824 don't cut it off), OR build a dedicated coverage path-forcer. Either way: GENERAL grammar-structure property, no hardcoded rule names. A real design pass, NOT a guess.

**Status:** `.7.1` investigation finding recorded here; the task-file `.7.1` leaf entry + commit are HELD pending the clean `.6` family-status re-verification run (`btv6ww3z1`) settling whether `.6` over-claimed (per [[feedback_always_signoff_decisions]] — don't stack new commits on an unverified one). Next: confirm `.6`, then commit `.7.1` (pure docs), then design `.7.2` fix.
