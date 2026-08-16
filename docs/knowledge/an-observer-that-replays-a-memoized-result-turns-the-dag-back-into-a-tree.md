---
id: an-observer-that-replays-a-memoized-result-turns-the-dag-back-into-a-tree
title: A cache makes work linear by SHARING a result; an observer that records the shared result once per USE materialises the DAG back into a tree — exponentially, and only where the cache was helping most
answers:
  - "my instrumented run hangs or OOMs on an input the uninstrumented run handles instantly"
  - "why does enabling coverage or tracing make a memoized parse blow up"
  - "how do I record per-event coverage without undoing my memoization"
  - "my profiler says the program is 99 % memcpy — where do I look"
  - "is it safe to derive failed-speculation counts as raw minus committed"
  - "how do I prove a slowdown is exponential rather than just large"
  - "a debug-only feature is unusable on one input — is that a real defect"
tags: [instruments, performance, caching, memoization, observability, measurement, evidence]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.22, PGEN. A 2 787-byte SystemVerilog file parses in 0.108 s bare and dumps per-rule ENTRY counts in 0.056 s (200 975 entries), but the OUTCOME dump — same execution graph, one extra recorder — peaks at 13.7 GB RSS within ONE SECOND on a 24 GB machine and never finishes. `/usr/bin/sample`'s own per-symbol table:  `_platform_memmove` 4302 of 4314 worker samples = 99.7 % of CPU. Cause:  `memoized_call` stored `coverage_stack[checkpoint..].to_vec()` in every memo entry and replayed it with `extend_from_slice` on every hit. Growth law measured on an else-if ladder: rule entries LINEAR (+19 295 per arm, constant), coverage stack EXPONENTIAL (x4.00 per arm), 353 005 042 slots at 7 arms; the real file has 9 else-if arms, and the extrapolation forecast 6 051 461 940 against a later-measured 5 648 150 434 — within 7.1 % of a number no instrument in the repository could produce at the time. FIXED the same day by storing an index and folding multiplicities: the file now dumps in 0.04 s and every previously-reported number is byte-identical.
reverify: "bash docs/tasks/artifacts/engine_universal_services/coverage_stack_blowup/verify_fix.sh   # ALL CHECKS PASS: the eight pinned rungs reproduce exactly and the pathological file dumps in 0.04 s. The DEFECT itself is no longer reproducible by command — it was removed from the engine — so its measurement is preserved as a file beside that script: probe_before_fix.txt"
---

**A memo's whole value is that a shared subresult is computed once and *reused*.** An observer that
records "what happened" per USE, by storing the recorded region and replaying it on every cache hit,
re-expands every reuse into a fresh copy. The computation stays a DAG; the *record of it* becomes the
tree the DAG was compressing. The blow-up is exponential in the nesting depth, so it is invisible in
testing and total in production.

```rust
// insert: copy everything the body recorded
let delta = Some(self.record[checkpoint..].to_vec());
// hit: replay it, as if the body had run again
self.record.extend_from_slice(delta);
```

The shape generalises far past parsers: a build cache with per-target provenance logging, a
memoized resolver that appends a trace span per hit, an incremental compiler recording touched
symbols per query. Anywhere a cache exists to collapse repeated work, a per-use recorder re-inflates
exactly what the cache collapsed.

## Three things this makes concrete

**1. The failure mode is opposite to intuition — it is worst where the cache helps most.** Inputs
with no reuse are unaffected. The pathological input is the one whose DAG has the highest sharing
factor, i.e. precisely the input the cache was added for. An instrumented run that is fine on a large
file and impossible on a small one is this signature.

**2. It corrupts a derived quantity, quietly.** The record's length was being read as *committed
work*, and `raw_invocations − committed` published as *wasted speculative work*. With replay, the
committed count includes subtrees never re-entered, so the difference can go **negative**: measured
`committed / entries` from **0.62× to 2 173.88×** across eight rungs. ⛔ It was positive on all 192
rows of the pinned sample — but that sample was selected from a census that *dropped* the file where
it fails, so the check was structurally unable to see it. **An identity that holds on your sample is
not an invariant; ask what your sample selection excluded.**

**3. "Bigger timeout" is not on the option list, and a growth law is what proves it.** A single hang
justifies a longer wait; a measured `×4.00 per arm` does not — the next input has one more arm. ⭐
Emit the construct at 0..N and put the linear column next to the exponential one. Two columns on the
same axis turn "it hangs" into "it cannot be waited out", and that changes which fixes are even
admissible.

## Why the obvious fix is usually wrong

Deleting the replay is not free: it was added for **completeness**. A subtree first computed inside a
speculation that later rolled back leaves a live cache entry with no surviving record; the later
cache hit is then the *only* place that subtree can be recorded. Remove the replay and the record
silently under-reports. ⇒ the correct fix keeps the replay and stops it from being a COPY — store a
reference to the cached region and fold the DAG once at read time, counting multiplicity in integers
rather than materialising it in slots. The true multiplicity really can be 10^10; that is a number,
not a list.

⭐ The general rule: **a recorder attached to a cache must record the cache's DAG, not its
expansion.** If your record's size is a function of the *unshared* work, the cache is decorative.

## What shipping it actually looked like

Store an index, not a copy; leave one tagged marker where the range used to be (so the miss path and
the hit path are symmetric and storage is strictly linear); fold once at read time in descending
id order, carrying an integer multiplier. The descending order needs no work list and no cycle
check: an id is allocated when a body COMPLETES, so every marker inside a record references a
strictly smaller id.

⭐⭐ **The verification is what makes it a fix rather than a rewrite.** Because the change is meant to
alter computation and not meaning, every previously-measurable number must come out identical — so
pin them BEFORE you touch anything. Here: eight exponentially-separated integers from the ladder
(reproduced exactly through 353 005 042), a 192-row measurement file (byte-identical), and a
coverage report (character-identical). An off-by-one in a multiplicity fold cannot survive any of
them. ⛔ Pinning them afterwards would have proved only that the new code agrees with itself.

⚠️ **One caveat survives the fix, and it is the useful half.** Making the multiplicity computable is
what proved `raw − committed` can go negative — so the derived-quantity hazard in point 2 above is
now *checkable* rather than merely *latent*. Fixing the blow-up did not fix the arithmetic.

⛔⛔ **And it stopped being hypothetical the moment the fix landed.** Over the whole 16 336-file
corpus the subtraction is now `899 264 997 − 5 682 584 657 = −4 783 319 660`. Before the fix the
same subtraction was **positive**, and only because the input that dominates the quantity was being
silently dropped. ⇒ **restoring a dropped input can invert an aggregate you had been reading for
months.** The number was never right; it was arithmetically comfortable.

Related: [[a-conservation-control-cannot-catch-a-misassignment]] — the control that would have caught
this earlier is a per-input comparison against the uninstrumented run, not a total.
