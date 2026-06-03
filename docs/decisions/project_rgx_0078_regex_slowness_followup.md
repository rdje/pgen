---
name: project-rgx-0078-regex-slowness-followup
description: QUEUED (director 2026-06-03) — right AFTER the SV main parser convincingly goes Mostly-Done -> Done, attempt another round at RGX issue PGEN-RGX-0078 (pgen regex parser slowness). Closure = geomean PGEN-parse / PCRE2-compile < 5x. Try NEW speed techniques; use the PCRE2 test corpus + the vendorable pgen_iteration_flow harness.
metadata:
  node_type: memory
  type: project
---

**Director directive (2026-06-03):** *"Right after the SV main parser convincingly goes from
Almost Done to Done, please attempt another round at
`/Users/richarddje/Documents/github/rgx/pgen-issues/PGEN-RGX-0078.yaml` from RGX. It is about
the slowness of the PGEN regex parser. You will need to try other techniques to speed up its
runtime. For that you can use the PCRE2 test corpus."*

**PRECONDITION (sequencing):** SV main parser reaches **Done** first. SV is currently
`Mostly Done`; the one short criterion is `focused_replay_target_debt_zero` (literal-0) — the
273-residual closed-loop work (`SV-EXH-PROOF.7.4.6` derivation-directed construction). RGX-0078
is queued AFTER that. Do NOT start RGX-0078 before SV is convincingly Done.

**The issue (PGEN-RGX-0078, status open):** pgen's `regex` parser parse-time is **~360x slower
than PCRE2's full no-JIT compile, ~85x slower than PCRE2 JIT** (geomean over the 8-pattern
bench corpus from RGX-0073). The PRIMARY absolute <50µs target was met (release 1.1.30); the
INTEGRATION closure criterion is the **PCRE2-RELATIVE ratio: geomean PGEN-parse / PCRE2-compile
< 5x**. Bug class: pathological performance. A prior round (RGX-0073) closed the absolute
target; 0078 is the relative-ratio follow-up — still open.

**Resources (provided in the issue):**
- A vendorable `pgen_iteration_flow/` directory (in the RGX issue area): standalone C PCRE2
  baselines, a self-contained Rust microbench, Cargo.toml/Makefile integration snippets, a
  one-shot driver — so pgen runs the same PCRE2-relative measurement without RGX. See
  `pgen_iteration_flow/README.md`.
- The **PCRE2 test corpus** (the director's named tool — for both correctness regression and a
  broader perf corpus than the 8-pattern bench).

**Methodology / approach (grounded by this session's TERMINATION.3 work):**
- **Profile first (WHY+WHERE before fix):** macOS `sample` on a hot regex parse (the technique
  that pinned SV's O(N²) this session) to find regex's actual bottleneck — then act on the
  profiled fact, not a guess. ([[feedback_tools_first_no_guessing]],
  [[feedback_research_grounded_sota_no_trial_and_revert]].)
- **NOTE — regex's cause is DIFFERENT from SV's:** the SV slowness was the per-rule
  `SemanticRuntimeState` clone in `with_semantic_runtime_rule_transaction`
  ([[stateful-packrat-not-linear]]); regex has **no semantic predicates**, so it takes that
  function's FAST-PATH (skips the clone). So `PARSE-TERMINATION.3.1` will NOT help regex —
  regex's bottleneck is elsewhere (likely the general parse machinery: memo allocation per
  rule, ParseNode/clone churn, the FxHashMap memo, per-char overhead vs PCRE2's hyper-tuned
  compile). Profile to pin it.
- **Try other techniques:** candidates to evaluate AFTER profiling — memo/allocation tuning,
  arena/zero-copy for ParseNode, reducing per-rule overhead, the cut operator / bounded memo
  (Mizushima 2010, PARSE-SOTA B2), avoiding redundant work. Pick based on the profile.
- **Measure the GLOBAL metric** (geomean PGEN/PCRE2 ratio) before/after each change; keep iff
  it improves; confirm no RGX conformance regression (the PCRE2 corpus is the oracle).

**Cross-repo:** the issue lives in the `rgx` repo (`rgx/pgen-issues/`); the WORK is on pgen's
regex parser (`grammars/regex.ebnf` + the engine/codegen) — both in this repo. Coordinate the
closure report back to the RGX issue. Likely owned by a new `RGX-0078` task tree when activated.

Composes with [[feedback_correctness_before_speed]] (this is the speed phase — regex is already
correct/conformant), [[feedback_no_codebase_change_without_tool_backed_facts]].
