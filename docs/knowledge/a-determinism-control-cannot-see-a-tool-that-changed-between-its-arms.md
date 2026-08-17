---
id: a-determinism-control-cannot-see-a-tool-that-changed-between-its-arms
title: A "run it twice, demand identity" control certifies the tool did not vary WITHIN the repeats — not that it was the same tool for the rest of the measurement
answers:
  - "my A/B says something other than the variable I changed also moved — what should I check first"
  - "is it safe to run a measurement while a build is finishing in the background"
  - "my determinism arm passed but the later arms all failed — what does that combination mean"
  - "how do I make a probe notice that its own tool was swapped mid-run"
  - "two arms of one script used different binaries — how is that even possible"
  - "why did my probe report a confident finding that turned out to be a stale binary"
  - "what does an A/B need to record besides its inputs and its result"
tags: [instruments, measurement, evidence, determinism, controls, build-flow, probe-banks]
date: 2026-08-17
status: current
evidence: |
  ENGINE-UNIVERSAL-SERVICES.31 slice 2, session #243. The `es31_label_hoist` bank ends by REBUILDING
  the tree's `ast_pipeline` from the restored source (~74 s). Seeing its final result line, I
  launched `es19_path_embedding/probe.sh` — which reads `rust/target/debug/ast_pipeline` — while that
  rebuild was still running. es19 generates four SystemVerilog parsers in order A1, A2, B1, B2. A1
  and A2 ran against the OLD (un-hoisted) binary and were byte-identical to each other, so **ARM 1,
  the determinism control, PASSED**; by the time B1 ran, cargo had replaced the binary with the
  hoisted one. The probe then reported **4 failed arms**, including *"normalised arm A != arm B —
  something OTHER than the path also moved"* and *"the path model is incomplete"*. Both are
  plausible, alarming, and false. The tell was in the printed header the whole time:
  `path sites: 31761 (long arm) / 1 (short arm)` — one number per emission era, in one run of one
  script. Re-run on a quiescent tree: **all 6 arms pass**, `path sites: 1 / 1`, live gap 3 == 1 × 3.
reverify: "bash docs/tasks/artifacts/engine_universal_services/es19_path_embedding/probe.sh   # exit 0, 6/6, and its header must print the SAME site count for both arms"
---

**A determinism control is a sample, not a guarantee.** "Run the tool twice at the same inputs and
demand byte-identity" answers exactly one question — *did the tool vary across those two
invocations?* It says nothing about the invocations that come after, and if the tool is a file on
disk that something else is rewriting, "after" is where the variation lives.

## The shape

```text
t0  bank B starts rebuilding rust/target/debug/ast_pipeline   (~74 s, in the background)
t1  probe P starts, reads that same path
      A1 = gen(long spelling)   <- OLD binary
      A2 = gen(long spelling)   <- OLD binary     ARM 1 "determinism" ✓  A1 == A2
      ...cargo finishes writing the binary...
      B1 = gen(short spelling)  <- NEW binary
      B2 = gen(short spelling)  <- NEW binary
t2  P reports: "arm A != arm B once normalised — something other than the path moved"
```

Every arm did its job. ARM 1 correctly reported that two adjacent runs agree. ARM 3 correctly
reported that A and B differ by more than the variable under test. The **conjunction** is the
diagnosis — and no arm was looking at the conjunction.

## Why this fails in the direction that invents work

The false verdict is not "no difference"; it is *"there is an unexplained difference"*. That reads
as a **finding**: a model gap, a new hypothesis, a leaf worth opening. In this repository that exact
sentence — *"something other than the path moved"* — is the false verdict that founded
`ENGINE-UNIVERSAL-SERVICES.19` on two hypotheses that were both wrong, and here it was manufactured
a second time by an operational mistake rather than a measurement one.

## What actually catches it

- ⭐ **Make each arm print something that identifies the TOOL, not just the input.** es19 already
  printed `path sites:` per arm; the two numbers disagreed by four orders of magnitude in a header a
  reader skims. Print it, and compare it — a per-arm tool identity that the script itself asserts
  equal is the cheap, general fix. (`PARSE-COST-RATCHET` arm 5 is the mature version of this: the
  probe is asked *which* generated parser it embeds.)
- ⭐ **Do not read a tool's mtime or a job's last log line as "the job is finished."** A background
  build's final *message* precedes its final *write*. Wait on the process, not on the prose.
- ⚠️ **Repeats detect flakiness; they do not pin provenance.** If a measurement can be contaminated
  by something outside it, only an explicit identity check on that something closes the hole.

## The generalisation

> A control answers the question it asks. Before trusting a suite, ask what the arms *jointly*
> fail to ask — here, *"was this the same tool for all four generations?"*, which no single arm
> owned and which the passing determinism arm made feel already answered.

Related: [[a-conservation-control-cannot-catch-a-misassignment]] (a control whose invariant is
preserved by the defect), [[a-check-whose-inputs-all-pass-has-not-been-tested]] (a control that
cannot go red), [[a-mutation-control-tests-the-layer-you-mutated-not-the-property]] (a control that
tests the wrong layer). This card is the fourth member of that family: **a control that tests the
right property over the wrong interval.**
