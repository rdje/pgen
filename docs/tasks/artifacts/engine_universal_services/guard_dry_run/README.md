# `guard_dry_run` — ENGINE-UNIVERSAL-SERVICES.17 slices 3 and 7

What option (iii) unlocks, measured by running the **real** elimination driver rather than a model
of it.

## The question this answers

Slice 2 priced the call-site follow-restriction guard at `guard-feasible 16/28` on SystemVerilog and
left slice 3 one mandatory first check, in its own words:

> whether the annotation-composability check also passes at `property_expr` is **not** measured here
> (the wrapper refuses it for a missing return annotation on `property_expr_sv_2017` alternative 5)
> and is slice 3's first check.

That check was unanswerable from any shipped tool, and the reason is structural: `plan_elimination`
is only ever reached from `survey.safe_candidates()`, so every refusal **downstream** of the
starvation gate — a hop with no declared return annotation, the trial re-lint, the ambiguity
comparison — is unobservable for a STARVED candidate. All 28 SystemVerilog rows are starved.

## The instrument

```bash
rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
  --report-indirect-lr-plan --indirect-lr-plan-guard-dry-run
```

It admits the guard-feasible candidates into the shipped driver **on a clone of the grammar** and
reports what the driver did. The admission policy is a parameter (`CandidateAdmission`); the shipped
entry point delegates with `StarvationSafe` and is otherwise the same function, which is why this
measures the real planner and cannot drift from it.

## The answer

```text
--- GUARD DRY-RUN: which guard-feasible candidates actually reach a PLAN ---
    inputs: annotations=present rules_with_branch_return_annotations=1069
    would_absorb=2 would_refuse=0 clone_rules=24 guard_chains=3 guard_rules=6
        left_recursive_rule_rows 28 -> 0
    ✅ would absorb 'casting_type'
    ✅ would absorb 'property_expr'
    🛡  guard 'casting_type_lr_guard0' [loop]  chain: casting_type  residual 'tick lparen constant_expression rparen'
        sites: constant_cast alt#0   rules: casting_type_lr_guard0_suffix, casting_type_lr_guard0
    🛡  guard 'casting_type_lr_guard1' [loop]  chain: casting_type  residual 'tick lparen expression rparen'
        sites: cast alt#0   rules: casting_type_lr_guard1_suffix, casting_type_lr_guard1
    🛡  guard 'property_expr_lr_guard0' [loop+trailing]  chain: property_expr  residual 'implies property_expr'
        sites: prop_primary_sv_2017 alt#13, prop_primary_sv_2023 alt#13   rules: property_expr_lr_guard0_suffix, property_expr_lr_guard0
```

- **`property_expr` composes** ⇒ the doubt slice 2 recorded against `.15`'s re-adjudication is
  discharged.
- **`28 → 0`**, re-derived from the rewritten clone by the same `detect_left_recursion` the lint
  runs — never inferred as *before minus absorbed*. Two rewrites clear all 28 rows because both are
  at DOMINATORS: 16 guard-feasible candidates are 16 rules on **two** knots, not 16 rewrites.
- The price is **24 clone rules**, plus (slice 7) **3 guard chains / 6 guard rules**.

## ⭐⭐ Slice 7 — what the planner EMITS, and two things the `🛡` lines say that nothing else did

- **The positions are a CROSS-CHECK, not a restatement.** `casting_type` gets `[loop]` and
  `property_expr` `[loop+trailing]` — which is exactly what their `seed:` lines say (`0/10` vs
  `78/80`) — but the emitter reaches it per SITE through `SeedVerdict` while the report prints a
  per-CANDIDATE aggregate. Two code paths, one answer; D10 pins the pair.
- **`variants` UNDER-COUNTS the chains.** The census reports `variants=1` at `casting_type` and the
  planner emits **two**: `guard_variants` keys on a FIRST byte set (what the *dead* byte-test form
  compared) while the emitted guard is a structural sub-parse, and `'( expression )` and
  `'( constant_expression )` share a FIRST set. D12 pins the pair `1,2` so the gap is the
  measurement.
