---
id: rigour-does-not-spread-by-proximity
title: A claim written next to a measured one is still unmeasured — and it is reliably the sentence that turns out to be false
answers:
  - "I drove red controls on part of a change — is the rest of the write-up trustworthy"
  - "someone challenged my findings — do I re-read them or re-run them"
  - "which sentence in my own report is most likely to be wrong"
  - "I deferred a verification leg to a later slice — can I publish the finding now"
  - "is it safe to store a commit distance or a count of days in a document"
  - "my instrument returned a number that does not change my verdict but that I cannot explain"
  - "how do I answer a director challenge without either caving or defending"
tags: [claims, controls, verification, retraction, derived-state, claim-verification, evidence, sv-corpus-grad]
date: 2026-08-25
status: current
evidence: "PGEN-SV-CORPUS-GRAD-0299, a re-verification of four findings published with -0296/-0298 after the director asked whether I still stood by them. Re-deriving (not re-reading) found: ONE FALSE — I published 'both wrong labels PASS their gates' about the SV revision register, one paragraph away from five red controls I HAD driven and reported honestly; measured, a NEUTRAL row whose digest moved is REFUSED rc=1 by tier C, and only RELEASE passes, so exactly one label is silent, not two. ONE UNDER-CLAIMED — the four L4 punctuation rows were published as 'candidates awaiting leg 3' while leg 3 was four probe invocations away; run, --dump-rule-outcome-counts-json shows each operator's own terminal rule COMMITTED (plus_assign 1, shift_left_assign 1, wildcard_equal 1, dot_star 3), confirming all four as verilog_2005 over-acceptances. TWO NUMERIC ERRORS — '53 days' for 2026-07-02..2026-08-25 (it is 54), and '1912 commits' which was correct when measured against HEAD 29001add, became 1913 at the commit that published it and 1915 later: a distance to HEAD stored in something that changes HEAD. None of the four would have moved on a careful re-read."
reverify: "Take any finding you published alongside driven controls and re-run ONLY the sentences you did not drive. On this one: label the register row NEUTRAL with a moved digest and run scripts/check_sv_contract_currency.sh -> rc=1 REFUSED; label it RELEASE with a version and no ledger row -> rc=0 PASSES."
---

# Rigour does not spread by proximity

**Question it answers:** I drove red controls on part of a change and wrote the finding up carefully.
Which sentence is most likely to be wrong?

**Answer:** the one sitting next to the controls. A measured claim does not lend its rigour to its
neighbours, and the write-up is exactly where an unmeasured sentence looks most trustworthy —
because everything around it *was* measured.

## Measured

Five red controls were driven on a new register disposition and reported honestly, arm by arm. One
paragraph away, in the same leaf, sat:

> "Both wrong labels PASS their gates."

Neither had been driven. Driving them:

| label | verdict |
|---|---|
| `NEUTRAL`, digest moved | ⛔ **REFUSED, `rc=1`** — tier C catches it by design |
| `RELEASE` + version, no ledger row | ✅ **passes, `rc=0`** |

Exactly **one** wrong label is silent. The sentence was false.

⭐ **And the corrected finding is sharper than the one it replaces.** "Both labels are silent" is a
diffuse complaint about a vocabulary; "exactly one label is silent, and it is the only remaining
slot" names the failure mode. Over-claiming does not just cost credibility — it costs *resolution*.

## Three companion failures from the same re-verification

1. **A deferred leg that was minutes away.** Four rows were published as *"candidates awaiting leg
   3"*. Leg 3 was four probe invocations and about ten seconds; run, it confirmed all four.
   ⇒ **if the missing leg is minutes away it is not a later slice, it is part of the claim.**
2. **A stored derived number that drifted inside one commit.** "1 912 commits back" was *correct
   when measured* and became wrong the instant my own commit landed — 1 913 at publication, 1 915
   later. ⇒ **a distance to HEAD cannot be stored in something that changes HEAD.** Store the
   command, never the count.
3. **A date subtraction done in my head.** 2026-07-02 → 2026-08-25 is **54** days; I published 53.
   Two lines of `datetime` would have settled it.

## The rule that falls out

When challenged, **re-derive; do not re-read.** Not one of these four would have moved on a careful
re-read: the false sentence *sounds* right, the under-claim was a deliberate deferral, and the two
numeric errors are arithmetic that reads fine. Re-reading checks whether the text is consistent with
what you believe. Only re-running checks whether what you believe is true.

⚠️ And note the direction the re-derivation ran in: one claim got **stronger**. A challenge is not a
prompt to retract — it is a prompt to measure, and measurement moves claims both ways. Caving is as
inaccurate as defending.

## See also

- [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] — the same disease in the claim's *scope*:
  reading a few implementations is sampling, not a census.
- [[when-your-delimiter-is-also-legal-content-anchor-the-parse]] — the same disease in the claim's
  *instrument*.
- `docs/CLAIM_VERIFICATION.md` — legs 1/2/3. This class fails leg 1: the claim was never re-derived
  at all, because its neighbours had been.
