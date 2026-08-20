---
id: a-provenance-block-must-say-whether-the-numbers-were-ever-right
title: "Which tree was this derived from?" and "were the numbers ever right about that tree?" are two questions — a provenance block that answers only the first turns a vague RED into a confident wrong answer
answers:
  - "what fields does an identity or provenance block actually need"
  - "can I stamp a freshness block on a baseline that is currently failing"
  - "my gate says provenance fresh and then reports a drift nobody can explain"
  - "is recording input hashes enough to make a stale baseline diagnosable"
  - "how do I adopt provenance on an artifact I cannot re-derive yet"
  - "should a provenance state have a default value"
  - "why did adding provenance make my gate worse"
tags: [provenance, identity, baselines, gates, doctrine, diagnosis, evidence]
date: 2026-08-20
status: current
evidence: "SV-CORPUS-GRAD.13c.2x.2. Slice 1 (`PGEN-SV-CORPUS-GRAD-0248`) stamped `systemverilog_recognized_cert_union_contract.json` with `verified_at_commit` + input digests taken at HEAD — while `.13c.2x` had already MEASURED those expectations wrong at HEAD by 71 rules and 53 UNKNOWNs. The block therefore asserted a derivation that never happened, and the consuming gate printed `identity fresh` and proceeded to a drift report nobody could attribute. Retracted the same day by `-0249`, which added a REQUIRED `expectations: confirmed|unconfirmed` field with no default; the contract is now `unconfirmed` and the gate refuses in under a second, naming the reason and the owning leaf."
reverify: "bash scripts/check_baseline_identity.sh --verify rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json   # rc=1, ARE UNCONFIRMED, names SV-CORPUS-GRAD.13c.2x"
---

**Input digests answer exactly one question: *have the inputs moved since this checkpoint?*** They
say nothing at all about whether the numbers stored beside them were ever correct about that
checkpoint. Those are independent facts, and a block that carries only the first silently implies
the second.

The failure is worse than the gap it was meant to close, and in a specific way: **it converts an
honest vague RED into a confident wrong answer.** Before the block, PGEN's certificate-union gate
said *"expected 1362, measured 1433"* — unattributable, but it did not claim to know why. After a
naive stamp it said *"identity fresh"* first, which reads as *"your baseline describes this tree,
therefore the tree regressed"*. That is the wrong diagnosis, delivered with new authority, and it
would have sent a reader hunting a parser regression that does not exist.

⭐ **The fix is a required state with no default:**

```json
"identity": {
  "expectations": "confirmed",     // the numbers WERE re-derived against the digests below
  "confirmed_by": "<the run that did it>"
}
"identity": {
  "expectations": "unconfirmed",   // the numbers are known NOT to describe the digests below
  "unconfirmed_reason": "<what measured them wrong>",
  "owner_leaf": "<who owes the re-derivation>"
}
```

Two properties make this work rather than becoming a loophole:

1. **No default.** A missing state is *malformed*, not *assumed fine*. A default is how the original
   defect would reappear the next time somebody adopts in a hurry.
2. **`unconfirmed` is RED for every consumer**, not a waiver. It still buys two real things — input
   drift is still detected, and the artifact now names the leaf that owes the re-derivation — but a
   gate calling it refuses to measure. The only route to green is an actual re-derivation.

⚠️ **The practical consequence, and it is the point:** you may adopt provenance on an artifact you
cannot re-derive today. That is often the *most* valuable case, because an artifact nobody can
re-derive is exactly the one that has been rotting. What you may not do is let the adoption imply a
confirmation you did not perform.

See also [[declare-the-producers-not-the-consumers-in-a-dependency-set]] and
[[a-check-whose-inputs-all-pass-has-not-been-tested]].
