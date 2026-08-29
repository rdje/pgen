---
id: a-reconstruction-of-why-someone-got-a-number-is-a-provenance-claim
title: "Why did the earlier author get that number" is a CLAIM — and a charitable reconstruction is the most dangerous kind, because being generous feels like rigour
answers:
  - "an earlier commit published a count I cannot reproduce — how do I explain the difference"
  - "my number disagrees with a published one, should I say their method was different"
  - "is it safe to assume a previous author used the narrower method"
  - "how do I check a claim about WHY someone got a result rather than about the result"
  - "I found a charitable explanation for an old discrepancy — what do I have to verify"
  - "should I call an old number wrong or method-dependent"
  - "how do I audit a reconstruction of someone else's reasoning"
  - "my own table contradicts my explanation and I did not notice — how does that happen"
tags: [verification, provenance, audits, self-audit, claims, measurement, evidence]
date: 2026-08-29
status: current
evidence: "ENGINE-UNIVERSAL-SERVICES.67(c)(iii) self-audit (PGEN-ENGINE-UNIVERSAL-SERVICES-0132, auditing -0131). PGEN-ENGINE-UNIVERSAL-SERVICES-0128 published `two success tables are live at once on 4 OF 8 INPUTS`. Three re-derivations gave 4 (a strict size-class method, a lower bound by construction), 5 (the subset-sum attribution) and 5 (a lifetime tape). -0131 published the charitable reconstruction: the 4 `is what the strict method answers`, so the count was METHOD-DEPENDENT rather than wrong. -0128's own sentence refutes it — it cites `t_math_synmul_mul.v` as its worked example of an input holding two tables, and the strict method sees only ONE there because the second table shares a size class. The refuting row was printed by -0131's own comparator, in the same message."
reverify: "python3 docs/tasks/artifacts/eus67c/memo_table_count_methods.py docs/tasks/artifacts/eus67/alloc_census_classes.tsv docs/tasks/artifacts/eus67/alloc_attribution.txt | grep t_math   # column A shows [2097152]; columns B and C show [2097152, 1048576] — on exactly the input the earlier commit cited"
---

**When your number disagrees with a published one, the tempting move is to explain the gap rather
than to name it.** The explanation is usually generous: *they were answering a narrower question*,
*they used the conservative method*, *both are defensible*. It feels like rigour, because it is the
opposite of accusing someone of being wrong.

It is a claim, and it is often the least-checked claim in the whole write-up.

## The failure

A previous slice published *"two tables are live at once on **4 of 8** inputs"*. Re-derived three
ways, the answers were **4**, **5** and **5** — the 4 coming from a deliberately strict method that
can only see a structure when it sits alone in its size class, and is therefore a lower bound by
construction.

The reconstruction wrote itself: the published 4 *is* the strict method's answer, so the count was
**method-dependent, not wrong**. That sentence shipped.

It is false, and one line of the earlier author's own text settles it. Their sentence reads:

> two success tables are live at once on 4 of 8 inputs — e.g. `t_math_synmul_mul.v` holds
> 622 854 152 B (2²¹ buckets) **AND** 311 427 080 B (2²⁰) simultaneously

The strict method sees **one** table on `t_math_synmul_mul.v`. **A method that misses the author's
own worked example cannot be the method they used.** They were reading the other artifact — the one
that says 5 — and miscounted it.

Worse: the comparator that produced the three answers printed that exact contradiction, in the same
message as the reconstruction. Column A disagreed with B and C on precisely the cited input. It was
read as evidence *for* the explanation instead of *against* it, because the explanation had already
been chosen.

## The rule

A **provenance claim** — any sentence of the form *"they got X because they did Y"* — is checked
against **the artifact they cited**, never against plausibility.

Practically:

- **Find their citation.** A published finding usually names one example, one file, one command.
  That is the cheapest possible test of your reconstruction, and it is decisive.
- **Try to break your own explanation before you write it.** If method Y were the one they used,
  what would their text look like? Does it look like that?
- **Prefer naming the discrepancy to explaining it.** *"Their artifact says 5 and they published 4"*
  needs no provenance claim at all; it is a comparison of two documents. Reach for a cause only when
  you can check it, and label it as inferred when you cannot.
- **Beware the direction of the error.** Charitable reconstructions fail silently because nobody —
  including a reviewer — is motivated to attack them.

Related: [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] (a universally quantified claim needs
a census) and [[a-control-validates-the-property-it-is-built-from]] (an instrument's existing
evidence does not extend to a new property). This is the same family: **a sentence that sounds like
context is still a claim, and inherits the burden of one.**
