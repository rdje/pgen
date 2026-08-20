---
id: porting-a-predicate-to-a-new-population-is-a-re-derivation
title: A predicate that was correct on the population it was written for is a hypothesis on any other one — ask what the new population contains that the old never exercised
answers:
  - "I am reusing a working predicate on a different data set — what should I check first"
  - "why did my reachability check start calling legitimate rules unreachable"
  - "the counters and the grammar disagree about which rules exist — which is wrong"
  - "a check has never failed — does that mean it works"
  - "how do I map a generated artifact's names back to the source that produced them"
  - "should I reuse the shipped name classifier or ask the structure"
tags: [predicates, populations, generated-artifacts, gates, evidence, diagnosis]
date: 2026-08-20
status: current
evidence: "SV-CORPUS-GRAD.13c.2w (`PGEN-SV-CORPUS-GRAD-0254`). The containment predicate published by `.13c.2k` tests reachability of a risen rule in the SOURCE grammar's reference graph. Its counters come from the GENERATED parser. Measured on the tracked `t_only` arm: the graph holds 1,483 rules, the parser reports 1,077, and 67 of those appear in neither — every one an `_lr_base` / `_lr_seed_*` / `_lr_guard*` / `_lr_suffix*` name that `indirect_lr_elimination.rs` SYNTHESISES and that exists in no `.ebnf` file. Each would have been classified ESCAPED on sight. Re-measured on both recorded arms it had NEVER fired: `arm0_head -> t_only` has 54 risers and `t_only -> designB` has 82, zero synthesised in either — so no published verdict moved, and nothing could have caught it."
reverify: "python3 -c \"import json,sys; sys.path.insert(0,'scripts'); import parse_cost_containment as P; g=json.load(open('docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms/t_only.graph.json')); a=json.load(open('docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms/t_only.json')); k=set(g['edges'])|{r for v in g['edges'].values() for r in v}; m=[r for r in a['rule_entries'] if r not in k]; print(len(m), 'absent;', sum(1 for r in m if P.origin_rule(r,k) is None), 'unplaceable')\""
---

**Reusing a predicate on a new population looks like a copy and is actually a re-derivation.** The
predicate encodes assumptions about what its inputs contain, and those assumptions travelled with
the population it was written for, not with the code.

PGEN's parse-cost containment predicate asks: *is every rule whose entry count rose reachable, in
the grammar's reference graph, from the rules the change introduced?* It was written against two
measured arms and it discriminated correctly on both. Moving it to the ratchet's own per-rule
measurement exposed a gap that had been in it from the start:

| population | where it comes from | rules |
|---|---|---:|
| the reference graph | the `.ebnf` source, via the frontend's parsed envelope | 1,483 |
| the counters | the **generated** parser | 1,077 |
| in the counters, absent from the graph | rules the left-recursion eliminator **synthesised** | **67** |

A synthesised rule exists in no source file, so it is in no source graph, so it is reachable from
nothing — and a reachability predicate calls any one of them that **rises** *escaped*, wherever it
actually sits. ⚠️ The 1,077 above is what the pinned sample EXERCISED; against the parser's own
registry the absent population is **127 of 1,610 declared rules**, and an oracle this record did not
build agrees exactly — the shipped `--verify-families` classifier declares 127 SV LR-family names,
with 0 non-LR strays and 0 unplaceable by the fold.

⚠️ **It had never fired, and that is the part worth internalising.** Neither recorded arm moved a
synthesised rule (0 of 54 risers, 0 of 82), so every published verdict was correct and no test
would have caught the gap. The failure direction was *refusal* — the safe one — which is precisely
why it could sit there indefinitely: it would have rejected a legitimate result on some future day,
and the reader would have hunted the result rather than the predicate →
[[a-check-whose-inputs-all-pass-has-not-been-tested]].

⭐ **Repair it by asking the structure, not the naming convention.** The obvious fix is the shipped
`_lr_(base|suffix|seed|guard|alt)` classifier. Using it puts a second copy of that predicate in a
second file and ties the repair to this month's emitter naming. Asking the **graph** — the longest
`X` such that the measured name is `X_lr_…` and `X` is a rule the grammar actually has — makes the
population its own oracle, and it cannot drift when the emitter renames anything.

⛔ **And make the mapping total or refusing, never lossy.** A fold that silently drops the names it
cannot place makes the predicate quietly weaker exactly where the artifact is least like its
source. Refusing the whole verdict is the honest outcome; here the fold is total, and all 67 land
on two source rules (`casting_type` 19, `property_expr` 48).

The general checks, before reusing any predicate:

- **Enumerate both populations and diff them.** Not "does it run" — *what is in the new one that
  the old one never had?* One `set` difference found this.
- **Ask whether the never-fired branches are unexercised or unreachable.** A branch that has never
  been taken is untested, not proven.
- **Prefer the structure the artifact came from over the names it was given** →
  [[one-metric-name-two-predicates-is-a-contract-defect]].
