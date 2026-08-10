---
name: feedback-pin-a-routed-defect-with-a-test-that-asserts-it
description: DISCIPLINE (2026-08-11, SV-CORPUS-GRAD.12c.1/.12c.2) — when you find a defect you are not fixing in this commit, routing it to a leaf protects it from being forgotten but NOT from changing silently. Land a test that ASSERTS THE DEFECT, with a comment ordering the fix to flip it. One test closes both directions at once: the fix cannot be silent (the test fails the moment behaviour changes) and cannot rot afterwards (it keeps failing on regression). Costs one test and a comment; here it paid inside one commit.
metadata:
  node_type: memory
  type: feedback
id: feedback-pin-a-routed-defect-with-a-test-that-asserts-it
title: A routed defect needs a test that ASSERTS it — routing protects against forgetting, not against silent change
date: 2026-08-11
answers:
  - "I found a bug I am not fixing right now — what do I do besides open a leaf?"
  - "how do I stop a deferred finding from being silently fixed or silently regressing"
  - "is it ever right to write a test that asserts wrong behaviour?"
  - "how do I make sure the next commit notices the thing I deferred"
  - "a routed finding disappeared and nobody knows when — how do I prevent that"
reverify: cd rust && cargo test --features generated_parsers --lib sv_preprocessor::   # the pins landed by .12c.1 and flipped by .12c.2
---

**The founding case.** `SV-CORPUS-GRAD.12c.1` was fixing how PGEN *reads* source files. Wiring the
SystemVerilog preprocessor into the new reader, its test asserted that a Latin-1 `©` survives — and
failed, with `©` arriving as `Â©`. That turned out to be a **second, unrelated, pre-existing
defect** one layer down: eight line scanners re-emitting with `out.push(bytes[i] as char)`, a
Latin-1 promotion rather than a UTF-8 decode, which doubles every non-ASCII character *including in
files that are valid UTF-8*.

Different root cause, so it belonged in its own leaf (`.12c.2`), not in the one in flight. The
question is what protects it in the meantime.

## Routing is necessary and it is not sufficient

A leaf stops a finding being **forgotten**. It does nothing about the two ways a deferred finding
goes wrong quietly:

- **Silently fixed.** Unrelated work touches the same code and the behaviour changes. Nobody
  notices, the leaf stays `todo` forever, and the repo now carries a leaf describing a defect that
  no longer exists — which is worse than no leaf, because a later reader trusts it.
- **Silently regressed.** The fix lands later, and something afterwards puts the old behaviour
  back. Nothing was ever asserting the corrected shape, because the correct shape was never
  encoded anywhere.

## The discipline

> **Land a test that asserts the defect, with a comment ordering the fix to change it.**

```rust
#[test]
fn a_plain_utf8_source_is_double_encoded_pinned_defect() {
    // ⛔ THIS TEST ASSERTS A DEFECT, DELIBERATELY. Found by `.12c.1`, owned by `.12c.2`.
    // It is pinned rather than merely noted so the fix is FORCED to notice it: `.12c.2` must
    // change this test, and a silent regression back to byte-as-char would resurrect it.
    assert_eq!(output.text.len(), source.len() + 7,
               "one extra byte per non-ASCII byte: ©=2 + —=3 + µ=2");
}
```

One test closes both directions. It fails the moment the behaviour changes, so the fix **cannot be
silent**; and once flipped, it keeps failing on regression, so the fix **cannot rot**. Here the pin
paid inside a single commit: `.12c.2` landed next and had to flip both pins by name.

Four rules make it work rather than becoming clutter:

1. **Say `PINNED TO A KNOWN DEFECT` in the test name or its first comment line.** A green suite
   asserting wrong behaviour is a trap unless it announces itself.
2. **Name the owning leaf in the comment**, and name the assertion the fix must write instead.
   *"When it lands, this assertion MUST be flipped to X."*
3. **Pin the QUANTITY, not a symptom.** `contains("Â©")` can pass for the wrong reason. `output.len()
   == source.len() + 7` over a line carrying a 2-, a 3- and a 2-byte character cannot be satisfied
   by any per-byte promotion — that is the assertion that survives contact with a partial fix.
4. **Pin the consequence, not only the appearance.** The visible mojibake was the cheap half; the
   half that outlives it was that every source-map byte range after a non-ASCII character named the
   wrong bytes. That got its own lock when the fix landed.

⚠️ **When NOT to pin.** If the defect is genuinely blocking, fix it instead — a pin is for a finding
you have consciously scheduled later, never a way to feel finished. And do not pin behaviour you
have not measured: a pin encodes a claim, so an unmeasured pin is a guess promoted to an assertion.

Companion to [[feedback_every_finding_must_be_fixed_not_logged]] and
[[feedback_every_finding_is_owned_and_scheduled_never_just_logged]] — those establish that a
finding must be OWNED and SCHEDULED; this adds the mechanism that makes the schedule
self-enforcing. Sibling of [[feedback_a_named_call_site_is_a_category_of_call_sites]], the other
discipline the same pair of leaves produced.
