---
id: a-rejection-the-design-demands-is-explained-only-by-a-parse-that-proves-it
title: A rejection the design demands is explained only by a parse that proves it — declare what the parser demanded, re-parse, and publish the minimal prelude; argue nothing
answers:
  - "the parser rejects a valid-looking suite file that uses an undeclared type — is that a defect"
  - "how do I move a row out of the axis-2 bar without relabelling it by argument"
  - "what is divergence:explained_store_gate and how is it proven per row"
  - "why does the store-gate cache carry a binary sha and a file sha"
  - "when is a store-gate rejection NOT explained (unresolved wildcard import, export, predefined type)"
tags: [sv-corpus, adjudication, store-gating, evidence, claim-verification, parser-elaborator-boundary]
date: 2026-09-03
status: current
evidence: "SV-CORPUS-GRAD.13w (PGEN-SV-CORPUS-GRAD-0346): 247 rejects-valid rows triaged BY PARSING through the dark-worklist search; the rows that parse once one demanded name is declared as a class left the bar with their prelude published; the two rows demanding `CrossQueueType` stayed (the LRM predefines it — a grammar defect, `.3.29`)"
reverify: "python3 stimuli/sv/store_gate_explanations.py --self-test && python3 stimuli/sv/store_gate_explanations.py --report   # the cache is the population; every explained row carries names+forms"
---

**A design decision can explain a divergence, but only the parser can prove the decision was the whole story.**

## The rule

`SV-PARSE-STRICT.2` made the parser reject a *provably*-undeclared type identifier at parse time
(the director-confirmed parser/elaborator boundary). An Annex-A answer key cannot see that
boundary, so a single-file suite test that never declares the type it uses is keyed `must_accept`
and sits in the axis-2 bar as if it were a defect. It is not — but *saying so* is an argument, and
arguments do not move bars here. What moves a row is a **parse**:

1. harvest the names the parser DEMANDED AND LACKED at the failure (its own trace);
2. declare them — nothing else changes — and re-parse; iterate to a fixed point;
3. minimise: drop every declaration the parse does not need;
4. publish the winning prelude beside the verdict, so anyone can re-run the proof.

A row that parses this way is `divergence:explained_store_gate`. A row that does not stays in the
bar, with the search's residual reason in its basis — the next defect slice starts from a triage.

## The three ways a rejection is NOT explained

- an **unresolved wildcard import** is in scope — the boundary says ACCEPT there, so a rejection
  is the gate being unsound: a defect for `SV-PARSE-STRICT`, not an explanation;
- a package **`export`** — visibility the file cannot settle on its own;
- a name the **LRM predefines** (`CrossValType`/`CrossQueueType`, IEEE 1800-2017 19.6.1.4) — the
  file owes no declaration, so the demand itself is a grammar defect.

Each is decided mechanically before the class is granted, and each has a RED arm.

## Why the memo carries two digests

The verdicts are memoised in a tracked cache keyed by file sha256 AND parser-binary sha256, and a
measurement is REFUSED when the binary on disk is not the one the lane's report names. A verdict
measured against a different parser than the results would be a number about nothing — the same
discipline `.13e.6` applies to the raw dumps.

## Related
[[a-denominator-row-is-classified-from-its-bytes-not-its-extension]] — the sibling: a class the adjudicator decides from bytes, not from a list.
[[a-commit-that-quotes-a-bank-must-be-gated-on-the-banks-exit-code]] — how the numbers this card cites were committed.
