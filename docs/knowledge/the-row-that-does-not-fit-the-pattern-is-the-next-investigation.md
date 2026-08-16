---
id: the-row-that-does-not-fit-the-pattern-is-the-next-investigation
title: When a table has one row that does not fit the pattern, that row is the next investigation — not a rounding error
answers:
  - "eight of my ten rows agree and two are slightly off — can I ignore the two"
  - "how much should a small anomaly in a measurement table be chased"
  - "my derived artifacts are mostly consistent; is 'mostly' good enough"
  - "how do I know my generated output matches the source that is supposed to produce it"
  - "what is the cheapest way to test whether a derived artifact is reproducible"
  - "I found the answer I was looking for — when do I stop"
tags: [measurement, evidence, derived-artifacts, reproducibility, process]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.19 slice 2, 2026-08-16 session #241. A codegen change was attributed by measuring its byte delta across all ten generated parsers against a pre-change snapshot. Eight rows read +2 578 bytes — a constant across artifacts spanning 260 KB to 143 MB and 1 488 to 1 608 rules, which is what proved the change was a FIXED emitted block. Two rows read +2 616. The attribution the slice was chasing was already complete without them; the +38 was small, on two rows, and orthogonal to the question. Chasing it anyway found that `generated/` is NOT reproducible from HEAD - both annotation parsers carry a `self.coverage_deltas.clear();` line the tracked generator cannot emit (`grep -c` -> 0, `git log -S` -> no commit ever) and whose absence the generator's own source comment deliberately defends. They had been emitted from an uncommitted editor state ~80 minutes before the commit that finalised the emission. Those two parsers are the pair the annotation backend links to generate every other parser, and the project's layer-A resume pointer recorded "generated/ FRESH" throughout.
reverify: "bash docs/tasks/artifacts/engine_universal_services/es19_residual_attribution/reproducibility_probe.sh   # ~3 s, exit 0. It was observed RED before ENGINE-UNIVERSAL-SERVICES.29 (a) repaired the pair — see reproducibility_probe_BEFORE_repair.txt beside it"
---

**A measurement table is two things at once: an answer to the question you asked, and a census of
the population you asked it about.** The second is free, and it is where the findings you did not go
looking for live. An outlier row is not noise around your answer — it is a *different fact about a
member of your population*, delivered at no extra cost.

The temptation to round it off is strongest exactly when the table has already answered the
question. That is the moment the outlier looks like a distraction rather than a lead.

```text
json_parser.rs                        +2578  ┐
regex_parser.rs                       +2578  │
rtl_const_expr_parser.rs              +2578  │  eight rows, one constant
rtl_frontend_parser.rs                +2578  │  across a 550x size range
scratch_parser.rs                     +2578  │  ⇒ a FIXED emitted block.
systemverilog_parser.rs               +2578  │     Question answered.
systemverilog_preprocessor_parser.rs  +2578  │
vhdl_parser.rs                        +2578  ┘
return_annotation_parser.rs           +2616  ← +38.  "close enough"
semantic_annotation_parser.rs         +2616  ← +38.  "same cause, surely"
```

The +38 was one emitted line. That line was the thread that unravelled a much larger fact: two of
the tree's derived artifacts could not be reproduced from the tracked source at all.

## Why the outlier is disproportionately informative here

The eight consistent rows are consistent *because they share a cause*. A row that breaks the pattern
is, by construction, telling you it has an **additional** cause — one your investigation has not
named. That is the definition of a lead. And the smaller the anomaly, the more likely it is that
nobody has looked, precisely because it is small.

⭐ Two properties make chasing it cheap, and both generalise:

- **The anomaly is already isolated.** You have a population, a control (the eight), and a treatment
  (the two). That is a better-conditioned experiment than most investigations start with.
- **The delta is usually literal.** `diff` the two artifacts and read the lines. Here it was 38
  bytes; the answer was one line of emitted code and it took a single `grep -c` against the tracked
  generator to know it was impossible.

## The general test this is an instance of

For any derived artifact — generated code, a lockfile, a compiled schema, a snapshot — the question
*"is this what the source produces?"* is answered by **re-deriving it and diffing**, and almost
nothing else answers it:

- a hash of the artifact proves only that it has not changed since someone recorded the hash;
- a determinism gate that regenerates from scratch and compares cycle *N* to cycle *N+1* proves
  regeneration converges — it **overwrites** the artifact in cycle 1, so a stale on-disk copy is
  invisible to it by construction;
- a green build proves the artifact compiles, not that it is current.

⛔ And when re-deriving, hold the **provenance inputs** fixed too. A generated artifact can embed its
own output path or a timestamp, in which case two byte-equivalent derivations differ in size for
reasons that have nothing to do with the source — see
[[a-hypothesis-list-is-a-snapshot-of-what-you-knew-that-day]], where exactly that cost a task leaf
three sessions on two wrong hypotheses.

## The honest bound

This card argues for chasing a *specific* kind of anomaly — one inside a population you have already
measured, where the follow-up is a diff. It is not an argument that every loose end deserves a
session. The discriminator is cost: if isolating the outlier is one `diff` away, the expected value
is high because the population work is already paid for. If it needs a new experiment, route it and
move on.

Related: [[a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness]],
[[a-hypothesis-list-is-a-snapshot-of-what-you-knew-that-day]].
