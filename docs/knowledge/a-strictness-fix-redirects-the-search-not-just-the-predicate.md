---
id: a-strictness-fix-redirects-the-search-not-just-the-predicate
title: Pricing a strictness fix by asking "where does the guard execute" gets the guard exactly right and misses the larger term — rejection is not free, it buys more speculation
answers:
  - "how do I predict what a grammar tightening will cost before writing it"
  - "my parser got stricter and slower at the same time — why"
  - "why did my cost model for a lookahead come out backwards"
  - "does making a rule reject more make a backtracking parser faster or slower"
  - "how should I price a change to a memoized PEG rule"
  - "what does flat committed with rising entries tell me"
tags: [performance, parsers, peg, cost-models, prediction, measurement, instruments]
date: 2026-08-19
status: current
evidence: SV-CORPUS-GRAD.13c.2k (PGEN-SV-CORPUS-GRAD-0235). PGEN moved a reserved-word exclusion out of a wrapper rule that never takes a memo hit (5,504,191 calls / 0 hits) and into `identifier`, which is ~90% memo hits (10,349,663 calls / 9,324,559 hits). A price instrument written BEFORE the fix predicted -8,958,174 rule entries. Measured after: +7,588,879 — wrong by 16,547,053 and in the flattering direction. The attribution says why: guard evaluations fell -8,941,038, within 0.19% of prediction, while `identifier` and the wrapper were RE-ENTERED +17,820,203 more times. `committed` moved -124 on 7.1M, so ~100% of the net rise is rolled-back speculation.
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/raw_identifier_census/price.py   # prints the predicate-relocation term the model gets right; compare its prediction against a post-fix `total_entries` from the same instrument, and expect the difference to be re-entry, not noise"
---

**A guard that says "no" does not end the parse. It sends a backtracking parser somewhere else.**

That is the whole lesson, and it is easy to miss because the obvious cost model is *local* and
*correct as far as it goes*: find where the predicate executes, count how often, multiply. In the
measured case that model was accurate to **0.19 %** — it predicted the guard-relocation saving
almost exactly. It was still wrong about the total by more than twice its own magnitude, because
the term it omits is not in the predicate at all.

⛔ **The omitted term is the search the predicate redirects.** Previously, a keyword-shaped token was
*accepted* as an identifier, so a speculative alternative succeeded early and the engine stopped
looking. Tightening the rule makes that alternative fail, and the engine then tries the next one,
and the next — each re-entering the same rules at new positions. Every one of those re-entries is a
rule entry that did not exist before. **Strictness moves work from "accept the wrong thing quickly"
to "reject it and keep searching."**

## The signature to look for

`committed` flat while `entries` rises. Committed entries are the work that survived into the parse
tree; raw entries include everything rolled back. When the accepted output is unchanged (`-124` on
7.1M) and entries move `+7.6M`, **every added entry is failed speculation** — which is exactly what
extra backtracking looks like and cannot be confused with the parser doing more real work.

## What to do instead

- **Price the predicate AND the redirection.** The first is modellable from per-rule entry/memo-hit
  counts. The second is not modellable from them at all — it depends on which alternatives now fail
  and what the engine tries next. Treat the model's output as a term, not a total.
- **Publish the prediction anyway, and label it.** A number that can be refuted is worth far more
  than a number that cannot; this one was refuted by its own follow-up measurement in one slice, and
  the refutation is what produced this card. ⭐ The instrument that made the prediction is the same
  instrument that measured it wrong — that is a feature.
- ⚠️ **Do not then reason your way to "irreducible."** The alternative spelling looks worse under the
  same model that just failed, so it cannot be dismissed by argument either. Build the arm and
  measure it, or the acceptance is an assertion wearing a measurement's clothes
  ([[a-deterministic-counter-cannot-see-a-per-entry-cost-rise]] retired a published bound for
  exactly this reason).

⭐ The generalisation beyond parsers: **a local model of a change to a search predicate does not
bound the change to the search.** Whenever a fix alters what a backtracking, speculative or retrying
system considers *done*, the cost lives in the paths it now explores, not in the test you edited.
