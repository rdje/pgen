# SV-EXH-PROOF.1 — Measured Baseline & Scope Lock (`2026-05-17`)

This is the deterministic measured baseline for the `SV-EXH-PROOF`
task tree (leaf `SV-EXH-PROOF.1`). It records the **true current
state** of the SystemVerilog main-parser exhaustive-closure surface as
measured on `main` (`fadf9107`), against which the live tracker is
reconciled. Every number below was produced by the canonical Makefile
gate targets with default policy (no bounded env overrides).

## How measured

```
make -C rust SHELL=/opt/homebrew/bin/bash sv_external_corpus_triage_gate
make -C rust SHELL=/opt/homebrew/bin/bash sv_syntax_closure_gate
make -C rust SHELL=/opt/homebrew/bin/bash sv_preprocessor_syntax_closure_gate
make -C rust SHELL=/opt/homebrew/bin/bash sv_formal_exhaustive_closure_gate   # FAILS (see Finding A)
```

## Findings

### Finding A — preprocessor syntax-closure is REGRESSED on `main` (hard prerequisite blocker)

`sv_preprocessor_syntax_closure_gate` **fails** (exit 2), clean
standalone run:

```
reachable_rules: 72
unreachable_rules: 1
reachable_branches: 37
unreachable_branches: 13
❌ systemverilog_preprocessor syntax closure gate failed with 1 violation(s):
  - unreachable_branches=13 > max_unreachable_branches=3
```

Provenance: `grammars/systemverilog_preprocessor.ebnf` was last
modified by **this session's** `PGEN-POST-SV-AUDIT-0002`
(POST-SV-AUDIT.2.1, `macro_formals` Category-A fix) and
`PGEN-INLINE-ALT-FIX-0001` (SVPP-0001, `pp_if_branch` inline-alt fix).
The matching contract
`rust/test_data/grammar_quality/systemverilog_preprocessor_syntax_closure_contract.json`
(`max_unreachable_branches: 3`) was last touched by the unrelated
`47e97606 "Add SV preprocessor syntax closure gate"` — i.e. it was
**never re-baselined in lockstep** with those grammar edits. The Cat-A
record-rule factoring + inline-alt lifting legitimately added named
rules/branches and shifted the static reach-set, raising
`unreachable_branches` from ≤3 to 13.

Consequence: this is a **real lockstep defect from a campaign
committed as Done** (POST-SV-AUDIT / INLINE-ALT-FIX). It also means
`sv_parser_family_status_gate` and `sv_formal_exhaustive_closure_gate`
**cannot run to completion / green on `main`** — they abort at this
stage. SV-EXH-PROOF's entire goal (an honestly-green SV
formal-exhaustive surface) is therefore blocked on remediating this
first. Owned by new leaf `SV-EXH-PROOF.2`.

### Finding B — SV-main static syntax-closure is HEALTHY (re-scope decision validated)

`sv_syntax_closure_gate` **passes** (exit 0):

```
defined_rule_count: 1447
reachable_rules: 1448
unreachable_rules: 1
reachable_branches: 1925
unreachable_branches: 20
status: pass   (contract: min_total_rules 366, max_unreachable_rules 1, max_unreachable_branches 25)
```

Confirms `PGEN-SV-EXH-PROOF-0001`: the SV-main *static* syntax-closure
surface is genuinely already closed and is **not** the SV exhaustive
-proof gap. It is a no-regression baseline only (the preprocessor
-trio-port hypothesis remains correctly falsified).

### Finding C — external-corpus parse surface is 0/14, NOT 10/14 (proven-false tracker drift)

`sv_external_corpus_triage_gate` totals (canonical, default policy):

```
corpus_count: 4   cases_declared: 7   cases_executed: 14   cases_blocked_total: 0
preprocess_pass_total: 14   preprocess_fail_total: 0
parse_pass_total: 0   parse_fail_total: 14   parse_skipped_total: 0
```

Per-case (all 7 declared × {sv_2017, sv_2023}): **every case is
`parse_fail`** with `preprocess=pass` —
`uvm_pkg`, `uvm_compat_pkg`, `scr1_core_top`, `scr1_top_ahb`,
`friscv_rv32i_core`, `friscv_pipeline`, `veer_el2_lsu`.

