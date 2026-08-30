---
id: the-sentence-explaining-your-numbers-is-a-separate-claim
title: The sentence EXPLAINING your numbers is a separate claim — and re-running the instrument can never check it
answers:
  - "every number in my finding re-derives, is the finding verified"
  - "I published a measurement plus an explanation of it — what still needs checking"
  - "how do I verify a causal claim like 'X is a function of Y'"
  - "my instrument measures lengths but I claimed a cause — is that covered"
  - "what does a two-sided predicate check that a sorted table does not"
  - "I read a pattern off a printout — is that a derivation"
  - "the numbers survived a re-audit but something still feels overstated"
tags: [evidence, verification, measurement, claims, publishing, causality]
date: 2026-08-30
status: current
evidence: "DOCTRINE-GAP-OWNERSHIP.20/.22 (PGEN-DOCTRINE-GAP-OWNERSHIP-0300/0301), corrected by -0302. Nine doctrine descriptions measured under 500 bytes; all nine sat at registry positions 2-12; every doctrine from position 13 on measured >= 975 B. Every one of those numbers re-derives exactly. The GLOSS published beside them — 'description depth is a function of registration era, not of importance' — was refuted by re-deriving it as a two-sided predicate: positions 2-12 hold ELEVEN doctrines, and TASK-ACCEPTANCE (1367 B) and LESSON-PROMOTION (783 B) are not thin, so it is 9 of 11, not 11 of 11. The follow-up hypothesis (the thick ones were revisited after a later incident) was refuted in the same pass: 4 of the 9 thin rows also cite a later leaf."
reverify: "python3 - <<'PY'  # re-derive the predicate two-sided over the population at the founding commit\nimport subprocess,re\ndoc=subprocess.run(['git','show','e154d83a^:DOCTRINE_ENFORCEMENT.md'],capture_output=True,text=True).stdout\nPY\n# The point is the SHAPE: assert (thin) <=> (position in 2..12) over ALL 33 rows, not over the 9 you noticed."
---

**A finding usually ships as two things: numbers, and a sentence saying what they mean. The
three-way check tends to cover only the first, because the second does not look like a claim — it
looks like the point.**

The asymmetry that makes this dangerous: re-running your instrument re-derives the numbers and says
**nothing at all** about the gloss. The instrument measured byte lengths; the sentence asserted a
cause. No amount of replication can move it, so a gloss can survive every audit you would normally
think to run, wearing the credibility the numbers earned.

## The measured instance

Nine doctrine descriptions were a single clause. Sorted by length, the thin ones were all old; the
first doctrine registered after them carried an order of magnitude more prose. Published:

> All nine are registry positions 2–12. **Description depth is a function of registration era, not
> of importance.**

Every number in that sentence is correct. The claim is still wrong, and the reason is visible the
moment it is written as a predicate over the whole population rather than read off a sorted table:

```text
claim:  (length < 500 B)  <=>  (registry position in 2..12)
population 33 · counter-examples 2
   position  6  TASK-ACCEPTANCE    1367 B   in 2..12 but NOT thin
   position  8  LESSON-PROMOTION    783 B   in 2..12 but NOT thin
```

Positions 2–12 hold **eleven** doctrines. Nine were thin. *"A function of era"* requires eleven.
What the data supports is an association with two unexplained exceptions — which is a weaker and
more honest finding, and it leaves a real question open instead of closing it with a phrase.

⭐ **The repair hypothesis failed too, and checking it was the whole point.** The natural rescue —
*the two thick old rows were revisited after a later incident* — is refuted in the same pass:
**4 of the 9 thin rows also cite a later leaf.** Asserting it would have been the more comfortable
move, and it would have been a second unverified gloss stacked on the first.

## The rule

**Underline every sentence in your finding that is not a number. Each one is a claim with its own
burden.** Words that should trigger the check: *is a function of, because, tracks, driven by, the
cause is, therefore, not X but Y*.

Two practical corollaries:

- **Write the claim as a predicate over the whole population and count counter-examples.** A pattern
  read off a sorted printout is an observation about the rows you looked at; the same pattern
  written as `A <=> B` and evaluated over every row is a measurement. Here the two differ by
  exactly the two rows the sorted view never invited anyone to check.
- **A causal gloss needs a mechanism you can falsify, or it needs to say it has none.** *"Unexplained"*
  is a publishable result. A mechanism invented to make the sentence read better is not, and it is
  the easiest thing in the world to write immediately after being right about the numbers.

Related: [[a-reconstruction-of-why-someone-got-a-number-is-a-provenance-claim]] — the same shape
aimed backwards at another author; this one is aimed at the sentence you are about to publish
yourself. And [[an-x-is-checked-by-nothing-claim-is-a-census-claim]], for why the population, not
the instances you noticed, is the thing to evaluate over.
