<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_tools_first_no_guessing.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_tools_first_no_guessing
description: User-set discipline (2026-05-24, emphatic) — given the toolbox pgen now has (furthest_position, predicate self-explaining trace, --trace-rules, --dump-rule-call-counts, fact-store-interaction trace), we should NEVER guess at fixes. Every fix proposal should be backed by a tool that SHOWED the root cause directly. If the existing tool doesn't show it, BUILD the tool that does — don't speculate.
metadata:
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User-set discipline (2026-05-24):** "Given all the new tools we landed and are going to land, we should[n't] guess a fix anymore, I mean normally."

Triggered when I applied a fix (Slice-64: has_fact gate on provisional), it didn't help (uvm_pkg regressed), and I started SPECULATING about why (cross-file refs, then type parameters). The user called this out:

> "For me the simple fact the fix didn't solve the issue is indicative that the analysis wasn't complete, because ultimately we didn't understood the real root cause of the issue, we are just speculating. We shouldn't. We should have tool that point us directly, or help us, clearly seeing the root cause. Right now that is not the case yet. It also imply that we need more information, Or that we didn't use properly the tool in our current toolbox."

**The discipline this enforces:**

1. **Before proposing a fix, name the tool that showed you the root cause.** If you can't name one, you don't have one — STOP. Don't propose the fix.

2. **If the existing tools can't show it, the next slice is the TOOL, not the fix.** The session just built furthest_position, the predicate self-explaining trace, etc., for exactly this reason — each was a "tool gap that blocked diagnosis." We should keep building tools until any defect class is self-diagnosing.

3. **"The fix didn't solve the issue" = diagnostic incomplete.** Don't double down with another guess. Stop. Add tooling. Re-diagnose.

4. **A correct understanding lets the fix work first try.** If the first fix doesn't work, the understanding was wrong. Per [[feedback_root_cause_before_fix_code_last_resort]] this matters; per this principle the REMEDY is tool-building, not more guessing.

**How to apply:**

- When proposing a fix in a session, the first paragraph of the commit message should say WHICH TOOL showed the root cause and WHAT IT SHOWED. If the answer is "I reasoned about it from the grammar source," that's a yellow flag — pause and verify with a tool first.
- When a fix doesn't work, the NEXT slice should NOT be another fix attempt. It should be either (a) using a better tool that we under-used, or (b) building a tool we don't yet have. Then come back to the fix when we can see directly.
- The fix-hierarchy ([[feedback_no_workarounds_fix_hierarchy]]) is necessary but not sufficient — you can pick the right level and still propose the wrong fix if you don't see the root cause. Tools-first comes BEFORE level-selection.

**SIBLING PRINCIPLE — fixes shall always be targeted (user 2026-05-24):**
> "Fix shall always be targetted."

Connects: tools-first lets you SEE the exact root cause; targeted means FIX exactly that, no more. Don't propose broad "audit and fix all similar" sweeps (Slice-63 did this with `view: shaped` on 4 sites — risky because each site might have nuances; safe-by-luck this time, but the discipline is to verify EACH site individually with the tool that shows its specific defect).

- One commit = one specific defect surgically fixed.
- The defect's identity = what the tool showed.
- The fix's scope = exactly what's needed to flip that one specific tool-shown defect, nothing more.
- Bigger "class of bug" fixes are valid only when EVERY instance has been individually verified to be the same defect via the same tool.

