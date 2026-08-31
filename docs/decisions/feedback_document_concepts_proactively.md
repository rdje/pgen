---
name: feedback-document-concepts-proactively
description: "STANDING DIRECTIVE (director, 2026-08-31): every concept, term or metric the engineer USES to describe their work must be documented proactively in the mdBook — not on request. The director had to ask twice in one session what `rc N` and `margin 5 115x` meant; both were load-bearing vocabulary in published findings, and neither was defined anywhere a reader could look them up. If it is worth saying in a status message, it is worth a definition in the book."
metadata:
  node_type: memory
  type: feedback
id: feedback-document-concepts-proactively
title: Document the vocabulary you use — proactively, in the book, without being asked
date: 2026-08-31
answers:
  - "do I need to document a term I used in a status message"
  - "the director asked what a column means — where should the answer live"
  - "is a glossary worth maintaining"
  - "I invented a metric for this measurement — does it need documenting"
  - "when does project vocabulary belong in the mdBook"
  - "the same word means two things in two leaves — is that a defect"
  - "do I wait for a question before explaining a concept"
reverify: grep -c '^| `' docs/book/src/glossary.md
---

**THE DIRECTIVE, verbatim (2026-08-31):** *"Just proactively document concepts, things you use to
describe whatever you are doing. You should not wait for me to ask."*

**WHAT PROMPTED IT.** In one session the director had to ask, separately, what `rc N` meant and what
`margin 5 115×` meant. Both were load-bearing vocabulary in findings already published to them —
`rc 134` was the entire severity claim of a shipped-artifact crash, and `margin` was the number every
before/after table turned on. Neither was defined in the book. A third term (`OVERSHOOT`, `B_FRAME`,
`DEPTH` vs `LEVELS`) had to be explained column by column on request.

**WHY IT MATTERS MORE THAN IT LOOKS.** The director reviews the BOOK, not the code
(`docs/decisions/`, standing). So an undefined term in a status message is not a small discourtesy —
it is a finding the reviewer cannot check. A number nobody can interpret is indistinguishable from a
number nobody can verify.

⛔ **AND IT SURFACED A REAL DEFECT, WHICH IS THE ARGUMENT FOR THE RULE.** Answering *what is a
margin* revealed that this repository calls TWO DIFFERENT QUANTITIES "margin", in the same unit, in
adjacent leaves: `reserve ÷ overshoot` (`.69`(a)/`.85` — how much headroom a parse ate) and
`budget ÷ (4 096 × B/frame)` (`.79`(a) — whether the guard can REACH its first sample). Both were
committed, published and compared in prose as if commensurable. **The ambiguity was invisible until
someone had to define the word**, which is precisely what writing a glossary forces.

## The rule

- **If a term appears in a status message, a leaf headline or a published table, it has a definition
  in `docs/book/src/glossary.md`** — added in the SAME slice that first uses it, not later.
- **Define the quantity, not the word**: formula, units, and what question it answers. A metric with
  no formula is a vibe.
- **Two things may not share a name.** If they do, say so at both sites until one is renamed.
- **State what a value CANNOT be**: censored ranges, sentinel values (a zero byte field meaning NOT
  MEASURED), and signed quantities whose negative arm is legitimate.
- ⚠️ NOT the lesson: write more prose. The glossary is a lookup surface — a row per term, linked from
  the chapters that use it — not another narrative.
