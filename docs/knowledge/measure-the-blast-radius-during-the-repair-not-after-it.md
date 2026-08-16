---
id: measure-the-blast-radius-during-the-repair-not-after-it
title: A blast radius that only exists in the transition must be measured DURING the repair — repair first and the baseline is gone, measure first and there is nothing to compare
answers:
  - "how do I prove a fix changed only what it was supposed to change"
  - "I argued my change is contained — how do I turn that argument into evidence"
  - "when should I snapshot before applying a fix"
  - "how do I measure the blast radius of regenerating a derived artifact"
  - "my change is obviously safe; is a before/after really worth it"
  - "what order do repair and verification go in"
tags: [evidence, measurement, derived-artifacts, controls, process, verification]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.29 slice 1, 2026-08-16 session #241. Two derived artifacts (the annotation parsers the code generator's own backend links, i.e. the pair that participates in generating every other parser) carried a line the tracked generator could not emit. The leaf argued their divergence could not reach any other artifact, because the line sits inside `enable_coverage()` and nothing on the parse path reads the field it touches - and recorded, explicitly, that this was an argument and not a byte comparison. One script turned it into one - snapshot all 8 downstream artifacts, repair through the canonical target, relink the generator against the repaired pair with its feature surface asserted, regenerate all 8 at the build system's own `-o` spelling, demand byte-identity. Result - 8 of 8 byte-identical in 2 m 12 s, and the probe that reported the defect was seen RED before and GREEN after on the same inputs.
reverify: "bash docs/tasks/artifacts/engine_universal_services/es29_generated_reproducibility/blast_radius.sh   # bare run prints the plan and touches nothing; --repair executes and demands byte-identity across all 8"
---

**Some claims can only be measured while the change is happening.** "This fix changes nothing else"
is one of them. After the repair, the pre-repair state is gone; before it, there is nothing to
compare. The measurement lives entirely in the transition, and if you do not capture it there you
are left with an argument — which is what most such claims actually are.

The tell that you are in this situation: you find yourself writing *"it cannot affect X, because…"*
That sentence is a hypothesis with no experiment attached.

## The shape

```text
1. SNAPSHOT   every downstream artifact, by hash            ← the step that disappears if you skip it
2. REPAIR     through the CANONICAL command, not by hand
3. VERIFY     the repair did what it claimed (the defect's own probe now passes)
4. RELINK     rebuild whatever consumes the repaired thing — and ASSERT it rebuilt correctly
5. REGENERATE every downstream artifact
6. DEMAND     byte-identity against step 1; any difference IS the finding
```

Steps 1 and 6 are the measurement. Steps 2–5 are the repair. Interleaving them is not tidiness — it
is the only ordering in which both exist.

## The three ways this goes wrong

- ❌ **Repair, then reason.** The most common. The artifact is fixed, the argument is written, and
  nothing ever tests it. The claim reads identically to a measured one.
- ❌ **Regenerate everything, then compare to nothing.** A full regeneration with no snapshot proves
  the pipeline runs; it says nothing about whether output moved.
- ⛔ **Compare with a variable you forgot to hold fixed.** A generated artifact can embed its own
  output path or a timestamp, so regenerating it to a scratch filename changes its bytes for reasons
  unrelated to the repair. Hold the *provenance* inputs identical too, and **assert** that you did —
  see [[a-hypothesis-list-is-a-snapshot-of-what-you-knew-that-day]], where exactly that difference
  founded a task leaf on two wrong hypotheses.

## Two guards worth building into step 4

**Assert the rebuild.** A relink that silently drops a feature may not produce *wrong* output — it
may produce *no* output, and a comparison loop that treats a missing file as "no difference" reports
a clean pass. Here an under-featured generator cannot emit any parser at all, so the script asserts
the feature surface and dies rather than comparing.

**Assert the comparability, not just the equality.** Before trusting a hash, confirm the two sides
are actually comparable — same embedded path, same site count, same spelling. A refusal (exit 2) is
the right verdict for "I cannot answer this", and it is a different verdict from "they differ".

## The free control

⭐ A defect that exists while you build its probe hands you something you would otherwise have to
fake: **the probe is observed failing on a real input, then passing on the same input minutes
later.** Keep both outputs. A control that has only ever been green is not known to work, and
constructing a synthetic failure for it afterwards is strictly weaker evidence than the one the
defect gave you for free.

Related: [[the-row-that-does-not-fit-the-pattern-is-the-next-investigation]],
[[an-ab-whose-arms-are-secretly-identical-does-not-fail-it-passes]],
[[a-grammar-rewrite-must-verify-its-own-postcondition-because-a-static-criterion-only-answers-what-you-encoded]].
