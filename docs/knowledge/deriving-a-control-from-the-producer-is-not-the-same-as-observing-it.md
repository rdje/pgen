---
id: deriving-a-control-from-the-producer-is-not-the-same-as-observing-it
title: Deriving a control from the PRODUCER'S SOURCE is not the same as observing the producer's OUTPUT — a control read off a `format!` is still a string you typed
answers:
  - "I derived my test cases from the emitting code instead of from the docs — is that enough"
  - "how do I know a control I wrote is actually testing something real"
  - "my control suite passes but I have never seen the thing it pins"
  - "is a positive control for a shape that never occurs worth keeping"
  - "how do I tell defensive coverage from live coverage in a classifier"
  - "my probe confirmed my hypothesis — why is that not evidence"
  - "how do I design a probe that separates two hypotheses instead of illustrating one"
tags: [instruments, controls, evidence, classifiers, probes, falsification, measurement]
date: 2026-08-15
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.21 slices 1-2. Slice 1 fixed a classifier by deriving its eight shapes from the two emitters (the right move, and it corrected a 4x error). One of the eight, `{rule}_lr_alt{n}`, was covered by a control string typed from reading `rust/src/ast_pipeline/mod.rs:3244`; **no parser had ever emitted one** (0 across all 10 generated parsers), so a mis-read of that `format!` would have reproduced as a PASSING control. Slice 2 observed it: 3 scratch-slot probes, the engine's own well-formedness checker naming `expr_lr_alt1`, and a discriminating probe that moved the alternative to position 1 and got `expr_lr_alt2` — refuting the rival "running count" reading.
reverify: "python3 stimuli/sv/corpus_parse_cost.py --verify-families   # 10 parsers, 131 declared LR names, 131 classified, `alt` absent from every shapes column — the arm is DEFENSIVE, and the instrument says so"
---

**Deriving a classifier from the code that emits the thing is the right move — and it does not, by
itself, make the control an observation.** A shape read off a `format!` string is a claim about how
you read that line. Nothing has emitted it. If you misread it, the control agrees with you.

This is the second-order version of *"derive from the producer, not from a description of the
producer"* (`docs/CLAIM_VERIFICATION.md` §3 leg 2). That rule kills the worst failure — a test and
an implementation descended from the same prose. It leaves a smaller one alive: a test and an
implementation descended from the same **reading of the same line**.

PGEN's parse-cost classifier had exactly this. Slice 1 correctly rebuilt it from the two
left-recursion eliminators, enumerating **eight** emission shapes where the previous version knew
three. Seven of the eight were pinned by names the shipped parser really declares. The eighth,
`{rule}_lr_alt{n}`, was pinned by `expression_lr_alt1` — a string typed while reading the emitter.
The parser declares **zero** `_lr_alt` rules; so do the other nine families. The control could
never have failed, and it could never have passed for the right reason either.

## What observing it actually took, and what it found

Three probes on the blessed scratch slot, all preserved as tracked artifacts:

1. **The well-formed case retracts it.** A plain `expr := expr "+" term | expr "-" term | term`
   hoists two `_lr_alt` rules; the planner consumes them and the retraction deletes them. The
   generated parser declares `expr_lr_base`/`expr_lr_suffix` and no `_lr_alt` at all.
2. **The surviving case is refused, and the refusal NAMES it.** The one shape that leaves a hoist
   unconsumed is rejected by grammar well-formedness before codegen:
   `rule 'expr_lr_alt1' has no finite terminal derivation`. ⭐ That is the observation — and it
   comes from an oracle built for another purpose entirely, which is what makes it worth more than
   another read of the emitter.
3. **The confirming probe was not yet a test.** Probe 2 hoisted the alternative at position 0,
   where *"the suffix is the alternative's index + 1"* and *"the suffix is a running count of
   hoists"* predict the same `_lr_alt1`. Consistent with both hypotheses ⇒ an illustration, not a
   test. Moving the only direct alternative to position 1 separated them: the engine emitted
   `_lr_alt2`. **Move the thing, then look.**

## The finding that only observation could produce

`_lr_alt` is unreachable in any shipped parser today, by **two independent mechanisms** — the
retraction, and the well-formedness refusal of the only shape that escapes it. That does not mean
delete the arm: a classifier must describe the EMITTER, not this month's grammar. It means the
honest label changes from *"one positive per emission site"* to *"seven live arms and one
**defensive** arm, and here is the measurement that says which is which."*

⛔ **The cost of not knowing which you have.** An unobserved arm is indistinguishable from a
correct one while it sits in a passing suite, and it is the arm that silently stops matching when
the emitter is refactored. Write down, per arm, whether an artifact has ever produced it.

## The rule

- Derive the shape list from the producer's **source**. (Necessary — it caught a 4x error here.)
- Then make the producer **emit** each shape and read the name off the output. Any oracle that
  already prints the name — a linter, a diagnostic, an error message — is free and is *not yours*,
  which is precisely its value.
- If a shape cannot be made to occur, say so **in the instrument**, and keep the arm as declared
  defensive coverage rather than letting it read as evidence.
- A probe that confirms at the default position has not separated your hypothesis from its rival.
  Change the input so the two predict **different** outputs, then run it once.
