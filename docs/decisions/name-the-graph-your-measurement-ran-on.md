---
name: name-the-graph-your-measurement-ran-on
description: "DISCIPLINE (2026-08-27, ENGINE-UNIVERSAL-SERVICES.56(b) / PGEN-ENGINE-UNIVERSAL-SERVICES-0107) — when a system ships one execution path and instruments another, every measurement names a graph, and writing that down is what keeps a diagnosis from migrating to code the customer never runs. MEASURED on the shipped SV parser: a closed census over the EMITTED artifact puts 0 of 1 544 per-rule entry counters, 0 of 4 memo-hit counters and 0 of 10 memo trace lines inside the fused rule-body region (lines 1 445 534-1 708 518, 1 044 cascade_match_* bodies), which holds 109 of 109 thin-memo probe sites and 218 of 220 depth-gated references. And the path cannot be observed by switching something on: the emitted selector is bare_parse = !coverage && !logger && !counters_observed && !report_memo_stats, so all four observation channels are disjuncts of the predicate that routes onto the OTHER graph — observing the fused path converts it into the protocol one. ⇒ before designing a fix from a measurement, name the execution mode the measurement ran in AND the one the fix must improve, and say out loud whether they are the same."
id: name-the-graph-your-measurement-ran-on
title: "Name the graph your measurement ran on — a shipped path and an instrumented path are two different subjects"
date: 2026-08-27
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .56 (b) (the emitted-artifact census table + the graph-selection predicate); producer docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_spine_memo_refusal.sh arm D; log docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/logs/spine_memo_refusal_census.txt"
reverify: "bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_spine_memo_refusal.sh   # arm D must keep reporting 0 counters / 0 trace lines inside the fused region beside non-zero thin-memo probes, and must keep printing the four-disjunct bare_parse selector — if a counter appears inside the fused region the shipped path has become observable and this record's premise is gone"
answers:
  - "my profiler and my product run different code paths, does my measurement transfer"
  - "why can I not just turn on a counter to measure the fast path"
  - "what is an observability twin and what does it cost me diagnostically"
  - "how do I check whether a diagnosis applies to the code that ships"
  - "when is a measurement-only build the right instrument"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

If a system has a **fast path that ships** and a **twin path that is instrumented**, then every
number you have is about one of them. Before a measurement is allowed to justify a fix, write down:

1. which path the measurement ran on,
2. which path the fix must improve,
3. whether (1) and (2) are the same — and if not, what licenses the transfer.

A wall-clock total often transfers. A *mechanism* usually does not.

## Why — the measurement

PGEN's generated parsers carry two graphs: a fused `cascade_*` graph that a plain parse executes, and
a protocol graph carrying the memo wrapper, counters and traces. A leaf had established — carefully,
with a control in the same run — that a hot rule's failing memo stores are all refused. Then a census
over the **emitted artifact** asked which graph that was about:

| emitted site class | total in the artifact | inside the fused rule bodies |
|---|---:|---:|
| per-rule ENTRY counter | 1 544 | **0** |
| memo-HIT counter | 4 | **0** |
| memo trace lines | 10 | **0** |
| thin-memo PROBE sites | 109 | **109** |
| thin-memo depth-gated references | 220 | **218** |

The fused rule bodies carry the whole cache and **nothing that counts or prints what it does**. The
diagnosis was true, and it was about the twin.

## The trap that makes this hard to notice

You cannot fix it by enabling an instrument, because **the instrument is the switch**:

```rust
self.bare_parse = !self.coverage_enabled
    && !self.logger_enabled
    && !self.counters_observed.get()
    && !report_memo_stats_enabled();
```

Those four disjuncts are the *complete* observation surface. Every way of looking also selects the
other graph. That is deliberate — it is what keeps the shipped path free of observer cost — and it
means the fast path is unobservable **by construction**, not by omission. So the usual reflex ("add a
counter and re-run") produces a number about the twin, again, and nothing warns you.

## What to do instead

- **Sort your instruments by whether they perturb the selector.** Here an external sampler
  (`/usr/bin/sample`) sets none of the four, so wall-clock and stack profiles of the *shipped* path
  are available and were never in doubt. Only the in-process counters are twin-only.
- **When the fast path genuinely must be instrumented, use a build that is not meant to ship** — the
  ceiling-probe pattern. PGEN's own precedent is a never-free bump `#[global_allocator]` that priced
  allocator removal at −40 %, was banked as a ceiling, and was never landed.
- **Env-gate both arms into ONE binary.** Two builds are two subjects again, which is the same
  mistake one level up.
- ⚠️ **Do not over-correct.** The right response is not "throw out the twin's numbers": it is to
  re-read each claim and say which path it is about. Wall-clock totals measured externally survive
  untouched; mechanism claims are scoped to the twin until something looks at the other graph.
