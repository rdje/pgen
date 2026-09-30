---
id: a-positive-control-built-on-a-live-defect-rots-when-the-defect-is-fixed
title: A positive control built on a live defect rots when the defect is fixed — calibrate a detector on a synthetic input it cannot lose
answers:
  - "why does silent_success_sentinel_gate print MISCALIBRATED"
  - "why did CAL-2-latent-is-visible start reading 0 sentinels"
  - "how should a detector's positive control be built so it does not rot"
  - "why did sota-exit-gate go red in the CI mirror"
  - "is there a latent invalid_sequence_access site left in the SystemVerilog grammar"
tags: [gates, calibration, controls, silent-success, sentinel, sota-exit-gate, ci_workflow_local_gate]
date: 2026-09-30
status: current
evidence: "DONE-BAR.5c correction (PGEN-DONE-BAR-0026): the sentinel gate's positive control parsed `this.x` at --entry-rule constraint_primary_sv_2017 and expected the ONE sentinel SV-AST-SHAPE-FIDELITY.2.4 had classified LATENT there; SV-CORPUS-GRAD.3.42 (8e8435db9, 2026-09-06) fixed the site, every other LATENT candidate of the .2.4 table reads 0 today under isolation (measured with the current probe AND the pre-.116 probe), and the gate sat MISCALIBRATED for 24 days until the push lane's mirror replayed sota-exit-gate on 2026-09-30"
reverify: "make -C rust SHELL=/bin/bash silent_success_sentinel_gate 2>&1 | grep -E 'CAL-2|MISCALIBRATED|^silent_success_sentinel_gate'   # CAL-2-every-sentinel-renders: observed 10 == expected 10"
---

A detector needs a positive control — a zero reads as a pass either way — and the sentinel gate's
founding control was a REAL latent defect: an SV rule that rendered `<invalid_sequence_access>`
only under `--entry-rule` isolation. Real defects get fixed. When `SV-CORPUS-GRAD.3.42` repaired
that site the control silently went to 0, the gate refused every sweep as `MISCALIBRATED`, and
nothing noticed for 24 days because the gate runs only inside `sota-exit-gate`, which nothing
between pushes replays.

- **Build the positive control on a synthetic input the detector cannot lose.** The dump path
  is what can go blind, so the probe now writes one root node per roster literal through the very
  serializer `--parse-dump-ast` uses (`parseability_probe --sentinel-calibration-dump <out>
  <literal>...`), and the gate requires EACH literal rendered exactly once — no grammar, no parse,
  no defect to rot.
- **Keep the founding site as the NEGATIVE control, and say why it reads 0.** `this.x` under
  `constraint_primary` now reads 0 in isolation as well as canonically because it is FIXED, not
  shadowed; the contract's `why` says so, or the next reader re-derives the stale story.
- **A ground-truth fact that names a live defect owes a re-check when that defect's leaf closes.**
  The `.2.4` LATENT table (7 sites) is fully green today; its calibration note in the shape manifest
  still describes them as latent. Grep the gate contracts for the rule you are fixing before you
  close the leaf.
- **Two findings sat behind one red gate.** The same run showed the semantic-annotation
  meta-grammar reaching 46 sentinels on its own stimuli (`power_expr`'s `$3` over a two-element
  body, since 2025-09-01) — a second defect the calibration failure had been hiding, because a
  miscalibrated gate offers no sweep verdict at all.

Related: [[a-filtered-acceptance-run-is-not-the-module]],
[[a-build-script-panic-must-respect-the-bootstrap-posture]],
[[porting-a-predicate-to-a-new-population-is-a-re-derivation]].
