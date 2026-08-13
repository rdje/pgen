---
id: revert-your-fix-and-re-run-a-fix-no-test-defends-is-not-finished
title: After fixing a defect, REVERT the fix and re-run — if nothing goes red, the fix is undefended and the work is not finished
answers:
  - "how do I know my regression test actually guards the fix I just made"
  - "I fixed a bug and all tests pass — what else do I owe before committing"
  - "my test suite is thorough about a mechanism but missed a criterion change — why"
  - "what fixture do I need when I add or tighten a criterion"
  - "how do I check a new guard can fail"
  - "the fix landed and the gates are green, is that enough evidence"
tags: [testing, verification, regression, doctrine, acceptance-checklist]
date: 2026-08-13
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .13 slice 5b (the measured revert — `cargo test --lib indirect_lr` reported 11 passed / 0 failed with the fix deleted, then 11 passed / 1 failed once a fixture expressed the distinction); rust/src/ast_pipeline/indirect_lr_plan.rs (`rules_transparent_to`, `a_transparent_holder_that_outlives_the_rewrite_starves_the_base_rule`); docs/tasks/artifacts/engine_universal_services/indirect_lr/p5_transparent_holder.ebnf
reverify: "revert the one-line criterion in collect_starvation_sites to `step.next_rule != base_rule`, run `cargo test --lib indirect_lr` — exactly one test must go RED, then restore"
---

**A fix is a change, and it owes the same evidence you demanded of the bug.** The usual acceptance
checklist asks *did the defect stop reproducing?* It does not ask *would I notice if this fix were
deleted?* — and those are different questions with different answers.

Measured here: a slice found a regression, root-caused it, corrected the criterion, re-verified end
to end on the real generated parser and the corpus ratchet, and shipped. Reverting the correction
afterwards left **every test green**. The single line standing between the repository and the
regression recurring could have been removed by a future refactor in silence.

**The procedure costs one command:**

1. Apply the fix and confirm the defect is gone.
2. **Revert the fix.** Re-run the suite.
3. Exactly the tests that should fail must fail. If none do, write the missing one *now* — you are
   not finished.
4. Restore, re-run, green.

**Why a thorough suite can still be blind — the part worth internalising.** The fixture everything
was built on could not express the property the criterion turns on. Every starvation test used a
synthetic in which the decisive rule *died with the rewrite*, so the old criterion and the new one
returned the **same verdict** on it. The suite was thorough about the mechanism and silent about the
distinction.

⇒ When you add or tighten a criterion, ask: **which fixture would report differently if I inverted
it?** If the answer is "none", the criterion is decoration and the fixture set is incomplete. Build
the fixture that separates them, and give it a **one-difference control** — the neighbouring shape
that must still report the *old* verdict — so the test proves a distinction rather than an outcome.

Related: [[a-check-whose-inputs-all-pass-has-not-been-tested]],
[[a-transparent-rule-inherits-the-greed-of-the-rule-it-forwards-to]],
[[a-grammar-rewrite-must-verify-its-own-postcondition-because-a-static-criterion-only-answers-what-you-encoded]].
