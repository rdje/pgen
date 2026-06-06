---
name: feedback-quality-over-speed-no-corners
description: STANDING, emphatic (director 2026-06-06) — NEVER cut corners, never quick-and-dirty. We are not in a hurry; there is all the time needed. Want quality, well-thought algorithms, elegant code, very high QoR — take the time to do it the RIGHT way. Speed is never a reason to compromise design.
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-06-06
---

**Director directive (2026-06-06), emphatic.** "You should not cut corners, we are not in
a hurry, we have all the time we need. I do not want quick and dirty work. I want quality,
well-thought algorithms, elegant code, very high QoR. I want to take the time to do the
right way."

**The standing rule.** Speed is NEVER a justification to compromise design. Every change
should be: the well-thought algorithm (not the first thing that works), elegant + minimal,
high quality-of-results, and structurally correct — even if it takes longer. There is no
deadline pressure; deliberateness is the expectation, not a luxury.

**The triggering mistake (own it).** While racing to produce the first real SV
certificate-coverage number (`GRAMMAR-WELLFORMED.G.4.4`), I hardcoded
`if grammar.grammar_name == "systemverilog"` in the PIPELINE (`main.rs`) and reached
straight for `parse_and_cover_systemverilog` — instead of the clean parser-agnostic
dispatch. That violated the absolute parser-agnostic doctrine
([[feedback_ast_pipeline_parser_agnostic]]) which the director stressed is project-critical
("the whole project might simply collapse"). The corner was cut purely to "get the run."

**How to apply.** Before writing code: think the algorithm through; choose the design that
is correct + elegant + agnostic + extensible, not the fastest to type. Prefer the
data-driven / registry / general-primitive structure over a special-case shortcut. When
tempted to special-case "just to see the result", STOP — build the general thing. Verify +
prove properties (e.g. grep that the pipeline has zero grammar names). A quick-and-dirty
result is worth less than no result, because it erodes the architecture.

Reinforces [[feedback_correctness_before_speed]], [[feedback_no_workarounds_fix_hierarchy]],
[[feedback_always_signoff_decisions]], and especially the absolute
[[feedback_ast_pipeline_parser_agnostic]].
