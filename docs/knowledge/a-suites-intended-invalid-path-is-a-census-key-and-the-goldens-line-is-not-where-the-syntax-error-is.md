---
id: a-suites-intended-invalid-path-is-a-census-key-and-the-goldens-line-is-not-where-the-syntax-error-is
title: A suite's intended-invalid path is a census key, and the golden's line is not where the syntax error is — re-adjudicate an answer-key class one production at a time, by parsing a twin
answers:
  - "why is a verilator _bad test keyed must_accept while PGEN rejects it — is the parser wrong or the key"
  - "the upstream golden names an error on one line but the parser stops on another — which one is the syntax error"
  - "how do I re-adjudicate a corpus answer-key row (LRM cite, measured furthest_position, two-sided control, class=invalid pin)"
  - "a construct is legal in IEEE 1800-2023 but the corpus lane judges 1800-2017 — must_reject or must_accept"
  - "the store gate rejects a name the file declares with the wrong kind (wire used as a type, package after extends, class after implements) — is that a parser defect"
  - "a probe bank went RED silently for several commits — why does a conservation law need every exit named"
  - "a control written in SV vocabulary fails under verilog_2005 — how do I scope reproducer pins per profile"
tags: [sv-corpus, answer-key, adjudication, systemverilog, lrm, store-gate, banks, edition-contrast]
date: 2026-09-03
status: current
evidence: "SV-CORPUS-GRAD.13x.a–.13x.f (PGEN-SV-CORPUS-GRAD-0350 … -0356): 48 bar rows on suite-intended-invalid paths keyed must_accept — 38 BNF violations the parser was RIGHT to reject (on 9 the golden named a semantic error on a line the parser passes), 7 store-gate refusals of a wrong-kind or later-declared name (+3 the census could not see, explained by the new route 1b), 2 IEEE 1800-2023 constructs on the 1800-2017 lane, 1 parser defect (.3.31); the axis-2 bar 212 → 162 with ZERO parse verdicts moved; the .13w bank had been RED since -0347 because its conservation law forbade the `match` exit."
reverify: "python3 stimuli/sv/run_adjudication_repros.py | tail -1   # the 69 .13x pins (invalid/control per construct, per-profile edition contrasts, the .3.31 defect pair) hold; bash docs/tasks/artifacts/sv_corpus_grad/store_gate/probe.sh | tail -1   # 9/9"
---

**A suite's `_bad` / `test/error/` / `_fail` path is a cheap, honest census key for an answer key that
rests on the suite's own driver: the driver records the stage a TOLERANT parser reached, not what the
standard derives — and the syntax error is routinely on a different line than the golden's message.**

## The rules

- **Census the class before touching a row.** Derive the population by a printed rule (a path regex
  over the bar), measure every row on the release probe under EVERY shipped profile, and classify each
  by parsing a MINIMAL reproducer against its one-token-legal twin. Only then rule — one production at
  a time, never "the golden says it fails, so it fails".
- **The golden's line is evidence about the tool.** On 9 of 38 rows the `%Error` was a SEMANTIC error
  (an enum over a struct, a slice out of range) on a line the parser PASSES, while the parser stopped on
  text Annex A derives nothing for (two packed dimensions on an enum base, three ranges in a select). A
  key derived from the stage the tool reached is wrong in both directions at once.
- **A ruling carries four things or it is an argument**: the production the text has no derivation in
  (with the LRM file and line), the corpus row's measured `furthest_position`, a two-sided control (the
  legal twin ACCEPTS), and a `class=invalid` reproducer pinned beside a `class=control` one — so a later
  relaxation that starts accepting the text goes RED instead of improving the pass rate.
- **Fix the group's rationale, not the row.** When a member of a keyed GROUP is wrong, re-read every
  sibling: the four sv2v/ivtest rows each left a group whose rationale (*"the text is BNF-parseable"*)
  was true for every other member — `A[1'sd1]` is an `integral_number`, `A['x]` is not.
- **A lane judges its edition.** `function :extends …` and `parameter type enum E_t;` are IEEE
  1800-2023 syntax; on the 1800-2017 lane they are `must_reject` (the parser REJECTS under `sv_2017`
  and ACCEPTS under `sv_2023`, measured), and the same bytes are pinned `invalid` on one profile and
  `control` on the other. A tool that speaks the newer edition is not an oracle for the older lane.
- **A name declared with the wrong kind is the sound store gate, not a defect** — proven by a named
  transformation of the row's own bytes: rename every OTHER occurrence at its declaration, declare the
  name as one prelude kind, and the file must PARSE (route 1b of `store_gate_explanations.py`). Inside
  the fixed point prefer the step that COMPLETES the parse, else the one that reaches FURTHEST — the
  first improver is the wrong pick (`class` advanced 56 → 67 where `iclass` reached 135).
- **A conservation law must name every exit.** The `.13w` bank asserted `explained + unexplained ==
  frozen count`; rows also leave via `match`, so it went RED at the first grammar fix and stayed RED for
  six commits because nothing ran it. Monotone (`≤`) plus the exact PARTITION is the law that holds.
- **A `verilog_2005` control must be written in 1364-2005 vocabulary** (`wire`, `()`), never `logic` or
  `.*` — the pin runner's per-profile arm caught three before they were committed.

## The measurement

Bar `212 → 178 → 174 → 172 → 162` across five ruling slices, every one `pass → fail 0` and observed
verdicts moved on 0 rows; the `sv_2017` key's `clause-cited` grounding 427 → 461. See
`docs/tasks/SV-CORPUS-GRAD.md` `.13x` and the frozen census under
`docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/after/`.
