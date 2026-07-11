---
name: project-speed-phase-profiler-root-cause-signature
description: DOCTRINE EXTENSION (2026-07-11, session #91) — the task-acceptance ROOT-CAUSE gate now recognizes PROFILER evidence (self-time / call-graph / flamegraph) as a valid diagnosis signature, because the SPEED phase makes perf defects first-class and they are root-caused by a profiler, not by a correctness tool.
metadata:
  node_type: memory
  type: project
---

**What changed.** `scripts/check_diagnosis_evidence.sh` — the `DIAGNOSIS_SIG` alternation that must
back a ticked **ROOT CAUSE (WHY + WHERE)** box in a code change's acceptance checklist gained a second
group of tokens: `self-time | call-graph attribution | call-graph samples | cargo flamegraph |
flamegraph`. The original set was entirely CORRECTNESS-defect diagnosis (`CERTIFICATE-COVERAGE:`,
`[plannable-probe]`, `furthest_position=`, `--trace-rules`, `--lint-grammar`, `--parse-dump-ast`, …).

**Why (the gap the SPEED phase exposed).** The director elevated **SPEED to a first-class,
continuously-tracked deliverable** (2026-07-11 — see [[feedback_correctness_before_speed]] elevated +
[[project_rgx_0078_regex_slowness_followup]]). A slowness defect is legitimately diagnosed by a
**profiler** (macOS `sample`, `cargo flamegraph`) via **self-time / call-graph attribution** — that is
the "WHY + WHERE" for a perf defect, exactly as a cert/probe/trace is for a correctness defect. The
acceptance-checklist enforcer predated perf work, so its recognized-evidence list had NO profiler
token. The first SPEED-phase *code* change (RGX-0078.3, the `rollback_to_named` scope-restoration
guard) hit this: its ROOT CAUSE was genuinely profiled (`sample` call-graph: `rollback_to_named` ≈ 24%
of regex parse self-time), the box was truthfully ticked and backed by that profile, yet the gate
false-failed because "self-time" was not a recognized signature.

**Why extend the enforcer rather than work around it.** The lower-hierarchy alternative — bolt an
unrelated correctness token (e.g. `--report-certificate-coverage`) onto every future perf leaf just to
satisfy the grep — is a WORKAROUND that MISATTRIBUTES the diagnosis (the cert did not diagnose the
slowness) and would repeat on every SPEED-phase change. Per [[feedback_no_workarounds_fix_hierarchy]]
the correct fix is the GENERAL one: teach the enforcer that a profiler IS a valid root-cause tool for a
perf defect. This is a non-weakening extension — it ADDS recognition for perf diagnosis without
relaxing any correctness-defect requirement, and the tokens are tight enough that they cannot match
unrelated text (notably `self-time`/`call-graph`/`flamegraph` never occur inside `sample_parse_failures`,
so the gate is not weakened).

**Scope / limits.** Only the ROOT CAUSE box's diagnosis-signature set changed. The ADDRESSED box
(before→after on the symptom — for a perf change that is the before→after geomean) and the NO REGRESSION
box (still the correctness global-gate signatures: seeds 0/7/42, byte-identical, `fully_certified`,
`spf=0`, shape-contract, clippy) are UNCHANGED — a speed lever still must prove the correctness floor is
intact, which is the ⛔ HARD constraint of the SPEED phase. `scripts/*` is outside the enforcer's own
`code_changed` trigger set (`grammars/*.ebnf | rust/src/* | generated/* | ast_shape_contract/*.json`), so
this enforcer edit did not itself require an acceptance checklist. Surfaced to the director as a doctrine
evolution (open to veto/refinement).
