---
name: feedback-ground-truth-control-must-not-pin-untracked-state
description: DISCIPLINE (LIVE-MEANS-LIVE.1b, 2026-07-31) — a ground-truth control pinned to UNTRACKED state decays SILENTLY. Measured — the done-bar audit's CTRL-4a pinned an error line recorded in rust/target/ and went red the moment that gate was re-run and passed, for a reason having nothing to do with the instrument it guards, while its sibling CTRL-4b (pinned to tracked script text) stayed green. A control must pin a TRACKED fact, or CONSTRUCT the state it observes via a testability seam.
metadata:
  node_type: memory
  type: feedback
---

The sharper sibling of [[feedback_instrument_needs_ground_truth]]. That record establishes that an
instrument must carry controls reproducing facts the project measured independently. This one is
about **where those facts are allowed to live**.

## The founding case

`docs/tasks/artifacts/done_bar/run_done_bar_probes.sh` carried two ground-truth arms:

| arm | what it pinned | where that fact lived | outcome |
|---|---|---|---|
| `CTRL-4a` | the string `computed 'In Progress' but tracker says 'Done'`, recovered from a status gate that RAN AND DIED in aggregate run 3 | `rust/target/…/regex_parser_family_status_gate.log` — **untracked build output** | ⛔ **went RED** |
| `CTRL-4b` | *"VHDL's only external-corpus lane is a TRIAGE gate"* | derived from **tracked** gate-script text | ✅ still green |

The driver was found at **11/12, exit 1** at commit `ce1df2b0` — before any edit. Root-caused with
two commands rather than assumed:

```
$ wc -c rust/target/regex_parser_family_status_gate/summary.txt          # 7377  (non-empty)
$ grep -c '^error:' rust/target/sota_exit_gate/logs/regex_…_gate.log     # 0
```

The regex gate had since been re-run and **passed**. So `find_artifact()` now finds a summary, and
`gate_ran_and_failed()` — the only code path that can emit the pinned string — is never reached.
The arm was green only until somebody ran a gate.

## Why this is worse than an ordinary stale test

1. **It decays for an unrelated reason.** Nothing about the audit changed. Someone ran an unrelated
   gate and a control guarding roster derivation went red.
2. **It is unreproducible on a fresh clone.** `rust/target/` is regenerable build output; a clean
   checkout has no such log, so the arm cannot pass there *at all* — the instrument is uncalibrated
   exactly where calibration matters most.
3. **The failure teaches nothing.** A red arm that means *"the build tree moved on"* trains the
   reader to ignore red arms — which is the failure mode a control block exists to prevent.

## The discipline

> **A ground-truth control must pin a TRACKED fact, or CONSTRUCT the state it observes.**

- **Prefer tracked.** Derive the fact from tracked sources — script text, grammars, manifests,
  registries. `CTRL-4b` and the register/registry cross-checks are this shape and do not decay.
- **Otherwise, construct it.** If the control must observe a *runtime* state (a gate that died
  mid-run, a 0-byte artifact, a stale mtime), give the instrument a **testability seam** and have
  the probe build that state from scratch. `CTRL-4a` was rebuilt this way — a synthetic target dir
  containing exactly the 0-byte-summary + error-log shape it wants — via a new
  `PGEN_DONE_BAR_TARGET_DIR` env seam. It now reproduces on an empty `rust/target/`.
- ⛔ **Never "fix" it by relaxing the assertion** to whatever the tree currently says. That converts
  a control into a mirror of the state it was supposed to judge — the same vacuous-floor shape as
  generating a claim from the gate that checks it.
- **Seam consistency is part of the fix.** Adding the seam is not enough: *every* path that reads
  the same state must honour it. The first cut left one control reading the real tree while the
  lookups read the seam, so it fired for gates it was not looking at — caught by running the probe
  driver, not by review.

## The tell

Ask of each control: **"could this go red without the instrument changing?"** If yes, its ground
truth is in the wrong place. Artifact directories, caches, logs, `mtime`s and anything under a
build-output root are all in the wrong place.

Related: [[feedback_instrument_needs_ground_truth]] (the parent discipline),
[[reference_self_referential_assertion_is_unsound]] (the other way a control quietly stops
asserting), [[feedback_delete_reclaimable_artifacts_regularly]] (build output is *meant* to be
deleted — which is precisely why nothing may depend on it).

Live instances: `scripts/audit_done_bar.sh` (`PGEN_DONE_BAR_TARGET_DIR`,
`PGEN_DONE_BAR_GRAMMARS_DIR`) and `docs/tasks/artifacts/done_bar/run_done_bar_probes.sh`
(`CTRL-4a`). Evidence: `docs/tasks/LIVE-MEANS-LIVE.md` leaf `.1b`.
