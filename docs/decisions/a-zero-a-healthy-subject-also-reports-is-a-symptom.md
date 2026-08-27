---
name: a-zero-a-healthy-subject-also-reports-is-a-symptom
description: "DISCIPLINE (2026-08-27, ENGINE-UNIVERSAL-SERVICES.44(b) step 3 / PGEN-ENGINE-UNIVERSAL-SERVICES-0105) — proving a capability census is ALIVE and proving its zero EXPLAINS your problem are two different controls, and the first is routinely mistaken for the second. MEASURED on the shipped SV grammar: four attempt-elision gates read 0 on systemverilog, and the same binary on regex read 60/33/853/102, which correctly established the instrument was not dead. It does NOT establish that the zero is why SV is slow — regex differs from SV in TWO ways at once (layout policy AND grammar scale). The control that settles it is a subject that SHARES the zero and is HEALTHY: json and rtl_const_expr are layout-skipping, report the same 0 on all four gates for the same reason, and reject an over-ceiling input at 0.8-1.0x their own deepest accepted parse, while SV reads 81-86x and VHDL 199-204x on the same input shape. ⇒ the zero was a SYMPTOM, the designed fix was capped at 2.5-5x against a 41x requirement, and the real term was elsewhere. Before spending a lane on 'make the failing gate applicable', find a subject that already fails the gate and check whether it is fine."
id: a-zero-a-healthy-subject-also-reports-is-a-symptom
title: "A zero a healthy subject also reports is a symptom — the alive-instrument control is not the does-it-matter control"
date: 2026-08-27
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .44 (b) step 3 (arms A-E, the three attribution bounds and the five-family ratio table); producer docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_layout_aware_dispatch.sh; log docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/logs/layout_aware_dispatch_census.txt"
reverify: "bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_layout_aware_dispatch.sh   # arm B must stay non-zero on regex (instrument alive) AND arm E must show json + rtl_const_expr PASSING with the same all-zero gate surface SystemVerilog has — if either half stops holding, this record's argument is gone"
answers:
  - "my capability census reports zero for the subject under study, what control do I need"
  - "the regex control proves the instrument works, is that enough"
  - "how do I tell a symptom from a cause when a gate reports zero"
  - "should I spend a lane making a refusing gate applicable"
  - "why did a large measured headroom turn out not to be the fix"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

A capability census reporting **zero** on your subject needs **two different controls**, and they
answer different questions:

1. **Is the instrument alive?** — the same binary, same invocation, on a subject where it reports
   large non-zero numbers. Without this, `0` is indistinguishable from a dead census.
2. **Does the zero explain my problem?** — a subject that **shares the zero** and is **healthy on
   the outcome you care about**. Without this, `0` is a correlation you found because you went
   looking for it.

Control 1 is the one everybody runs, because it is the one a careful engineer thinks of. It is also
the one that feels conclusive and is not.

## Why — the measurement

`ENGINE-UNIVERSAL-SERVICES.44` was chasing a 12-second over-ceiling rejection on SystemVerilog.
Step 2 measured PGEN's four attempt-elision gates and found them at exactly zero — `0` of 535
top-level sites, `0` of 535, `0` of 1 934 branches, `0` of 1 157 min-0 sites — for one reason,
`terminals skip leading layout (R2 raw-byte peek unsound)`, firing on 100 % of every population. It
ran control 1 properly: the same binary on `grammars/regex.ebnf` reads **60 / 33 / 853 / 102**. The
zero was a fact about SystemVerilog, not a dead instrument.

Step 3 was then directed to design a layout-aware dispatch, and began by measuring the headroom. It
is large and real — a one-line facet flip on a copy of the tracked grammar takes the per-branch
admission gate from **0 to 1 906 of 1 934** branches, and at the site that dominates the workload
**12 of 15 alternatives are refutable on the first byte**.

Then control 2 arrived, and the design collapsed:

| family | layout facet | elision gates | ratio (reject ÷ deepest accept) | verdict |
|---|---|---|---:|---|
| json | skipping | **all four 0** | **0.9–1.0×** | ✅ healthy |
| rtl_const_expr | skipping | **all four 0** | **0.8×** | ✅ healthy |
| regex | sensitive | 60/33/853/102 | 0.8–0.9× | ✅ healthy |
| systemverilog | skipping | all four 0 | **81–86×** | ⛔ |
| vhdl | skipping | all four 0 | **199–204×** | ⛔ |

`json` and `rtl_const_expr` have **exactly** SystemVerilog's elision surface — zero, for exactly the
same layout reason — and they are fine. So the zero cannot be the cause.

## Why control 1 could not have caught it

regex differs from SystemVerilog in **two** ways at once: its layout policy *and* its scale. An
instrument that changes two things at once cannot attribute an effect to either — a lesson this same
leaf had already recorded one step earlier and then re-committed, because control 1 *confirmed* the
hypothesis instead of testing it. The healthy-subject control is cheap precisely because it does not
need to be a near-neighbour: it needs only to share the reported property and differ on the outcome.

## What it changed

Priced against the workload, a perfect layout-aware dispatch removes at most **80.2 %** of rule
entries — the entire guardable population — which is a **2.5–5×** ceiling against the **41×** the
latency bar requires. The term that can supply 41× was in the other factor: the dominant spine rule
is entered **601 658** times on an input with at most **1 634** memo keys, at **0 % memo hits**,
while the same tournament's leaves are ~100 % served. Breadth and repetition multiply; the designed
fix divided the smaller one. The parked leaf owning repetition was promoted onto the critical path
and the designed emission was routed behind it.

## The shape to reuse

- Name the outcome you actually care about (here: a ratio, not a gate count).
- Find a subject that **fails the same gate** and **passes the outcome**. If one exists, the gate is
  not your cause — whatever else it may be worth.
- Keep the headroom measurement anyway. *Insufficient* is not *worthless*: a measured 2.5–5× that
  cannot close a 41× is still the largest elision win the grammar has been shown to have, and it
  survives as its own routed leaf instead of being lost with the hypothesis it was born under.
