# `guard_dry_run` — ENGINE-UNIVERSAL-SERVICES.17 slice 3

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
    would_absorb=2 would_refuse=0 clone_rules=24 left_recursive_rule_rows 28 -> 0
    ✅ would absorb 'casting_type'
    ✅ would absorb 'property_expr'
```

- **`property_expr` composes** ⇒ the doubt slice 2 recorded against `.15`'s re-adjudication is
  discharged.
- **`28 → 0`**, re-derived from the rewritten clone by the same `detect_left_recursion` the lint
  runs — never inferred as *before minus absorbed*. Two rewrites clear all 28 rows because both are
  at DOMINATORS: 16 guard-feasible candidates are 16 rules on **two** knots, not 16 rewrites.
- The price is **24 clone rules**.

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

## ⛔ What none of this claims

The dry run **emits no guard**. The grammar it builds is the one `.13` slice 5 measured as a
REGRESSION — and the driver's first pick is `casting_type`, the exact rule `TOOLBOX.md` §5.5 warns
about (*rewriting it turns the accepted `int'(3)` into a rejection*, probe P2). `28 → 0` is
**plan-stage reachability**, not closure. Stacked with slice 2's exactness result (0 of 129 sites
exact), the guard's *effectiveness* on real SystemVerilog text remains unmeasured and is slice 4's
burden.

## Files

| file | what it is |
|---|---|
| `probe.sh` | the self-checking bank, 9 cases, rc 0 iff every case matches its DECLARED expectation |

Run it from the repository root:

```bash
bash docs/tasks/artifacts/engine_universal_services/guard_dry_run/probe.sh
```

⛔ If a case fails, re-adjudicate in `ENGINE-UNIVERSAL-SERVICES.17` — do not edit the expectation to
match. D1/D2 are the evidence that `.15`'s founding premise is re-adjudicated; D5/D6 are the evidence
that the wrapper's refusal is not evidence about the grammar that ships.
