---
name: a-green-counter-can-be-the-one-hiding-the-counter-that-moved
description: "DISCIPLINE (2026-08-25, SV-CORPUS-GRAD.13e.10(c4) / PGEN-SV-CORPUS-GRAD-0305) — a lint prints many counters, and the one you are watching going GREEN is not a statement about the others. MEASURED: applying the eleven minimal fixes `--lint-grammar` itself derived took `profile_orphans` 11 -> 0, the error-class counter the edit was aimed at, while `left_recursion_unhandled` went 0 -> 15 and the rule count fell 1537 -> 1513 in the same run. `left_recursion_unhandled` is WARNING-class, so `--lint-grammar` exited 0 and no doctrine blocked it — and the warning's own text says the consequence is that the grammar `declares strictly more than its parser accepts`, i.e. an UNDER-acceptance on the two profiles being released. The green counter was the one that made the moved counter easy to skip. ⇒ diff the WHOLE summary line before and after, never the counter you are steering, and treat any counter that MOVES as a finding regardless of its severity class — a class says how loud a state is, never whether your change caused it."
id: a-green-counter-can-be-the-one-hiding-the-counter-that-moved
title: "A green counter can be the one hiding the counter that moved — diff the whole summary line"
date: 2026-08-25
evidence: "docs/tasks/SV-CORPUS-GRAD.md leaf .13e.10(c4) (the seven-row bisect table, baseline vs each candidate variant); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .50 (the root cause the moved counter led to: route_profile_gate at rust/src/ast_pipeline/indirect_lr_elimination.rs:1364 refuses a route whose @profiles intersection no single rule on it declares)"
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar 2>&1 | grep -o 'grammar lint:.*' | grep -oE 'left_recursion_unhandled=[0-9]+|profile_orphans=[0-9]+'   # must print left_recursion_unhandled=0 and profile_orphans=0 — BOTH, because either alone is the reading this record exists to refuse"
answers:
  - "my lint went green, is the change safe"
  - "why did a clean profile_orphans not mean a clean grammar"
  - "what should I compare before and after a grammar edit"
  - "is a warning-class counter safe to ignore"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-25
---

## The rule

Capture the lint's **entire summary line** before the edit and after it, and diff them. Any counter
that moved is a finding you must explain — including, especially, one whose severity class is
`warning` or `note`.

## Why — the measurement

`SV-CORPUS-GRAD.13e.10`(c4) had to gate seventeen IEEE-1800-only punctuation terminals out of the
`verilog_2005` profile. Gating the seventeenth, `tick`, raised eleven `profile_orphans`, and
`--lint-grammar` **derives a minimal fix for each orphan by name**. Applying all of them is the
obvious, tool-sanctioned move, and it produces this:

| variant | rules | `left_recursion_unhandled` | `profile_orphans` | lint exit |
|---|---|---|---|---|
| baseline | 1537 | 0 | 0 | 0 |
| + 17 terminal gates | 1537 | 0 | **11** | 1 |
| + 17 and all 7 source companions | **1513** | ⛔ **15** | **0** | **0** |

The last row is the one a reader steering `profile_orphans` would ship. It exits **0**. It is also
the row in which PGEN's indirect-LR elimination has silently stopped running on one candidate,
trading 24 synthesized rules for 15 cycles served only by the runtime cycle guard — and the lint says
what that costs, in the warning it just printed: *"every derivation that needs the recursion at the
seed position is UNREACHABLE and this grammar declares strictly more than its parser accepts."*

An **under-acceptance on `sv_2017`/`sv_2023`** — the profiles under a release order — introduced by a
fix for an over-acceptance on a third profile, with a green error-class counter and exit 0.

## The two corollaries

- **A severity class describes how loud a STATE is, never whether YOUR change caused it.**
  `left_recursion_unhandled=15` is a legitimate warning on a grammar that has always had surviving
  cycles. The same number is a regression on a grammar that read 0 one commit earlier. The counter
  cannot tell the difference; only the diff can. (Pricing a ratchet on it is
  `ENGINE-UNIVERSAL-SERVICES.50`(d).)
- **The derived-fix list is what made this easy to walk into.** The tool named all eleven repairs, so
  applying them felt like following the instrument rather than exercising judgement — see
  [[a-tool-derived-list-is-not-thereby-an-actionable-list]], which is the other half of the same
  measurement.

Related: [[feedback_check_the_summary_line_on_the_failing_run]] is the mirror image — there the
summary line lied on the RED run; here it told the truth and was read selectively on a GREEN one.
