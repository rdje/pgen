---
id: a-probe-sample-says-where-generation-ended-up-never-what-the-plan-asked-for
title: A witness probe's SAMPLE says where generation ended up; it never says what the plan ASKED for — diagnosing a `parsed=true witnessed_target=false` from the sample alone produced a confident, wrong root cause that reached a task leaf AND a tracked gate contract
answers:
  - "why did my witness probe render a sibling alternative instead of the target"
  - "what does parsed=true witnessed_target=false actually mean"
  - "is a cert-coverage UNKNOWN a reach-planner bug"
  - "how do I tell a reach/routing gap from a generation failure"
  - "why can a rule be unwitnessed when the reach path is correct"
  - "why is an LR-eliminated _lr_suffix rule never witnessed"
  - "how do I check what the stimuli reach plan instructed"
tags: [instrument-soundness, diagnosis-protocol, stimuli-generation, reach-planner, left-recursion, certificate-coverage]
date: 2026-08-11
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .10; TOOLBOX.md Protocol A (4 steps) + 4.4; docs/book/src/diagnosing-unknowns.md; rust/src/ast_pipeline/stimuli_generator.rs `generate_quantified` re-entry guard + test `forced_quantifier_stands_down_on_re_entry_so_an_lr_suffix_can_witness`
reverify: "cd rust && cargo test --features 'generated_parsers ebnf_dual_run' --lib ast_pipeline::stimuli_generator::tests::forced_quantifier_stands_down_on_re_entry_so_an_lr_suffix_can_witness -- --nocapture"
---

`[plannable-probe] rule='X' parsed=true witnessed_target=false sample="…"` is the cert census saying
*"I generated something, it parsed, and it did not exercise X."* The natural next move is to read the
sample, see which sibling it took, and conclude the planner routed badly. That move is wrong often
enough to be worth a card, because the sample and the plan answer **different questions**:

| instrument | question it answers |
|---|---|
| `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` (the sample) | where generation **ended up** |
| `PGEN_REACH_PATH_DUMP=1` (the hop chain) | what generation was **instructed to do** |

**The diagnosis is the gap between them.** With only the first, you cannot distinguish "the plan was
wrong" from "the plan was right and rendering it failed".

## The measured instance

Two SystemVerilog rules reported `parsed=true witnessed_target=false`. Their samples looked exactly
like routing failures — one rendered `cross f, f { option.f = 3.5; }`, the `option` arm that never
enters `bins_selection`, so `select_expression` is not reached at all. From that, the cause was
recorded as *"the reach planner cannot route a probe to a rule that did not exist when it built its
graph"* (the rules are synthesized by left-recursion elimination). That sentence was written into a
task leaf **and** into `systemverilog_recognized_cert_union_contract.json`'s `rebaseline_note` — a
tracked gate contract, i.e. established fact for whoever picked the work up next.

One command falsified it. `reach_hops_pass` BFSes the grammar tree *after* elimination, so the
synthetic rules are in its graph, and the dump printed a **complete, correct** 21-hop chain:

```
[reach-path] target='select_expression_lr_suffix' hops=[…,
  ("bins_selection_or_option","root/o1/s1"), ("bins_selection","root/s3"),
  ("select_expression","root/s1/q")]
```

Every OR steered, the left-recursion quantifier forced. The planner was never the defect. The real
one was a stack frame away, in the **render**: `generate_quantified` re-applied its forced minimum on
every recursive re-entry of the rule that owns the site — and the left-recursion eliminator's own
output (`X := X_lr_base ( X_lr_suffix )*`, `X_lr_suffix := op X`) puts the forced site's body back
inside that rule. 119 forced firings at one site in a single probe; the descent died on depth; and the
enclosing choice, which falls back **by design so generation terminates**, quietly rendered a sibling.

## Read the verdict as three causes, not one

| hop dump shows | cause | fix lives in |
|---|---|---|
| chain missing / short / wrong carrier | **plan-side** — a real reach gap | the reach planner |
| chain complete, sample ignores it | **render-side** — steered, failed, fell back silently | whatever made the forced descent fail |
| sample renders the target construct, still unwitnessed | **parser-side** — the parser never *commits*, typically a longest-match sibling spanning the same syntax | the grammar shape, or seed selection |

The third row is the easiest to misread: rule **entry** counts will show the target entered
speculatively and tell you nothing. Use `--parse-dump-ast-pretty` and read the discriminator `kind`.
(SystemVerilog's `select_expression` catch-all reaches the full expression hierarchy, which parses
`&&` itself — so a bare-identifier seed is swallowed and the suffix never commits, yielding kind
`cross_set` where the declaration says `and`.)

## The generalizable rule

⭐ **A "silent fallback for robustness" turns any failure below it into a misleading symptom
somewhere else.** `generate_or`'s forced-first-*with-fallback* exists so generation always terminates,
which is right — but it converts "the forced descent failed" into "a sibling was rendered", and the
census can only see the second. Whenever a component recovers silently, budget for an instrument that
reports what it *tried*, not only what it produced; otherwise every downstream diagnosis starts from
the recovery instead of the failure.

Corollary for this repo: a forced quantifier prints `candidates=[1]` — exactly one repeat count, so it
has no fallback of its own and its failure always propagates to the nearest choice. Counting those
lines (`PGEN_TRACE_VERBOSITY=high … | grep "Quantifier decision"`) is how a runaway forcing loop is
caught: 119 before the fix, 5 after.
