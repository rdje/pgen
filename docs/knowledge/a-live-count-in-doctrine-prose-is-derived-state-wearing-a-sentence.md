---
id: a-live-count-in-doctrine-prose-is-derived-state-wearing-a-sentence
title: A live count in doctrine prose is derived state wearing a sentence — cite history freely, never a present-tense number the tree can move; and a "cited number must appear in the enforcer" gate is ~95 % false positives, so the rule is R1/R3, not a gate
answers:
  - "how many of the doctrine descriptions disagree with their own enforcer"
  - "should the enforcer header be derived from the registry row"
  - "a doctrine's description quotes a count — how do I know it is still true"
  - "can I gate that every number a doctrine row cites appears in its check script"
  - "where may a founding measurement live and where may a live count not"
  - "the registry row, the §10 row and the book say the same number — is it current"
tags: [doctrine, prose, derived-state, census, false-positives, governance, sv-corpus-grad]
date: 2026-09-02
status: current
evidence: "SV-CORPUS-GRAD.13v / PGEN-SV-CORPUS-GRAD-0338. `docs/tasks/artifacts/sv_corpus_grad/doctrine_prose_census/census.py` over all 34 doctrines: 44 numbers cited in a registry or §10 row that the enforcer's text never contains. Read one by one: 38 founding history, 1 register-read constant, 3 live and true, 2 LIVE AND ROTTED — `LIVE-DOC-CURRENCY` said `CHANGES.md scores 175` (the enforcer's own `20\\d{2}-\\d{2}-\\d{2}` pattern finds 206), `PARSE-COST-RATCHET` said `131 declared LR names, 131 classified` (`--verify-families` reads 41 / 41; `git log -S` dates the 131 to 2026-08-15). Each sat in three reader-facing homes. A presence gate would have fired on 42 of 44 correct numbers."
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/doctrine_prose_census/census.py 2>&1 >/dev/null | tail -1   # DOCTRINE-PROSE-CENSUS: rows=34 … — compare against after_headline.txt beside it; every new lead is READ, not counted"
---

**Doctrine prose is a live document.** The registry row and the §10 row explain a doctrine to a
maintainer; the derived book explains it to the director; the enforcer's header explains it at the
point of edit. A number typed into any of them stays there until someone re-reads it, and the
meta-checks hold only the NAME roster equal.

## Two kinds of number, one rule

| the number is… | example | may it live in prose? |
|---|---|---|
| **history** — the incident that founded the doctrine | `138,403 bytes at 60 lines`, `160 disagreements, not one a count` | yes — that is what the prose is for |
| **live** — present tense, movable by the tree | `CHANGES.md scores 175`, `131 declared LR names` | **no** — name the producer (*"the count is the instrument's own output on every run and is never stored here"*) |

This is `DERIVED_STATE_CONTAINMENT` R1/R3 — *a field a command answers exactly is looked up, never
written down* — applied to the one class of document that had escaped it.

## Why the obvious gate is refused

"Every number a doctrine row cites must appear in its enforcer's output or header" reads like a
clean invariant. Measured over the whole registry it fires on **42 of 44** correct numbers: the
founding history is legitimately absent from the enforcer, and one constant is read from a register
rather than the script text. A ~95 % false-positive gate is past the bar this repository set when it
refused a 91 % trigger, and it is a gate people learn to waive. Deriving the header from the row is
refused for the same reason — the header is where the full history belongs.

⇒ the census stays a **lead generator** (0.3 s), its `after` run the baseline for the next; live vs
history is a **reading**, and the reading is the deliverable. ⭐ Read the population — 44 leads at
twenty seconds each is fifteen minutes, and publishing "44 disagreements" would have been a count
that was not a count ([[an-x-is-checked-by-nothing-claim-is-a-census-claim]]).

⚠️ And the instrument mis-reported twice before it was trusted — a digit-group split (`138,403` →
`138`, `403`) and a register-read constant scored as absent. A census instrument's false positives
are its own first finding; fix them before the headline, not after.
