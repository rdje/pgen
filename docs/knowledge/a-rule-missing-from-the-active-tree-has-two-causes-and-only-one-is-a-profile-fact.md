---
id: a-rule-missing-from-the-active-tree-has-two-causes-and-only-one-is-a-profile-fact
title: "A rule absent from the active `grammar_tree` is EITHER profile-removed OR never defined, the generator cannot tell which, and treating both as dead broke a passing test"
answers:
  - "can I treat a missing rule reference as gated-out by the profile"
  - "why did my profile-aware pruning break a grammar with no profiles"
  - "how do I tell a profile-filtered rule from an undefined one in the generator"
  - "does apply_grammar_profile_filter leave the removed rules in rule_order"
  - "how should a reach or reachability pass handle an include() external reference"
  - "my analysis pass over-pruned and I cannot tell why"
tags: [profiles, stimuli-generation, reach-planner, engine-universal, instrument-soundness, regression, include-system]
date: 2026-08-26
status: current
evidence: "ENGINE-UNIVERSAL-SERVICES.51 slice 1, PGEN-ENGINE-UNIVERSAL-SERVICES-0089. The reach BFS was taught to skip a hop site whose enclosing alternative the profile filter had killed, keyed on `!grammar_tree.contains_key(name)`. That predicate is TRUE for two unrelated situations: a rule the profile filter removed, and a rule never defined in this grammar at all (PGEN's include(...) system; `node_satisfiable` in grammar_wellformedness.rs deliberately treats the latter as TOP with the comment 'no false positives'). There is no residue to discriminate on: `apply_grammar_profile_filter` (rust/src/main.rs:2657) builds a `retained_rule_order`, so the filter prunes `rule_order` as well as `grammar_tree`, and `StimuliGenerator` holds no profile field at all. Measured consequence: the first cut took the pre-existing test `spurious_store_entry_block_keeps_its_own_rule_witness` RED, because its fixture reaches `soft_target` through an UNDEFINED `epsilon` -- so the whole alternative was pruned and `set_reach_plan_for_rule` would not install. Attribution measured, not assumed: `git checkout HEAD -- stimuli_generator.rs` -> test passes; re-apply -> test fails. Fix: `reach_hops_pass` PREFERS the pruned walk and FALLS BACK to the unpruned one, so pruning can only ever improve a route and never deletes the last one; pinned by `an_undefined_reference_must_not_prune_a_route_the_way_a_profile_gate_does`. Final: 224/224 in the module, verilog_2005 cert UNKNOWN 18 -> 14 with the fallback in place (identical to without it)."
reverify: "cd rust && cargo test --lib stimuli_generator::tests::an_undefined_reference_must_not_prune_a_route_the_way_a_profile_gate_does && cargo test --lib stimuli_generator::tests::spurious_store_entry_block_keeps_its_own_rule_witness"
---

# `contains_key` is not a profile predicate

A pass that reasons about what the active profile allows will reach for the obvious test:

```rust
if !self.grammar_tree.contains_key(name) { /* the profile removed it */ }
```

It is true for **two** situations, and only the first is about profiles:

| the rule is missing because | correct treatment |
|---|---|
| the profile filter removed it | unrenderable under this profile — prune |
| it was never defined in this grammar (an `include(...)` external reference) | ⛔ **nothing to do with profiles** — treat as ⊤ |

And there is **no residue to tell them apart**. `apply_grammar_profile_filter` builds a
`retained_rule_order`, so it prunes `rule_order` alongside `grammar_tree`; `StimuliGenerator` holds
no profile field at all. Both cases present identically: absent from the tree, absent from the order.

The well-formedness lint already faces this and resolves it explicitly — `node_satisfiable` returns
`true` for a reference to a rule "NOT defined in THIS grammar", with the reason stated as *"no false
positives"*.

## What it cost

Teaching the reach BFS to prune dead branches, keyed on that predicate, took a **passing test red**:
its fixture routes to the target through an undefined `epsilon`, so the branch was pruned and the
reach plan would not install. The test was pre-existing and correct; the new code was wrong.

⭐ Attribution was **measured, not argued** — `git checkout HEAD -- <file>` and the test passes,
re-apply and it fails. That takes a minute and replaces a guess about whose change it was.

## The shape of the fix — prefer, then fall back

Rather than trying to recover a distinction the data no longer carries, make the optimisation
**unable to lose**:

```rust
self.reach_hops_bfs(entry, target, store_gated, /* prune */ true)
    .or_else(|| self.reach_hops_bfs(entry, target, store_gated, /* prune */ false))
```

Now pruning can only ever IMPROVE a route. A target reachable both ways takes the renderable one; a
target reachable only through a dead branch is planned exactly as before; a grammar using includes is
untouched. ⇒ **when a heuristic cannot distinguish two cases and being wrong about one is a
regression, do not sharpen the heuristic — make its failure recoverable.**

## The generalisation

Any pass that concludes something *negative* from an absence should ask what else produces that
absence. Here the absence had two producers and the pass assumed one. The same question is worth
asking of every `!contains`, `is_none`, and empty-set test that feeds a pruning decision.

**See also:** [[a-per-branch-fact-joined-against-a-rule-level-profile-set-over-reports]] — the sibling
error one layer up, where a per-branch fact was checked against a per-rule set;
[[an-x-is-checked-by-nothing-claim-is-a-census-claim]].
