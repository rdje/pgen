---
id: a-report-scraper-must-anchor-on-structure-not-on-a-substring
title: A bank that scrapes a report by substring is coupled to every line that report will ever grow — and the drift is silent in the passing direction
answers:
  - "my probe bank greps a tool's stdout — what is the safe way to extract a count"
  - "I added a line to a report and a pinned bank went red — is the number wrong or the bank"
  - "how do I stop a pinned N/N ratio from silently changing meaning"
  - "a bank counts lines with grep -c — what can go wrong"
  - "how should a self-checking probe extract a per-row value from a human-readable report"
  - "my tool grew a new output column and a downstream count moved — which one do I fix"
  - "is it editing the expectation to change a pinned number after re-adjudicating it"
tags: [instrument-honesty, probe-banks, report-scraping, ground-truth, verification, drift]
date: 2026-08-14
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .17 slice 5 RESULT 3 (the pinned 157/157 was 129 sites + 28 candidate rows); docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh C7/C7b (extraction re-anchored on `[guard=` and `~ hops=`, wrapper arm added); the three earlier drafts of the same case — tautological `want` derived from `got`, then a site-capped report measuring 123 and calling it "every"
reverify: "bash docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh   # 18/18; C7 pins 129/129 anchored on [guard=, C7b pins the wrapper 77/77"
---

**`grep -c 'first='` counts LINES CONTAINING a substring, not the rows you meant.**

The concrete failure: a bank pinned *"every guard byte test on SystemVerilog is over-approximated,
157 of 157"*. The report it scraped prints one `first=` per starvation-site line — and also one
`suffix_first=` per **candidate summary** line. So `157` was `129` sites plus `28` candidates, and
the bank had been quoting a candidate count as a site count in TOOLBOX, the book, two decision
records and two READMEs. It only surfaced when a later slice added a `seed_first=` line and the
number jumped to 185.

⛔ **The drift direction is what makes this worth a card.** A substring that also matches a sibling
line INFLATES the denominator, and an inflated `N of N` reads as *broader* coverage. Nothing looks
wrong. Compare the opposite error — a scraper that matches too little — which shows up immediately as
a ratio that no longer sums.

⭐ **Anchor on something STRUCTURAL in the line you mean:**

| anchor | why it holds |
|---|---|
| a bracket group opened by the row (`[guard=`) | one per row, and no other line opens it |
| a marker in its ONLY legal position (`~ hops=`) | also rules out the same character appearing as *data* inside a rendered set |
| a field the row alone has (`— residual '`) | usable as an INDEPENDENT second extraction to cross-check the first |

Two independent extractions agreeing is the cheap proof that either one is right. In the case above,
`[guard=` and `— residual '` both returned 129, and the surviving-site census (`68 + 29 + 29 = 126`)
plus the 3 benign rows accounted for it exactly.

⭐ **Re-adjudicating is not "editing the expectation to make it pass".** The bank's own instruction
was *"do NOT edit the expectation; re-adjudicate"*, and the two are distinguishable by a question:
**did the thing being measured move, or did the measuring move?** Here the site count and the
exactness result were both unchanged — every candidate summary is over-approximated too, so the
claim held under either count. Only the number quoted for it was wrong. Fixing that is a correction;
raising a threshold because a real regression made it fail is not.

⛔ **A bank is a consumer of a report's FORMAT, so a report's format is an interface.** When you add
a line or a column to a diagnostic, sweep its scrapers the same way you would sweep callers of a
changed function signature — `grep -rl` for the tool's name across `docs/tasks/artifacts/**`, not
recall. Sibling of [[a-check-whose-inputs-all-pass-has-not-been-tested]] (a control that never
exercises its refusing branch) and of
[[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]] (a control whose output
does not separate two explanations): this one is a control whose *input* was never the thing it
named.