- **`max_hops` UNDER-COUNTS the clones, for a different reason.** `hops` is the SHORTEST transparency
  distance; the per-site `chain=` is every rule on any transparent path. They disagree wherever
  transparency BRANCHES, which SystemVerilog's dialect twins do — `primary` reaches `cast` through
  `primary_sv_2017` AND `primary_sv_2023`, so five rules get cloned where `hops=3` reads as four, and
  leaving either twin unguarded leaves a live unguarded route to the same starvation. D13/D13b pin
  the population (**6** of 129 SV sites, **5** of 77 wrapper sites) rather than one example.
  ⛔ Two independent under-prices in one census, both invisible until something was built against
  them — and the second was found only because the first prompted the same question of the other
  number.
- ⛔ **Every SV chain has `chain:` of length 1**, i.e. `max_hops=0`. So this bank never exercises the
  guarded HOP clone `X_lr_guard{v}_<hop>` — the rule that carries the whole call-site-scoping
  argument. Its only coverage is the unit test
  `the_guarded_clone_chain_reproduces_the_hand_written_g7_shape`, asserted rule-for-rule against
  `guard_effectiveness/g7_guarded_clone_chain.ebnf`.
- ⛔ **D11/D11b are the SHIPPED-path check**: `indirect_guard_chains=0` on all three grammars. The
  guard planner runs unconditionally, and what keeps it silent on a shipped parser is that the
  admission criterion is *"no surviving starvation site"* while a guard exists only for one. That is
  an argument; these two rows are the check on it.

⛔ **The `[positions]` string is read back off the EMITTED grammar, not off the plan** — and that is
a correction, not a design note. The first version of `GuardChain::summary` reported the plan's own
booleans, and a plant that deleted the trailing-lookahead emission left this bank **14/14 green**
while describing a lookahead the grammar no longer carried. It now derives each position from
whether the rule that should carry it ends in a `Lookahead`, and the same plant flips D10 by name.

## The contrast, and why it is not a contradiction

| grammar | would_absorb | would_refuse | rules with branch return annotations |
|---|---|---|---|
| `systemverilog` | **2** | **0** | **1069** |
| `systemverilog_lrm_profiled_wrapper` | 0 | **13** | **21** |
| `ebnf` | 0 | 0 | 143 |

All 13 wrapper refusals are `declares no return annotation`. The wrapper is a 33-line file over
`systemverilog_lrm_profiled_generated.ebnf` — 148 KB generated from the LRM Annex A markdown, with
one `->` line in total. ⇒ **the two views are structurally comparable and NOT
annotation-comparable**, and no composability conclusion transfers between them in either direction.

⭐ This is why the report prints its own `inputs:` line. `compose_route_template` returns *"nothing
to compose"* on a grammar with no annotations, so `would_refuse=0` has two readings; the annotation
census separates them, and probe cases D4/D6 pin both sides.

## ⛔ What none of this claims — and the bound MOVED at slice 7, inward

Through slice 6 the dry run **emitted no guard**, so the grammar it built was the one `.13` slice 5
measured as a REGRESSION — the driver's first pick is `casting_type`, the exact rule `TOOLBOX.md`
§5.5 warns about (*rewriting it turns the accepted `int'(3)` into a rejection*, probe P2). It now
synthesizes the guarded clone chains.

What is **still** unclaimed is every parse. Nothing here generates a parser or runs an input, so
`28 → 0` remains **plan-stage reachability** and the `🛡` lines are a claim about SHAPE. The guard's
effectiveness was measured separately, on a synthetic, by `../guard_effectiveness/` — that bank's
`g7` row is the shape these lines reproduce, and no measurement yet connects the two on the shipped
grammar.

## Files

| file | what it is |
|---|---|
| `probe.sh` | the self-checking bank; rc 0 iff every case matches its DECLARED expectation, and the headline total is DERIVED from the cases that ran |

Run it from the repository root:

```bash
bash docs/tasks/artifacts/engine_universal_services/guard_dry_run/probe.sh
```

⛔ If a case fails, re-adjudicate in `ENGINE-UNIVERSAL-SERVICES.17` — do not edit the expectation to
match. D1/D2 are the evidence that `.15`'s founding premise is re-adjudicated; D5/D6 are the evidence
that the wrapper's refusal is not evidence about the grammar that ships.
