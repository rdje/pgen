---
id: an-instrument-firing-is-not-the-defect-reproducing
title: An instrument FIRING is not the defect REPRODUCING — a synthetic emitted exactly the expected trace, from a different pass, while reporting the opposite headline
answers:
  - "how do I know my minimal reproducer actually reproduces the bug"
  - "my synthetic shows the expected trace but the wrong result"
  - "how should I validate an isolating synthetic grammar"
  - "why does my scratch-slot probe report UNKNOWN=0 when I expected UNKNOWN=1"
  - "what makes a reduction faithful"
  - "why do I need a cheap alternate route to a rule in a probe grammar"
tags: [instrument-soundness, diagnosis-protocol, reproducers, stimuli-generation, parse-harness]
date: 2026-08-12
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .11 slice 2; docs/tasks/artifacts/engine_universal_services/forced_branch_depth_budget.ebnf (the `decl := "chain" deep ";"` alternative and the comment explaining why it is load-bearing); TOOLBOX.md 1.3 (the scratch slot)
reverify: "cp docs/tasks/artifacts/engine_universal_services/forced_branch_depth_budget.ebnf grammars/scratch/scratch.ebnf && make -C rust SHELL=/bin/bash focus_scratch && (cd rust && cargo build --features 'generated_parsers ebnf_dual_run' --bin ast_pipeline) && ./rust/target/debug/ast_pipeline grammars/scratch/scratch.ebnf --report-certificate-coverage --entry-rule scratch --count 1 --seed 0; git checkout grammars/scratch/scratch.ebnf"
---

Building an isolating synthetic is the discipline this repository keeps asking for. This card is
about the way that discipline still fails: a reduction that **emits the trace you went looking for,
produced by a mechanism you were not investigating.**

## The measured instance

The defect: a reach-plan-FORCED branch whose own derivation is deeper than the witness pass's budget
dies `depth exceeded`, and the enclosing choice silently renders a sibling, so the rule stays
`UNKNOWN`. The first synthetic put the deep chain only under the arm that was supposed to be
under-funded. It produced exactly the expected records —

```
[forced-override] rule='expr_lr_suffix' path='root' forced_branch=2/3 outcome=failed
    reason="Stimuli generation depth exceeded max_depth=…"
[forced-override] rule='expr_lr_suffix' path='root' forced_branch=2/3 outcome=overridden
    rendered_branch=0
```

— 96 of them. And reported `CERTIFICATE-COVERAGE: … UNKNOWN=0 fully_certified=true`.

Both facts were true, and together they meant the reduction was wrong. With no other route to the
chain rules, **every** one of them was itself a residual target, so the *plannable* pass forced that
branch on each of their behalfs. The override traffic came from a pass that was not under
investigation, and the rule the leaf was about never became residual at all. Adding one cheap
alternative elsewhere in the grammar (`decl := "chain" deep ";"`) gave the chain rules a shallow
route from the entry — the analogue of the real grammar's expression hierarchy being reachable from
everywhere else — and the synthetic then reproduced the real headline: `UNKNOWN=1 ["expr_lr_suffix"]`
with all four reach passes reporting zero witnessed.

## What to check, in order

1. **The headline the leaf actually claims** — the verdict, count or metric the defect is defined by.
   Check this FIRST; it is the one that fails while the trace looks perfect.
2. **The trace** — that the mechanism fired.
3. **The attribution** — that it fired *from the pass/component under investigation*. A trace line
   names a rule and a site; it does not name who asked.

⭐ **A reduction is faithful when it reproduces the SYMPTOM, not when it exhibits the MECHANISM.** The
mechanism is easy to provoke in isolation — that is what makes a stripped-down grammar attractive and
what makes it misleading. The symptom is what nobody can fake.

## The structural cause, worth knowing in advance

Stripping context does not merely remove noise; it **promotes what remains**. Every rule left in a
minimal grammar carries proportionally more of the coverage/reachability pressure, so passes that
would never have targeted it in the real grammar now do. When a probe's numbers look wrong, suspect
this before suspecting the engine: ask which rules became important *because* you deleted the others,
and give them back a cheap route.

See [[a-probe-sample-says-where-generation-ended-up-never-what-the-plan-asked-for]] for the sibling
failure (reading a sample as if it reported the plan) and
[[a-recorded-failure-reason-is-not-a-readable-one]] for the instrument used here.
