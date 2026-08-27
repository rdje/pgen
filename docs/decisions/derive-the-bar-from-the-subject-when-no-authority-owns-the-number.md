---
name: derive-the-bar-from-the-subject-when-no-authority-owns-the-number
description: "DISCIPLINE (2026-08-27, ENGINE-UNIVERSAL-SERVICES.44(d) / PGEN-ENGINE-UNIVERSAL-SERVICES-0104) — when a quality bar seems to need a number nobody in the room owns, do not invent one and do not ship without one: check whether the SUBJECT already contains a defensible unit, and derive the bar from that. MEASURED: .44(d)'s first ruling declined to set a latency budget for a pathological-input rejection because 'a latency budget is a consumer-contract question and inventing one here would be a number with no authority behind it' — correct about invented numbers, and one step short. The parser's own DEEPEST ACCEPTED parse is a measured quantity on the same host, so the bar became 'a rejection may cost no more than 2x the deepest accepted parse', which needs no external authority, cannot rot across hardware the way an absolute second count does, and carries a one-sentence correctness argument: a parser must not spend more effort FAILING than SUCCEEDING. Both released families then measured RED by two orders of magnitude (SystemVerilog 82x, VHDL 267x), turning an open judgement call into a falsifiable exit condition."
id: derive-the-bar-from-the-subject-when-no-authority-owns-the-number
title: "When no authority owns the number, derive the bar from the subject — a refusal to invent is not a licence to ship without"
date: 2026-08-27
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .44 (d) — the first ruling (PGEN-ENGINE-UNIVERSAL-SERVICES-0097) declining a number, and the second (PGEN-ENGINE-UNIVERSAL-SERVICES-0104) deriving one; the accept/reject anchors are plain fused-graph runs of rust/target/release/parseability_probe on nested-parenthesis inputs; the consumer half is routed to leaf .57 (pending-external) and published in docs/book/src/embedding-and-downstream-integration.md
reverify: "P=rust/target/release/parseability_probe; python3 -c \"import sys;sys.stdout.write('module m; assign x = %s1%s; endmodule' % ('('*318, ')'*318))\" > /tmp/a.sv; python3 -c \"import sys;sys.stdout.write('module m; assign x = %s1%s; endmodule' % ('('*800, ')'*800))\" > /tmp/r.sv; for f in /tmp/a.sv /tmp/r.sv; do /usr/bin/time -p $P --parse systemverilog $f --profile sv_2017 >/dev/null; done   # the ACCEPT arm must still accept and the RATIO reject/accept is the bar; >2x is RED"
answers:
  - "how do I set a quality bar when the stakeholder who owns the number is not available"
  - "should I invent a performance threshold or leave it open"
  - "why is a ratio a better acceptance bar than an absolute time"
  - "what is the latency bar for a PGEN over-ceiling rejection"
  - "how do I turn a judgement call into a falsifiable exit condition"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

There is a tempting third option between **inventing a number** (indefensible) and **leaving the bar
open** (unshippable):

> **Look for a defensible unit inside the subject itself, and state the bar as a relation to it.**

A bar derived this way needs no external authority, because it does not import one. It is not a
compromise between the two options — it is usually *better* than the number the absent stakeholder
would have given, because it moves when the subject moves.

## The case it comes from

`ENGINE-UNIVERSAL-SERVICES.44`(d) asked whether a ~12 s CPU burn on ~1.6 KB of pathological
SystemVerilog is acceptable. The first ruling said the burn was **not** acceptable but explicitly
declined to set a target:

> *"A latency budget for an over-ceiling reject is a consumer-contract question that belongs with the
> `embedding_api` contract and Nexsim's own limits, and inventing one here would be a number with no
> authority behind it."*

Every clause of that is true, and it still left the leaf with no exit condition — open until somebody
*judged* the burn tolerable. The missing move was to notice that the parser already contains a
measured quantity of the right kind: **the cost of its own deepest ACCEPTED parse.**

**The bar:** an over-ceiling rejection may cost no more than **2×** the same parser's deepest accepted
parse of the same input shape, same host, same run.

| family | deepest accepted | its cost | reject (depth 800) | ratio | verdict |
|---|---:|---:|---:|---:|---|
| systemverilog | depth 318 | 0.151 s | 12.43 s | **≈ 82×** | RED |
| vhdl | depth 310 | 0.071 s | 18.98 s | **≈ 267×** | RED |

## Why this particular form is the production-grade one

- **It cannot rot.** *"Under one second"* is a claim about one laptop. It needs a rebaseline on every
  new machine, and — worse — it goes quietly **green on faster hardware without the defect moving**.
  A ratio cancels host variance by construction.
- **The subject defines its own budget**, which answers the original objection instead of dodging it.
  Nobody's authority was borrowed.
- **It carries a one-sentence correctness argument**, which is what makes it reviewable: *a parser
  must not spend more effort failing than succeeding.* A success yields an AST; a rejection yields one
  diagnostic. Paying 82× more for the lesser product needs no appeal to anyone's latency budget to be
  indefensible.
- **Pick the multiple for a reason.** `2×` rather than `1×` because a reject legitimately pays for
  work an accept does not — the guard trip, the unwind, the diagnostic — so the bar leaves engineering
  headroom rather than demanding the parser beat its own accept path.

## The test that says you got it right

**A bar every subject already passes is worth nothing.** If the derivation lands somewhere everything
is already green, it was chosen to be comfortable rather than to be right. Here both released families
fail by two orders of magnitude — which is the finding, not an embarrassment.

## What it does NOT license

Deriving the engineering bar does not settle the stakeholder's question; it makes the work independent
of it. Keep the real consumer question **alive and separately owned** — here as a `pending-external`
task leaf plus a note on the consumer-facing book page — and be explicit that no work parks behind it.
State the direction the answer can move the bar (here: **stricter only**), because that is what makes
proceeding safe.

Related: [[feedback_instrument_needs_ground_truth]] is the same instinct one level down — a measurement
needs something outside itself to be judged against, and often the subject can supply it.
