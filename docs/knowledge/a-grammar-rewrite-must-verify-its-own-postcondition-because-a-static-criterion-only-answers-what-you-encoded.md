---
id: a-grammar-rewrite-must-verify-its-own-postcondition-because-a-static-criterion-only-answers-what-you-encoded
title: A grammar-rewriting pass must APPLY to a copy and re-run the checker before committing — a static "can I rewrite this?" criterion answers the question you encoded, not the question you asked
answers:
  - "my elimination pass picked a base rule that the analysis said was safe and the grammar is still broken — why"
  - "how do I stop a grammar transform from half-fixing a cycle"
  - "is a pre-pass analysis enough to decide whether a rewrite is legal"
  - "where should the postcondition of a codegen pass live — in the pass or in a downstream gate"
  - "why did indirect left-recursion elimination pick the wrong rule three times"
  - "how do I catch a rewrite that trades one defect for another"
tags: [engine, left-recursion, codegen, grammar-transform, verification, ast-pipeline]
date: 2026-08-13
status: current
evidence: rust/src/ast_pipeline/indirect_lr_elimination.rs (the trial-then-commit block in `eliminate_indirect_left_recursion`); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .13 slices 3/4/5 (three successive base-rule choices, the third found only by the trial); the emitted refusal naming the surviving path `constant_primary_sv_2017 -> …_lr_seed_constant_primary -> constant_primary_sv_2023 -> constant_cast -> casting_type -> constant_primary -> constant_primary_sv_2017`
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-indirect-lr-plan | sed -n '3,9p'   # every REFUSED line carries the measured reason, not a guess"
---

**Three slices in a row picked the wrong base rule, each on good evidence.** A synthetic said one
rule. A criterion derived from the shipped grammar said two others. The transformation, applied to a
copy and re-checked, said a fourth — and printed the cycle that was still there.

The pattern generalises past left recursion. A pre-pass analysis encodes a *model* of what the
rewrite will do. The rewrite is the ground truth. Wherever the model is narrower than the checker
that judges the result — here the route walk follows only bare leading rule references and only
*simple* paths, while the left-recursion lint follows nullable prefixes and any path — the model can
call a plan safe that the checker will reject.

**So put the postcondition inside the pass:**

```rust
let mut trial_grammar = grammar_tree.clone();
let mut trial_order   = rule_order.clone();
apply_plan(&plan, &mut trial_grammar, &mut trial_order, /* … */);
let after = detect_left_recursion(&trial_grammar, &trial_order);
if after.names(&plan.base_rule) || after.len() >= before { return Err(reason_with_the_surviving_path); }
```

Cost on the real grammars: one grammar clone per rewrite, single-digit rewrites per grammar.

**Why not leave it to a downstream gate.** A gate tells you the tree is broken. A postcondition tells
you *which plan* broke it, refuses to apply that one, and lets every other plan land — so the pass
degrades to "eliminates what it can, names what it cannot" instead of "all or nothing". The refusal
text is the diagnosis you would otherwise have to reconstruct.

**The corollary that cost the most time: an inherited criterion needs its JUSTIFICATION re-derived,
not just its name.** `no_acyclic_seed` — "a rule whose every alternative is on the cycle has no seed
to chain from" — is sound for a transform that DROPS the cyclic alternative, and false for one that
CLONES it with the cycle edge sheared, because then the seeds come from *under* the alternative.
Same words, different transform, opposite verdict; it excluded the one rule that works.

Related: [[a-by-verification-oracle-is-not-evidence-outside-the-shape-that-verified-it]],
[[a-check-whose-inputs-all-pass-has-not-been-tested]].
