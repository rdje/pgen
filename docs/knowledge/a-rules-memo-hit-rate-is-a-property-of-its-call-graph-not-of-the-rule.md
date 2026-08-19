---
id: a-rules-memo-hit-rate-is-a-property-of-its-call-graph-not-of-the-rule
title: A rule's memo-hit rate is a property of how many places call it, not of the rule — so pricing a fix from that rate prices a graph the fix itself moves
answers:
  - "why did my packrat rule report zero memo hits when it is called millions of times"
  - "where should I spell a guard — in the shared rule or at its call sites"
  - "my cost model used measured memo-hit rates and still came out backwards — why"
  - "does adding call sites to a rule make a parser slower"
  - "what does a generated parser's inlining do to per-rule counters"
  - "how do I compare two spellings of the same grammar fix"
tags: [performance, parsers, peg, packrat, memoization, codegen, cost-models, measurement]
date: 2026-08-19
status: current
evidence: "SV-CORPUS-GRAD.13c.2k (PGEN-SV-CORPUS-GRAD-0236). Three parser arms of ONE correctness fix, measured over a pinned 192-file sample, all verdict-identical. Guard inside the shared rule (`designB`): 425,174,240 entries, +1.82 % over baseline — refused by the cost ratchet. The SAME guard spelled at its 45 call sites (`designA`): 413,858,778 entries, -0.89 % BELOW baseline. The difference, read out of the generated parser rather than inferred: the wrapper rule `non_keyword_identifier` is reached through 341 generated `inlined_frame_call` sites in designB, which bypass its own memoization (14,320,942 entries / 0 memo hits); giving it 45 more grammar references makes it memo-served (18,758,343 entries / 17,724,671 hits = 94.5 %) and the inner rule then runs 1,516,324 bodies instead of 19,353,115 entries."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms/inline_census.sh   # asks the GENERATED parser which rules are memoized on their executed path and which are inlined at their call sites; re-run it against two arms of the same change and compare"
---

**"This rule is 90 % memo hits, that one is 0 % — so put the work in the first one."** The two
measurements were right. The conclusion was backwards, and it cost a correctness fix two sessions of
being held back as *"a rise we cannot eliminate."*

⛔ **A hit rate is not a property of the rule.** A code generator that inlines a rule's body at its
call sites gives those calls a full observable frame — entry counter, trace, coverage — and **no
memo lookup at all**. So a rule reached only through inlined sites reports **zero** memo hits no
matter how hot it is, and a rule reached through its own method reports a high rate for the same
work. The number you read off a profile is a fact about *the call graph*, not about the rule.

⛔⛔ **And the fix you are pricing is a change to that graph.** This is what makes the mistake so
hard to see: the model was built from the current hit rates, and the edit being priced *moves
references between rules*, which is exactly the input the inlining decision keys on. The measured
case:

| spelling | where the guard lives | shared wrapper | inner rule | total |
|---|---|---|---|---|
| baseline | wrapper only, ~100 sites | 5.5 M entries / **0 hits** | 10.3 M / 9.3 M hits | — |
| "cheap" (`designB`) | moved INTO the inner rule | 14.3 M / **0 hits** (inlined at 341 sites) | 19.4 M / 18.3 M hits | **+1.82 %** |
| "expensive" (`designA`) | left in the wrapper, 45 call sites rewritten to use it | 18.8 M / **17.7 M hits** | 1.5 M / 0 hits | **−0.89 %** |

The spelling the model rejected **answers the repeated question one level higher**. Once the wrapper
has enough call sites to be memoized rather than inlined, it absorbs the repeats, and the inner rule
runs only on the misses. The spelling the model chose left the wrapper a bare alias — trivially
inlinable — so every one of its 14.3 M calls paid a full frame *and then* re-entered the inner rule.

## The rules that follow

- **Never price a change to a call graph from counters measured on the old call graph.** Build both
  arms and measure. Two arms of one fix cost ~6 minutes each here; the reasoning they replaced had
  already been wrong twice.
- **Ask the generated code which rules are memoized on their executed path**, not the counters. A
  0-hit rule is ambiguous — never queried twice, or never memo-served at all — and those two have
  opposite implications for where to put work.
- **Compare arms on verdicts before comparing them on cost.** Costs of two arms that accept
  different languages are not comparable. Here `committed` moved +51,611 between the arms and it
  would have been easy to read as a regression; per-file verdicts showed all 192 files unchanged and
  a per-rule split attributed the move exactly (one extra committed frame per committed identifier
  at the 45 rewritten sites, minus 4 × 31 reserved words that no longer commit anywhere).
- ⭐ **A "prohibitive" cost is a hypothesis about one spelling.** It is evidence about the spelling
  you built, never about the fix.

⭐ The generalisation beyond parsers: **whenever an optimiser's decision (inline, cache, specialise,
JIT) is keyed on a structural property your change edits, the profile you measured before the change
describes a program that will not exist after it.** Memoization, inlining thresholds, monomorphisation
and branch prediction all have this shape.

Sibling of [[a-strictness-fix-redirects-the-search-not-just-the-predicate]], which is the same fix's
*previous* lesson and ends by warning against reasoning your way to "irreducible" — this card is
what happened when that warning was obeyed. See also
[[a-counter-that-cannot-tell-a-cache-hit-from-work-prices-them-alike]].
