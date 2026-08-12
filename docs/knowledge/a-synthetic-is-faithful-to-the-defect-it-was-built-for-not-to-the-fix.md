---
id: a-synthetic-is-faithful-to-the-defect-it-was-built-for-not-to-the-fix
title: A minimised synthetic is faithful to the QUESTION it was built for — re-validate it against the real artifact before the next question, because a "harmless" compression can decide the fix
answers:
  - "my minimal repro reproduces the bug exactly — can I design the fix on it"
  - "how do I know a minimised grammar is still faithful when I change the question I ask it"
  - "the synthetic says rule X is the right place to fix this — does that transfer to the real grammar"
  - "is it safe to collapse a chain of bare rule references when minimising a grammar defect"
  - "why did my fix target the wrong rule even though the repro was correct"
tags: [debugging, minimisation, synthetics, left-recursion, grammar-authoring, engine]
date: 2026-08-12
status: current
evidence: docs/tasks/artifacts/engine_universal_services/indirect_lr/p1_knot_a_defect.ebnf (the synthetic, whose README states the one compression as harmless — "Nothing on the cycle's shape changes"); docs/tasks/artifacts/engine_universal_services/indirect_lr/survey/systemverilog.dumpall.txt (`constant_primary [no_acyclic_seed]` — the real grammar refuses the rule the synthetic pointed at); grammars/systemverilog.ebnf:1641 (`constant_primary := constant_primary_sv_2017 | constant_primary_sv_2023`, both on the cycle); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .13 slices 3 and 4
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-indirect-lr-plan 2>/dev/null | grep -E 'constant_primary \\[no_acyclic_seed\\]|candidate\\] constant_primary_sv_2017'"
---

**A synthetic's fidelity is scoped to the question it was built for.** `p1_knot_a_defect.ebnf`
minimises SystemVerilog's cast/call left-recursion knot to six rules and states its one compression
openly: SystemVerilog reaches `constant_cast` from `constant_primary` through
`constant_primary_sv_2017` — two hops where the synthetic has one. Its README priced that as
harmless (*"Nothing on the cycle's shape changes"*), and for the two questions it was built to answer
it was exactly right:

- *Does the defect reproduce?* Yes, with SystemVerilog's signature byte for byte — one cast level
  parses, every level needing the recursion does not.
- *Which of two candidate rewrites is correct?* The comparison came out right too: eliminating at
  the inner rule regresses, eliminating at the consumer works.

Then the work asked a **third** question — *which rule may absorb the chain?* — and the compression
is fatal to that one, because the elided hop is precisely where SystemVerilog's dialect split lives:

```ebnf
constant_primary := constant_primary_sv_2017 | constant_primary_sv_2023   # BOTH reach the cycle
```

A rule whose every alternative reaches the cycle has no seed, so `X := X_base ( suffix )*` has no
`X_base` and the rule cannot be the rewrite's base. The synthetic's merged `prim := lit | cast_expr`
has a seed and reads as if it can. The fix had been specified against a rule that cannot host it.

**Why this is not "the synthetic was wrong".** It was a good synthetic: it named the mechanism, it
refuted a plausible design, and it cost one command to run instead of a multi-minute regeneration.
The error is in treating *faithful for question A* as *faithful for question B*. Minimisation
deletes structure by design; whether the deleted structure matters depends entirely on what you
then ask.

**The cheap discipline.** Before a synthetic's answer becomes a design decision, re-ask the same
question of the real artifact — one command, and here it moved the fix's target rule:

```bash
./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-indirect-lr-plan
#   constant_primary [no_acyclic_seed]: …            ← the synthetic's answer, refused
#   [candidate] constant_primary_sv_2017  …  verdict=MAY-ABSORB
```

The same failure mode has a cost twin: a quantity argued from a synthetic is an argument, not a
price. The clone cost of this rewrite read as **2** on the six-rule synthetic and measures **13** on
the shipped grammar, because the real cycle is 13 rules long rather than the 4-rule cycle the linter
prints first.

Related: [[a-by-verification-oracle-is-not-evidence-outside-the-shape-that-verified-it]] — the same
shape of error one level up, where the *tool* rather than the *fixture* is trusted outside the
domain that validated it.
