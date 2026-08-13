---
id: an-oracle-pair-is-matched-on-one-axis-and-not-the-others
title: Two views of the same language make a matched pair on the axis they were built to hold constant, and only that one — quoting a refusal from the wrong view is a sourced, checkable, wrong answer
answers:
  - "can I use the raw transcription of a standard as an oracle for the shipped grammar"
  - "my two grammars disagree about the same rule — which one is right"
  - "how do I tell whether a cross-grammar comparison is evidence or a category error"
  - "an analysis refuses a rule on one grammar and accepts it on another — what does that mean"
  - "why did a generated grammar refuse every candidate for the same reason"
  - "how do I stop a report saying zero-refusals when it had nothing to check"
  - "what should an instrument print next to its verdict"
  - "is a well-sourced quote from a tool run automatically evidence"
tags: [oracles, evidence, grammars, annotations, instruments, systemverilog, ast-pipeline]
date: 2026-08-13
status: current
evidence: "grammars/systemverilog.ebnf declares return annotations on 1069 rules; grammars/systemverilog_lrm_profiled_wrapper.ebnf (a 33-line file over the 148 KB LRM-Annex-A-generated systemverilog_lrm_profiled_generated.ebnf) declares 21, and its generated body carries a single `->` line. `--report-indirect-lr-plan --indirect-lr-plan-guard-dry-run` prints would_absorb=2/would_refuse=0 on the first and 0/13 on the second, every wrapper refusal being 'declares no return annotation'. ENGINE-UNIVERSAL-SERVICES.17 slice 3"
reverify: "bash docs/tasks/artifacts/engine_universal_services/guard_dry_run/probe.sh   # 9/9; D1 vs D5 are the split, D4 vs D6 are the axis that explains it"
---

**A pair of grammars that describe the same language is a controlled experiment only for the
variable it was constructed to isolate.** Change the question's axis and the same pair silently stops
being a pair — while continuing to produce confident, quotable output.

PGEN carries two SystemVerilog views: the hand-maintained `systemverilog.ebnf` that ships, and
`systemverilog_lrm_profiled_wrapper.ebnf`, whose body is generated from the IEEE 1800 Annex A
markdown. They are a genuine matched pair for **structure** — `ENGINE-UNIVERSAL-SERVICES.15`
identified a hand-written precedence cascade as scar tissue precisely by comparing the shipped
grammar's cycle shape against the raw transcription's.

They are not a pair for **annotation**, and nothing in either grammar, either report or either task
leaf said so. The shipped grammar declares return annotations on **1069** rules; the wrapper declares
**21**, all of them its own entrypoints, because a grammar generated from standards prose has no AST
shape to declare. So when an analysis asked *"can this chain's AST be composed?"*, the wrapper
answered **no, thirteen times, for the same reason every time** — and one slice recorded that as an
open question about the shipped grammar. Measured through the real planner, the shipped grammar
composes both knots with zero refusals.

## The tell

The quote was impeccable: a real tool run, a named rule, a named alternative, reproduced on demand.
What made it wrong was not sourcing but **reach** — the same failure shape as
[[a-by-verification-oracle-is-not-evidence-outside-the-shape-that-verified-it]]. Two signatures are
worth checking before a cross-view quote becomes a premise:

- **the refusals are uniform.** Thirteen candidates, one reason, verbatim. A property of the
  population usually means a property of the *grammar*, not of the candidates.
- **the axis was never named.** If nobody can say what the two views hold constant, nobody can say
  when the comparison expires.

## What to do about it

Make the instrument **state its own input** next to its verdict. The composition check returns
"nothing to compose" on an unannotated grammar, so `would_refuse=0` has two readings — *the chain
composes* and *there was nothing to check* — that print identically. One extra line closes it:

```text
inputs: annotations=present rules_with_branch_return_annotations=1069
would_absorb=2 would_refuse=0 clone_rules=24 left_recursive_rule_rows 28 -> 0
```

That is the general move, not a SystemVerilog one: **a verdict whose vacuous case and whose real
case render the same is not falsifiable by its reader**, and the fix is one line of provenance
rather than a second tool ([[a-check-whose-inputs-all-pass-has-not-been-tested]]).

And when a leaf does rely on a cross-view comparison, write down the axis it is matched on. The pair
above stays the right oracle for structure; it was only ever the wrong one for annotation.