**Cross-references:**
- Strengthens [[feedback_root_cause_before_fix_code_last_resort]] (don't fix before understanding) by adding HOW: through tools that show, not through reasoning that speculates.
- Strengthens [[feedback_corpus_expected_from_spec_not_fix]] / [[feedback_prove_independence_with_decisive_baseline]] (independent oracles catch what hopeful self-checks miss) by extending it: tools that show are independent oracles for "what is true now."
- Reinforces the whole session's tooling investment (furthest_position, predicate self-explaining trace, dashboard, etc. — each one a "fill the gap so we don't have to guess again").
- Counterpoint to [[feedback_no_workarounds_fix_hierarchy]]: picking the right level matters but ISN'T enough; you can pick level 1 (correct hierarchy) and still apply too broadly (wrong target). Both are needed.

---

## AMENDMENT (user 2026-05-25): "When something works, stick to it; don't redo old habits"

User-set this session, after Slice-68 was committed with three SPECULATED hypotheses for the corpus regression — without first running any of the tools we built to distinguish them:

> "The only thing I want is that when we learn something that works that we stick to it and not redo old habits. We have build many tools to helps debug faster and more efficiently but it looks like we are not using them enough."

**The pattern this names precisely:** building tools feels like progress; reaching for those tools at diagnosis time does not happen by default. The session keeps lapsing back to "I have a hypothesis, let me code the fix" instead of "I have a question, let me run the tool that answers it." Building a tool we then don't USE is the loop to break.

**Concrete inventory of tools BUILT in this campaign that must be USED FIRST, by question type:**

| Question                                          | Tool that answers it (use BEFORE hypothesizing)                                  |
| ------------------------------------------------- | -------------------------------------------------------------------------------- |
| "Why is parse X slower / hung?"                   | `--dump-rule-call-counts [N]` (live top-N dashboard; works on timeout)           |
| "Which exact rule + position is failing?"         | furthest-position output on parse failure (`.b.6.2.25`)                          |
| "What is rule X doing on this input?"             | `--trace-rules X,Y,Z` (`.b.6.2.17` — 100-1000× volume reduction vs `--trace`)    |
| "Did my change cause the regression?"             | decisive-baseline stash/edit + rebuild + re-measure ([[feedback_prove_independence_with_decisive_baseline]]) |
| "What facts/scopes does the parser see?"          | `--explain` / `dump_facts` (`.b.6.2.5.1.6`)                                      |
| "Did my grammar edit change the AST shape?"       | `parseability_probe --parse-dump-ast-pretty` before/after diff                   |
| "Are the trace events themselves trustworthy?"    | self-explaining HIGH/DBG verdict + mechanism levels (Slice-65/66)                |

**The new commit-message discipline (addition to point 1 above):** the first paragraph of every diagnostic/fix slice MUST list (a) the question being investigated, (b) the tool above that was run to answer it, (c) the actual output captured. Three explicit sentences. If any one cannot be filled in, the slice is premature; loop back to running the tool.

**This session's specific lapses to remember:**

1. Slice-68 timeout regression: I formed three hypotheses (allocation churn, propagation, predicate drift) and committed the speculation as analysis in the commit message — without running `--dump-rule-call-counts` or `--trace-rules rollback_to`. After user prompt, the two-test factor experiment (Test-C `filter(false)` vs Test-D `filter(true)`) cleanly distinguished H1 from H2 in two builds. That experiment was *fifteen minutes of work* and should have been the FIRST move after seeing the regression, not the move after being called out.

2. The implicit-type port-list defect (`function f(string a, b);`): four sessions of work have produced predicate gates, engine fixes (C3-A, C3-B), and a failed Architecture B — and no commit message yet has led with `--trace-rules port_declaration,non_keyword_identifier` output on a minimised repro. That trace is the FIRST thing to look at the next time this defect is opened.

3. The friscv corpus regression: I could have dumped the regressed parser's fact-store size at intervals via a 3-line instrumentation (we have `dump_facts`); I instead reached for `filter(|_|true)` factor-ablation. Both work; the instrumentation would have been faster and would have produced a number ("fact-store grew to X entries before timeout") instead of a comparison ("D regresses, C doesn't"). Bias the choice toward direct-measurement tools when both are available.

**Sticking discipline:** when a tool is shown to work for a defect class (e.g., `--trace-rules` cleanly diagnosed `.b.6.2.21` stuck-rule issues), it becomes the DEFAULT first move for that class. Don't drift back to "let me think about this first." Re-reading the tool's output IS thinking.
