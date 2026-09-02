---
id: a-denominator-row-is-classified-from-its-bytes-not-its-extension
title: A denominator row is classified from its BYTES, never from its extension or its suite — and an answer key outranks a shape only in the direction that is a verdict
answers:
  - "why are files that can never parse sitting in the DARK worklist"
  - "how should a corpus census treat a file that is not source text at all"
  - "should I remove non-source files from the denominator or leave them counted"
  - "what happens when an answer key says must_accept on a memory image"
  - "how do I stop a new hex image or command-line file from landing silently in a parse census"
tags: [corpus, adjudication, denominator, detectors, answer-key, sv-corpus-grad]
date: 2026-09-03
status: current
evidence: "PGEN-SV-CORPUS-GRAD-0345 (SV-CORPUS-GRAD.13c.1): 346 friscv `$readmemh` memory images (IEEE 1800-2017 §21.4) wearing `.v`, 3 slang bare-string-literal fixtures and 1 Surelog command line saved as `.sv` sat 23 days in DARK `chained_only` because the design-suite rule keyed on the SUITE and the fragment rule on the EXTENSION. A byte detector (`not_source_text()`, three stated shapes) moved exactly 350 rows into `deferred:not_source_text` — DARK 4 363 → 4 013, zero verdicts moved, zero adjudicated rows hit over all 16 336 files."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/not_source_text/probe.sh   # arm 6 runs the detector over every corpus file and demands it fire on exactly the labelled rows"
---

**An extension is a claim the file makes about itself. The bytes are what the parser saw.**

## The rule

1. **Classify a row from its bytes.** A list of paths is wrong the day the corpus adds a 347th image; an
   extension test is exactly what let 346 memory images into a parse census. State each shape so a reader
   can refute it (what the file contains, and why no `source_text` derivation exists), cite the format
   (`$readmemh`: white space, comments, hexadecimal numbers, `@hh…h` addresses — §21.4), and keep the
   detector CONSERVATIVE: a shape it cannot decide stays where it was, visible.
2. **Apply the detector AFTER the key has spoken, and let the key win only where it is a verdict.**
   `must_reject` on non-source text is a correct expectation — rejecting it IS the verdict — so it stays.
   `must_accept` on non-source text is a broken key: REFUSE and name the row; never let the detector
   paper over a key. Every unkeyed row (`chained_only`, `out_of_scope`) becomes the new class.
3. **Neither remove silently nor count as a failed parse.** The rows stay in the denominator as a stated
   fact about the corpus's composition — their own class, PERMANENT (no key, chaining or grammar change
   can produce a verdict about SystemVerilog from a memory image) and ceiling-bounded at the measured
   count, so growth is a deliberate act.
4. **A census stratum is not a parse outcome.** `observed=fail` on a memory image is not DARK — DARK means
   *the parse of source text failed and the deferral is why nobody looked*. Give the class its own stratum
   so the burn-down population is the one somebody can burn down.
5. **Re-measure the headline before closing the leaf.** The leaf said 351; its sibling census said 1
   Surelog file, not 2; the bytes said 350. Hold the detector to the labelled set with a whole-corpus arm.

## Related
[[a-live-count-in-doctrine-prose-is-derived-state-wearing-a-sentence]] — the book's *530* was already 528
and is now 178; the gate held the artifact, never the sentence.
[[a-red-arm-that-returns-rc-1-from-a-crash-is-not-a-red-arm]] — every RED arm here asserts the refusal text.
