# PGEN-RGX-0078-0187 result

Verdict: **HOLD; telemetry does not supply implementation margin.**

The exact rollback telemetry expansion finds 63 inlined bare-target sites:
39 ordinary two-counter sequences and 24 tournament four-counter sequences.
Their 402-PC union is disjoint from accepted G1-C/thin-memo-success work,
`-0185` bare diagnostics, and `-0186` lookup work.

Only **3/7/3** raw samples hit the mechanism. Weighted across the three bands,
that is **0.017994867% = 0.227347146 ns**. Added to the pre-lookup held bundle,
30% capture remains **1.003506256 ns short** of the 28.8 ns noise floor. Added
to `-0186`'s deliberately optimistic lookup composite, it creates only a
**0.143875763 ns** sliver before mandatory direct-index replacement work that
is still priced at zero. This is not honest margin.

The source reader audit also catches a contract constraint: the rollback
counters are diagnostic-only for verdicts, but the public
`SemanticRuntimeState::counters()` accessor promises cumulative values and has
no pre-parse observer latch. Maintained outcome dumps already force the
protocol twin and are safe; external after-parse inspection is observable.
Counter elision therefore requires an explicit public opt-in/compatibility
design even if a later batch becomes measurable.

No source, emitter, generated artifact, corpus, probe, capture, floor, MAX,
public contract, or product behavior changed.
