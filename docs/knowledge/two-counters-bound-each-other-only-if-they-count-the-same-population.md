---
id: two-counters-bound-each-other-only-if-they-count-the-same-population
title: Two counters bound each other only if they count the same population — and the consumer usually already knows they don't
answers:
  - "one of my instrument's counters exceeds another that ought to bound it — is the instrument broken"
  - "a rule reports committed more times than it was entered — what does that mean"
  - "how do I tell an instrument defect from an assumption I brought to the instrument"
  - "where do I look first before opening a leaf on a suspicious number"
  - "is `raw - committed` the failed-speculation count in a PGEN parse"
  - "my two counter maps have wildly different key counts — is that a bug"
  - "does a memo hit count as a rule entry"
tags: [instruments, counters, memoization, coverage, toolbox, sv-corpus-grad, engine-universal]
date: 2026-08-25
status: current
evidence: "PGEN-SV-CORPUS-GRAD-0300 (leaf SV-CORPUS-GRAD.13e.12). --dump-rule-outcome-counts-json reported dot_star rule_committed_counts=3 against rule_entry_counts=1 in one parse of one file, with the maps holding 35 and 298 keys. Routed as a possible instrument defect on the premise that entries 'ought to bound' commits. THE PREMISE WAS THE ERROR. rule_entry_counts counts INVOCATIONS of a rule method (ast_based_generator.rs:4285/:7122, never rolled back, and a memo HIT still invokes the method); rule_committed_counts counts OCCURRENCES in the fully expanded derivation tree of the accepted parse — a post-parse FOLD (:1973) that expands each REPLAY|d marker (:9586/:9776) with a multiplicity. A memo hit on an ANCESTOR of R therefore adds an occurrence of R without invoking R at all. Measured: the enclosing list_of_port_connections was invoked 3x = 1 memo miss + 2 memo hits (confirmed independently by --trace-rules -> 'Memo hit for rule 571' x2), and its body delta holds dot_star once. Control: two `.*` sites -> committed 6, memo hits 4. The 35-vs-298 half dissolved on inspection — the maps are NESTED, committed-not-entered = 0 exactly. AND THE ANSWER WAS ALREADY IN THE CONSUMER: --fusibility-outcome-counts carries a named committed_overshoot field, accumulates it with saturating_sub so a per-rule discard can never go silently negative (fusibility_census.rs:2493), and prints a loud WARNING (:5528). The hazard was known and handled downstream; only TOOLBOX had never stated it."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/committed_vs_entered/probe.sh   # 16 arms, pass=16 fail=0, ~1s; arm [1] reproduces dot_star entered 1 / committed 3, arm [2] is the multiplicity law (2 sites -> 6), arm [3] proves committed keys are a SUBSET of entry keys, arm [6] is a deliberately wrong expectation observed FAILING. Refuses rc=2 if the release probe is absent rather than reporting a flattering zero."
---

# Two counters bound each other only if they count the same population

**Question it answers:** one counter in my instrument exceeds another that obviously ought to bound
it. Is the instrument broken?

**Answer:** first establish that the two counters count the same *population*. "Ought to bound" is
an assumption you brought with you, not something the instrument claimed — and when it is wrong,
the number is the instrument working.

## Measured

`--dump-rule-outcome-counts-json` reported, in one parse of one file:

| | `dot_star` |
|---|---|
| `rule_entry_counts` | **1** |
| `rule_committed_counts` | **3** |

A rule committed more often than it was entered. It was routed as a possible instrument defect on
the stated premise that entries *"ought to bound"* commits. **The premise was the error.**

| map | counts | rolled back? |
|---|---|---|
| `rule_entry_counts[R]` | **INVOCATIONS of `R`'s rule method** — one monotone `fetch_add`. ⚠️ a memo **HIT** still invokes the method, so a hit counts. | never |
| `rule_committed_counts[R]` | **OCCURRENCES of `R` in the fully expanded derivation tree** of the accepted parse. Not a counter — a post-parse **fold** that expands each memo `REPLAY` marker *with its multiplicity*. | truncated on failed speculation |

⇒ **a memo hit on an *ancestor* of `R` adds an occurrence of `R` without invoking `R` at all.** The
memo makes the parse a DAG; the fold reports the TREE. Neither number is wrong; they are answers to
different questions.

The mechanism was confirmed by an instrument that shares none of the counting code: the enclosing
`list_of_port_connections` was invoked **3×** — **1 memo miss + 2 memo hits** — and `--trace-rules`
independently printed `Memo hit for rule 571` twice. The **multiplicity law** is the control that
makes it a measurement rather than a story: two `.*` sites must give committed **6** and memo hits
**4**, and they do. A miscounting counter has no reason to land on exactly 2×.

## The second half was not even a discrepancy

The same routing note flagged the maps holding **35** vs **298** keys as evidence they were
"populated by different mechanisms". They are — and the maps are **NESTED, not disjoint**:
`committed-not-entered = 0`, measured as exact set containment. 298 rules had their method invoked;
263 of them only inside speculation that `try_parse` truncated; 35 survive into the accepted tree.
That direction is the instrument's whole purpose.

⭐ **A gap between two key counts is only evidence of anything once you know which direction the
containment is supposed to run.** Checking it costs one set difference.

## ⭐⭐ Read the CONSUMER before you open a leaf on the producer

The strongest tell was not in the instrument at all. Its primary consumer,
`--fusibility-outcome-counts`, already carried:

- a named field, `committed_overshoot`,
- accumulated with `saturating_sub` so a per-rule discard can never go silently negative,
- a loud `WARNING: committed_overshoot=…`,
- and a doc-comment naming **both** contributing mechanisms.

The hazard was known and handled *downstream* the whole time. What was actually defective was the
operator-facing documentation, which asserted `raw − committed` = failed-speculation entries as an
**identity** and had never stated the per-rule form.

⇒ **a handled hazard leaves a named field behind.** Before opening a defect on a number, grep the
consumers for a name that sounds like the anomaly — `*_overshoot`, `*_unmatched`, `saturating_*`,
a warning string. Finding one turns "is this a defect?" into "why did the docs never say so?",
which is a different and much cheaper leaf.

## The operating rule

- Use a committed/coverage count as an **occurrence oracle** (`≥ 1` ⇔ present in the accepted
  parse), never as an arithmetic operand. **Check the sign before subtracting.**
- Pair every `≥ 1` verdict with a **matched control** where the construct is absent and the count
  must read `0` — the sibling discipline in
  [[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]]. Doing exactly that here
  exposed that the *entry* counter reads **2 in both arms** and is provably blind to the question,
  while the committed map separates them.
- ⚠️ **A caveat the fold does not remove:** speculation is truncated only on the failure path, so a
  **positive lookahead that SUCCEEDS** keeps its inner pushes. In PGEN's SV grammar all **7**
  positive-lookahead sites are *peek-the-token-the-next-element-consumes*, so today this inflates a
  multiplicity rather than manufacturing a phantom — but that is a property of *that grammar*, not
  of the instrument.

Companion to [[an-observer-that-replays-a-memoized-result-turns-the-dag-back-into-a-tree]] (why the
fold exists) and [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] (the same reflex: the
question is a census, so go count).
