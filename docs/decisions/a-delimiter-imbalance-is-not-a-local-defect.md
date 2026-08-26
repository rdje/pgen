---
name: a-delimiter-imbalance-is-not-a-local-defect
description: "DISCIPLINE (2026-08-27, LIVE-DOC-CONTAINMENT.8 / PGEN-LIVE-DOC-CONTAINMENT-0012) — when a format pairs delimiters LEFT TO RIGHT over a whole record, ONE unmatched delimiter shifts every pairing after it, so the position an imbalance detector REPORTS is the RESIDUE of the cascade and almost never the typo. MEASURED on a shipped contract: `SV-0034` in the released-parser bug ledger needs exactly ONE pipe escaped (a BNF alternation inside a code span), but a stray trailing backtick elsewhere in the row made a CommonMark code-span scan report THREE, so a mechanical fixer escaping all three landed the row at 11 cells against a 13-column header. Four rows were diagnosed by chasing the reported unmatched position; in all four the span AT that position was correctly closed and the real stray was upstream. ⛔ The corollary that makes this expensive: a per-row 'balanced?' test is not a per-row test — a defect in row N is invisible in row N and shows up as damage in the rest of the record. ⇒ when a detector reports an imbalance, treat the position as a BOUND ('somewhere at or before here'), re-pair from a known-good anchor, and validate the repair against a semantic property of the record rather than against the detector that found it."
id: a-delimiter-imbalance-is-not-a-local-defect
title: "A delimiter imbalance is not a local defect — the reported position is the cascade's residue, not the typo"
date: 2026-08-27
evidence: "docs/tasks/LIVE-DOC-CONTAINMENT.md leaf .8, field `Step_2_READ_and_LANDED_2026-08-27` (the SV-0034 worked case: one real join reported as three, landing 14 -> 11 against a 13-column header); the HAND_JOINS table and its header comment in docs/tasks/artifacts/live_means_live/table_pipe_repair.py; the renderer that adjudicated it, docs/tasks/artifacts/live_means_live/table_render_oracle.sh"
reverify: "bash docs/tasks/artifacts/live_means_live/delimiter_cascade_demo.sh   # prints `balanced -> 0` then `one stray up front -> 1`, and exits non-zero if that stops holding: nothing changed at the pipe, and the pipe changed meaning"
answers:
  - "my markdown table row breaks even though the code span looks closed"
  - "an imbalance checker points at a line that looks correct, why"
  - "why did escaping every pipe my scanner found make the row worse"
  - "how do I find the real unmatched delimiter"
  - "is a per-record balance check enough to localise a delimiter bug"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

When a format pairs delimiters by scanning left to right — CommonMark code spans, quotes, brackets,
comment markers — an unmatched delimiter does not fail *where it is*. It re-pairs everything after
it. So the position a checker reports is where the scan **ran out**, which is a bound, not a
location. Localise by re-pairing from a known-good anchor, and validate the repair against a
**semantic** property of the record, never against the checker that found the problem.

## Why — the measurement

`SV-0034` in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` carries one genuinely broken
cell boundary: a BNF alternation written inside a code span,
`` `associative_dimension := lbrack data_type rbrack | lbrack star rbrack` ``. GFM splits a table
row on `|` *before* it parses inline content, so that pipe really does break the row, and escaping
it is the whole repair.

A CommonMark code-span scan over that row reported **three** in-span pipes. Escaping all three took
the row from 14 cells to **11**, against a 13-column header — worse than the defect. The two extra
"findings" were real cell separators, swallowed into phantom spans because a stray trailing backtick
later in the row had shifted every pairing.

Four rows were then diagnosed by chasing the reported "unmatched backtick" position. In **all four**
the span at that position was correctly closed:

| row | reported at | what was actually there |
| --- | --- | --- |
| `REGEX-0108` | `` `descending_class_range `` | correctly closed — `` `descending_class_range` `` |
| `SV-0035` | `` `.3 `` | correctly closed — `` `.3` `` |
| `SV-0048` | `` `undefineall `` | correctly closed, a proper double-backtick span |
| `SV-0034` | end of the Fix-Proof cell | this one *was* the stray — and only because it was last |

Greedy pairing leaves the leftover at the **end**, so the reported position is informative only when
the stray happens to be the final delimiter.

## What actually localises it

Two things, neither of them the checker:

1. **A known-good anchor.** The bug ledger declares 13 columns with recognisable shapes — an id, a
   bare ISO date in column 6, a release string in column 12, free prose in column 13. Reading the
   fragments back onto the columns they belong to reconstructs the intended boundaries without any
   delimiter model at all.
2. **The renderer.** It reports what a reader loses, so it adjudicates between two disagreeing
   models instead of adding a third.

## The transferable shape

Any left-to-right pairing has this property, and the tell is always the same: the detector's output
is *plausible everywhere* and *wrong in aggregate*. Watch for it in code-span and quote scanning,
in comment-marker balancing, in bracket matching over a file, and in any lint that reports "N
occurrences" of something whose parse depends on the prefix of the record.

⚠️ Note what is NOT the lesson: the scan was not badly written. A greedy pairing IS what CommonMark
specifies. The defect was reading its output as a list of independent findings when the format makes
them a single dependent chain.

Related: [[the-optional-half-of-a-delimiter-is-where-a-counter-goes-blind]] — the other way a
delimiter model fails silently, found in the same leaf.
