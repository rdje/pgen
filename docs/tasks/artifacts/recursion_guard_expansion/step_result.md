# PGEN-RGX-0078-0188 result

Verdict: **HOLD; the fixed name-stack subset has only 0.179 ns margin.**

The source audit confirms that modern generated frames already carry a
complete `(RuleId, position)` stack. The parallel name stack is still required
by public legacy methods and error/trace text, so the viable design is an
additive generated bare path with ID-to-name reconstruction—not a silent
change to `RecursionGuard`'s public methods or fields.

The custody-pinned classifier owns only four inlined name pushes, the name half
of one outlined `enter_id`, and nine inlined name pops. The exact **131-PC**
subset is disjoint from every accepted/held mechanism. It excludes cycle/depth
work, speculative rollback/name lookup, allocator children, and ABI traffic.

Dynamic counts are **71/74/92**, weighted **0.311889302% = 3.940409443 ns**.
Added to the honest pre-lookup bundle, the total becomes **96.595388589 ns**;
30% capture is **28.978616577 ns**, only **0.178616577 ns** above the 28.8 ns
noise floor. That sliver cannot absorb unpriced ID-to-name reconstruction or
ordinary model uncertainty. The deliberately optimistic thin-index composite
would have 1.325998596 ns margin, but still omits mandatory lookup replacement
work and is not a licensing basis.

No parser, runtime, emitter, generated artifact, corpus, probe, capture, floor,
MAX, public contract, mdBook, or product behavior changed. The next read-only
neighbor owns the non-overlapping `try_parse` name-stack rollback and exact
ID-to-name replacement proxy.
