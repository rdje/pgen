---
id: a-census-keyed-on-a-decoration-reads-the-author-s-mood-not-the-status
title: A census keyed on a decoration reads the author's mood, not the status — count the status WORD, exclude the ✅-headed line, declare the heading level, and keep the retired expression as a self-test arm so the old number stays explicable
answers:
  - "the release distance did not move when I opened a leaf — why"
  - "why did report_sv_release_distance.sh count 3 open SV leaves when the tree had 10"
  - "how does the SV release distance count open leaves now"
  - "a heading begins with ⛔⛔ or ⚠️ — does the distance census see it"
  - "an engine leaf opened by SV-CORPUS-GRAD was not in the engine count — what was wrong with the matcher"
  - "the trailing closure rate divided by the wrong number of releases — what was in the history"
  - "I am writing a census over task-tree headings — what should it key on"
  - "a published number jumps after an instrument fix — what has to happen before it is republished"
  - "the task tree files leaves at several heading levels — how does the distance census avoid counting an umbrella beside its children"
tags: [instrument, task-tree, release-distance, census, self-test, derived-state, status-word]
date: 2026-10-09
provenance: SV-CORPUS-GRAD.49 (PGEN-SV-CORPUS-GRAD-0437); SV-CORPUS-GRAD.51 (PGEN-SV-CORPUS-GRAD-0438, the level rule); SV-CORPUS-GRAD.25 (the instrument, PGEN-SV-CORPUS-GRAD-0406); SV-CORPUS-GRAD.46 (close-on-measurement)
reverify: "bash scripts/report_sv_release_distance.sh --self-test   # SV-RELEASE-DISTANCE-SELF-TEST: pass=7 fail=0; the retired decoration form reads 1 and the #####-only form 6 of the same seven open lines"
---

# A census keyed on a decoration reads the author's mood, not the status

**The shape.** The SV release distance (`scripts/report_sv_release_distance.sh`, `SV-CORPUS-GRAD.25`) counted an
open leaf of `docs/tasks/SV-CORPUS-GRAD.md` with `^##### (⛔|⏳) \``: one mark, then a space, then the id. The book
defined the term by the STATUS WORD — a `todo` / `active` leaf. The two agreed on the day the instrument shipped
and diverged the first time an author wrote `⛔⛔`, put the mark after the id, or opened a leaf with `⚠️`. Measured
on 2026-10-09: the tree carried **10** `#####` headings with the status word and the instrument published **3**.
The engine matcher had the same defect in a different coat: it demanded `opened <date> by \`SV-CORPUS-GRAD`, and a
heading carrying `session #251` between the date and the `by` (`ENGINE-UNIVERSAL-SERVICES.41`) was not counted.

**Why it is the author's mood.** A decoration is chosen per heading, by feel, at the moment of writing: one ⛔ for
a defect, two for an urgent one, three for a director challenge, ⚠️ for a routed finding, ⭐ for an order. None of
that is the status, and the tree already has a vocabulary for the status — the word in backticks. An instrument that
keys on the decoration measures how worried the author was, not whether the work is open.

**The fix, in order.**
1. **Adjudicate before the number moves.** Implementing the definition moved the count by +5 at once; at least two of
   the headings were the *done-under-open* class (`.13c.2x.4`: ALL FOUR MOVES LANDED the day it opened, heading
   `NEW todo` for fifty days; `.13c.2x.9`: every clause discharged, heading `todo` for forty-eight). Each of the ten
   was read and measured first — the runner's line order, the probe's verdict on seven directives, a register's
   `deferred` rows, a comment-line probe against the identity gate, a tuple census — and the heading set to its
   true status. Then the number moved, and every unit of the jump names a leaf.
2. **Count the word.** `grep -cE '^##### [^✅]*\`(todo|active)\`'` — any decoration before the id, and a ✅-headed
   line excluded even when it quotes the word (a done heading may say *was `todo`*). The convention that makes the
   exclusion sound: a done heading is ✅-headed.
3. **Declare the level — then read every level, each leaf once.** `.49` read `#####` and named the eight `###` /
   `####` headings it did not read; `.51` (the same day) reads every level 3–6 with the umbrella rule: an open
   heading is counted only when no open heading lies in its own section, so `.13` is counted through its children
   and never beside them. Four of those eight were done-under-open; three joined the number (18 → 20).
4. **Keep the retired expression as an arm.** `--self-test` drives both censuses on scratch trees whose lines are
   the real shapes, and one arm runs the OLD expression on the same five open lines and asserts it reads **1**.
   The number the instrument used to publish stays explicable from the script itself.
5. **The record has the same class of hole.** One history row carried an EMPTY release label (a commit whose subject
   named no release), and the trailing rate counted the empty string as a release. The label is now required — read
   from the register's last RELEASE row when the subject is silent, refused when neither says.

**The general rule.** When an instrument counts headings, rows or lines by a *mark* the author chooses freely, the
instrument's number is a function of style. Key it on the field that carries the fact, exclude the field that
carries the opposite fact, state the scope you did not read, and self-test on the shapes the corpus actually
contains — including the shape the old form got wrong.
