---
id: a-gate-left-red-for-a-known-reason-cannot-report-an-unknown-one
title: A ratchet parked RED for a known reason has stopped ratcheting — adjudicate the known rows and get it GREEN, because until you do every other regression it watches is free
answers:
  - "a gate is red for a reason we already understand — is it ok to leave it until we fix the cause"
  - "how much does a known-red gate actually cost us"
  - "should I raise a ceiling to make a ratchet green, or is that cheating"
  - "when is raising a ratchet ceiling legitimate"
  - "my ratchet count went up after several commits landed — how do I find which commit did it"
  - "git log -S did not find the commit that changed my rule, why"
  - "how do I predict the count after I fix the divergences a ratchet is counting"
  - "the count after my fix was not count minus the rows I fixed"
  - "how do I attribute a ratchet delta when both of the tool's inputs have moved"
  - "a gate that nobody runs automatically — how do I tell how long it has been wrong"
tags: [gates, ratchets, ci, evidence, controls, claim-verification, attribution, envelope-differential]
date: 2026-08-23
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.20 (`PGEN-GRAMMAR-WELLFORMED-0176`), 2026-08-23.
  `ebnf_frontend_dual_run_gate` was RED at HEAD on `systemverilog` — `envelope divergences REGRESSED:
  155 > ceiling 151` — and was the ONLY red row of 14; the other thirteen sat exactly at their
  ceilings. It had been red since 2026-08-18 and was found on 2026-08-22 BY ACCIDENT, while an
  unrelated leaf measured its own blast radius. Ten commits touched `grammars/systemverilog.ebnf`
  across those five days and THREE of them moved this gate. Nothing runs it automatically; measured
  cost is 35 s wall-clock warm / 2 070 MB peak tree RSS.
  ATTRIBUTION. The instrument has TWO inputs — the subject grammar, and `grammars/ebnf.ebnf`, which
  builds the arm-2 parser — and both had changed since the ceiling was pinned. Pinning the binary at
  HEAD and feeding it the SV grammar at `0fd53da4` reproduced EXACTLY 151, proving the `ebnf.ebnf`
  repair contributed zero. `git log -S` named only 2 of the 4 new rows (it reports a changed COUNT of
  a string, and two of the four rules only had their bodies moved); running the differential at all
  ELEVEN vintages named all four, and additionally showed seven vintages moved the count by zero and
  that ZERO sites disappeared. Three surgical one-edit controls proved each mechanism; one of them
  removed TWO divergence rows for ONE deleted line, so four rows were three defects.
  THE MASKING RESULT. Pre-committed prediction for the post-fix count: 155 -> 151 (drop the four
  comment-bearing rows). MEASURED: 155 -> 152. The four dropped and a FIFTH row appeared at a rule
  that already had one — `scoped_or_hierarchical_tf_identifier` returned as
  `semantic_annotation_inline -> semantic_annotation`, which the removed row had been hiding.
  ⚠️ HONEST BOUND: that arm (C4) strips the comments from the GRAMMAR, whereas the real fix strips
  them from the PAYLOAD, so 152 is itself a prediction, recorded for GRAMMAR-WELLFORMED.H.20.1 to
  falsify in turn.
reverify: "make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate   # GREEN, 14/14 at their ceilings, exit 0. MEASURED two-arm control for this card's own claim: (A) set `systemverilog) echo 151` in rust/scripts/ebnf_frontend_dual_run_diff_gate.sh, re-run -> exit 2, '1 failing grammar flow(s)'. (B) ALSO set `json) echo 1`, re-run -> exit 2, '2 failing grammar flow(s)'. So the EXIT CODE is identical in both (2) while the printed count does move — the saturation is at the pass/fail level a human and a CI step branch on, NOT in the report body. `git checkout rust/scripts/ebnf_frontend_dual_run_diff_gate.sh` to restore. Attribution method: `git show <old-commit>:grammars/systemverilog.ebnf > tmp/x.ebnf && (cd rust && PGEN_ENVELOPE_DUMP_ALL=1 ./target/debug/ebnf_dual_run_diff --input ../tmp/x.ebnf --output ../tmp/r.json --envelope-differential ../tmp/e.json)` — one run per vintage, binary pinned."
---

A ratchet exists to tell you about the regression you did **not** predict. The moment one of its rows
is red for a reason you already understand and have accepted, it has stopped doing that job — because
the verdict everyone acts on is already `fail`, and a second, unknown breach changes nothing about it.

