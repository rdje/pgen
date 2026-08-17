---
id: an-unmeasured-honest-bound-is-an-unverified-claim-wearing-a-humble-face
title: An "honest bound" you reasoned rather than measured is an unverified claim — and because it errs modestly, re-reading it will never catch it
answers:
  - "do the limitations I write next to a fix need the same verification as the numbers"
  - "I wrote an honest bound from how I think the tool works — is that good enough"
  - "my fix's stated limitation says it does not close the demo it was written for — now what"
  - "why would an understated claim be dangerous if it errs on the safe side"
  - "how do I verify a negative claim like 'this cannot detect X'"
  - "what should a control's pass predicate accept"
  - "a probe printed a verdict that contradicted its own evidence — which do I trust"
tags: [evidence, measurement, claim-verification, controls, gates, honest-bounds]
date: 2026-08-17
status: current
evidence: |
  ENGINE-UNIVERSAL-SERVICES.32, slices 1 and 2, session #243. Slice 1 hardened
  `GENERATED-REPRODUCIBILITY` tier 2 by invoking cargo, and published, in the check's header and in
  its leaf: *"it cannot detect a binary hand-COPIED over cargo's output path, because cargo keys on
  its own fingerprint of the sources rather than on the output bytes."* Reasoned, never tested.
  Measured under a director challenge, twice, two perturbations — replacing the binary with a
  different valid one, and truncating it to 1 000 bytes — cargo restored byte-identity in **0.57 s**
  both times, because `target/debug/<bin>` is a hardlink/copy of `target/debug/deps/<bin>-<hash>`
  that cargo re-establishes every run. ⭐ The consequence was not cosmetic: the false bound said the
  fix did NOT close the constructed demonstration that opened the leaf. Re-run end to end, it does —
  the gate goes from `✓ json re-derives byte-identically (208 sites)` to a refusal at exit 2.
reverify: "cp rust/target/debug/ast_pipeline_bootstrap rust/target/debug/ast_pipeline && (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline) && ls -la rust/target/debug/ast_pipeline   # restored to the real 216 MB artifact in <1 s"
---

**A project that verifies every number it publishes can still publish an unverified sentence.**
`docs/CLAIM_VERIFICATION.md` asks *"is this number earned?"*. The limitation you write underneath it
is a claim about the world too — usually a *negative* claim ("this cannot detect X", "this does not
cover Y") — and negative claims are exactly the ones that feel too obvious to test.

## Why this one survives review

An honest bound is written in the **modest** direction by construction: you are saying your work is
weaker than it looks. That earns it three protections no overclaim gets.

1. **It reads as integrity.** A reviewer skims it as evidence of care, not as an assertion to check.
2. **Its failure direction is conservative** — the fix turns out *stronger* than advertised — so
   nothing downstream breaks, no gate goes red, no number disagrees with another number.
3. **It is self-consistent.** Nothing in the tree contradicts it, because it describes an absence.

⇒ the ordinary tripwires (a gate, a second instrument, a co-published figure that must agree) are
all blind to it. The only thing that catches it is running the experiment.

## The cost is not the sentence, it is what the sentence licenses

Measured here: the false bound implied the adopted fix did **not** close the demonstration that
justified the leaf. A reader asking the obvious question — *"does this actually fix the thing you
showed me?"* — would have been told **no**, by the author, in the check's own header. A modest
sentence had quietly reduced the value of a real fix to zero on the page.

⛔ So the test is not "would this bound embarrass me if wrong" but **"what does this bound tell a
reader they still have to worry about?"** If the answer is "the thing I just fixed", measure it.

## How to verify a negative claim

- **Perturb and observe, twice, differently.** One perturbation tests your model of the mechanism;
  two different ones test the *claim*. Here: replace the artifact with a valid substitute, and
  truncate it. Same verdict from both = a property, not a coincidence.
- **Take the claim end to end.** "Cargo cannot see X" is a fact about cargo; "the gate still passes"
  is the fact that matters. They can differ — and they did, because a *different* assertion in the
  same gate caught what cargo's repair then exposed.
- **Name the legs you did not run**, as the standard already requires for numbers. A bound with no
  experiment behind it should say so in the same breath.

## ⚠️ And check what your control's predicate accepts

The probe written to settle this printed **`✗ the fix does NOT close M1`** — because its pass
predicate accepted only `rc=1` (a breach) while the real, correct outcome was `rc=2` (a refusal).
Both are non-passes; only one was in the predicate. Reading the verdict line instead of the evidence
beneath it would have "confirmed" a retraction that was itself wrong.

> **A control's predicate must enumerate every outcome that counts as passing — not the one its
> author expected to see.**

Related: [[a-conservation-control-cannot-catch-a-misassignment]],
[[a-check-whose-inputs-all-pass-has-not-been-tested]],
[[a-mutation-control-tests-the-layer-you-mutated-not-the-property]],
[[a-determinism-control-cannot-see-a-tool-that-changed-between-its-arms]]. Those four are about
controls that cannot fail, or fail on the wrong thing. This one is about the **prose beside them**,
which no control watches at all.
