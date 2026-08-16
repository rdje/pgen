---
id: a-ranking-is-robust-to-an-undercount-that-a-share-is-not
title: A classifier wrong about 75.1 % of a rule family moved the SHARE it published by 4x, moved 12.5 % of the sample rows it ranked, and moved the quantity that sample exists to cover by 0.45 % — three different blast radii from one defect
answers:
  - "my classifier was wrong — which of the things built on it actually have to be redone"
  - "do I have to re-derive my sampled benchmark after fixing the metric that selected it"
  - "how do I decide whether to re-pin a benchmark sample or keep the old one"
  - "is it worth resetting a performance baseline to adopt a corrected selection"
  - "a measurement bug affected two published numbers — are both equally wrong"
  - "how do I price a correction instead of assuming it must propagate everywhere"
tags: [instruments, measurement, ratchets, sampling, benchmarks, evidence, proxies]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.21 acceptance (b), 2026-08-16. One regex classified PGEN's left-recursion-elimination rule family. It matched 97 of the parser's 127 LR rule names and missed **75.1 %** of the family's corpus rule entries (18 518 719 of 24 644 435). Two things were built on it. (1) A published SHARE, `0.681 %` of all corpus entries — wrong by **4x**; the true figure is 2.741 %, and every surface quoting it was stale. (2) A pinned 192-file benchmark sample whose `lr` tier is *"the 40 heaviest by guarded-admission entries"* — re-derived under the corrected classifier over a reproducible full-corpus census (two censuses, RAW BYTE-IDENTICAL, 16 336 rows each), it moves `hot` **0/40**, `lr` **5/40**, `breadth` **19/112** — **24 of 192 slots, 12.5 % of the sample** — while the LR-family entries that sample COVERS change by only **+0.45 %** (12 440 690 -> 12 496 291). ⛔ Both numbers, always: a first draft of this card published only the 0.45 %, which makes the decision to decline look easier than it is. The correction was necessary and the selection barely noticed it.
reverify: "python3 stimuli/sv/corpus_parse_cost.py --census --outdir <a> && python3 stimuli/sv/corpus_parse_cost.py --census --outdir <b> && python3 docs/tasks/artifacts/engine_universal_services/sample_rederivation/compare_sample.py --census-a <a>/census.tsv --census-b <b>/census.tsv   # 4 legs, exit 0; leg 4 is an external oracle — the fresh census must reproduce the tracked entries.tsv totals 416 841 264 / 12 440 690 exactly"
---

**A measurement defect does not propagate uniformly to everything downstream of it.** It propagates
in full to whatever consumed the *magnitude*, and often barely at all to whatever consumed the
*order*. Those are different consumers, and pricing them together is how a correction turns into a
rewrite that buys nothing.

PGEN corrected a classifier that had been missing three quarters of a rule family. Two published
things depended on it:

```text
consumer                                   what it used   how wrong it was
-----------------------------------------  -------------  -----------------
the published family SHARE (0.681 %)        the MAGNITUDE  4x   (true: 2.741 %)
the pinned sample's MEMBERSHIP              the ORDER      12.5 % of rows move
the quantity that sample exists to COVER    the ORDER      0.45 %
```

The share was wrong by a factor of four and had to be corrected on every surface carrying it. The
sample — selected by ranking files on the very same broken count — churned **12.5 % of its rows**
while the quantity it exists to cover moved **less than half a percent**. ⛔ Quote both or neither:
the row churn is what a re-derivation costs, and the 0.45 % is what it buys. A summary that keeps
only the second makes the decision look free.

## Why the ranking survived

A classifier that misses a *sub-population* of the family behaves, for ranking purposes, like a
noisy monotone transform of the true count. The files that are heaviest in `_lr_base`/`_lr_suffix`
entries are largely the same files that are heaviest in `_lr_seed`/`_lr_guard` entries, because
those rules are emitted together, for the same constructs, in the same source. The undercount was
**systematic, not selective** — so it shifted every file's score in roughly the same direction and
left the order nearly intact.

⇒ **the question to ask is not "was the input wrong" but "was it wrong in a way that reorders
things".** A share reads the absolute value and inherits the whole error. A top-N selection reads
only the comparisons, and inherits the error only where it crosses a boundary — here, 5 of 40 rows.

## What that changes about the decision

The tempting move after fixing a metric is to re-derive everything it touched. Price it instead:

| | cost of adopting | benefit |
|---|---|---|
| the published share | none — it is prose and a gated constant | correctness; it was 4x wrong |
| the pinned sample | a **one-time reset of the performance baseline**, ending comparability with every prior measurement | +0.45 % coverage of the mechanism |

The share was adopted immediately. The sample re-derivation was **declined** — and the declination
is written into the manifest's own header with the numbers, so the next reader does not rediscover
the question or, worse, assume the tier means something it does not.

⛔ **Declining is not the same as ignoring.** The `lr` tier's charter says *"the 40 heaviest by
guarded-admission entries"* and 5 of its 40 rows do not satisfy that under the corrected
classifier. That is a real, if small, discrepancy between a file and its own description, and the
honest resolution is to **declare it where the file is read**, not to leave it for someone to find.

## The precondition nobody can skip

This decision was blocked for a session, and not by an opinion: the census the sample derives from
was **not reproducible**, because one corpus file was being silently dropped, and its absence
perturbed the per-suite pools and moved 5 breadth rows on its own. A delta measured against a
moving baseline is not a delta.

⇒ **before comparing a re-derivation against a pinned artifact, prove the derivation reproduces.**
Here that meant two independent full-corpus censuses, compared *raw*, byte for byte — and then a
fourth leg nobody had asked for: the fresh census, restricted to the pinned sample's 192 paths, must
reproduce the tracked baseline's totals exactly (`416 841 264` entries, `12 440 690` family entries).
It did, across two code paths and two sessions, which is what makes the 0.45 % believable rather
than merely computed. See [[a-check-whose-inputs-all-pass-has-not-been-tested]].

## ⛔ And the audit script reproduced the very defect it was auditing

The comparison instrument above was written with `breadth = 112` — the number of rows in the
manifest — while the derivation's actual default is **120**. Both realize the identical sample (each
floors to a per-suite quota of 8 across 14 sub-corpora), so the re-typed constant was silent and the
output was correct. It was caught by reading the producer, not by any test, and the fix was to give
the tier sizes one home and import them.

⇒ *a constant copied from an OUTPUT agrees with that output by construction, and tells you nothing
about the PRODUCER.* That is the same failure as
[[deriving-a-control-from-the-producer-is-not-the-same-as-observing-it]], one turn later, in the
script written to audit it.
