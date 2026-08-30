---
id: a-decided-designs-premise-is-a-claim-not-an-instruction
title: A decided design's PREMISE is a claim, not an instruction — measure it before you execute on it
answers:
  - "the previous session decided the design so the next one can execute — what still needs checking"
  - "my task leaf says the source already carries X, do I need to verify that"
  - "how do I execute a decided plan without re-litigating it"
  - "I inherited a design decision — which parts of it are facts and which are assumptions"
  - "the leaf says derive from source A, but is A good enough"
  - "what is the difference between re-opening a decision and checking its premise"
  - "a plan written to stop the next session re-deciding also stopped it from measuring — how"
tags: [planning, task-trees, continuity, evidence, verification, handoff]
date: 2026-08-30
status: current
evidence: "DOCTRINE-GAP-OWNERSHIP.20 (PGEN-DOCTRINE-GAP-OWNERSHIP-0300). The leaf's own decided design read: 'DERIVE the chapter from the registry (which already carries a full paragraph per doctrine)'. Measured before building on it: the registry's 33 descriptions run min 56 / median 526 / max 6235 bytes, and 6 of 33 are a single clause under 120 chars — four of those six being among the 14 doctrines the leaf's census had found ABSENT from the book. The DOCTRINE_ENFORCEMENT.md §10 mirror carries the same claims at min 48 / median 1755 / max 12478, 2.5x the total prose. Executing the design as written would have shipped a structurally complete chapter that still only MENTIONED the six, which is exactly the bar the director had raised that day."
reverify: "bash scripts/report_doctrines.sh | tail -32   # the roster the corrected design renders; the depth line at the top of `bash scripts/report_doctrines.sh` reports median/thinnest/under-floor counts derived from the sources, so the premise is now re-measured on every run rather than asserted once."
---

**A leaf that says "the design is decided so the next session executes rather than re-litigates" is
doing the right thing. But a design is made of two different substances, and only one of them is a
decision.**

- The **decision** — *derive it, do not hand-write it; render it as a chapter, not 33 pages; gate it
  two-sided* — is a judgement the earlier session was entitled to make, and re-opening it is the
  waste the note exists to prevent.
- The **premise** — *because source A already carries a full paragraph per item* — is a **claim
  about the repository**. It was measured or it was not, and the note does not say which.

Executing a premise you did not check is not obedience to the plan. It is inheriting an unverified
measurement and spending it as though it were evidence.

## The measured instance

A leaf ordered a book chapter describing all 33 enforced doctrines, and pre-decided the design so a
later session would not have to re-derive it. One clause of that design carried a premise:

> DERIVE the chapter from the registry (**which already carries a full paragraph per doctrine**)

Checked before it was built on:

```text
registry description bytes : min 56   median 526   max 6235   total 35737
§10 "Proves" cell bytes    : min 48   median 1755  max 12478  total 90256
registry descriptions under 120 chars: 6 of 33
```

The premise is *partly* true — a median of 526 bytes really is a paragraph — and that is what makes
it dangerous. Six rows are a single clause, and **four of those six were among the fourteen
doctrines the leaf's own census had found missing from the book entirely.** Rendering the registry
alone would have moved those four from *absent* to *mentioned*, and the whole leaf existed because
the director had said, that same day, that a mention is not documentation.

The correction was small precisely because it was made early: take the population, the IDs and the
enforcer paths from the registry (which really is the source of truth for *which* doctrines exist)
and the description from the richer mirror, holding the two two-sided so the population still cannot
drift.

## The rule

**When you inherit a decided design, sort its sentences into decisions and claims. Execute the
decisions. Measure the claims.** A claim in a plan is not weaker for being in a plan — it is more
dangerous, because the framing invites you to read it as an instruction and instructions are not
things one checks.

Two practical corollaries:

- **A parenthetical is where premises hide.** The load-bearing assumption above was inside brackets,
  written as background rather than as a finding, and nothing in the leaf said whether it had been
  measured. If a plan's clause would change what you build when false, it is a claim.
- **Prefer a premise the artifact re-measures.** The fix here was not merely to check the number
  once: the produced chapter now derives and publishes its own depth census on every run, so the
  premise cannot quietly rot back into being wrong. A checked premise is a fact about one day; a
  derived one is a fact about today.

Related: [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] — the same shape aimed outward at the
repository; here it is aimed at your own plan.
