---
id: a-filtered-acceptance-run-is-not-the-module
title: A filtered acceptance run is not the module — the module's other tests are the regression suite for the arm you touched
answers:
  - "why did the CI mirror fail a test my filtered cargo test run passed"
  - "which tests do I run after changing stimuli_generator.rs"
  - "what is the minimum cargo test scope for an engine change"
  - "why did lib-unit-test-gate go red when my leaf's tests were green"
tags: [testing, acceptance, cargo-test, stimuli_generator, ci_workflow_local_gate, regression]
date: 2026-09-30
status: current
evidence: "ENGINE-UNIVERSAL-SERVICES.98 correction (PGEN-ENGINE-UNIVERSAL-SERVICES-0190): .98's acceptance ran the spacing-family filter (12 tests green) while its new compound-repetition arm broke the standing whitespace_only_greedy_tail_gets_no_separator in the same module; the CI mirror's lib-unit-test-gate — the first full run of the lib suite after -0188 — refused with 1291 passed; 1 failed"
reverify: "cd rust && cargo test --lib --features \"generated_parsers ebnf_dual_run\" -- stimuli_generator 2>&1 | tail -1   # the module, not a family filter; expect 0 failed"
---

A test filter is a lens, not a suite. `ENGINE-UNIVERSAL-SERVICES.98` changed one arm of the
stimuli generator's separator chooser and accepted it on a family filter (`spacing`, 12 tests):
green. The same module held a standing test (`whitespace_only_greedy_tail_gets_no_separator`,
`GRAMMAR-WELLFORMED.H.5.1.1`) that pins the exact input the new arm now mis-answered — and it was
outside the filter. The first full run of the lib suite was the CI mirror's `lib-unit-test-gate`
at the push cadence, two commits later.

- **The minimum acceptance scope for an engine change is the MODULE of the function touched**
  (`cargo test --lib --features "generated_parsers ebnf_dual_run" -- <module>`), and the full
  lib suite before the commit when the module is a shared service (the stimuli generator, the
  registry, the contract instruments). A family filter proves the new tests; the module's OTHER
  tests are the regression suite for the arm.
- **A pinned counter-example lives where the pin was written, not next to the new arm.** The
  whitespace-only exemption's test sat 6 000 lines above the new code; grep the module for the
  predicate you are bypassing (`regex_class_is_whitespace_only`) before adding a sibling arm.
- **The mirror is not the first line.** It runs at the push cadence (400 commits); a red there
  is a defect that shipped commits ago. `.117`'s cold-clone class had the same shape.

Related: [[a-build-script-panic-must-respect-the-bootstrap-posture]],
[[a-comment-syntax-the-frontend-does-not-know-is-a-silent-language]] (bullet 4: a filter that
matches nothing is a green run of nothing — this card is the filter that matches too little).
