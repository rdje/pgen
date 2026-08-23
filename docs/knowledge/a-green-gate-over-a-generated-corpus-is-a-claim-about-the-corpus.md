---
id: a-green-gate-over-a-generated-corpus-is-a-claim-about-the-corpus
title: A green gate over a GENERATED corpus is a claim about the corpus — add the discriminating row first and require it to go RED
answers:
  - "my differential gate is green but I suspect a divergence — how do I tell"
  - "how do I verify a fix when the gate already passes before I start"
  - "should I add the test before or after the fix"
  - "a certified byte-identity claim has a bug in it — how did the gate miss it"
  - "what does a green generated-corpus gate actually prove"
  - "how do I stop a corpus gap from reading as a proof"
  - "my stimuli generator never produces the shape that breaks my code"
  - "how do I prove my new test case is discriminating and not just passing"
  - "is it enough to add a regression test after fixing the bug"
tags: [gates, corpus, evidence, controls, regression-tests, parse-harness, differential]
date: 2026-08-23
status: current
evidence: GRAMMAR-WELLFORMED.H.22 (PGEN-GRAMMAR-WELLFORMED-0179). `return_annotation` sat in `parse_harness_equivalence_gate`'s CERTIFIED list — interpreter byte-identical to the generated parser — while carrying a live divergence: the parser emits `match_regex("[^']*", false)` ×5 and the interpreter passed `true`, so on `-> {k: ' abc'}` the parser captured `" abc"` and the interpreter `"abc"`. The discriminator is a quoted string whose CONTENT starts with layout, and the bounded stimuli generator never emitted one in 89 samples. Adding the rows to `CURATED_CORPUS` with NO code change turned the standing green claim RED (`DIVERGE samples=89 agree=80 diverge=8`); the fix then returned it to `4/4 ok`.
reverify: "Add the discriminating input to the gate's corpus and run the gate BEFORE touching any code — it must FAIL. make -C rust SHELL=/bin/bash parse_harness_equivalence_gate. Then fix, and it must pass. If step 1 does not go red, your row is not discriminating and the fix that follows is unverified."
---

**A gate that runs over a *generated* corpus proves a property of that corpus, not of your code.**
When the generator cannot produce the shape that separates two implementations, "byte-identical" and
"never asked" are the same reading — and the second one is indistinguishable from success for as long
as nobody looks.

`return_annotation` was on PGEN's CERTIFIED list, asserted byte-identical between the interpreter and
the generated parser over a deterministic corpus at three seeds. It carried a real divergence the
whole time. The discriminator was a single-quoted string whose content begins with a space or tab —
an entirely ordinary input the bounded stimuli generator simply never emitted across 89 samples.

## The rule

**Add the discriminating row FIRST, with no code change, and require the gate to go RED.**

```text
step 1 — corpus row only, NO code change   →  DIVERGE samples=89 agree=80 diverge=8   ← must happen
step 2 — land the fix                      →  4 passed ✅
```

Step 1 is the load-bearing one. It converts *"I believe there is a hole"* into *"the gate now
demonstrates the hole, and did so before I touched anything."* Do it the other way round — fix first,
add the test after — and you can never distinguish a row that catches the defect from a row that
would have passed anyway. That is the difference between a regression test and a decoration.

## Pair every discriminating row with its control

A corpus addition that fails everywhere is not evidence either. Each leading-layout row went in
beside its no-layout twin:

```
"-> {k: 'abc'}",     "-> {k: ' abc'}",
"-> {k: \"abc\"}",   "-> {k: \" abc\"}",
```

The controls pass in both arms; the discriminators fail before and pass after. Now the corpus is
shown to return **both** readings, which is what
[[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]] asks of any
instrument before its output is cited.

## The corollary about the fix itself

Once the row exists, prefer a fix that removes the possibility rather than one that re-synchronises
two copies. Here the interpreter was hand-mirroring two codegen decisions; the fix made it **call
codegen's own kernel** instead, so the two sides are the same code and cannot drift again. A synced
duplicate closes this instance; a deleted duplicate closes the class.

## When the corpus is generated, ask this early

- What shape would separate these two implementations?
- Can the generator emit it? (Depth ladder, seeds, bounded quantifiers, character classes.)
- If not, the green reading is about the generator. Write the row by hand.
