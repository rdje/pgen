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

---

**PROFILE LANDED (session #90, 2026-07-11) — the WHERE is now known.** `sample` (release, 1 ms) on
regex/json/vhdl. Regex baseline geomean ≈ **496µs/parse** (8-pattern bench). Self-time: **59% in
`libsystem_malloc`** (the symptom). Call-graph attribution: **`SemanticRuntimeState::rollback_to_named`
≈ 24% of total** — the backtrack path taken by every failed speculation; its allocation is the
unconditional `active_chain.clone()` (`semantic_runtime.rs:2793`) + `self.scopes = …collect()` rebuild
(`:2804`). **SYSTEMIC: confirmed** — json (`create_contextual_error` 174, `RecursionGuard::check_cycle`
130) + vhdl show the same shared-engine cost shape. **Two priors CORRECTED:** (1) regex is NOT fully
predicate-free — it emits `regex_capture_group` facts + opens/closes lookaround scopes, so the semantic
runtime IS on its hot path (the 2026-06-03 "no semantic predicates" note above is too strong). (2) The
"per-parser construction rebuilds the annotation table ⇒ 160µs floor" guess was **REFUTED** — `new()` is
~0.7%; the PARSE dominates (the per-char `atom`-tournament under `longest_match` backtracks constantly).
`.3` = gate the rollback active-chain/scopes restore on scope-state-changed (no regen, correctness-neutral).
