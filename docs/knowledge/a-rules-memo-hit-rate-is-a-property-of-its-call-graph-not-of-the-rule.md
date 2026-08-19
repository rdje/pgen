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
evidence: "SV-CORPUS-GRAD.13c.2k (PGEN-SV-CORPUS-GRAD-0236/-0237, corrected under director challenge in -0238). FOUR parser arms of ONE correctness fix, measured over a pinned 192-file sample, all verdict-identical on every one of the 192 files. The generator's own census gives the decision: a rule is INLINED (and takes no memo lookup at all) while refs x body_nodes stays under a named duplication budget. Baseline: `identifier` refs=47 over-budget/memoized, wrapper refs=7 INLINED. Moving 45 references flips both - `identifier` to refs=1 INLINED, wrapper to refs=52 over-budget/memo-served at 94.5 % hits. designB (guard inside `identifier`, wrapper a bare alias) changed NEITHER decision and cost +1.78 % on entries purely through redirected speculation - refused. designA (wrapper delegates) -0.89 % entries but +0.72 % committed - refused on the other counter. designC (wrapper matches the token) -1.07 % entries, -5.11 % committed, -2.41 % memo hits - shipped. The difference, read out of the generated parser rather than inferred: the wrapper rule `non_keyword_identifier` is reached through 341 generated `inlined_frame_call` sites in designB, which bypass its own memoization (14,320,942 entries / 0 memo hits); giving it 45 more grammar references makes it memo-served (18,758,343 entries / 17,724,671 hits = 94.5 %) and the inner rule then runs 1,516,324 bodies instead of 19,353,115 entries."
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms/inline_decision.py   # prints the GENERATOR's own inline verdict (class, refs, body_nodes, INLINED|over-budget) for each arm of the fix, beside the duplication budget it is judged against. Do NOT substitute counts of `inlined_frame_call` in the generated parser: those are emitted sites after transitive expansion, and reading them as the decision is the error this card was corrected for"
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

⛔ **ASK THE GENERATOR, DO NOT INFER FROM ITS OUTPUT.** PGEN's `--report-fusibility-census` prints
its own decision per rule — `INLINE-DECISIONS … duplication_cap`, then `<class> refs=<n>
body_nodes=<n> INLINED|over-budget`. A rule is inlined, and therefore takes **no memo lookup at
all**, while its reference count × body size stays under that budget. Measured across four arms of
one correctness fix:

| arm | `identifier` | `non_keyword_identifier` (the wrapper) | `entries` | `committed` |
|---|---|---|---|---|
| baseline | refs=**47** → over-budget ⇒ *memoized* | refs=**7** → **INLINED** ⇒ 0 memo hits | — | — |
| `designB` — guard moved INTO `identifier`, wrapper a bare alias | refs=47 → over-budget | refs=7 → INLINED | **+1.78 %** ⛔ | −0.00 % |
| `designA` — wrapper keeps the guard and DELEGATES; 45 refs moved | refs=**2** → **INLINED** | refs=**52** → **over-budget** ⇒ memo-served, 94.5 % hits | −0.89 % | **+0.72 %** ⛔ |
| `designC` — wrapper MATCHES THE TOKEN; 45 refs moved | refs=**1** → INLINED | refs=52 → over-budget | **−1.07 %** | **−5.11 %** |

**Moving 45 references carried both rules across the budget in opposite directions**, and the memo
boundary followed. That is the lesson, and it is why no profile of the old parser could have
predicted it.

⛔⛔ **AND HERE IS THE PART THIS CARD ORIGINALLY GOT WRONG, corrected under challenge.** It first
explained `designB`'s +1.78 % as an inlining effect — *"a bare alias is trivially inlinable, so the
generator inlined it at 341 sites"* — citing counts of `inlined_frame_call` in the generated parser
(46 at baseline, 341 for the alias). Those counts are real, but they are **emitted sites after
transitive expansion, not the decision**, and the table above shows `designB` changed **neither
rule's decision**. Its cost was never inlining at all: it was redirected speculation
([[a-strictness-fix-redirects-the-search-not-just-the-predicate]]), full stop. ⇒ **when a compiler
will state its decision, never reconstruct it from the code it emitted.**

⭐⭐ **THE THIRD ARM IS THE SAME LESSON AGAIN, ON A DIFFERENT COUNTER.** `designA` looked like the
answer and was refused on `committed`: wrapping *delegates*, so every committed identifier at the 45
rewritten sites paid one extra committed frame. `designC` has the wrapper match the token itself,
which deletes `identifier`'s committed frame outright (415,534 → **0**) and takes one off the 7
rules that already used the wrapper. **Two of three candidate spellings were refused on cost, each
on a DIFFERENT binding counter, and neither reason was visible without building the arm.**

## The rules that follow

- **Never price a change to a call graph from counters measured on the old call graph.** Build the
  arms and measure. An arm cost ~6 minutes here; the reasoning it replaced had already been wrong
  twice, and the third arm — built only because the second was refused — is the one that shipped.
- **Ask the GENERATOR for its decision, not the generated code and not the counters.** A 0-hit rule
  is ambiguous — never queried twice, or never memo-served at all — and those two have opposite
  implications for where to put work. The emitted code is ambiguous in a second way: inlining is
  transitive, so the number of emitted sites moves for reasons the decision did not.
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
