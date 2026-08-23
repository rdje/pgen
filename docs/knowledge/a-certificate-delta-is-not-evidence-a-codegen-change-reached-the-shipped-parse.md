---
id: a-certificate-delta-is-not-evidence-a-codegen-change-reached-the-shipped-parse
title: The certificate observes the PROTOCOL graph — a cert delta does not prove a codegen change reached the parse a consumer runs
answers:
  - "my codegen change moved the certificate — does that prove the shipped parser changed"
  - "which graph does certificate_coverage actually parse on"
  - "why does bare_parse matter when I am measuring a grammar or codegen fix"
  - "the cert moved but a real parse behaves the same — which one is wrong"
  - "which oracle proves a change reached the fused cascade graph"
  - "does enable_coverage change how the parse executes"
  - "I fixed one codegen emission site — how do I know there are not others"
  - "why do all the counter dumps agree with each other and still miss the defect"
  - "how do I stop a decision from being spelled at several codegen sites"
tags: [certificates, codegen, cascade, observability-twin, gates, evidence, parse-harness]
date: 2026-08-23
status: current
evidence: GRAMMAR-WELLFORMED.H.16.4a (PGEN-GRAMMAR-WELLFORMED-0177). The layout-skip decision was spelled SIX times across four modules; changing only `generate_atom_logic` (the PROTOCOL emitter) moved `ebnf` cert `144/0/112/32` → `144/0/113/31` at seeds 0/7/42, delta attributable by rule name, while a `bare_parse` production parse was untouched. `parse_harness_equivalence_gate` caught it — `ebnf DIVERGE samples=81 agree=60 diverge=8`, interpreter `"elements":[{"content":"   ","type":"whitespace"}]` vs oracle `"elements":[]`. Fixing three of six then made `cascade_match_*` and `cascade_build_*` disagree on `start_dynamic` and codegen emitted a parser that panicked on its own tape (`derivation-tape drift in rule 'whitespace': expected TokStart, found TokEnd(157319)`).
reverify: "make -C rust SHELL=/bin/bash parse_harness_equivalence_gate   # the ONLY standing oracle whose generated-parser side is a bare parse_full_*; 11/11 certified grammars byte-identical. Then: cargo test --features 'generated_parsers ebnf_dual_run' --lib layout_owning_terminal_tests   # pins the six callers of the one shared decision"
---

**`certificate_coverage` runs the parse on a different execution graph than your users do, and it is
not optional — the certificate cannot be computed any other way.**

Every generated parser opens `parse()` with:

```rust
self.bare_parse = !self.coverage_enabled && !self.logger_enabled
    && !self.counters_observed.get()
    && !crate::ast_pipeline::report_memo_stats_enabled();
```

The certificate's witness hook (`parser_registry::parse_and_cover_<family>`) must call
`parser.enable_coverage()` — the parser's own transactional coverage stack is *how* the
committed-rule set is recorded. So `coverage_enabled` is true on every certificate run,
`bare_parse` is false, and the certificate observes the **PROTOCOL** graph. An ordinary
`parse_full_*` with no diagnostic consumer is `bare_parse` and executes the **fused `cascade_*`**
graph, emitted from entirely different codegen sites (`cascade.rs`, `cascade/value.rs`, `scan.rs`).

## Why this is worse than it sounds

The trap is not that the certificate is wrong — it is exact about what it measures. The trap is that
**the certificate is the metric this repository reports before→after on**, and it is blind to five
of the six places the change had to land. Every other instrument you would reach for to
cross-check — `--dump-rule-entry-counts-json`, `--dump-rule-outcome-counts-json`,
`PGEN_REPORT_MEMO_STATS`, the live call-count dashboard — *also* forces the protocol graph
(`TOOLBOX.md` §3.1–§3.6 documents each one's routing). They cannot disagree with the certificate,
because they are all watching the same graph.

`TOOLBOX.md` §3.8 already states this for **profiling** ("the counters cannot answer this — they all
route to the PROTOCOL graph, and a production parse runs the FUSED one"). It is a **correctness**
hazard too, and `H.16.4a` is the case that proved it: a fix that produced the correct certificate at
all three seeds, with the delta attributable by rule name, and changed nothing a consumer would ever
observe.

## What to do

1. **Name the graph.** When a change edits an emission path, say which graph your measurement
   observed. A cert number alone is not an answer to "did this ship?".
2. **Use an oracle that runs a bare parse.** `make -C rust SHELL=/bin/bash
   parse_harness_equivalence_gate` compares the interpreter against `parse_full_*` with no
   diagnostic consumer, so it is the standing check for the fused graph. `parseability_probe --parse`
   without any `--dump-*`/`--trace` flag is the ad-hoc equivalent.
3. **Do not rely on remembering the site count.** Grep for the decision's *inputs*
   (`skip_leading_whitespace`, `match_regex`), not for the predicate name — the predicate name only
   exists at the site you already edited. Better: collapse the decision to **one definition with N
   callers** and land a test that pins N. `H.16.4a`'s does exactly that, and its control fires
   `left: 5, right: 6` when a site is re-spelled by hand.
4. **Note which pairs are coupled.** `cascade_match_*` writes the derivation tape and
   `cascade_build_*` reads it; both derive `start_dynamic` from the same boolean. They fail loudly
   when they disagree, which is a mercy — the other four sites fail silently.
