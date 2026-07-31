---
name: feedback-enumerating-instrument-must-refuse
description: DISCIPLINE (2026-07-31, session #229, LIVE-MEANS-LIVE.2) — when an instrument has to ENUMERATE the shapes of the thing it looks for, the list is the defect. A shape it does not know produces an ABSENT ROW, not a reported miss, so it under-reports and looks clean doing it — silent in the PASSING direction. One population measured 10, then 16, then 18 across three passes. The fix is not a longer list: require TOTAL CLASSIFICATION of the input and REFUSE on anything unmatched.
metadata:
  node_type: memory
  type: feedback
---

**The founding case.** `LIVE-MEANS-LIVE`'s instrument B asks a simple question: does a document's
self-declared `Last updated:` disagree with the newest date in its own body? It measured the same
tracked-markdown population three times and got three answers:

| pass | population | knew | blind to |
|---|---:|---|---|
| `.4` | **10** | `Last updated: 2026-05-14` (root docs) | ~50 `docs/tasks/` trees written `` - Last updated: `2026-05-31` ``, plus one phantom row from prose |
| `.5` | **16** | + the backtick-quoted form | the `docs/contracts/` **continuation** form — `- Last updated:` with the date on the *following* line — 6 files, **2 self-refuting** |
| `.2` | **18** | + continuation + template, fences and quoted text excluded | — (it refuses instead) |

Each pass was a careful correction of the one before, and each was still wrong.

## Why this failure mode is worse than an ordinary bug

**A shape the extractor does not know is not a reported miss — it is an absent row.** The output
looks like a clean, complete population. There is no error, no warning, no count of skipped input.
The instrument under-reports *and looks healthy doing it*, which is failure in the **passing
direction**: the direction where nobody investigates.

Both corrections above were possible only because a **prior published number existed to disagree
with**. Without `.4`'s 10 on the page, `.5`'s 16 would have been accepted as fact on first sight;
without `.5`'s 16, so would 18. The cheapest ground truth available to any instrument is *the last
time somebody measured this*.

**And the blind spot is a category, not a scatter.** Dialects cluster by directory, so an
extractor's gap tends to be a whole surface class. Here the two files the second pass missed were
the published **SystemVerilog and VHDL integration contracts** — the class where a stale claim is
most expensive, and the class where `DONE-BAR.5a` had already measured ~77 releases of undetected
drift. ⇒ when an extractor's coverage is uneven, expect to lose a category.

## The discipline

> **If an instrument must enumerate the shapes of what it looks for, it must also REFUSE on any
> input it cannot classify. Enumeration without refusal is silent under-reporting.**

Concretely:

1. **Classify the whole input space, not just the matches.** Every candidate the instrument
   *anchors on* must resolve to a pinned shape or to an explicit exclusion. Nothing may fall
   through.
2. **REFUSE (a distinct nonzero exit) on anything unclassified** — do not skip it, do not warn and
   continue. A fifth spelling appearing tomorrow must be a loud refusal, not a shorter list.
3. **Pin the exclusions as deliberately as the matches.** "Not a declaration" is a claim too:
   here, fenced code blocks (sample output) and indented text (quoted material) are excluded *by
   construction*, each backed by a control built on a real measured occurrence.
4. **Give every shape AND every exclusion a ground-truth control**, so a change in the extractor's
   meaning aborts before a number is published ([[feedback_instrument_needs_ground_truth]]).
5. **When the instrument fires on your own documentation, that is a real class — do not reword
   around it.** Instrument B refused on the very task leaf documenting it, because a wrapped
   quotation of its output put `Last updated:` at the start of an indented line. Reflowing the
   paragraph would have left the class live for the next author and taught them to route around the
   gate — the failure mode `DOCTRINE_ENFORCEMENT.md` §6.1 names, where a hand-written waiver sat
   unread inside a ticked box. The fix belonged in the anchor.

## How this differs from its sibling record

[[feedback_instrument_needs_ground_truth]] calibrates an instrument's **output** — pin facts the
number must reproduce, refuse when it cannot. This record is about the instrument's **input
coverage**: even a perfectly calibrated extractor reports a clean, wrong population if it silently
never saw part of its corpus. Controls catch a *changed* meaning; total classification catches an
*unseen* dialect. Both are needed, and only the second has a chance against a shape nobody has
thought of yet.

Related: [[feedback_read_prior_art_before_designing]] (re-measure, never quote — the habit that
surfaced all three disagreements), [[feedback_flow_findings_are_routed_not_worked]] (the two new
findings this produced became leaves `.6`/`.7` rather than scope creep).

Live instance: `scripts/check_live_document_currency.sh` (doctrine `LIVE-DOC-CURRENCY`), whose
`UNCLASSIFIED` refusal path is the reference shape and is itself covered by a control. Evidence and
the full shape census: `docs/tasks/LIVE-MEANS-LIVE.md` leaf `.2`.
