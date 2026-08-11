---
id: a-fused-counter-is-not-evidence-about-any-of-its-parts
title: A counter that sums several mechanisms answers questions about none of them — split it before concluding, because the aggregate reads as the reassuring component
answers:
  - "my rule shows lots of memo hits but the parse is still exponential — is the memo working"
  - "is packrat actually collapsing this rule, or just caching its failures"
  - "why does rule_memo_hit_counts look healthy while the parse blows up"
  - "how do I tell a replayed success from a cached failure in the packrat memo"
  - "what does a stale-tainted memo eviction mean and how do I count them"
  - "the memo hit rate is constant as input grows but cost is exponential — what is that telling me"
  - "is every rule in a generated parser actually memoized"
  - "why do some rules have entry counts but emit no memo trace lines at all"
  - "my per-rule instrument disagrees with rule_entry_counts — which one is wrong"
  - "how do I measure memo insert vs evict per rule without changing the engine"
tags: [memo, packrat, performance, instrument-honesty, measurement, systemverilog, codegen]
date: 2026-08-10
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .11a (the `-0194` reading and the `-0195` correction); docs/tasks/artifacts/sv_corpus_grad/memo_insert_evict_census.py; rust/src/ast_pipeline/ast_based_generator.rs (three `record_memo_hit` sites feeding one counter); rust/src/parser_registry.rs:136 (the field doc that already said "fail-set + valid tainted-failure + success replays")
reverify: "test \"$(grep -c 'record_memo_hit' rust/src/ast_pipeline/ast_based_generator.rs)\" = 3 && echo FUSED-THREE-WAYS"
---

**`rule_memo_hit_counts` sums three unrelated mechanisms, and only one of them is the packrat
guarantee.** Codegen increments the same counter from three sites — a cached clean failure, a cached
tainted failure, and a replayed success. The first two say "we cheaply re-refused a dead end"; only
the third says "we skipped work we would otherwise have redone".

The reading error and its cost, measured on the same rule in two consecutive sessions:

| | what was seen | what was concluded | what was true |
|---|---|---|---|
| `-0194` | `conditional_statement` **287 entries / 208 memo hits** | "the memo already collapses the rule everyone would have blamed — *add memoization* is already true and already insufficient" | **all 208 are cached FAILURES; success replays = 0** |
| `-0195` | the same parse, split per path | the memo is not serving this construct **at all** | packrat had silently stopped working across the whole statement surface |

⇒ the aggregate did not merely lose detail, it pointed the investigation **away** from the cause: a
healthy-looking hit count retired the memo as a suspect for a whole session. ⛔ **The failure
direction of a fused counter is reassurance** — the cheap, frequent component dominates the sum, so
the metric looks best exactly where the expensive component has stopped working.

⭐ **The fix needed no engine change — the split was already being logged.** The generated
`memoized_call` emits a distinct line per memo transition under `PGEN_TRACE_VERBOSITY=debug`
(miss / hit-success / hit-failure / hit-tainted-failure / insert / evict-stale-success /
evict-stale-failure), keyed by numeric rule id. Joining those against the parser's own `RULE_NAMES`
table reconstructs the per-path census for free
(`docs/tasks/artifacts/sv_corpus_grad/memo_insert_evict_census.py`). **Before building an
instrument, check whether the engine is already saying it and only the summary is lossy.**

⭐ **A per-path census makes the mechanism self-proving.** A *stale-tainted eviction* can only run on
an entry that was epoch-stamped, so counting evictions **is** the proof that the entry was tainted —
no separate taint instrument needed. Measured on the `if / else if` chain: inserts follow `2ⁿ − 1`,
evictions follow `2ⁿ − 1 − n`, success replays are **0** at every depth. *Every entry that was ever
looked up again had been thrown away first.*

⛔ **Cross-check a derived census against an independent counter, and let it REFUSE.** The census
reads a trace log; `--dump-rule-outcome-counts-json` reads atomic counters. For a memoized rule
`miss + hits` must equal `rule_entry_counts` exactly. That control fired on the very first run — and
it is the only reason the next fact was found instead of averaged away.

⛔⛔ **NOT EVERY RULE IN A GENERATED PARSER IS MEMOIZED, and the rule's own source will tell you it
is.** A rule reached through the generated `inlined_frame_call` helper carries a full observable
frame — entry counter, coverage push, enter/exit trace — but **no `memoized_call`**. On
`systemverilog.ebnf` that is **663 of 1481 rules across 2871 call sites**. So:

- `grep memoized_call` inside `parse_<rule>` proves the **method** is wrapped, never that the
  **executed path** was — a rule can be inlined at every call site that matters;
- a rule with entry counts and *zero* memo trace lines is not a broken instrument, it is an inlined
  rule, and a control that cannot express that distinction fails on ~305 live rules and gets
  weakened for the wrong reason.

⇒ partition the population (strictly-memoized rules must balance exactly; inlined-reachable rules may
only fall short) and **report the shortfall rather than dropping it**.

⚠️ **Generalise the shape, not the noun.** The same trap is any metric that adds a cheap frequent
event to a rare expensive one: cache hits + refusals, retries + successes, "files processed" over two
pipelines. Ask of every headline number: *which mechanisms does this sum, and which one did I
actually want?* If the answer is more than one, the number is not evidence yet.

⭐ **A COVERAGE DENOMINATOR fuses the same way, and there the fused reading is PESSIMISTIC — which is
just as blocking.** `SV-CORPUS-GRAD.13` published *"38.7 % of the SV corpus (6 321 rows) carries no
verdict"*, one number over three strata of opposite worth (`.13a`, same instrument re-run):

| stratum | rows | worth |
|---|---:|---|
| one-sided positive, unit-shaped | 1 824 | the parse consumed the WHOLE file ⇒ **no rejects-valid defect hides here**; silent on accepts-invalid |
| ⚠️ one-sided, FRAGMENT-shaped | 99 | an `.svh` payload / excerpt fixture — accepting one is **not** testimony, it may BE the over-acceptance |
| ⛔ dark (the parse failed) | 4 398 | nothing is known — **this** is what a burn-down attacks |

⇒ the program is 26.9 %, not 38.7 %, and the difference required **no relabelling at all**: the
manifest already carried the observed outcome per row, unused. Note the two failure directions in one
record — a fused *performance* counter reads reassuring, a fused *coverage* denominator reads alarming
— and note that splitting it also produced a stratum whose sign is the opposite of the obvious one
(accepting a fragment is bad news). **Split first; decide what each part is worth second.**

See also [[a-memo-key-must-name-every-context-the-outcome-depends-on]] (the memo's other honesty
bound — what a `(rule, position)` key may hold),
[[a-rising-pass-rate-is-not-evidence-of-correctness]] (the same failure direction on a corpus
metric), [[an-instrument-that-prints-an-unjudged-column-reports-a-defect-as-green]] and
[[a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness]].
