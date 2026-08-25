---
name: a-tool-derived-list-is-not-thereby-an-actionable-list
description: "DISCIPLINE (2026-08-25, SV-CORPUS-GRAD.13e.10(c4) / PGEN-SV-CORPUS-GRAD-0305) — a tool that names N repairs has told you the POPULATION, not the WORK. MEASURED: `--lint-grammar` named eleven `profile_orphans` and derived a minimal `@profiles` tag for each; the sizing slice (-0303) priced them as `11 companion tags, every one tool-derived` and published an edit budget of 28. FOUR of the eleven — `casting_type_lr_suffix`, `casting_type_lr_suffix_r4`, `casting_type_lr_guard0_suffix`, `casting_type_lr_guard1_suffix` — DO NOT EXIST IN THE SOURCE GRAMMAR at all: they are synthesized by PGEN's own left-recursion elimination pass, cannot be edited, and resolve on their own once their hosts are tagged (measured: 17 + 7 source tags => profile_orphans=0). The real budget was 24. ⛔ The sizing read the list off the tool and counted it without asking, of each row, `is this a thing I can WRITE TO` — the one question the tool never answers, because a lint reports on the grammar it ANALYSES (post-pass), not the file you MAINTAIN (pre-pass). ⇒ before pricing a tool-derived list, resolve every row back to a writable location and price the residue separately."
id: a-tool-derived-list-is-not-thereby-an-actionable-list
title: "A tool-derived list is not thereby an actionable list — resolve every row to a writable location first"
date: 2026-08-25
evidence: "docs/tasks/SV-CORPUS-GRAD.md leaf .13e.10(c4) (the correction, with the 17+7 => profile_orphans=0 measurement) against the same file's (c4) SIZING section from PGEN-SV-CORPUS-GRAD-0303 (which published `Total edit budget: 17 terminal tags + 11 companion tags = 28, every one tool-derived`); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .50"
reverify: "for r in casting_type_lr_suffix casting_type_lr_suffix_r4 casting_type_lr_guard0_suffix casting_type_lr_guard1_suffix; do grep -c \"^$r *:=\" grammars/systemverilog.ebnf; done | sort -u   # must print a single `0` — all four are engine-synthesized and appear in NO tracked grammar"
answers:
  - "the lint derived a fix for each finding, can I just apply them all"
  - "why was my edit budget wrong"
  - "why does a lint name a rule that is not in my grammar file"
  - "how do I price a repair from a tool-produced list"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-25
---

## The rule

A tool-derived list gives you the **population**. Before it can give you a **budget**, resolve every
row back to a location you can write to. Rows that resolve to nothing writable are not zero-cost —
they are a *different* question, and they belong in a separate line of the estimate.

## Why — the measurement

`--lint-grammar` is unusually helpful here: it does not merely name a `profile_orphan`, it derives
the minimal `@profiles` tag that would fix it, per rule. Eleven orphans, eleven named repairs. The
sizing slice priced exactly that: *"17 terminal tags + 11 companion tags = 28, every one
tool-derived."*

Seven of the eleven are rules in `grammars/systemverilog.ebnf`. The other four —
`casting_type_lr_suffix`, `casting_type_lr_suffix_r4`, `casting_type_lr_guard0_suffix`,
`casting_type_lr_guard1_suffix` — appear in **no tracked grammar in the repository**. PGEN's
left-recursion elimination pass synthesizes them, and the lint runs *after* that pass, so it is
reporting on a grammar that has more rules in it than the file anyone maintains. Tagging the seven
real hosts takes `profile_orphans` to **0** and the four synthetic rows resolve with them.

The published budget was wrong by 4 of 28, in the direction of overstating the work — which is the
harmless direction here, and would not have been on a list where the unwritable rows implied a
*larger* problem rather than a smaller one.

## The transferable shape

The lint's population is the **analysed** artifact; your edit budget is over the **maintained** one.
Whenever a tool runs after a transformation pass — LR elimination, desugaring, macro expansion,
inlining, a profile filter — those two artifacts are different, and the tool has no way to say which
of its rows crosses back. Asking *"can I write to this row?"* is the reader's job, not the tool's.

⚠️ Note what is NOT the lesson: the lint was not wrong. All eleven orphans are real, and naming them
is what let the seven be fixed at all. The defect was in the pricing step that treated *named* as
*actionable*.

Related: [[a-green-counter-can-be-the-one-hiding-the-counter-that-moved]] — the other half of the same
measurement, and the reason applying all eleven would have been worse than merely over-priced.
