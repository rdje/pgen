---
id: a-recorded-failure-reason-is-not-a-readable-one
title: A failure reason that is RECORDED is not a failure reason you can READ — the per-branch `failure_reasons` map holds the answer, and the one path where residual `UNKNOWN`s are produced discards it at process exit
answers:
  - "why can't I get failure_reasons from a --report-certificate-coverage run"
  - "how do I see whether the reach plan's forced branch actually rendered"
  - "how do I tell whether a witness probe was ever driven down the intended branch"
  - "why does a forced branch silently render a sibling"
  - "what does forced-override outcome=failed mean"
  - "what does forced-override outcome=overridden with no failed line mean"
  - "is a generator depth-exceeded a too-small budget or a wrongly-scoped one"
  - "why does raising --max-depth fix an UNKNOWN and break something else"
tags: [instrument-soundness, diagnosis-protocol, stimuli-generation, reach-planner, certificate-coverage, observability]
date: 2026-08-12
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .11 slice 1 (PGEN-ENGINE-UNIVERSAL-SERVICES-0008); TOOLBOX.md 6.1 + 6.4; docs/book/src/diagnosing-unknowns.md; rust/src/ast_pipeline/stimuli_generator.rs `dump_forced_branch_failure` / `dump_forced_branch_overridden`; rust/src/main.rs:1149 (the early return) vs :1373 (the coverage write)
reverify: "PGEN_REACH_FORCED_OVERRIDE_DUMP=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0 2>&1 | grep forced-override | grep select_expression_lr_suffix"
---

[[a-probe-sample-says-where-generation-ended-up-never-what-the-plan-asked-for]] ends by saying that
whenever a component recovers silently, you should *"budget for an instrument that reports what it
tried, not only what it produced."* This card is what happened when that instrument was finally
built — and the surprise was that the data had been there all along.

## The trap

`record_branch_failure` files the generator's own error string for every failed OR branch, keyed by
`(rule::node_path, branch_index)`. [[branch-failure-reasons-are-the-witness-why]] and `TOOLBOX.md`
6.1 both say — correctly — that this record is *"already written by every run that emits reports, no
re-run needed."* A task leaf quoted that and told the next session to go read it.

It is not readable on the path that matters:

* `--report-certificate-coverage` **returns** at `rust/src/main.rs:1149`, long before the
  `--coverage-output` write at `:1373`;
* the two flags cannot even be combined — `--coverage-output` is declared to `require`
  `--generate-stimuli`, so asking for both is a clap error and no file is produced.

⇒ on the single path that *produces* residual `UNKNOWN`s, every failure reason is computed, stored,
and dropped when the process exits. 6.1 is the tool for the closed-loop replay gap report. It is not
a tool for the witness passes, and nothing said so.

⭐ **The generalizable form:** *"the value is recorded"* and *"the value is reachable from where the
question is asked"* are different claims, and a doc that only checks the first will send readers to
an empty file. When a leaf's plan says "just go read X", verify the path emits X **before** building
on it — even when the plan is your own from two sessions ago.

## The instrument that closed the gap

`PGEN_REACH_FORCED_OVERRIDE_DUMP=1` (`TOOLBOX.md` 6.4) emits the pair `generate_or` never left behind:

```
[forced-override] rule='select_expression_lr_suffix' path='root' forced_branch=2/3 outcome=failed
    reason="Stimuli generation depth exceeded max_depth=67 while expanding rule 'real_number'"
[forced-override] rule='select_expression_lr_suffix' path='root' forced_branch=2/3
    outcome=overridden rendered_branch=0
```

Read it as two independent facts:

| you see | it means |
|---|---|
| `outcome=failed` + `outcome=overridden` | the directive was issued, attempted, and lost — the reason string names the mechanism |
| `outcome=overridden` with **no** `outcome=failed` for that site | the forced branch was never *attempted*; look at the self-recursion stand-down and the bypass-fuel re-admission, not at the branch's body |
| neither, on an unwitnessed target | this site was never forced at all — the plan is the suspect, so go back to `PGEN_REACH_PATH_DUMP` |

## Wire a recovery instrument at EVERY exit, and enumerate them mechanically

`generate_or`'s attempt loop has **seven** exits — four `return Ok` (literal-hint short-circuit, plain
success, depth-slack-retry success, constructive-reach-retry success) and three
`record_branch_failure` sites. The obvious two are the plain success and the plain failure. An
instrument wired only there reports *"never overridden"* on exactly the runs where a retry rescued
the fallback: a blind spot in the passing direction, inside a tool built to close a blind spot in the
passing direction. Enumerate the exits with a tool (`awk` over the loop's line range for
`return`/`continue`/`break`) rather than by reading — "I think I found them all" is precisely the step
that fails.

## "Depth exceeded" does not mean "needs more depth"

The number is the diagnosis, not the verdict. `max_depth=67` above is
`reach_prefix_budget + min_derivation_depths[rule]` = `2×24 + 19`: a **rule**-scoped depth, i.e. the
depth of that rule's *shallowest* alternative — computed once, outside the loop that then FORCES a
specific, much deeper alternative. So two different situations print the identical reason:

* the budget is genuinely too small → raise it;
* the budget is measured against the wrong thing → **scope it to what you forced**.

Only the second is a defect, and this was the second. The A/B that separates them is a
`--max-depth` ladder, and on SystemVerilog it also refutes the tempting fix: raising the **global**
knob does reach `UNKNOWN=0`, while `sample_parse_failures` climbs `0→8→17` — the gate trading a known
residual for witness samples its own parser rejects. ⛔ A global knob cannot pay for a locally
mis-scoped budget; it only moves the cost onto a different column of the same report.
See [[sv-residual-depth-budget-cause]] and [[stimuli-residual-coverage-model]].