That is the whole cost, and it is easy to underestimate because it is invisible by construction. The
exposure is not *"four un-adjudicated rows"*. It is **every regression the gate watches, on every
grammar it covers, for as long as the red row sits there.** Here that was fourteen grammars for five
days, and the red was found by accident rather than by anyone looking.

⚠️ **Measured precisely, because the sloppy version of this claim is false.** A two-arm control on this
very gate: with only `systemverilog` breached it exits **2** and prints *"1 failing grammar flow(s)"*;
with `json` breached as well it exits **2** and prints *"2 failing grammar flow(s)"*. So the report
body does still carry the new breach — what saturates is the **exit code and the pass/fail verdict**,
which is precisely the layer a CI step branches on and a human glances at. ⇒ the blindness is real but
it is *at the verdict*, not in the artifact; say it that way, because a reader who checks will find the
count moved and discard the whole card.

⛔ There is a second, quieter cost: a gate whose verdict you can predict is a gate you stop running.
This one is operator-invoked, and across the five days it was red, ten commits touched its subject and
nobody ran it once.

## So adjudicate — which is not the same as bumping

The rule that keeps a ratchet honest is **a ceiling is LOWERED as a fix lands; it is never RAISED to
let a change pass.** That prohibition is about *your* change buying itself a pass. It does not cover
the different case where work already shipped — reviewed, gated, days old — and the ratchet simply was
not run. Adjudicating that is not cheating; leaving it red is the more expensive choice.

The bar for the raise is that the delta stops being a number and becomes a list:

1. **Re-confirm the RED at HEAD first.** A stale RED misleads exactly as much as a stale GREEN.
2. **Name every added row**, individually.
3. **Attribute each to an exact commit.**
4. **Prove each mechanism**, ideally with a one-edit control that removes that row and nothing else.
5. **Show no new *class* appeared** and that nothing silently disappeared — a net `+4` can hide `+7/−3`.
6. **Route anything that turns out to be a real defect** to a leaf that will LOWER the ceiling again.

Step 6 is the one that keeps this from becoming a laundering ritual. If a row survives the analysis as
a genuine defect, the raise is a bookmark with an owner, not an acceptance.

## Attribution: the tool has two inputs, and `git log -S` is not the instrument

⭐ **Pin one axis and vary the other.** A differential compares two implementations over a subject. If
both the subject and the thing that *builds* one of the arms have moved, a two-arm control that varies
both proves nothing. Build the binary once, then feed it the subject at two vintages. Here that
isolated the whole delta to the subject grammar in one run.

⛔ **`git log -S` reports a changed COUNT of a string, not a changed body.** A rule that was edited in
place, or moved, or had an annotation added above it, is invisible to it. It named two of four rows and
said nothing about the rest — and said nothing about *that*, either. The census that worked was blunt:
run the measurement at every vintage in the range and diff the result lists as a multiset keyed on the
divergence **site**, not on its text.

That census pays for itself twice, because it also answers a question no `-S` sweep can: **did anything
disappear?** A ratchet delta of `+4` is consistent with `+7 / −3`, and those are very different stories.

## You cannot subtract your way to the post-fix count

⭐⭐ **A divergence can mask another divergence at the same site.** Predicting `155 − 4 = 151` was
wrong: the fix removed four rows and *revealed* a fifth that had been hidden behind one of them, landing
at 152. Comparisons align position by position, so the first mismatch at a site is the only one that
gets reported; whatever sits behind it is invisible until the first is gone.

⇒ **arithmetic on a ratchet count is a prediction, never a result.** Write the prediction down before
you measure — a wrong one you kept is worth more than a right one you assumed — and re-measure after the
fix rather than asserting the arithmetic in the fix's own acceptance.

## The instrument that finds a defect is often the wrong one to size it

The one row here that turned out to be a real frontend defect — a comment swallowed into a return-
annotation payload, shipping verbatim in a generated artifact — showed up as **4** divergences. A direct
census of the shipped artifacts found **9** instances over 7 rules. The gate under-reported its own
finding by more than half, because it only sees the instances that happen to produce a kind mismatch.

⇒ once a gate hands you a defect, stop measuring the defect *with that gate*. Go count the population
directly, over a closed set, and state what the set is.

See also [[a-control-that-cannot-fail-is-not-a-control]],
[[an-instrument-can-be-green-in-both-arms-of-the-defect-it-describes]] and
[[a-control-that-clears-your-hypothesis-has-not-cleared-the-symptom]]. Those three are about gates that
pass when they should not. This one is the mirror image: **a gate stuck failing is just as blind, and it
looks responsible while it happens.**
