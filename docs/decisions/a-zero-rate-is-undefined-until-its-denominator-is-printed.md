---
name: a-zero-rate-is-undefined-until-its-denominator-is-printed
description: "DISCIPLINE (2026-08-27, ENGINE-UNIVERSAL-SERVICES.56(a) / PGEN-ENGINE-UNIVERSAL-SERVICES-0106) — a 0 % rate is not a finding until you know how many chances it had. MEASURED on the shipped SV parser: constant_primary reports a 0 % memo-hit rate at EVERY depth, and the same zero means opposite things at the two ends. Sub-ceiling the rule is entered once per NESTING LEVEL — 0.1 to 0.5 entries per available (rule, position) key — so a hit is arithmetically impossible and the zero says nothing; three commits nonetheless read it as a defect signature. Past the recursion ceiling the same rule reaches 368.4 entries per key with the same zero, and it is then a large real cost (3 707 misses over 350 keys at depth 350, max 113 probes at ONE key, 0 failure stores, while a control rule in the same run stores 32 failures and takes 4 160 hits). One number, two regimes, opposite meanings; what separates them is the DENOMINATOR, which nobody had printed beside it. ⇒ print the opportunity count next to every rate, and treat a rate whose denominator is ~1 as UNDEFINED rather than as zero."
id: a-zero-rate-is-undefined-until-its-denominator-is-printed
title: "A 0 % rate is undefined until its denominator is printed — count the chances before calling it a defect"
date: 2026-08-27
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .56 (a) (the eleven-depth regime table + the depth-350 event census); producer docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_spine_memo_refusal.sh; log docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/logs/spine_memo_refusal_census.txt"
reverify: "bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_spine_memo_refusal.sh   # arm A's sp_per_key column must stay BELOW 1 for the sub-ceiling rows and rise past the ceiling — that rise is the finding; the control column must be non-zero from the first row, or the arm is measuring a broken memo rather than a refused one"
answers:
  - "a rule reports a 0 % cache hit rate, is that a defect"
  - "how do I tell a real zero from an arithmetically forced one"
  - "what should I print beside a rate"
  - "why did the same measurement mean opposite things at two input sizes"
  - "how do I choose a control rule for a per-rule cache measurement"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

A rate is a fraction. Before reading a **0 %** as a finding, print its **denominator** — the number
of opportunities the numerator could have counted. If the denominator is ~1, the rate is
**undefined**, not zero, and it carries no information about the thing you are investigating.

## Why — the measurement

`constant_primary` — the SystemVerilog expression spine — reports a **0 % memo-hit rate at every
input size measured**, from a 39-byte file to a 1 633-byte one. Three commits treated that as a
defect signature. It is two different facts:

| depth | entries | available keys | **entries / key** | hits | what the 0 % means |
|---:|---:|---:|---:|---:|---|
| 3 | 3 | 40 | **0.1** | 0 | nothing — one entry per nesting level, no key is ever revisited |
| 300 | 300 | 634 | **0.5** | 0 | nothing — same |
| 320 | 433 | 674 | 0.6 | 0 | the ceiling is contacted; repetition begins |
| 350 | 3 707 | 734 | 5.1 | 0 | real |
| 500 | 88 443 | 1 034 | 85.5 | 0 | real |
| 800 | **601 963** | 1 634 | **368.4** | 0 | **a large real cost** |

Sub-ceiling the rule is entered **exactly once per nesting level** (depth 3 → 3 entries, depth 300 →
300). A rule that never revisits a key **cannot** hit its cache. The zero is forced by arithmetic and
says nothing about caching, gating, or anything else.

Past the recursion ceiling the same rule is entered 368 times per available key and still reports
zero — and there the zero is the whole finding. At depth 350 a single traced run counts **3 707
misses over 350 distinct keys, mean 10.6 and a maximum of 113 probes at one key, with zero failure
entries stored**.

## The other half: the control

The denominator tells you whether the zero *could* be informative. A **control measured in the same
run** tells you whether the instrument works. Here `primary_literal` — a leaf of the same tournament
— reports **78.9 %** hits at depth 3 and **99.9 %** at depth 800, and stores 32 failures where the
spine stores none. Same input, same binary, same trace. Without that column, "0 hits" is
indistinguishable from "the counter is dead"; with it, the spine's zero is a fact about the parse.

## The shape to reuse

- Alongside any rate, print **numerator, denominator and the opportunity count**. `0/0` and `0/601963`
  should never render as the same string.
- Sweep the parameter that moves the denominator (here input depth). If the rate is constant while the
  denominator moves by three orders of magnitude, you are looking at two regimes, not one measurement.
- Put a **control in the same run**, not in the same script — two runs of a backtracking or
  concurrent system are two different executions.
- ⚠️ The corollary that costs the most: a rate that has been *stable and zero for a long time* is the
  easiest kind to over-read, because its stability looks like evidence. It is often just an undefined
  quantity being recomputed.
