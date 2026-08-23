---
id: a-corpus-generated-from-the-artifact-under-test-is-part-of-the-measurement
title: When the corpus is generated from the thing you are testing, the corpus vintage is part of the measurement's identity — and a net pass/fail count over it cannot say which direction a fix moved
answers:
  - "I re-ran the recorded stimuli recipe and got a different number — did something regress"
  - "the row count matches the record but my result does not, what moved"
  - "does regenerating a corpus count as re-running a measurement"
  - "my fix makes the self-rejection count go UP, is the fix wrong"
  - "how do I tell a narrowing fix from a regression"
  - "what does 'the grammar self-rejects N of its own stimuli' actually mean"
  - "should I score a grammar fix against the old corpus or a regenerated one"
  - "my instrument says 0 failures on 1000 samples, is that closure"
  - "how many seeds before a zero is real"
tags: [measurement, corpora, stimuli, claim-verification, instruments, ledgers, grammar-wellformedness]
date: 2026-08-23
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.16.6a → H.16.6c (`PGEN-GRAMMAR-WELLFORMED-0171`, `-0180`).
  `H.16.6a` published that a `=>` disambiguation arm took the grammar's self-rejection of its own
  generated stimuli from 18 of 1 000 to 8, and recorded the recipe verbatim: `--generate-stimuli
  --count 200 --seed {0,7,42,123,999}`, deduped, 1 000 unique.

  `H.16.6c` re-ran exactly that recipe one commit later and scored **3**, not 8. The recipe was
  right, the dedupe was right, and the row count reproduced to the row — 1 000 unique. The corpus
  was still a different 1 000: `--generate-stimuli` derives samples FROM the grammar, and the
  intervening arm had added four rules (`map_key` and friends), moving the generator's choice
  structure. `cmp` on the two corpora: different files. Rebuilding the corpus from the pre-arm
  grammar reproduced 18 and 8 exactly.

  The same leaf then hit the second half. Four terminals in `grammars/semantic_annotation.ebnf` are
  spelled `[^\s]`, so a path or URL abutting `]`, `}`, `)` or `,` swallows the delimiter and its
  enclosing collection can never close. A scratch arm narrowing those four character classes scored
  **10** rejects against the control's **8** on the fixed corpus — read as a total, the fix is worse.
  The per-input ledger read `widen=5 narrow=7`, and all seven narrowed rows were inputs whose path
  literal CONTAINS a delimiter (`./}.`, `~/}vP`, `/SH}2`, `/!,`, `file://,T`, `../]&}`, `~/bRk]`) —
  rows the narrowed generator, by construction, can no longer emit. Scored against stimuli each
  grammar generates itself, the same arms read 15 → 1 and 15 → 0 over 3 200 samples.

  And the small sample lied in the flattering direction: at 1 000 samples (5 seeds) BOTH arms read
  zero self-rejects; at 3 200 (16 fresh seeds) the weaker arm has one survivor — a URL that eats the
  ARROW rather than a bracket — which only the stronger arm prevents.
reverify: "for s in 0 7 42 123 999; do ./rust/target/debug/ast_pipeline grammars/semantic_annotation.ebnf --generate-stimuli --count 200 --seed $s -o rust/target/kmv_head_$s.txt >/dev/null 2>&1; git show dcc2e2d8:grammars/semantic_annotation.ebnf > rust/target/kmv_pre.ebnf && ./rust/target/debug/ast_pipeline rust/target/kmv_pre.ebnf --generate-stimuli --count 200 --seed $s -o rust/target/kmv_pre_$s.txt >/dev/null 2>&1; done; cat rust/target/kmv_head_*.txt | grep -v '^[[:space:]]*$' | sort -u > rust/target/kmv_head.txt; cat rust/target/kmv_pre_*.txt | grep -v '^[[:space:]]*$' | sort -u > rust/target/kmv_pre.txt; wc -l < rust/target/kmv_head.txt; wc -l < rust/target/kmv_pre.txt; cmp rust/target/kmv_head.txt rust/target/kmv_pre.txt   # SAME recipe, SAME 1000-row count, DIFFERENT corpus"
---

