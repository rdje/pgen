---
id: a-classifiers-residual-is-its-most-important-output
title: A classifier's residual is its most important output — print it verbatim, cap it rather than zero it, and extend the rule by enumeration
answers:
  - "how do I classify a corpus field without over-fitting the rule"
  - "my classifier left half the population unclassified — is the result still valid"
  - "should a gate require zero unclassified rows"
  - "how do I extend a classification vocabulary safely"
  - "why did my regex classify tool testimony as a spec citation"
  - "is mentioning IEEE 1800-2017 the same as citing a clause"
  - "how do I publish a share or percentage over a corpus honestly"
  - "how do I stop a classifier from matching a task-leaf id as a clause number"
  - "what is the trust bound on the SV corpus answer key"
tags: [classification, census, instruments, controls, false-positives, residual, corpus, corpus-key-audit]
date: 2026-08-24
status: current
evidence: "CORPUS-KEY-AUDIT.2 (PGEN-CORPUS-KEY-AUDIT-0004), classifying the provenance of all 10 090 keyed rows in the SV adjudication manifests. Cut 1 ('any basis naming IEEE 1800-2017 is clause-cited') classified all 623 Surelog rows — basis 'parses under Surelog's IEEE 1800-2017 grammar' — as spec citations; they are the purest tool testimony in the corpus. Cut 2 ('a clause cite needs an A.n.n production') then missed 72 rows citing a numbered clause directly. Both settled by enumerating the token after every edition mention: 623 'grammar' / 72 a dotted number / 63 'clauses|Annex|production', exhaustive over all 758. Separately, the first complete run left 4 961 rows (49 %) unclassified while printing a clean-looking table and a headline; enumerating those shapes took the residual to 4, all printed verbatim. Final: clause_cited=627 tool_testimony=2287 suite_convention=7172 unclassified=4."
reverify: "python3 docs/tasks/artifacts/corpus_key_audit/key_provenance_census.py   # rows=10090 ... unclassified=4; the artifact prints all four verbatim. python3 docs/tasks/artifacts/corpus_key_audit/key_provenance_census.py --self-test   # arms=16 failed=0, including both historical mistakes pinned in OPPOSITE directions and the negative arm proving a task-leaf id (.8b.3) is not read as a clause number. bash scripts/check_corpus_key_integrity.sh   # tier A4 caps the residual"
---

Writing the rule before reading the vocabulary produces a rule that is wrong in **whichever
direction you were not thinking about** — and the output looks fine either way, because a
classifier always returns a complete-looking table.

## Wrong twice, in opposite directions, from the same cause

Classifying what evidence each corpus expectation rests on:

```text
cut 1: "a basis naming IEEE 1800-2017 cites the standard"
       -> swept in all 623 Surelog rows, whose basis reads
          "...parses under Surelog's IEEE 1800-2017 grammar"
       the possessive is the meaning: that is the upstream TOOL's grammar.
       The most tool-dependent class in the corpus, filed as its exact opposite.

cut 2: "a clause cite needs an A.n.n production"
       -> missed 72 rows citing a numbered clause directly:
          "IEEE 1800-2017 22.8 / IEEE 1364-2005 19.2 permit it only OUTSIDE..."
```

Neither was caught by re-reading the regex. **A regex is never refuted by staring at the regex** —
cut 1 died when one sample per group was printed and read.

The fix was to stop guessing and **enumerate**: print the token that follows every edition mention
across the whole population.

```text
623  'grammar'          -> tool testimony
 72  a dotted number    -> clause cite
 63  'clauses'|'Annex'|'production' -> clause cite
      ...and nothing else.  758 = 623 + 72 + 63
```

⭐ **The property that makes the answer trustworthy is that the partition came out EXHAUSTIVE**, not
that the third rule looked better than the second. The real rule was never *does it mention the
standard* — it is ***does it point at a place in it***.

## The residual is where it nearly shipped wrong

The first complete run printed a clean table and a headline share — over a population that was **49 %
unclassified**. Every number in it was arithmetically correct and the conclusion was false: a share
computed over half a corpus is a **lower bound wearing the clothes of a measurement**, and nothing in
the output said so. Enumerating those shapes took the residual from 4 961 to 4.

**Make the residual a first-class output and print every member of it.** A residual folded into the
nearest bucket is exactly how a classifier stops being checkable — and a residual reported only as a
count is a number nobody can audit.

## Cap the residual; do not require zero

It is tidier to gate on `unclassified == 0`. Don't. That turns the next unknown shape into *pressure
to widen a rule until the number falls* — which is precisely how 623 tool-testimony rows became spec
citations. A **ceiling** turns the same event into a signal that says *enumerate*. In the gate:

```text
A4 RED: an unmatched basis SHAPE appeared (residual above the ceiling)
        -> ENUMERATE the new shapes and extend the vocabulary
        -> do NOT widen a rule until the number falls
```

## Two more habits the same case earned

- **Say which direction your bound errs in.** Two residual rows here *are* clause cites, in a bare
  dotted-number spelling the classifier deliberately does not chase — the corpus writes task-leaf
  ids (`.8b.3`, `.13e.2`) in the same strings, so a general rule would classify every pinned row as
  spec-cited **on its own bookkeeping**. So the count is a *lower* bound, short by exactly two, and
  saying so is part of publishing it.
- **Cross-check the population against an unrelated producer.** This census's `sv_2017` keyed count
  is 7 556; `scripts/check_sv_corpus_denominator.sh` independently publishes `adjudicated = 7556`.
  Two producers sharing no code reaching the same population is worth more than either one's
  internal consistency.

Same trap, different corpus: [[a-heading-census-is-only-as-good-as-the-heading-grammar]]. The
instrument-health version: [[the-first-real-consumer-is-a-test-of-the-producer]]. What the resulting
number is *for*: [[an-answer-key-that-records-why-it-decided-can-audit-itself]].
