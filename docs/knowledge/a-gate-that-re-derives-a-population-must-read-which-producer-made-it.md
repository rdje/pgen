---
id: a-gate-that-re-derives-a-population-must-read-which-producer-made-it
title: A gate that re-derives its producer's population must read WHICH producer made it — and the member where the wrong method happens to agree is the one that makes it look correct
answers:
  - "my freshness gate re-counts a corpus and disagrees with the report — which side is wrong"
  - "how do I wire a second lane of the same family to a census gate"
  - "a denominator arm breaches on some sub-corpora and not others — is that a real drift"
  - "should a gate hard-code how a population is collected, or read it from a declaration"
  - "one of my controls agrees with the instrument — is that evidence the instrument is right"
  - "how do I know a green arm is measuring anything at all"
  - "what does it mean when a check passes on one member of a population and fails on the rest"
tags: [instruments, gates, corpus, denominator, controls, evidence, doctrine, sv-corpus-grad]
date: 2026-09-02
status: current
evidence: "SV-CORPUS-GRAD.13m / PGEN-SV-CORPUS-GRAD-0332. `CORPUS-CENSUS-CURRENCY` arm (4) re-counted every declared sub-corpus by WALKING the tree, because the two lanes it had ever been pointed at were collected with `find`. SystemVerilog's third lane is not: `stimuli/run_external_corpus.sh:403-405` reads `stimuli/sv/characterization/v2005_lane_files.tsv`, an adjudicator-derived list. Under the lane's OWN `-name '*.v'` extension set the walk reads 360 / 3 799 / 341 against a published 356 / 1 909 / 341 — 2 of 3 suites breach; under `.sv`/`.svh`/`.v` it reads 1 266 / 3 799 / 953 and all 3 breach. The suite that AGREES, `sv2v`, agrees set-identically (341 = 341, `diff` of the sorted lane paths against the sorted `find -type f` output = 0 lines) — a fact about what that suite happens to contain, not about the rule."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/v2005_lane_denominator/probe.sh   # V2005-LANE-DENOMINATOR: 9/9 as declared; arm 0 drives the same report under the filesystem source and must go RED"
---

**A denominator is only re-derived if you re-derived it the way the producer produced it.** Compute
it another way and the gate is not checking the report — it is comparing the report to a number
nobody claimed.

## What happened

A corpus-freshness doctrine had a denominator arm: re-count the population under each declared
sub-corpus and hold it equal to the row that publishes it. It was written against two lanes, and
both collect their files with `find … -type f`, so the arm walked the tree. The rule looked like a
property of *corpora*.

It is a property of *producers*. The same family's third lane parses only the files named in a
tracked list that an adjudicator derives from the first lane's results — it never touches a
directory. Pointing the walking arm at it produced a breach on two of its three suites, and the
breach was the gate's: the report was current, and the arm was answering a different question.

## The fix shape — declare the source, never infer it

```jsonc
"denominator_source": "filesystem",   // default: walk the tree (every earlier entry unchanged)
"denominator_source": "lane_list",    // the tracked list the producer reads IS the population
"lane_list": "…/v2005_lane_files.tsv"
```

Two properties make this a re-derivation rather than an exemption:

- **it still counts.** The list is counted per suite, the published rows are held equal to it in
  BOTH directions (a suite in one and not the other breaches either way), the total is held equal to
  the totals row, and every listed path must be a regular file the producer could have been handed.
  ⛔ An arm that *opts out* is a reachability defect wearing a census.
- **an unimplemented source REFUSES.** Falling back to the walk is how a gate silently resumes
  answering the wrong question.

## The half worth carrying elsewhere

⭐ **The member where the wrong method agrees is the most dangerous row in the table.** `sv2v` read
341 against a published 341, and the two populations were not merely equal in count — they were
**set-identical**. A reviewer sampling that one suite would have concluded the walk was correct. It
is correct *there*, because every `.v` file in that suite happens to be in the lane, which is a fact
about that corpus's contents and not about the collection rule.

That is the same shape as this doctrine's earlier symlink defect, which had been green since it
shipped "by accident of what those corpora happen to contain". Two independent instances, one
doctrine, one year apart in the tree's own history:

> **Before reading an agreement as confirmation, ask whether it is a property of the RULE or of the
> MEMBER.** If a single member's contents would flip it, you have measured that member.

And the control that separates a working arm from a silent one is the arm's own subject run the
*old* way: arm 0 of the bank drives the identical report under `filesystem` and requires RED. A
clean run and an arm that stopped looking read identically — see
[[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]] and
[[an-x-is-checked-by-nothing-claim-is-a-census-claim]].
