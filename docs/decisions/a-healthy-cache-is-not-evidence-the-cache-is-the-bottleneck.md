---
name: a-healthy-cache-is-not-evidence-the-cache-is-the-bottleneck
description: "DISCIPLINE (2026-08-27, ENGINE-UNIVERSAL-SERVICES.44(b) step 1 / PGEN-ENGINE-UNIVERSAL-SERVICES-0100) — when a memoized parse is slow, 'is the cache working?' and 'is the cache the problem?' are different questions, and the first answering YES is exactly what hides the second. MEASURED on the shipped SV parser over an over-ceiling parse: the depth-gated failure channel is reached, stores 4 765 entries at the first over-ceiling depth rising to 36 964 at depth 800, and hits — it is healthy. Between depth 300 and 800 the input grows 2.67x and memo ENTRIES grow 3.4x (42 066 -> 141 332), while rule CALLS grow 109x (173 697 -> 18 901 646) and wall clock 85x (0.144 s -> 12.25 s). The cache bounds what is KNOWN; nothing bounds how many times the engine ASKS. ⇒ compute CALLS PER MEMO KEY (here 4.1 -> 133.7) before touching the cache: a hit rate and an entry count cannot distinguish 're-deriving' from 're-asking', and only the second is fixed by attempt elision at the choice site."
id: a-healthy-cache-is-not-evidence-the-cache-is-the-bottleneck
title: "A healthy cache is not evidence the cache is the bottleneck — measure calls per key"
date: 2026-08-27
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .44 (b) step 1 (the five-depth census table + the depth-800 spot row); producer docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/measure_memo_collapse.sh; log docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/logs/memo_collapse_census.txt"
reverify: "bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/measure_memo_collapse.sh 300 325   # the cal/key column must RISE with depth while ent-plain barely moves — that divergence IS the finding, and the script refuses when the probe's SV fingerprint differs from generated/systemverilog_parser.rs"
answers:
  - "my parser memoizes and is still slow, where do I look"
  - "the memo hit rate is high, why is the parse quadratic"
  - "how do I tell re-derivation from re-asking"
  - "is a bigger or smarter cache the fix for a slow packrat parse"
  - "what does calls per memo key measure"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

Before changing a cache to fix a slow memoized parse, measure **rule calls ÷ distinct memo entries**
at two or more input sizes. If entries stay flat while calls climb, the cache is **working** and the
cost is in the code that ISSUES the calls — a cache change cannot touch it.

## Why — the measurement

`ENGINE-UNIVERSAL-SERVICES.43` fixed an unbounded hang past the 4096-frame recursion ceiling by
adding depth-gated failure memo channels. `.44` then asked why SystemVerilog *still* spends ~72 % of
a 12-second over-ceiling reject in the parse cascade. The framing everyone reaches for — mine
included — is *the cache must be failing*.

| depth | rule calls | memo entries | wall clock | calls / key |
|---:|---:|---:|---:|---:|
| 300 (sub-ceiling) | 173 697 | 42 066 | 0.142 s | 4.1 |
| 400 | 889 146 | 63 982 | — | 13.9 |
| 800 | **18 901 646** | 141 332 | **12.33 s** | **133.7** |

Input grows **2.67×**. Entries grow **3.4×**. Calls grow **109×** and time **86.8×** — both measured
on the SAME graph, which the first cut of this record did not do. Time tracks the call count and
nothing else; per-call cost falls only 1.25× across a 109× change in volume. Every one of those 18.9 M calls is cheap and correctly answered — a hash
probe returning `Backtrack`. **Nothing is re-derived. The engine is paying to ask.**

## The corollary that changes what you build

A *better* memo cannot help when the memo already answers ~99 % of calls correctly. The only term
that can move a 109× is the fan-out that issues the calls — sibling alternatives at a choice site
re-entering the same rules to be told, one probe at a time, what the cache already knows. ⇒ the fix
lane is **attempt elision at the choice site**, not caching.

## ⚠️ Honest bound — what is MEASURED and what is not

**MEASURED**: every number above, on the shipped release probe with its SystemVerilog fingerprint
checked equal to `generated/systemverilog_parser.rs` before any number was read.

**NOT measured**: which choice sites issue the calls. That is the next census, and this record must
not be read as naming a fix site.

⛔ **Routing caveat, which is prior art and not a finding**: `TOOLBOX.md` §2.1 records that a trace or
a counter clears `bare_parse` and routes the parse onto the PROTOCOL graph, so the call and entry
columns describe that graph. What licenses reading them as the plateau's COST is a separate
measurement — the graph switch ALONE costs **~1.08×** at depth 300 and **nothing** at depth 800, so
the two graphs agree within 8 % across the censused range. ⛔ An earlier cut of this record said
**~2.0×**, measured against `--dump-rule-call-counts`, which switches the graph AND counts every rule
entry; the counter was the missing 1.99×. Corrected 2026-08-27 under a director re-verification
challenge (`PGEN-ENGINE-UNIVERSAL-SERVICES-0101`).

Related: [[a-green-counter-can-be-the-one-hiding-the-counter-that-moved]] is the same failure at the
level of a single run — a number you are steering going green while the one that matters moves. Here
the healthy number is the hit rate, and what it hides is the denominator it was never computed over.
