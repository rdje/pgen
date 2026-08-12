---
id: a-reported-limit-without-its-provenance-invites-the-wrong-fix
title: A limit reported WITHOUT its provenance invites the wrong fix — "the budget is too small" and "the budget is measured against the wrong thing" print the identical `depth exceeded max_depth=N`, and only the value of N tells them apart
answers:
  - "is a generator depth-exceeded a too-small budget or a wrongly-scoped one"
  - "why does raising --max-depth fix an UNKNOWN and break something else"
  - "how do I decide whether to raise a generation budget"
  - "why is my witness pass budget too small for one alternative but not others"
  - "what is witness_target_depth_budget for"
  - "how should a diagnostic report a limit it hit"
  - "why did a forced branch run out of depth"
tags: [diagnosis-protocol, instrument-soundness, stimuli-generation, budgets, certificate-coverage]
date: 2026-08-12
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .11 slice 2 (PGEN-ENGINE-UNIVERSAL-SERVICES-0009); rust/src/ast_pipeline/stimuli_generator.rs `generate_target_own_structure_witnesses` per-branch budget + `witness_target_depth_budget`; rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json rebaseline_note; TOOLBOX.md 6.4
reverify: "make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate   # union_unknown=0, union_residual_rules=[], sample_parse_failures=0 at seeds 0/7/42"
---

A generation budget that is exceeded prints one message:

```
Stimuli generation depth exceeded max_depth=67 while expanding rule 'real_number'
```

Two different defects produce it, and they want opposite fixes:

| the situation | the fix |
|---|---|
| the budget is genuinely too small for the work | raise it |
| the budget is **measured against the wrong thing** | scope it correctly — raising it papers over the bug and costs elsewhere |

The message alone cannot separate them. **The value of `N` can**, if you know where it came from.

## The measured instance

`67 = 2×24 + 19`. The `2×24` is the reach prefix at the default `--max-depth`; the `19` is
`min_derivation_depths["select_expression_lr_suffix"]` — a **rule**-scoped depth, computed once,
outside a loop whose entire job is to force each of that rule's alternatives in turn. A rule-scoped
depth is the depth of the rule's *shallowest* alternative, which is precisely the alternative a
forced branch is not. The forced arm descended SystemVerilog's whole expression hierarchy; it never
had a chance, and `generate_or` silently rendered a sibling
([[a-recorded-failure-reason-is-not-a-readable-one]]).

⭐ The engine already held the right formula one function away. `witness_target_depth_budget`
budgets a BRANCH target by the targeted alternative's own `min_full_derivation_depth_of_node`, and
its docstring states the failure verbatim: *"a rule-scoped depth is the depth of that rule's
SHALLOWEST alternative — which is precisely the alternative a residual branch target is NOT."* A
later pass re-derived a budget by hand and reproduced the older, wrong shape. **Reusing the existing
function beats writing an equivalent expression** — not for brevity, but because two hand-derived
budgets drift apart and one shared one cannot.

## Why the tempting fix is a regression, not a shortcut

Raising the **global** `--max-depth` does clear the residual. Measured on the same configuration:

| `--max-depth` | union `UNKNOWN` | `sample_parse_failures` |
|---|---|---|
| 24 (default) | 1 | **0** |
| 32 | 0 | **8** |
| 40 | 0 | **17** |

The `UNKNOWN` column improves while the column beside it degrades — those are witness samples the
real parser then REJECTS. ⛔ **A global knob cannot pay for a locally mis-scoped budget; it moves the
cost to a different column of the same report.** The scoped fix reached `UNKNOWN=0` with
`sample_parse_failures` still `0`.

## The generalizable rule

**A diagnostic that reports a limit should report the limit's PROVENANCE, not just that it was hit.**
`depth exceeded max_depth=67` sends you to the knob. `depth exceeded max_depth=67 (= reach_prefix 48
+ rule-scoped 19)` sends you to the arithmetic. The second costs one format string and is the
difference between a fix and a trade. The corollary when reading one: before concluding *"needs more
budget"*, decompose the number and ask what each addend was scoped to.
