---
name: the-optional-half-of-a-delimiter-is-where-a-counter-goes-blind
description: "DISCIPLINE (2026-08-27, LIVE-DOC-CONTAINMENT.8 / PGEN-LIVE-DOC-CONTAINMENT-0011) — a count derived from a field splitter inherits that splitter's assumption about the EDGES of the record, and the edge is exactly where a format says OPTIONAL. MEASURED: a tracked census counted a GFM table row's cells as awk's `NF-2` under `-F'|'` — subtract the empty field before the leading pipe and after the trailing one — which is correct only for rows carrying BOTH, and GFM makes both optional. Every row omitting its trailing pipe counted one cell LOW, read at exactly the declared arity, and vanished from a census whose entire job is to find rows that do not: `REGEX-0098` in a contract shipped to a downstream consumer lost its `Notes` cell in two independent renderers while the census called it healthy, so the owning leaf published `25 of 197` for a population that is `26`. The same bug read a 5-cell row as a 4-cell UNDER row and a 22-cell row as 21. ⛔ The failure is in the PASSING direction and leaves no trace — no build breaks, no gate reddens, and the rendered page looks fine because a dropped cell leaves no gap. ⇒ before counting anything by splitting it, enumerate what the format marks OPTIONAL and write a case for each; the optional element is not an edge case of the format, it is the blind spot of every counter built on the mandatory one."
id: the-optional-half-of-a-delimiter-is-where-a-counter-goes-blind
title: "The optional half of a delimiter is where a counter goes blind — enumerate what the format calls OPTIONAL before you split on it"
date: 2026-08-27
evidence: "docs/tasks/LIVE-DOC-CONTAINMENT.md leaf .8, field `Instrument_defect_FOUND_AND_FIXED_2026-08-27` (the three re-derived corrections) against the same leaf's `Population_recensused_2026-08-26`, which published `25 of 197`; the corrected instrument docs/tasks/artifacts/live_means_live/table_arity_census.sh (function `cellcount`); the renderer that settled it, docs/tasks/artifacts/live_means_live/table_render_oracle.sh"
reverify: "printf '| A | B | C |\\n|---|---|---|\\n| x | y | z\\n' | awk -F'|' 'NR==3{print \"NF-2 says:\", NF-2}'   # prints 2 for a row carrying 3 cells — GFM's trailing pipe is OPTIONAL, so the mandatory-edge count is one LOW and the over-wide row disappears"
answers:
  - "why does my table-column census miss some rows"
  - "how should I count the fields of a delimited record"
  - "my counter and my renderer disagree by one row, which is right"
  - "what is the blind spot of a count derived from a field splitter"
  - "why did an instrument fail silently in the passing direction"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

When you count by splitting a record on a delimiter, the count depends on how the record's **edges**
are treated — and a format that marks an edge delimiter *optional* has two record shapes, not one.
Enumerate them and write a case for each **before** the count is used for anything. A counter built
on the mandatory shape does not report an error on the optional one; it reports a wrong number.

## Why — the measurement

`table_arity_census.sh` existed to find GFM table rows carrying more cells than their header declares,
because GFM discards the surplus silently. It computed the count as awk's `NF-2` under `-F'|'`:
subtract the empty field before the leading pipe and the empty field after the trailing one.

GFM makes **both** of those pipes optional. A row written without the trailing pipe has one fewer
field, so `NF-2` reports one cell fewer than the row actually has. An over-wide row therefore reads at
*exactly* the declared arity and drops out of the census.

Three published figures were wrong because of it:

| published | true | why |
| --- | --- | --- |
| ledger `25 of 197` over-wide | **26** | `REGEX-0098` omits its trailing pipe |
| index UNDER population `3` | **2** | `README-POLICY` carries 5 cells, not 4 |
| widest index row `21` | **22** | `SV-CORPUS-GRAD` omits its trailing pipe |

`REGEX-0098` sits in a contract this project ships to a downstream consumer, and it was losing its
`Notes` cell — the consumer-impact column — in *both* renderers while the census called it healthy.

## What actually caught it

Not re-reading the census. The census and an independently written repair tool answered **25** and
**26** on the same file; both are short, both look right, and the disagreement is one row in 197.
What settled it was neither implementation but the **renderer** — mdbook and pandoc, agreeing — which
reports what a reader actually loses rather than what a model of the rule predicts.

⇒ two implementations of the same rule are worth more than one careful implementation, and an oracle
outside both is worth more than the pair.

## The transferable shape

Any format with an optional affix has this hazard: a trailing record separator, an optional BOM, an
optional final newline, an optional leading `+` on a signed field, an optional enclosing quote in CSV.
The count is derived from *structure you assumed*, and the assumption is invisible in the code because
the mandatory case is the one you tested on. The tell is that the failure is **silent and in the
passing direction** — the counter does not crash on the optional shape, it agrees with you about it.

⚠️ Note what is NOT the lesson: `NF` was not the wrong tool. The defect was stating the record's shape
once, in arithmetic, instead of handling each shape the spec permits.

Related: [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] — the same failure one level up, about
a claim over the tree rather than a count over a record. And the scope corollary this slice also
produced: *"the instrument has 0 false negatives"* is itself a census claim, true only over the
population it was measured on.
