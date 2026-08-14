---
id: a-report-must-be-computed-from-the-artifact-it-describes
title: A report built from the DECISION instead of the ARTIFACT cannot see a defect in the artifact — and every bank pinned to it goes green anyway
answers:
  - "my probe bank scrapes the report correctly and still did not catch a real defect — what is wrong"
  - "where should a summary field get its value from, the plan or the thing that was built"
  - "I planted a break and cargo test failed but the bank stayed green — which one is broken"
  - "how do I make an outcome struct describe what a pass DID rather than what it intended"
  - "is it enough to prove a test falsifiable with cargo test, or must I plant through the CLI too"
  - "a code change and a report change landed together — what makes the report trustworthy"
  - "I found one number in a census wrong — where do I look next"
tags: [instrument-honesty, probe-banks, ground-truth, verification, falsifiability, derived-state]
date: 2026-08-14
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .17 slice 7 RESULT 4 (GuardChain::summary reported the plan's booleans; a plant deleting the trailing-lookahead emission left GUARD-DRY-RUN 14/14 GREEN describing a lookahead the emitted grammar no longer carried) and RESULT 5 (the same question put to the neighbouring price found guard_hops == guard_chain.len()-1 false on 6 of 129 SV sites, every one a dialect twin); rust/src/ast_pipeline/indirect_lr_elimination.rs (summary now reads the tree apply_plan wrote, and is built AFTER the apply for that reason)
reverify: "bash docs/tasks/artifacts/engine_universal_services/guard_dry_run/probe.sh   # N/N; D10 pins each chain's [positions], derived from whether the rule that should carry a lookahead ends in one"
---

**A value you report about a thing must be computed FROM that thing.** Not from the plan that
intended it, not from the flag that requested it, not from the branch you took to produce it.

The concrete failure: a planner emitted guarded rules into a grammar, and its outcome struct reported
which guard positions each rule carried. The struct built that string from the *plan's* two
booleans — `loop_guard`, `trailing_guard` — which is the honest answer to *"what did I decide to
emit?"* and says nothing about what was emitted. Deleting the emission of one lookahead in the apply
step left the bank that pins those positions at **`14/14`, green**, printing `[loop+trailing]` for a
rule whose body no longer had a trailing lookahead.

⛔ **This survives every check that usually catches instrument rot.** The extraction was correct
(the bank read exactly the field it meant — so
[[a-report-scraper-must-anchor-on-structure-not-on-a-substring]] does not cover it). The expectation
was declared in advance. The bank exits non-zero on a mismatch. It had a REAL failing mode and a real
passing mode. It was simply reading a variable that could not disagree with itself.

⭐ **The fix is a read-back, and it usually moves code.** Derive each reported property from the
artifact: *is a position present? — iff the rule that should carry it ends in a lookahead.* That
forces the summary to be built AFTER the mutation rather than beside the plan, which is the tell:
**if your summary can be constructed before the work happens, it is not describing the work.**

⭐⭐ **And the falsifiability plant belongs on the INSTRUMENT, not only on the code under test.**
Here the unit test caught the planted break (it asserts the emitted rule body) while the bank did
not, and only a plant that rebuilt the binary and ran the CLI could tell them apart. A plant that
only ever runs under `cargo test` measures the assertion layer and leaves every report-scraping row
untested — those rows are a separate instrument with a separate blind spot.

| where the value comes from | what a green run proves |
|---|---|
| the plan / the request / the flag | the decision was taken |
| the artifact, read back | the decision took effect |

⭐⭐ **Once one number is caught, the cheapest next move is to ask the same question of the numbers
beside it.** In the same slice, the neighbouring field carried the comment *"`guard_hops` is
`guard_chain.len() - 1` by construction"* — false on 6 of 129 sites, because it was the *shortest*
distance while the artifact clones *every* branch. That took one sweep over a report that already
existed. A census is a set of prices sharing a set of assumptions; when one is measured against a
superseded model, its neighbours are where to look. ⛔ And note whose sentence was wrong the second
time: the same author's, written an hour earlier, contradicting a docstring three files away that
they had also written. Being the author of both halves is no protection — only executing them
together separates them.

⛔ **Same family, different failure, all silent in the flattering direction:**
[[a-report-scraper-must-anchor-on-structure-not-on-a-substring]] is an extraction that matched the
wrong rows; a hand-typed total beside a table that grew is a number nothing re-derives
(`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3); this one is a correct extraction of a value that was
never about the artifact. Sibling of [[a-check-whose-inputs-all-pass-has-not-been-tested]] and of
[[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]] — those are controls whose
inputs or outputs cannot discriminate; this is one whose *source* cannot.
