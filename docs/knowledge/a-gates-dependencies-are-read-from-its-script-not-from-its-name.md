---
id: a-gates-dependencies-are-read-from-its-script-not-from-its-name
title: A gate's dependencies are read from its script, not from its name — a "differential" gate was dispositioned as an operator campaign and described as needing reference tools, and both claims were false by grep
answers:
  - "why were sv_preprocessor_curated_differential_gate and sv_preprocessor_template_differential_gate run by nothing"
  - "do the SV preprocessor differential gates need verilator or iverilog"
  - "what does the SV preprocessor formal exhaustive closure umbrella run"
  - "how was DONE-BAR.3e's opening premise wrong and how was it corrected"
  - "when must an accepted-operator-invoked disposition be re-priced"
  - "what does the umbrella do with a differential stage whose diff mode is disabled"
tags: [done-bar, gate-reachability, dispositions, sv-preprocessor, toolbox-first, published-claims, umbrella]
date: 2026-10-08
status: current
evidence: "DONE-BAR.3e (`PGEN-DONE-BAR-0031`). The done-bar audit read `systemverilog_preprocessor … 9 covering gate targets; 7 run, 2 run by nothing` — the curated and template differential gates, both `accepted-operator-invoked` with the reason *run per SV campaign rather than on every commit*. `.3d` opened `.3e` saying they were differential against `verilator -E` / `iverilog -E` and so campaign instruments; `grep -n -e verilator -e iverilog -e require_tool` over both scripts finds `require_tool jq` and nothing else — the curated gate compares `ast_pipeline --preprocess systemverilog` with CHECKED-IN expected artifacts (9 cases), the template gate with a deterministic template predictor (32 generated cases). Under the guard: `passed` in 10 s and 5 s. Wired as stages of `sv_preprocessor_formal_exhaustive_closure_gate.sh` (pinned `state_dir`, counts lifted into `metrics`, paths under `proof_surfaces`, a refusal when `diff_mode_effective` is not `enabled` or zero cases were checked); the umbrella then passed end to end in 307 s with the two stages costing 5 s and the closure verdict unchanged. Reachability `99 reachable, 29 orphan` → `101 / 27`; the audit `7 run, 2 run by nothing` → `9 / 0`."
reverify: "grep -n -e verilator -e iverilog -e require_tool rust/scripts/sv_preprocessor_curated_differential_gate.sh rust/scripts/sv_preprocessor_template_differential_gate.sh (expect require_tool jq only); grep -n 'differential' rust/scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh | head (expect the two stages and the refusal); bash scripts/audit_done_bar.sh | grep -A1 '^systemverilog_preprocessor' | tail -1 (then read its leg-2 line: expect 9 run, 0 run by nothing); bash scripts/check_gate_reachability.sh (expect 27 orphan + 1 policy-only)"
---

**A gate's name is a hypothesis about what it depends on; its script is the fact.** *Differential* suggested
a reference tool, a reference tool suggested a host-only campaign, and the sentence reached three durable
surfaces before a grep was run. The repository's standing rule — verify before publishing, a challenge
must be answerable in one word — applies to a dependency as much as to a number.

- **Re-price a disposition whose reason is a cost.** `accepted-operator-invoked` carried a sentence about
  price and no measurement behind it. A price is a measurement with a date; one without a date is a
  guess that outlives the gate it describes. Both gates run in seconds, so the remedy is `.3d`'s again:
  wire the lane where the family already runs its proofs, and delete the disposition.
- **A stage that evaluated nothing is not a covering gate's green.** Both gates carry a `diff_mode` knob
  that can disable the comparison while the script still exits 0. The umbrella refuses that state
  (`diff_mode_effective` not `enabled`, or zero cases checked) rather than counting it — the same
  principle as a gate that must say NOT EVALUATED loudly instead of passing by absence.
- **Additive summary keys, readers by name.** The family-status gate and its contract gate read the
  umbrella's summary by named keys (a has-every-key check over the expected surfaces), so the new
  surfaces and counts join without a schema bump; the closure verdict stays the zero-gap proof surface the
  contract names, and a failing stage fails the umbrella through its stage runner.
- **A gate's build line is a dependency too.** Both scripts built the shared debug `ast_pipeline` with a
  narrower feature set than the Makefile's rule, so one run relinked it without the `.ebnf` frontend and
  three doctrine tiers that read that path on disk refused. Wiring a gate into an umbrella means its
  build recipe now runs inside every chain that umbrella sits in: read that line before wiring, and make
  it the repository's one recipe (`CI-PARITY-GATE-ROT.24` owns the class; its guard,
  `scripts/require_ast_pipeline_features.sh`, still has no callers).
- **The artifact-backed audit reads STALE the moment the umbrella moves.** The script is now an input of
  the family's verdict; artifacts produced before the edit are older than their inputs, and the audit says
  UNPROVEN until the next run. That is the instrument working, and the record quotes it rather than the
  earlier MET.
