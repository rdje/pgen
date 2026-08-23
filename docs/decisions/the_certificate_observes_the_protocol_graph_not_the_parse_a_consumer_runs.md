# The certificate observes the PROTOCOL graph, not the parse a consumer runs

- **Category:** feedback (measurement discipline)
- **Date:** 2026-08-23
- **Found by:** `GRAMMAR-WELLFORMED.H.16.4a` (`PGEN-GRAMMAR-WELLFORMED-0177`)

## Context

`certificate_coverage` is this repository's headline trustworthiness number, and it is the
metric almost every grammar/codegen leaf reports a before→after on. It verifies a witness by
parsing a generated sample through the **real generated parser** — which is exactly why it is
trusted, and exactly where the trap is.

Every generated parser computes, at the top of `parse()`:

```rust
self.bare_parse = !self.coverage_enabled && !self.logger_enabled
    && !self.counters_observed.get()
    && !crate::ast_pipeline::report_memo_stats_enabled();
```

The witness hook (`parser_registry::parse_and_cover_<family>`) calls `parser.enable_coverage()`
before parsing — it has to, because the parser's own coverage stack is how the committed-rule
set is recorded. So **`coverage_enabled` is true for every certificate run**, `bare_parse` is
false, and the parse the certificate observes is the **PROTOCOL** graph.

A parse a real consumer runs (`parse_full_*`, no logger, no counters, no memo stats) is
`bare_parse` and runs the **FUSED `cascade_*`** graph — a *separate emission* from a separate
codegen site.

## Decision

⛔ **A certificate delta is not evidence that a codegen change reached the shipped parse.**
When a change edits an emission path, say WHICH graph the measurement observed, and confirm the
fused graph with an oracle that runs it.

MEASURED, and this is the founding case rather than a hypothetical. `H.16.4a`'s first cut
changed the layout-skip decision at **one** of the six codegen sites that spell it — the
`generate_atom_logic` site, which emits the protocol graph. The certificate moved to its
honest post-fix value (`ebnf` `144/0/112/32` → `144/0/113/31`, at all three seeds, with the
delta attributable by rule name) while a production parse was **completely untouched**. Every
counter-based instrument in `TOOLBOX.md` §3.1–§3.6 routes to the protocol graph too, so none of
them could have disagreed.

What caught it was `parse_harness_equivalence_gate` (TOOLBOX 1.6), the one oracle whose
generated-parser side is a bare `parse_full_*`:

```
ebnf   DIVERGE samples=81 agree=60 diverge=8 (+13 suppressed)
       first=Ast: AST differs at byte 32 (interp_len=149 oracle_len=112)
       interp …"elements":[{"content":"   ","type":"whitespace"}]…
       oracle …"elements":[],"type":"grammar_file"…
```

## Consequences

- The oracle for "did this reach the shipped parse?" is
  `make -C rust SHELL=/bin/bash parse_harness_equivalence_gate`, or any
  `parseability_probe --parse` without a diagnostic consumer. Not the certificate, and not any
  `--dump-rule-*-counts` reading.
- ⭐ The generalisation is the one `TOOLBOX.md` §3.8 already states for *performance* — "the
  counters cannot answer this: they all route to the PROTOCOL graph, and a production parse runs
  the FUSED one" — but §3.8 frames it as a profiling hazard. It is a **correctness** hazard too,
  and on the metric the repository leads with.
- The structural repair in `H.16.4a` was to give the decision **one definition with six
  callers** and a test that pins the caller count, so a future edit cannot reach one graph and
  miss the others. The observability twin makes that class of miss silent in the PASSING
  direction; a single definition is what removes the opportunity.

Related: [[an-instrument-can-be-green-in-both-arms-of-the-defect-it-describes]],
[[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]],
[[a-control-that-clears-your-hypothesis-has-not-cleared-the-symptom]].
