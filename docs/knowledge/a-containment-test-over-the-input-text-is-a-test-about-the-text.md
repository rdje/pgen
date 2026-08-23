---
id: a-containment-test-over-the-input-text-is-a-test-about-the-text
title: Searching the failing input for the suspect's spelling is a test about the TEXT — only an arm that removes the mechanism is a test about the MECHANISM
answers:
  - "how do I attribute a set of failing inputs to a cause"
  - "is it enough that the failing input contains the construct I suspect"
  - "my grep over the repro says 6 of 8 are caused by X, can I publish that"
  - "how do I prove which of two nearby constructs caused a rejection"
  - "what is a controlled arm and when do I need one instead of a search"
  - "I shrank the repro and the suspect is still in it — is that attribution"
  - "why did my by-eye split of failing cases come out wrong in both directions"
tags: [attribution, controls, claim-verification, evidence, instruments, debugging]
date: 2026-08-23
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.16.6c (`PGEN-GRAMMAR-WELLFORMED-0180`).
  Eight stimuli that `grammars/semantic_annotation.ebnf` generates and then refuses had to be
  attributed between a known `=>` ambiguity and anything else. Three attempts, two of them wrong:

  1. BY EYE (inherited from `H.16.6a`, and correctly flagged there as a hypothesis): 4 arrow / 4 not.
  2. BY TEXT SEARCH (mine): shrink each input to a minimal rejecting core, then ask whether the core
     contains `=>`. Result **6 arrow / 2 not**.
  3. BY CONTROLLED ARM: build a scratch copy of the grammar with the suspected mechanism REMOVED —
     the four `[^\s]` path/URL terminals narrowed so they cannot swallow `]`, `}`, `)`, `,` — and
     re-score every input against the unmodified control. Result `ACCEPT-SET-LEDGER: widen=5
     narrow=7`, naming the five newly-accepted rows. True split: **5 delimiter-swallowing · 3 arrow**.

  Both wrong answers failed for the same reason: three of the five delimiter rows have an arrow
  standing next to the path as innocent bystander text (`@public : J7Pc_ => { … ";" => ../3|z}`
  fails because `../3|z` eats the `}`, not because of either arrow), and one arrow row was read as
  "not arrow" because its outer shape was a spread. Text adjacency and causation are unrelated.

  The mechanism also shows why no property of the text could have worked: `ftp://98eS]` — closing
  bracket included — parses as a WHOLE `annotation_value`, while `[ftp://98eS]` is rejected and
  `[ftp://98eS ]` accepts. The guilty terminal is invisible in isolation and misbehaves only in
  context.
reverify: "printf 'ftp://98eS]' > rust/target/kmv_a.txt && printf '[ftp://98eS]' > rust/target/kmv_b.txt && printf '[ftp://98eS ]' > rust/target/kmv_c.txt && for f in a b c; do ./rust/target/debug/ast_pipeline grammars/semantic_annotation.ebnf --interpret-parse rust/target/kmv_$f.txt --interpret-entry-rule annotation_value >/dev/null 2>&1 && echo \"ACCEPT $(cat rust/target/kmv_$f.txt)\" || echo \"reject $(cat rust/target/kmv_$f.txt)\"; done   # the bracket-eating form accepts; the bracketed one does not"
---

When several failing cases must be sorted between two candidate causes, the tempting instruments are
the cheap ones: read the inputs, or search them for the suspect's spelling. Both answer a question
about the **text**. Attribution is a question about the **mechanism**, and the two come apart exactly
where it matters — when the suspect is *present but innocent*.

## The three instruments, in ascending order of what they can actually claim

| instrument | question it answers | what it costs |
|---|---|---|
| read the inputs | "what do these look like to me" | free |
| search / shrink-then-search | "does the failing fragment contain X" | minutes |
| **controlled arm** | "does removing X change these verdicts" | an hour, and it is the only one that attributes |

Shrinking helps — a minimal reproducer is worth having — but shrinking *then searching* is still
searching. A minimal core can contain the suspect and be caused by something standing beside it.

## Building the arm

A controlled arm is a copy of the artifact with **one mechanism removed**, scored input-by-input
against the unmodified control. Three properties make it evidence rather than decoration:

1. **It never touches the shipped artifact.** Scratch copies only, on the repository volume.
2. **Every edit REFUSES on a missing or non-unique anchor.** A silently-skipped substitution produces
   an arm identical to the control, which then scores a clean `widen=0 narrow=0` and reads as *"the
   mechanism is cleared"* — the most expensive possible false negative.
3. **It reports per input, both directions, by name.** The count alone cannot attribute; the named
   rows are the attribution.

The output is the claim: *"removing this mechanism widens exactly these five rows and moves nothing
else."* Nothing about the input text can establish that, and nothing weaker should be published as
attribution.

## When a search is still fine

Use the cheap instruments to **generate** hypotheses and to bound the work — reading eight inputs is
how you learn there are two candidate causes at all. Just never let their output cross into the
write-up as a result. Label them as what they are (*"by reading"*, *"text containment"*), and if you
publish an intermediate split, mark it NOT CAUSAL **in the instrument itself** so a later reader
cannot mistake it for the verdict. The instrument in this slice now prints
`⛔ text-containment, NOT causal — see ledger_arms.py` in its own headline, because the wrong number
had already been recorded once.

Related: [[a-control-that-cannot-fail-is-not-a-control]] ·
[[a-control-that-clears-your-hypothesis-has-not-cleared-the-symptom]] ·
[[a-corpus-generated-from-the-artifact-under-test-is-part-of-the-measurement]]
