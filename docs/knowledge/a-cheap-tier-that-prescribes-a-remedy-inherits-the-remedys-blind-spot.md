---
id: a-cheap-tier-that-prescribes-a-remedy-inherits-the-remedys-blind-spot
title: A cheap gate tier that prescribes a remedy inherits the remedy's blind spot — and the recording step makes the wrong answer permanent
answers:
  - "my cheap tier fires on any change to the inputs — doesn't that protect the expensive oracle"
  - "the gate breached, I did exactly what its message told me to do, and the tree is still wrong"
  - "is a two-tier gate sound if its cheap tier's identity set is complete"
  - "where does a duplicated fact inside a gate actually do its damage"
  - "should a check fall back to a default when it cannot derive a fact"
  - "why is a mirror inside an oracle worse than a mirror beside one"
  - "what must a --rebaseline / --accept path refuse to record"
tags: [gates, doctrine, two-tier-checks, false-pass, ops-build-flow, claim-verification]
date: 2026-08-17
status: current
evidence: |
  ENGINE-UNIVERSAL-SERVICES.33 slice 1, session #244 (docs/tasks/ENGINE-UNIVERSAL-SERVICES.md, and
  docs/tasks/artifacts/engine_universal_services/es33_makefile_mirror/measurements.md M1/M2/M3).
  GENERATED-REPRODUCIBILITY tier 2 held its own copy of rust/Makefile's generator flags. With
  `--indirect-lr-admit-starvation-safe-only` added to RUST_GENERATOR and the artifacts left alone:
  tier 1 breached at rc 1; `--rebaseline` (what the breach message instructs) re-derived with the
  STALE flags, matched, and RECORDED the baseline at rc 0; tier 1 then reported OK. `make` would
  have emitted a 130 878 616 B SystemVerilog parser against the 143 072 420 B on disk.
reverify: "bash scripts/check_generated_reproducibility.sh --self-test   # 18/18, incl. 7 REFUSE(2) arms proving the oracle now refuses rather than falling back; and `make -C rust SHELL=/bin/bash generated_reproducibility_gate | grep 'recipe DERIVED from'` shows the recipe is read, not mirrored"
related:
  - an-unmeasured-honest-bound-is-an-unverified-claim-wearing-a-humble-face
  - an-ab-whose-arms-are-secretly-identical-does-not-fail-it-passes
  - an-instrument-firing-is-not-the-defect-reproducing
  - a-gate-must-be-able-to-fail-and-able-to-run
  - derive-the-comparison-key-from-the-artifact-not-from-the-caller
---

# A cheap tier that prescribes a remedy inherits the remedy's blind spot

The two-tier gate is a good shape: a **cheap identity** tier that runs on every commit and proves
*"nothing that could have changed the answer has changed"*, plus an **expensive oracle** tier, run on
demand, that establishes the answer. The cheap tier is honest about inheriting whatever the oracle
last established — that bound is usually written down.

What is easy to miss is the other direction. The cheap tier does not merely *inherit* the oracle's
authority; when it breaches it **prescribes re-running the oracle**. So if the oracle has a blind
spot, the cheap tier is not a second line of defence against it — it is the mechanism that walks the
operator into it, and the oracle's `--rebaseline` step then **records** the wrong answer, which
silences the cheap tier for good.

## The measured instance

`GENERATED-REPRODUCIBILITY` (`ENGINE-UNIVERSAL-SERVICES.33`). Tier 1 re-hashes the emission sources,
which include `rust/Makefile`. Tier 2 re-derives every generated artifact and demands byte-identity.
Tier 2 held its **own copy** of the Makefile's generator flags instead of reading them.

A flag was added to the Makefile's `RUST_GENERATOR` and the artifacts were left alone — the ordinary
state during a recipe change:

| step | outcome |
|---|---|
| tier 1 | ✅ breached: *"the EMISSION SOURCES moved"*, rc 1 |
| the operator does exactly what the breach message says: `--rebaseline` | ⛔ tier 2 re-derives with its **stale** flags, matches, prints `✓ systemverilog re-derives byte-identically`, **records the baseline**, rc 0 |
| tier 1, again | ⛔ `OK … tier 2 last proved them byte-identical to HEAD`, rc 0 |

What `make` would actually have emitted: **130 878 616 B** against the **143 072 420 B** on disk —
a different left-recursion admission policy in the shipped SystemVerilog parser. Three green lines,
one wrong artifact, and the alarm permanently reset.

## Why "the identity set is complete" is not the property you need

`emission_sha` was complete: it covered the Makefile, so the change *was* detected. Completeness of
the identity set buys detection of **movement**. It buys nothing about whether the oracle can judge
the new state — and the cheap tier's message hands the judgement straight to the oracle.

## What to do

- **A mirror inside an oracle is strictly worse than a mirror beside one.** Everything downstream
  inherits what the oracle last established, so the oracle is the one place duplication must not sit.
  Derive the fact; do not keep a second copy in step with it.
- **When a check cannot derive the fact, make it REFUSE, never fall back.** A fallback that is right
  today is indistinguishable from one that is silently wrong tomorrow, and the wrong direction here
  is a pass.
- **Read every cheap tier's breach message as a specification of the oracle.** If it says *"re-run
  X"*, then X's blind spots are this tier's blind spots. Audit X for duplicated inputs before
  trusting the pair.
- **The recording step is the one that makes it permanent.** A `--rebaseline` / `--accept` path must
  refuse on anything less than a clean, complete verification — a partial or breaching run recorded
  as a baseline converts a transient wrong answer into the new definition of right.
