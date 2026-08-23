---
id: half-apply-a-mirrored-change-and-let-the-differential-find-the-inputs
title: To find the inputs a change actually moves, apply it to ONE side of a differential and let the gate enumerate them
answers:
  - "how do I find which inputs my behaviour change actually affects"
  - "I have three reproducers and they all pass — how do I know the accept set did not move"
  - "how do I measure the blast radius of an engine change without a corpus of my own"
  - "what is a cheap corpus-wide oracle for a codegen or runtime guard change"
  - "my change is mirrored in codegen and the interpreter — can I use that"
  - "how do I prove a clean differential sweep is not just a corpus gap"
  - "is a green equivalence gate evidence that my change is inert"
  - "how do I turn a regression gate into a discovery instrument"
tags: [differential, evidence, blast-radius, parse-harness, gates, controls, interpreter]
date: 2026-08-23
status: current
evidence: GRAMMAR-WELLFORMED.H.16.2b (PGEN-GRAMMAR-WELLFORMED-0178). The comment-skip guard is spelled identically in codegen and in `parse_harness_interpreter.rs`. Applying the prefix-test change to the INTERPRETER ONLY and running `parse_harness_equivalence_gate` turned a regression gate into a discovery instrument over 11 grammars × a deterministic stimuli corpus × seeds 0/7/42; it came back 4/4 CLEAN. The RED control — `allow_comment_skip = false` on the same line — failed immediately with `rtl_frontend DIVERGE samples=143 agree=138 diverge=5`, `systemverilog_preprocessor DIVERGE samples=75 agree=73 diverge=2`, `ebnf DIVERGE samples=81 agree=78 diverge=3`.
reverify: "Apply the change to one side only, then: make -C rust SHELL=/bin/bash parse_harness_equivalence_gate   # divergences = the inputs the change moves. Then replace the same expression with an obviously-wrong value and re-run; it MUST go red, or the sweep cannot see your change and its clean reading means nothing."
---

**A differential gate normally answers *"are these two implementations still equal?"*. Break the
symmetry on purpose and it answers a much more useful question: *"which inputs does my change
move?"* — enumerated, over the whole corpus, for free.**

The setup is any place where one behaviour is deliberately implemented twice and held equal by a
gate. In PGEN that is codegen versus `parse_harness_interpreter.rs`, held byte-identical over a
deterministic stimuli corpus (11 certified grammars, seeds 0/7/42) by
`parse_harness_equivalence_gate`.

## The technique

1. Apply your change to **one** side.
2. Run the differential.
3. **Every divergence it reports is, by construction, an input on which your change matters.** Not a
   guess, not a hand-built reproducer — the corpus found it.
4. Apply the change to the other side and confirm the gate returns to green.

Hand-built reproducers cannot do this. `H.16.2b` had three, all carefully chosen at the one site the
census said was behavioural, and all three passed in both arms. Three passing reproducers license
nothing about a corpus.

## ⛔ The step that makes it evidence

A clean sweep is worth **nothing** until you have shown the sweep can go red *on the thing you
changed*. The corpus may simply not reach it, and "no divergences" reads identically in both cases.

So run a deliberately-wrong version of the same expression and require it to fail:

```text
prefix test      →  gate 4/4 CLEAN
allow_comment_skip = false  (RED CONTROL)
                 →  rtl_frontend DIVERGE 5 · systemverilog_preprocessor DIVERGE 2 · ebnf DIVERGE 3
```

Now the clean reading is a measurement. Without the control it is an absence of evidence wearing the
shape of evidence — the failure
[[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]] names.

## When it applies, and when it does not

- ✅ Any mirrored implementation with a differential gate over a real corpus.
- ✅ Especially good for changes whose blast radius is a *language* question ("does the accept set
  move?") rather than a *count* question.
- ⛔ It measures only what the corpus reaches. A clean result plus a passing red control says *"no
  input in this corpus discriminates"*, which is strictly weaker than *"no input exists"*. Say which
  one you are claiming.
- ⛔ Restore the symmetry before committing. A half-applied change left in the tree is a divergence
  the next author will inherit as a mystery.
