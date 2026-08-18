---
id: a-delta-is-a-claim-about-a-join
title: A before/after delta is a claim about a JOIN — join on the outcome and you measure only the rows that crossed it, systematically under-reporting the fix
answers:
  - "I re-ran a corpus before and after a fix and diffed the results — is that delta trustworthy"
  - "how should I compute what a parser change actually moved"
  - "my fix moved N files from fail to pass — is N the whole benefit"
  - "a routed leaf already measured the delta; do I re-derive it or promote it"
  - "why did the published bar disagree with the transition set it was derived from"
  - "a row's classification changed but its pass/fail did not — how do I catch that"
  - "how do I audit a corpus promotion before publishing the number"
tags: [measurement, corpus, oracles, instrument-honesty, verification, adjudication]
date: 2026-08-14
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .13h (routed 309->303, re-derived 309->302); docs/tasks/artifacts/sv_corpus_grad/es17_slice9_flip/corpus_transitions.tsv (the 13th row and the join rule); stimuli/sv/characterization/adjudication_manifest.tsv (unexplained_rejects_valid 288->281, explained_svpp_include 139->140)
reverify: "bash scripts/check_sv_corpus_denominator.sh   # re-derives the adjudicated/routed/no-verdict/dark/bar tuple from the tracked manifest and fails if a published anchor disagrees — the digits are deliberately NOT quoted here, because this card had carried .../302 for several slices after the bar moved"
---

**Pick the join column wrong and the delta is not merely imprecise — it is blind to a whole class of
movement, and blind in the flattering-to-nobody direction of under-reporting your own fix.**

The concrete failure: a parser fix was measured by re-running a 16 336-file corpus and joining the
two runs per path **on the observed `pass`/`fail` verdict**. That gave 12 transitions and a defect
bar of 303. The real bar was **302**. One row is `fail` on both sides and moved anyway:

```text
sv2v  test/core/string_byte_order.sv
      divergence:unexplained_rejects_valid  ->  divergence:explained_svpp_include
```

Because the row's *classification* did not depend only on whether it parsed. It depended on **where
the parse stopped**:

```systemverilog
localparam b = 64'("abcd");     // ← the construct the fix repaired
…
`include "string_byte_order.vh" // ← a SECOND, independent blocker
```

Before, the parse died on the cast — ahead of anything a preprocessor could reach — so the failure
was *unexplained*. After, the cast parses, the furthest position advances to the `` `include ``, and
the failure is *explained*. Same outcome, different meaning, one fewer known defect.

**The rule.** Join over the record that carries the *classification*, not the one that carries the
outcome — here `(suite, relpath) → (observed, adjudication)` from the manifest, not `(path, status)`
from the results file. Then assert the row SET is unchanged on both sides, so you can tell "a row
moved" from "a row appeared".

**Why the error runs one way.** A row with two independent causes hides its first repair completely:
the outcome cannot improve until the last cause is gone. So an outcome-joined delta systematically
**under**-counts a fix, and the more layered your corpus, the worse it gets. That is an argument for
measuring class transitions rather than outcome transitions whenever the classes carry the meaning.

**Corollary, and the reason this was caught at all: re-derive a routed number, never promote it.**
The delta arrived pre-measured in a routing note, with instructions amounting to "re-run and
publish". Re-deriving it independently cost one extra join and moved a published figure. A number
handed to you with its provenance is still a number you have not measured — and the cheapest audit
is to compute it a second way and require the two to agree.

⛔ **The tell that something is off is usually arithmetic that does not close.** Here: six rows moved
`unexplained → match`, but the `unexplained` class fell by **seven**. That single missing unit is the
whole finding. Before publishing any bucketed delta, make the buckets balance — every row that left
one class must arrive in a named other one.