Two traps live on any measurement whose corpus is **derived from the artifact under test** — a
grammar's own generated stimuli, a schema's own sample documents, a model's own synthetic data. They
are independent, they both fail in the direction that looks like an answer, and neither is visible in
the headline number.

## Trap 1 — the recipe is not the corpus

A recorded recipe (`--count 200 --seed {0,7,42,123,999}`, deduped) is a *procedure*. Run it against a
changed artifact and you get a different corpus with the same shape — often the same row count, which
is exactly what makes it convincing. Re-running the recipe reproduced `1 000 unique` to the row and
scored **3** where the record said **8**. Nothing had regressed and nothing was wrong with the
record; the two runs were different experiments.

⇒ **a regenerated corpus is a new experiment, not a re-run of the old one.** If you want to reproduce
a published number, reconstruct the corpus from the *vintage the number was measured on* — for a
grammar, `git show <commit>:<grammar>` and generate from that.

This also means the metric's own name is ambiguous, and both readings are legitimate:

| reading | corpus | when it is right |
|---|---|---|
| **fixed-corpus** | held constant across arms | a before/after ledger — a moving corpus makes the delta unattributable |
| **own-corpus** | generated by the artifact under test | the honest reading of *"its own"*; the only one that does not penalise a narrowing fix |

Record which one you measured. A future session that regenerates and reads a different number
otherwise cannot tell a regression from a vintage change.

## Trap 2 — a net count cannot say which direction a fix moved

The containment arm rejected **10** where the control rejected **8**. That reads as *the fix is
worse*, and it is the point at which an investigation stops. The per-input ledger said
`widen=5 narrow=7`: five rows fixed, seven rows the narrowed generator **cannot produce any more**.

The asymmetry is structural, not incidental. On a derived corpus, a **narrowing** fix is charged for
every input its own predecessor emitted outside the new language — inputs that cease to exist the
moment the fix lands. A widening fix gets the opposite free ride. The net is therefore not a weak
signal; on this class of measurement it is *systematically* biased against the correct change.

⇒ **always print the per-input ledger — WIDEN and NARROW, by name.** `widen=5 narrow=7` and
`10 > 8` were the same run. One of them names five fixed rows and seven generator artifacts; the
other says "worse". See [[an-accept-set-watch-cannot-see-a-replaced-ast-shape]] for the companion blind spot: a ledger that names both directions still says nothing about a replaced AST *shape*.

## Trap 3, and it comes free with the other two — a zero on a small sample

Both arms read **zero** self-rejects over 1 000 samples from five seeds. At 3 200 samples over
sixteen fresh seeds, one arm has a survivor with a *different* sub-mechanism (a URL eating the
arrow rather than a bracket) that only the stronger arm prevents. Publishing the five-seed zero would
have been a claim the larger sample refutes — and the residual, once found, is what distinguishes the
two candidate fixes.

⇒ a zero is a claim about the sample before it is a claim about the artifact. Scale the sample until
either the residual appears or the count stops moving, and **name the residual** rather than rounding
it away.

## The practice

1. Record the corpus **vintage** beside any number derived from a generated corpus — the commit, not
   just the recipe. The recipe is reproducible; the corpus is not.
2. Never publish a net pass/fail delta on a derived corpus. Publish `widen`/`narrow` with the moved
   rows named, and say which reading (fixed-corpus or own-corpus) you measured.
3. When a fix *narrows* a language, score it against a corpus **its own generator** produces, and say
   so — that is the only reading under which the number means what its name says.
4. Treat a zero on a first sample as a hypothesis. Enlarge the seed set once before quoting it.

Related: [[a-control-that-cannot-fail-is-not-a-control]] ·
[[a-control-that-clears-your-hypothesis-has-not-cleared-the-symptom]] ·
[[a-containment-test-over-the-input-text-is-a-test-about-the-text]]
