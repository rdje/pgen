---
id: rgx-0078-regex-slowness-followup
title: QUEUED — after SV reaches Done, attempt RGX-0078 (regex parser slowness vs PCRE2)
answers:
  - "what to do after the SV main parser reaches Done"
  - "what is PGEN-RGX-0078 / the regex parser slowness follow-up"
  - "how to speed up the pgen regex parser"
  - "what is the closure criterion for regex parse performance"
  - "is the SV stateful-packrat fix relevant to regex slowness"
tags: [regex, performance, rgx, queued, pcre2]
date: 2026-06-03
status: current
evidence: docs/decisions/project_rgx_0078_regex_slowness_followup.md; /Users/richarddje/Documents/github/rgx/pgen-issues/PGEN-RGX-0078.yaml (issue, in the rgx repo)
reverify: cat the RGX issue yaml; run the pgen_iteration_flow harness (PCRE2-relative bench)
---

**Director-queued (2026-06-03):** right AFTER the SV main parser convincingly goes
`Mostly Done → Done`, attempt another round at **PGEN-RGX-0078** — the pgen **regex parser is
slow** (~360× PCRE2 no-JIT compile, ~85× JIT). **Closure criterion = geomean
PGEN-parse / PCRE2-compile < 5×.** Try NEW speed techniques; use the **PCRE2 test corpus** +
the vendorable `pgen_iteration_flow/` harness (standalone C baselines + Rust microbench +
driver, in the RGX issue area).

**Precondition:** SV → Done first (its one short criterion is `focused_replay_target_debt_zero`
/ literal-0 = the 273-residual `SV-EXH-PROOF.7.4.6` work). Don't start RGX-0078 before that.

**Key insight (don't conflate with SV's slowness):** SV's O(N²) was the per-rule
`SemanticRuntimeState` clone ([[stateful-packrat-not-linear]]); **regex has no semantic
predicates → takes the fast-path that skips that clone**, so `PARSE-TERMINATION.3.1` does NOT
help regex. Regex's bottleneck is elsewhere (general parse machinery vs PCRE2's tuned compile)
— **profile it first** (macOS `sample`, the technique that pinned SV) then act on the fact.

Full plan + candidate techniques in the decision record [[project-rgx-0078-regex-slowness-followup]].
Cross-repo: issue in `rgx/pgen-issues/`; work on pgen's `grammars/regex.ebnf` + engine.
