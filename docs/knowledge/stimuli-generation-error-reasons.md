---
id: stimuli-generation-error-reasons
title: Stimuli generation error-reason taxonomy (and severity is never gated by verbosity)
answers:
  - "what do the stimuli generation error reasons mean"
  - "what is GenerationErrorReason / classify_generation_error"
  - "how do I tell a depth failure from a visit-limit or timeout failure"
  - "where are depth_exceeded_errors / target_timeout_errors counted"
  - "are warnings and errors hidden at low verbosity"
tags: [stimuli, diagnostics, severity]
date: 2026-06-03
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (GenerationErrorReason, classify_generation_error, *_errors counters, TargetDriveSummary); rust/src/ast_pipeline/mod.rs (Severity, emit_diagnostic, pgen_warn!/error!/fatal!); docs/tasks/DIAG-SEVERITY.md
reverify: grep -n "GenerationErrorReason\|classify_generation_error" rust/src/ast_pipeline/stimuli_generator.rs
---

A generation attempt that fails is classified into a **canonical reason** by
`classify_generation_error` → `GenerationErrorReason`:

- `DepthExceeded` — hit `max_depth` (see [[sv-residual-depth-budget-cause]]).
- `RuleVisitLimit` — hit the second budget, `max_rule_visits` (default 8).
- `TargetTimeout` — the primary canonical-entry target-drive attempt budget expired.
- `HelperTimeout` — an alternate helper-entry probe budget expired.
- `Other` — anything else (e.g. the context-gating tail, which emits **no** message).

Counts are surfaced honestly all the way up: `TargetDriveSummary` carries
`depth_exceeded_errors`, `rule_visit_limit_errors`, `target_timeout_errors`,
`helper_timeout_errors`; validator-backed parseability reports and stimuli corpus bundles
preserve them, so a future session distinguishes "generic churn" from "budget fired"
without scraping trace by hand.

**STANDING RULE (DIAG-SEVERITY, emphatic):** severity (`Warning` < `Error` < `Fatal`) is
**NEVER** gated by a trace/verbosity level — verbosity governs `INFO` only; warnings,
errors and fatals emit unconditionally (stderr/log). The original sin this fixed: the SV
depth-exceeded error was emitted *only* as a high-verbosity trace event, so the dominant
cause of the 888 residual stayed invisible for an entire campaign. See
[[feedback_severity_never_gated_by_verbosity]].
