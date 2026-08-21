---
id: a-conservative-encoding-in-a-channel-that-cannot-express-the-fact-is-not-free
title: When one field carries two kinds of fact, the one that does not fit gets encoded with the most conservative available value — and that encoding is a cost nobody prices
answers:
  - "one field carries verdicts from several sources and one of them has no natural value to write — what do I write"
  - "is it safe to widen a value to the most conservative option so the consumer stays correct"
  - "my cache went from fast to catastrophically slow after a soundness fix that measured clean — where do I look"
  - "a guard that is supposed to be a bound turned into an unbounded search — what class of bug is that"
  - "when is it right to split one taint channel into two"
  - "how do I know a taint scope is expressible in my cache key at all"
  - "a soundness fix was correct and measured — why did it still cost something enormous"
  - "should a resource-exhaustion verdict and a semantic verdict share a propagation path"
  - "how do I cache a failure whose validity depends on context the key does not carry"
tags: [caching, memoization, packrat, soundness, recursion, design, taint, engine, cost-model]
date: 2026-08-21
status: current
evidence: "PGEN `ENGINE-UNIVERSAL-SERVICES.43`. `SV-CORPUS-GRAD.3.12` routed three recursion-guard verdicts through one field — a parse-stack FRAME INDEX. Two of them (`Infinite`, `LeftRecursive`) name a blocking frame. The whole-stack DEPTH CEILING names none, so it wrote `0`: the index outside every rule. `.3.12`'s gate refuses to cache a failure whose block came from a frame outside the memoized rule, so `0` made that true for EVERY rule at `entry_depth >= 2` and ONE ceiling trip disabled FAILURE memoisation for the rest of the parse. Measured on the shipped SystemVerilog parser: 315 nested parens 0.15 s accepted, 316 0.23 s, 317 4.90 s, 318 NO RESULT in 30 s. `.3.12` itself was correct, measured, and fixed real wrong-rejections; the `0` was the unpriced half."
reverify: "bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/measure_cliff.sh 60   # SV rejects past ~320 in bounded time; before the split it returned nothing at 318"
---

**A channel designed around one kind of fact will eventually be handed another kind, and the fact
that does not fit gets written with whatever value is safest.** That value is chosen for
correctness, it *is* correct, and its cost is invisible at the moment of writing — because the cost
is not paid by the code that writes it. It is paid by every consumer downstream that reads the field
as if it meant what the channel was designed to mean.

PGEN's packrat memo refuses to cache a failure whose recursion-guard block came from a frame
*outside* the memoized rule: such an outcome depends on which rules the caller had on the stack, and
the memo key `(rule, position)` does not carry that. Sound, and it fixed real wrong rejections. The
field it reads is a frame index, and two of the three guard verdicts genuinely name a frame.

The third — a whole-stack depth ceiling — names no frame. There is no honest index for *"the entire
stack"*, so it wrote `0`. Frame `0` is outside every rule, so the gate became true everywhere, and a
single ceiling trip switched the cache off for the remainder of the parse. A bound became a search:
one character of extra input turned a 0.23-second parse into one that never returned.

**The diagnostic question is not "is this value sound?" but "does this channel have a vocabulary for
this fact?"** If the answer is no, a conservative encoding is a silent widening of scope, and scope
is what the consumer is deciding on.

**The repair is usually a second channel, not a cleverer value** — and the test for whether the
second channel can be *validated* rather than merely *refused* is whether the fact is monotone in
something the entry can carry. Here it was: a depth ceiling prunes strictly more from a deeper
stack, and extra pruning cannot turn a failure into a success, so a failure at depth `D` holds at
every depth `>= D`. That is a condition a cache entry can stamp and check. The frame-scoped verdicts
have no such ordering — a different stack blocks differently in either direction — so they stay
refused. Same subsystem, two verdicts, two disciplines, and the gates compose: the content-scoped
gate runs first, so an entry only reaches the depth-scoped one once content-dependence is excluded.

⛔ **The trap in reviewing this class is that the original change is not wrong.** Looking for the
defect in `.3.12` finds nothing: it is correct, it was measured, its own regression evidence is
real. The cost lives in the one arm whose fact the channel could not express — an arm the change's
own evidence had no reason to exercise. Ask which callers had to *widen* a value to use the new
interface, and price that widening as part of the design.
