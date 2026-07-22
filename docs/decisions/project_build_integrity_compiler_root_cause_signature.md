---
name: project-build-integrity-compiler-root-cause-signature
description: DOCTRINE EXTENSION (2026-07-22, session #196) — the task-acceptance ROOT-CAUSE gate now recognizes COMPILER evidence (`error[EXXXX]`, `could not compile`) as a valid diagnosis signature, because a target that no longer BUILDS is root-caused by the compiler, not by a correctness tool or a profiler.
metadata:
  node_type: memory
  type: project
---

**What changed.** `scripts/check_diagnosis_evidence.sh` — the `DIAGNOSIS_SIG` alternation that must
back a ticked **ROOT CAUSE (WHY + WHERE)** box in a code change's acceptance checklist gained a THIRD
group of tokens: `error\[E[0-9]{4}\] | could not compile`. Group 1 is correctness-defect diagnosis
(`CERTIFICATE-COVERAGE:`, `[plannable-probe]`, `furthest_position=`, `--trace-rules`, …); group 2 is
performance-defect diagnosis (`self-time`, `call-graph attribution`, `flamegraph` — see
[[project_speed_phase_profiler_root_cause_signature]], the precedent this extension follows exactly).

**Why (the gap a BUILD-INTEGRITY defect exposed).** `BIN-BUILD-INTEGRITY.1` repaired
`rust/src/bin/ebnf_dual_run_diff.rs`, which had not compiled since `-0212` (the `ParseNode.span` →
`Span{u32,u32}` migration missed this one feature-gated binary). For that defect class the diagnosis
tool IS the compiler: `error[E0308] expected usize, found u32` at `:167`/`:168` names both the WHY
(a type migration missed a call site) and the WHERE (exact file:line) with more precision than any
other instrument could. There is no parse to trace and no run to profile — the target does not
build. The enforcer's recognized-evidence list had no token for this, so a truthfully-ticked,
genuinely compiler-diagnosed ROOT CAUSE box false-failed the gate — the same shape of false failure
the SPEED phase hit in 2026-07-11.

**Why extend the enforcer rather than work around it.** The lower-hierarchy alternatives were both
worse. (a) Bolting an unrelated correctness token onto the leaf just to satisfy the grep would
MISATTRIBUTE the diagnosis (no cert/probe/trace diagnosed this) — a workaround forbidden by
[[feedback_no_workarounds_fix_hierarchy]], and it would recur on every future build-integrity fix.
(b) Using the documented `PGEN_DIAG_EVIDENCE_WAIVER` escape hatch would land a commit that the same
script REJECTS when CI re-runs it over the range — kicking a known failure downstream rather than
closing it. Per the fix hierarchy the correct fix is the GENERAL one: teach the enforcer that the
compiler IS a valid root-cause tool for a build defect.

**Why this does not weaken the gate.** The tokens are verbatim rustc output. `error[EXXXX]` requires a
real four-digit rustc error code; `could not compile` is rustc's exact failure line. Neither can be
produced by casual prose (verified: `"we ran cargo check and it was fine"` does NOT match, while
`"error[E0308] expected usize"` does), so quoting one means a real compiler diagnostic was actually
read. Deliberately EXCLUDED as too loose: `cargo check` / `cargo build` (they appear in ordinary prose
and in every build recipe), which would have been the easy but gate-weakening choice.

**Scope / limits.** Only the ROOT CAUSE box's diagnosis-signature set changed. The ADDRESSED box
(before→after on the symptom — for a build defect that is "the check now exits 0 across every target",
ideally plus a functional run) and the NO REGRESSION box (unchanged correctness global-gate
signatures) are untouched. `scripts/*` is outside the enforcer's own `code_changed` trigger set
(`grammars/*.ebnf | rust/src/* | generated/* | ast_shape_contract/*.json`), so this edit did not itself
require an acceptance checklist. **Surfaced to the director as a doctrine evolution — open to veto or
refinement**, exactly as the profiler extension was.

**Watch item.** The enforcer's `NOREGRESS_SIG` includes the bare token `clippy`, which matched
INCIDENTAL PROSE elsewhere in the `BIN-BUILD-INTEGRITY` leaf rather than the NO REGRESSION box itself
— i.e. that check can pass on a mention anywhere in the file. The leaf was corrected to carry a real
measured clippy result, but the underlying looseness (whole-file grep, not box-scoped) is a known
soundness gap in all three signature groups and is worth a future hardening slice (scope the grep to
the ticked box's own bullet).
