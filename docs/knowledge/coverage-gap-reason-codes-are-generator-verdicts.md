---
id: coverage-gap-reason-codes-are-generator-verdicts
title: Coverage-gap reason codes are STIMULI-GENERATOR verdicts, not parser verdicts
answers:
  - "what does selected_but_failed mean on a coverage target"
  - "is never_selected about the parser or the generator"
  - "a residual coverage target says selected_but_failed — should I trace the parser?"
  - "which subsystem owns a residual closed-loop coverage target"
  - "what is the difference between never_selected, selected_but_failed, never_hit and below_threshold"
  - "why is a Protocol D branch-selection trace the wrong instrument for a coverage gap"
tags: [stimuli, coverage, diagnostics, gap-report]
date: 2026-08-01
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (reason assignment :2686-2698; record_branch_selected at the TOP of the generator's ordered-choice attempt loop :10302; record_branch_success only where a body generates :10350/:10382/:10435/:10535); docs/tasks/SV-EXH-PROOF.md leaf .7.4.6.9 (WHY_WHERE_CLASS_A_2026-08-01)
reverify: sed -n '2686,2698p' rust/src/ast_pipeline/stimuli_generator.rs; grep -n 'record_branch_selected(\|record_branch_success(' rust/src/ast_pipeline/stimuli_generator.rs | grep -v 'fn record' | head -5
---

A residual target in the closed-loop gap report carries a `reason`. **Every one of these is a
statement about the STIMULI GENERATOR, not about the parser** — the counters behind them are written
by the generator's own ordered-choice attempt loop:

| reason | condition | means |
|---|---|---|
| `never_selected` | `selected_hits == 0` | the generator never even *attempted* this branch |
| `selected_but_failed` | `selected_hits > 0 && success_hits == 0` | the generator **attempted it and could not emit a sample** |
| `below_threshold` | `success_hits > 0` but under the required count | it generates, just not often enough |
| `never_hit` | rule target, zero successes | the rule itself was never produced |
| `unreachable_from_entry` / `references_rule_missing_from_active_grammar` | not branch-reachable | structural, before generation is even tried |

`record_branch_selected` fires at the **top** of the attempt loop — before the outcome is known,
immediately preceding the `Trying OR branch: …` trace line. `record_branch_success` fires only where a
branch body actually generates. So `selected_but_failed` is a **generation failure**: no witness for
that branch exists at all.

⛔ **THE TRAP, and it is a measured one.** `selected_but_failed` is easy to paraphrase as *"a witness
was generated but the target was not credited"* — which sounds like an attribution bug in the parser
and is the opposite of what the code says. `SV-EXH-PROOF` carried that paraphrase across three slices,
and it had already produced a plan: leaf `.7.4.6.9` was written to open with a **parser-side**
`Protocol D` branch-selection trace (`🏁 selected branch N/M`). That instrument measures the parser
and cannot see this defect. Two of that leaf's three candidate mechanisms — PEG ordered-choice
shadowing, and "the post-transform parser never selects that branch index" — were disqualified by this
fact alone, before any trace was run.

**Consequence for triage.** Read the reason code FIRST and let it choose the subsystem:

- `never_selected` / `selected_but_failed` / `never_hit` → **generator**. Reach for the reach-plan and
  generation-budget instruments ([[stimuli-generation-error-reasons]], [[prove-rule-dead-or-reachable]]),
  not the parse trace.
- A target that *is* witnessed but the parser rejects → **parser**. That is where `--trace-rules`,
  the `🏁` selection line and `Protocol D` belong.

Note that a zero `depth_exceeded` counter does **not** clear the generator: a branch can be attempted
and still emit nothing without the depth budget firing. See
[[stimuli-generation-error-reasons]] for the error-reason taxonomy that distinguishes the budgets.

**The portable rule:** when a diagnosis rests on a status value, open the code that assigns that value
before designing an instrument around it. A reason code is an API; a paraphrase of it is a guess.
