---
id: a-stale-identity-block-is-a-prompt-to-re-measure-not-evidence-of-drift
title: An instrument-identity block says whether your INPUTS moved, never whether your ANSWER moved — treating a stale one as proof of drift replaces a cheap check with an expensive assumption
answers:
  - "my baseline's input hashes no longer match — do I have to re-adjudicate everything"
  - "a tracked report says it no longer describes my tree — how bad is that"
  - "how do I re-measure without confounding my change with unrelated drift"
  - "should I defer work because an oracle it depends on is stale"
  - "how do I make a delta attributable to my change by construction"
  - "when do I run the control — before or after the edit"
  - "I am moving items between two owners; what do I check first"
tags: [evidence, instruments, baselines, staleness, attribution, controls, oracles]
date: 2026-08-23
status: current
evidence: "SV-CORPUS-GRAD.13e.1 (PGEN-SV-CORPUS-GRAD-0282). A corpus lane's tracked results oracle carried an identity block pinning generated parser bfaca030… while the tree held e53cb4a2…, and the block's own wording is `if any hash differs, this report no longer describes your tree`. That was read as: a re-run would confound three days of parser drift with the change under test, so the unit must be deferred as an atomic cascade. Measured instead: the lane re-parses in 10 s (2 459 files, peak 94 MB) and results_v2005.tsv came back BYTE-IDENTICAL — the identity was stale, the outcomes were not. Separately, re-deriving the manifest from the UNCHANGED transform before editing it produced a byte-identical file, which made every later difference attributable by construction: one transition, 147 rows, 0 rows whose observed value moved."
reverify: "python3 stimuli/sv/adjudicate_external_corpus.py --out-manifest rust/target/attr_control.tsv --out-summary rust/target/attr_control.md >/dev/null && cmp stimuli/sv/characterization/adjudication_manifest.tsv rust/target/attr_control.tsv && echo BYTE-IDENTICAL-BEFORE-CONTROL   # the pattern: re-derive from the UNCHANGED transform first"
---

**An instrument-identity block answers one question: did the inputs I was derived from move?** It
does not answer, and cannot answer, whether the *output* would come out different today. Those two
come apart constantly — most changes to a producer leave most of its verdicts alone — and the gap
between them is where a cheap check quietly gets replaced by an expensive assumption.

The failure looks responsible, which is why it lands:

> The oracle's identity pins an older producer. Re-running it now would mix that drift into my
> change, so I cannot attribute my delta. This needs its own unit.

Every clause is sound. The conclusion is still wrong whenever the outcomes did not actually move —
and you find that out by running the thing, not by reasoning about it.

⭐ **The tell is the cost ratio.** In the founding case a unit of work was about to be deferred to
avoid a **ten-second** command. Whenever avoiding a measurement costs more than the measurement,
the analysis has stopped paying for itself.

## Re-measure, then decide

```text
identity stale  ->  re-run the producer, unchanged, to a scratch location
                ->  diff against the tracked artifact
                    · byte-identical  ->  no drift exists; proceed, delta fully attributable
                    · differs         ->  NOW you have a real cascade, and its size, not a fear
```

Both branches are better than the assumption. The second one is strictly better than the assumption,
because "it differs" without a magnitude is not actionable either.

## Run the attribution control BEFORE the edit — afterwards it is unavailable

If a producer is a pure transform of tracked inputs, re-derive its output from the **unchanged**
producer first and compare. Byte-identical means every subsequent difference is yours *by
construction*, and the write-up stops needing an argument:

```text
row set identical
one transition:  class A -> class B,  147 rows
rows whose upstream-observed value moved: 0
headline metric: unchanged
```

Do it first. Once the file is edited, a clean "before" no longer exists and attribution degrades
into inference.

## Verify the RECEIVING side before you change the sending side

When a fix moves items from one owner to another, the interesting failure is not a bad move — it is a
move into a place that cannot accept them. In the founding case both sides classified by the same
rule, so teaching only the sending side would have routed items into a lane structurally unable to
answer them: the "unanswered" count would have fallen while nothing real changed, and it would have
looked like success. ⇒ **check that the destination produces an answer for the moved items, and
report that as a number** — here, *107 of 147 answered, 40 on the destination's own named deferrals*.

⭐ And expect the honest version to create work. That reclassification surfaced **7 new defect
candidates** that had been invisible while the rows sat silent. A move that only tidies totals and
finds nothing is the result to distrust.

⚠️ **Bound.** None of this says identity blocks are noise — a stale one is exactly the signal that
should trigger the re-measure, and without it nobody would have looked. The error is stopping at the
signal and substituting a conclusion for the run.
