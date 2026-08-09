---
id: a-routed-finding-describes-its-witness-not-its-construct
title: A finding routed from the single row that surfaced it will UNDERSTATE the construct — re-sweep the whole population when the leaf is worked, and share the sweep so old families re-measure themselves for free
answers:
  - "a routed leaf says one row — do I fix just that row"
  - "how much of a defect class does a routing note actually describe"
  - "should I copy a corpus sweep to look for a second construct or parameterize it"
  - "how do I get regression evidence that a previously-closed defect family stayed closed"
  - "my fix closed every row in the worklist — what re-proves it later"
tags: [corpus, routing, diagnostics, instrument-honesty, regression-evidence, scope]
date: 2026-08-09
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaves .3.15 and .3.16; docs/tasks/artifacts/sv_corpus_grad/gen_block_family/sweep_begin_family.py (`in_family`, the `--family` predicate table); the four banked sweeps sweep_result*.txt, all reading 0 at HEAD
reverify: "S=docs/tasks/artifacts/sv_corpus_grad/gen_block_family/sweep_begin_family.py; python3 $S --family begin | grep -q '^`begin` family: 0 / ' && python3 $S --family inside | grep -q '^`inside` family: 0 / ' && echo BOTH-FAMILIES-CLOSED"
---

**A routing note is written from the row that happened to surface, and that row is a *witness*,
not the construct.** The construct's real extent is a property of the population, and nothing in
the routing moment measures it.

Worked example. `SV-CORPUS-GRAD.3.15` routed a finding as *"the `inside` set in a generate
condition"* — one row, described exactly as observed. When `.3.16` swept the whole
`unexplained_rejects_valid` population for that stuck token it returned **two**, and the second was
a `localparam` initializer nowhere near a generate block. The true class was *`inside` wherever the
grammar demands a `constant_expression`* — strictly larger than, and differently shaped from, the
routing note. Had the leaf fixed "the routed row", it would have closed one of two and reported
done.

## The rule

**When a routed leaf is worked, re-derive its population before fixing anything.** Treat the
routing note as a pointer to a construct, never as the construct's definition. The cost is one
sweep; the alternative is a fix that reports complete at partial coverage, which no pass-rate,
cluster map or family table will contradict.

## The dividend: share the sweep instead of forking it

The cheap way to sweep a second construct is to copy the existing sweep and edit one string — the
defect [[a-copied-diagnostic-covers-only-where-it-was-pasted]] names. Make only the **predicate**
selectable and keep the probe, the manifest walk and the controls shared, and something useful
falls out immediately:

```bash
S=docs/tasks/artifacts/sv_corpus_grad/gen_block_family/sweep_begin_family.py
python3 $S --family begin     # 0 / 310  <- re-proves the PREVIOUS leaf
python3 $S --family inside    # 0 / 310  <- proves this one
```

**The older family re-measures itself, from an instrument that was not written to confirm it.**
That is stronger regression evidence than any assertion the closing leaf could have made about
itself, and it costs nothing because the predicate lives beside its siblings rather than in a dead
one-shot artifact. Every construct added this way makes the whole set cheaper to re-verify.

⚠️ Pair this with constructed controls per `(family, lane)`. A control pinned to a corpus row can
be invalid in a lane it was never checked against — see
[[feedback_ground_truth_control_must_not_pin_untracked_state]] — and a sweep that cannot control
itself in a lane must REFUSE, so that an unasked lane never reads as a measured zero.
