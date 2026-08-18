---
id: a-counter-that-cannot-tell-a-cache-hit-from-work-prices-them-alike
title: A counter that cannot tell a CACHE HIT from real work prices them alike — so a "regression" it reports may be pure cache traffic, and the gate that reads it needs a typed way to say so
answers:
  - "my deterministic perf counter rose but I don't think anything got slower — how do I tell"
  - "how do I prove a cost rise is memoization lookups rather than new work"
  - "my perf gate refuses every rise and my fix is correct — what is the sanctioned way out"
  - "how do I let a gate accept a measured regression without turning it into a waiver"
  - "what makes an exception record different from a rubber stamp"
  - "should a cost baseline ever move upward, and what has to be true first"
tags: [instruments, performance, ratchets, gates, evidence, memoization, waivers, measurement]
date: 2026-08-18
status: current
evidence: "SV-CORPUS-GRAD.13c.2j + ENGINE-UNIVERSAL-SERVICES.36 (2026-08-18). Restoring an IEEE 1800 A.8.4 branch the SV grammar had lost raised PARSE-COST-RATCHET's binding counters: entries 416,424,205 -> 417,009,457 (+0.14 %) and memo_hits 186,926,969 -> 187,512,221 (+0.31 %). The two deltas are IDENTICAL to the unit (+585,252 each) and `committed` is FLAT at 7,123,491 -> every added rule entry was answered by the memo table, so the change asked 585,252 more CACHED questions and did zero new parsing work. The counter cannot see that: a memo hit is one rule entry, exactly like a parse. ⛔ The gate's breach text already said 'record it as irreducible with the measurement that proves it' and there was nowhere to record it -- every rise was a fail() and the rebaseline refuses while a failure stands, leaving exactly two outcomes: abandon a correctness fix, or skip a doctrine."
reverify: "PGEN_PARSE_COST_REMEASURE=1 bash scripts/check_parse_cost_ratchet.sh   # a rise is accepted only by a docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/accepted_rises.tsv row naming the EXACT from/to plus a coded invariant, which the gate RE-EVALUATES on the run's own numbers; the five refusal arms are bash docs/tasks/artifacts/engine_universal_services/accepted_rise_gate/probe.sh"
---

**A memo hit and a parse are both one rule entry.** So a counter keyed on rule entries prices a
cached lookup exactly as it prices real work, and a change that only asks the parser more questions
it has already answered shows up as a regression indistinguishable from one that made the parser do
more.

This is the CONVERSE of the blind spot already banked as
[[a-deterministic-counter-cannot-see-a-per-entry-cost-rise]]: that card records that the counter
cannot see cost growing *inside* an event. This one records that it cannot see an event being *free*.
Same metric, same structural cause, opposite sign — and unlike a sensitivity bound, both follow from
what the metric counts and so cannot go stale.

## What made it decidable rather than arguable

The three binding counters are published together, and their JOINT pattern is the evidence:

```text
entries    416,424,205 -> 417,009,457    +585,252
memo_hits  186,926,969 -> 187,512,221    +585,252     <- identical, to the unit
committed    7,123,491 ->   7,123,491          +0     <- nothing new survived
```

`delta_entries == delta_memo_hits` with `delta_committed == 0` is not a comfortable reading of a
number; it is a statement with a mechanism. Every entry the change added was served from the memo
table, and none of them committed. ⭐ **One counter alone could not have said this.** The insight is
not "publish more counters" but *publish counters whose RELATIONSHIPS are diagnostic* — the reason
this ratchet binds on entries **and** committed **and** memo hits rather than picking the best one.

And the elimination question was answered by the same mechanism instead of by trying things: a
`!( identifier dot )` guard to skip the new alternative would replace one memoized lookup with an
`identifier` match plus a `dot` match — themselves rule entries. The obvious optimisation would
likely cost MORE of the metric it was meant to save.

## The gate-design half, which is the transferable part

The gate's own breach message ended *"… or record it as irreducible with the measurement that proves
it"*, and no such record existed. Every rise was a hard failure; the rebaseline refused while a
failure stood. That leaves a correct fix exactly two outcomes: **abandon it, or bypass the gate.**

⛔ **A gate that leaves a legitimate outcome unrepresentable does not prevent that outcome — it moves
it out of the record.** The same shape had been found one day earlier in a completely different
instrument in the same repository: a reproducer manifest that could express *valid text wrongly
rejected* and *invalid text correctly rejected*, but had no class for *invalid text wrongly
ACCEPTED*, so a known over-acceptance could live in prose and nowhere a runner could see it. Two
instruments, two directions of the same asymmetry. ⭐ **When a check has a direction it cannot
express, look for the outcome people are quietly taking instead.**

## What separates an acceptance from a waiver

The fix was a typed acceptance record, and three properties are what keep it from becoming a rubber
stamp — each one worth copying:

1. **Exact integers, not thresholds.** A row names the precise `from` and `to`. It cannot cover any
   rise but the one it was written for; a different number finds no row and fails.
2. **The justification is CODE, not prose.** `invariant` must name a predicate implemented in the
   gate. Accepting a rise of a *new shape* therefore takes a code change with its own owning task —
   the data file cannot invent new reasons.
3. **The justification is RE-EVALUATED, never trusted.** The invariant runs against the current
   run's numbers. When it stops holding the gate fails with *"the acceptance's own justification is
   gone; re-attribute the rise, do not re-word the row"*.

⚠️ And the honest bound, in the record itself: once the baseline is promoted the row matches no rise
and stops being re-derived. It survives as the audit trail for **why the baseline moved upward** —
which is the one thing a promoted baseline can no longer tell you.

## The mechanism that proved the mechanism

Every way the acceptance must refuse was fired and observed (unaccepted rise, broken invariant,
unknown invariant name, malformed row) plus a control asserting the unperturbed tree still passes —
because without that control a probe that simply broke the gate would score full marks. ⭐ Both bugs
the probe itself shipped with were caught by its own assertions rather than by review: it resolved
the repository root one level short, so every arm returned `rc=127`, refused because an arm asserts
its exit code *and* its message; then it perturbed the totals printed in the human-readable report
while the gate compares totals summed from the machine-readable one, and the first arm scored a
pass. **A probe that perturbs the wrong file reports the gate as holding when it was never
challenged** — see [[a-check-whose-inputs-all-pass-has-not-been-tested]].