These are **genuine grammar rejections** (`parseability_probe --parse
systemverilog … --profile …` → "Parser did not consume full input at
position N"; e.g. `scr1_core_top@2017` pos 1724, `veer_el2_lsu@2017`
pos 947, `friscv_pipeline@2017` pos 135), produced by a freshly-built
probe over the parser generated from `grammars/systemverilog.ebnf`
via the canonical gate — **not** a probe/build artifact.

The pre-baseline `LIVE_ACHIEVEMENT_STATUS.md` "`systemverilog` main
parser" row stated `parse_pass_total=10`, `parse_fail_total=4`, and
that `scr1_top_ahb`, `friscv_rv32i_core`, `friscv_pipeline`,
`scr1_core_top`, `el2_lsu` were "green in both `sv_2017` and
`sv_2023`". Measured reality is `0/14`. This is a tracked
tracker↔codebase correctness defect; the tracker is corrected to the
measured truth same-commit (the user's only window — no drift, no
compromise). The exact regressing commit is triage work owned by the
grammar-hardening leaf, not this baseline.

### Finding D — formal-exhaustive closure-green is a hard-coded literal (unproven claim)

`rust/scripts/sv_formal_exhaustive_closure_gate.sh:245` sets
`external_corpus_backed_proof_surface_present=true` as a **literal**,
then derives `systemverilog_formal_exhaustive_closure_surface_green=true`
and `closure_criteria_satisfied_count=1` from it. The gate runs
`sv_external_corpus_triage_gate` and captures every metric
(`…_parse_fail_total`, etc.) into its summary, but the closure-green
boolean **does not consume any of them**. The SV
`systemverilog_formal_exhaustive_closure_contract.json` still
documents the surface as *missing*
(`required_surface_missing_detail`). So the closure claim is
currently doubly unfounded: asserted by a literal `true` (D) over a
real `0/14` parse surface (C). (Runtime not observable here because the
gate aborts earlier at Finding A.)

## Scope-lock conclusions

1. SV-main static syntax-closure: **already closed** — no-regression
   baseline only (Finding B).
2. SV `Done` sole closure criterion = `external_corpus_backed_proof_surface`.
   Honoring it honestly requires (i) replacing the hard-coded
   `surface_present=true` with a derived per-case proof contract
   (Finding D), over (ii) a corpus parse surface that is genuinely
   green — which today is `0/14` (Finding C), so genuine SV grammar
   hardening on real-world UVM/scr1/friscv/veer corpora is required,
   not a "promote a mostly-green bounded surface" exercise.
3. **Prerequisite blocker:** the preprocessor syntax-closure
   regression (Finding A) must be remediated first — the SV
   family-status / formal-exhaustive umbrella cannot be green until
   then.
4. The LIVE tracker drift (Finding C) is corrected same-commit.

## Re-planned `SV-EXH-PROOF` tree (post-baseline)

| Leaf | Goal |
| --- | --- |
| `.1` | **(this)** measured baseline + scope lock + mandatory LIVE-drift correction (docs-only) |
| `.2` | Remediate the preprocessor syntax-closure regression (Finding A) — re-baseline contract + honestly classify the newly-unreachable branches via the proven preprocessor zero-plausible-gap pattern; restore `sv_preprocessor_syntax_closure_gate` + the SV family-status/formal-exhaustive umbrella to green |
| `.3` | SV grammar hardening: drive the external-corpus parse surface `0/14 → green` (multi-slice; per-corpus/defect sub-leaves; each grammar fix probe-verified + lockstepped) |
| `.4` | Build the real `external_corpus_backed_proof_surface`: derive `surface_present` from a checked-in per-case disposition contract over `sv_external_corpus_triage_gate` (replace the hard-coded `true`; no false closure) |
| `.5` | Wire the surface into `systemverilog_formal_exhaustive_closure_contract.json` + `sv_formal_exhaustive_closure_gate.sh` + `sv_parser_family_status_gate` + `sota_exit_gate` + `sv_combined_telemetry_contract_gate` parity |
| `.6` | LIVE rows → `Done` with the machine-checkable surface as evidence; SV book + integration contract same-commit lockstep; closeout |

Every grammar / contract / gate change above is task-tree-leaf-owned
(Code-Change Doctrine). This baseline doc changed no code.
