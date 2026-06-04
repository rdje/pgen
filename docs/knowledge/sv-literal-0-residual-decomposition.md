---
id: sv-literal-0-residual-decomposition
title: The SV literal-0 residual — what it is, why it is not yet 0, and the two named causes
answers:
  - "why is the SystemVerilog main parser still Mostly Done not Done"
  - "what is the closed-loop replay residual / focused_replay_target_debt_zero"
  - "why is the SV stimuli residual not literal-0"
  - "what is the property_expr construction-cost residual"
  - "what are the non-covering branch targets in SV stimuli coverage"
  - "what does the witness construction speedup (Cow) do"
tags: [stimuli, systemverilog, coverage, literal-0, property_expr, construction]
date: 2026-06-04
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (generate_target_witnesses construct_mode + strip_probability_prefix Cow + timeout_failure_samples/unresolved_after_samples); docs/tasks/SV-EXH-PROOF.md leaves .7.4.6.3/.4/.5/.6 (PGEN-SV-EXH-PROOF-0148..0151)
reverify: run the witness pass (--target-report-input <150-sample> --target-max-attempts 0 --target-generation-timeout-ms 200 --seed 712001 --output /tmp/o.json) and read the "Witness 'target_timeout'-failure samples" + "still-UNRESOLVED samples" stdout lines; or `make sv_stimuli_quality_gate` for closed_loop_replay_targets_total
---

The `systemverilog main parser` is `Mostly Done`, not `Done`, because of ONE remaining
Done-criterion: `focused_replay_target_debt_zero` — the closed-loop replay leaves
`closed_loop_replay_targets_total > 0` coverage targets. The residual fell ~2660 → 888 (.7.2
steering plateau) → 753 (.7.4.4 Purdom ordering) → 273 (.7.4.5 dedicated budget) → **97**
(.7.4.6.3 derivation-directed CONSTRUCTION, `construct_mode`). See
[[sv-witness-purdom-ordering]] and [[stimuli-residual-coverage-model]].

**The 97 is fully DECOMPOSED (PGEN-SV-EXH-PROOF-0150/0151) into two distinct, named causes**
— pinned by tool, not guessed (the witness pass now SAMPLES the `target_timeout` class with
construct-vs-search attribution, and the still-UNRESOLVED set incl. the non-covering class):

1. **property_expr deep-branch TIMEOUTS** (the bulk). All sampled timeouts are branches of
   `property_expr_sv_2017` (the single most recursive SV rule), `construct_reason=TargetTimeout`
   — construction does NOT structurally dead-end (no DepthExceeded/RuleVisitLimit); its minimal
   derivation tree is just genuinely large, so it exceeds the gate's tiny per-witness budget.
   **Budget-solvable:** a post-Cow witness-floor sweep (150-sample) resolves them at ~3000 ms
   (parser-agnostic env `PGEN_WITNESS_TIMEOUT_FLOOR_MS`), and the .7.4.6.5 **2× speedup** makes
   a raised gate floor affordable.

2. **NON-COVERING branches** (the small hard core). A witness IS generated (construct Ok, no
   error/timeout) but does not cover the target — invisible to the failure counters until the
   `unresolved_after_samples` tool surfaced them. On the 150-sample: exactly 4 —
   `net_type_declaration_sv_2017` #0/#1 (overlapping branches: branch 0's `data_type` can match
   branch 1's `net_type_identifier net_type_identifier` form) and `sequence_expr` #4/#8 (a
   LEFT-RECURSIVE rule, eliminated automatically at transform-time, see [[pgen-parsing-model]]).
   **Root cause NOT yet pinned** (needs a branch-coverage-mechanism trace — esp. why branch 0,
   the FIRST ordered choice, is uncovered); live hypotheses: PEG ordered-choice SHADOWING (a
   later branch is dead → PARSE-SOTA `.9` shadowing-lint territory) and/or LR-transform
   branch-index artifacts → those targets may be genuinely PEG-UNREACHABLE and should be
   classified out of the target universe (proven rigorously, NOT gamed per
   [[parser-signoff-four-pillars]]).

**What landed (all parser-agnostic + monotone):** `construct_mode` (build the minimal
derivation tree directly, backtrack-free) cut 273→97; the `strip_probability_prefix` → `Cow`
refactor removed the per-`generate_or` clone of every alternative (profiled construction hot
spot) for a **2.1× construction speedup** that benefits EVERY parser's generation. DISPROVEN
and reverted (no codebase change on a disproven hypothesis): OR-width + quantifier-count
limited-backtrack (k=1..8 no effect) and the budget-only lever (200 ms = 1000 ms).

**Lever map to literal-0** (leaf `.7.4.6.6`): (A) raise the gate witness floor (cheap
post-Cow) → resolves property_expr; (B) trace + fix/classify the non-covering branches. Until
both close, the honest status is `Mostly Done` with a characterized, two-cause remainder — never
over-claimed ([[parser-signoff-four-pillars]]).
