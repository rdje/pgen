---
name: feedback-systematically-use-debug-toolbox
description: STANDING, emphatic (director 2026-06-22) — SYSTEMATICALLY reach for PGEN's full debug toolbox FIRST, before any UNKNOWN / rejected-parse / reach-gap / "why not witnessed" investigation. Never wait to be told a tool exists; never eyeball a grammar or guess a root cause. The complete toolbox is cataloged in the book (diagnosing-unknowns.md + parseability-probe-debug.md) and KM (cert-coverage-unknown-diagnostics), so there is no excuse to ignore it. If the tools cannot surface WHY+WHERE, BUILD a tool — do not speculate.
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-06-22
id: feedback-systematically-use-debug-toolbox
title: Reach for the FULL debug toolbox systematically and FIRST — it is mechanically enforced, not advisory
date: 2026-06-22
answers:
  - "what is the UNKNOWN protocol"
  - "which tool do I run first for a rejected parse or a reach gap"
  - "is the toolbox-first rule enforced or just recommended"
  - "where is the complete tool catalog"
reverify: grep -c '^### ' TOOLBOX.md; grep -n 'UNKNOWN protocol' TOOLBOX.md | head -2
---

**Director directive (2026-06-22), emphatic.** "I really don't understand why you do not
SYSTEMATICALLY use all the debug capabilities that PGEN has implemented... You seem to wait until I
complain or point to them before you start actually using them. We could be a lot more efficient but
[for] this strange behavior of yours we are not." And: "put all the debugging capabilities PGEN has
left/right/center in the mdBook, if not already, in KM cards, if not there already, in claude project
local memory. Put it everything so that you can't ignore them."

**The rule.** For ANY `UNKNOWN`, rejected parse, hang, reach gap, or "why isn't this witnessed"
question, the FIRST action is to run PGEN's purpose-built debug tools — not read the grammar, not
infer, not propose a strategy menu. The tools surface the exact mechanism and location; only then is
a fix proposed. Waiting for the director to name a tool is the failure mode being corrected.

**The toolbox (the catalog you must not ignore).**
- **Certificate-coverage `UNKNOWN` diagnosis** — the 3-step protocol: `PGEN_CERT_COVERAGE_DUMP_ALL=1`
  (full residual list + dead-rule candidates) → `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` (per-rule
  `[plannable-probe]` forced-sample + `parsed`/`witnessed_target` verdict) → scoped semantic trace
  (`PGEN_TRACE_VERBOSITY=debug … --trace-rules <r>`) to name the exact `@predicate` rejection.
  `PGEN_REACH_PATH_DUMP=1` shows the planner's BFS hop chain. NOT-depth check via `--max-depth 24/32/40`.
- **Parser-level** — `parseability_probe` trace levels (none/low/medium/high/debug), `--trace-rules`
  (100-1000× volume reduction), `--dump-rule-call-counts` dashboard, furthest-position error
  augmentation, the predicate self-explaining trace, `--parse-dump-ast-pretty`.
- **Generation/static** — `--dump-gen-ast` (the IR the generators consume), `--lint-grammar`
  (LR / non-terminating / shadowing), `PGEN_REPORT_MEMO_STATS`, the `PGEN_WITNESS_*` knobs.

**Where it lives (so it is impossible to ignore).**
- Book: `docs/book/src/diagnosing-unknowns.md` (master index + the 3-step protocol) and
  `docs/book/src/parseability-probe-debug.md` (parser-level detail).
- KM card: `docs/knowledge/cert-coverage-unknown-diagnostics.md` (retrieval-indexed) + the existing
  `ast-pipeline-cli-reference`.
- Claude project local memory: `reference_pgen_debug_toolbox` (a pointer card).
- This decision (layer C) is the standing enforcement.

Reinforces [[feedback_tools_first_no_guessing]] (the tool must DIRECTLY show the truth),
[[feedback_why_and_where_before_solution]], [[feedback_no_codebase_change_without_tool_backed_facts]],
[[feedback_understand_subsystem_holistically_first]], and the 2026-06-22 companion
`feedback_pinpoint_real_blocker_not_menu` (pinpoint the real blocker, no strategy menus).
