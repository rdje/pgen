---
id: an-instrument-that-prints-an-unjudged-column-reports-a-defect-as-green
title: An instrument that PRINTS a column it does not JUDGE will file a defect as green — judge every axis you display, or refuse to display it
answers:
  - "my repro matrix says 0 cases differ but the verdicts look wrong — how"
  - "is it safe to run a one-profile matrix under a second profile just to look at it"
  - "why did a committed passing artifact contain nine over-acceptances"
  - "how do I add a second profile or dialect to an existing pass/fail matrix"
  - "what should an instrument do when asked about an input it has no expectation for"
  - "how do I get a real before measurement after I already regenerated the parser"
tags: [instrument-honesty, ground-truth, profiles, repro-matrix, proof-discipline, systemverilog]
date: 2026-08-09
status: current
evidence: docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/matrix.py (the per-profile EXPECT table and the unknown-profile REFUSE, both added by SV-CORPUS-GRAD.3.20); the pre-fix artifact after_v2005.txt from .3.19, which shows nine ACCEPT verdicts under a "cases differing from the post-fix expectation: 0" footer; before_3_20_v2005.txt, the same binary re-measured by the fixed instrument, reporting 9; docs/tasks/SV-CORPUS-GRAD.md leaves .3.19 and .3.20
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/matrix.py --profile pcre2; test $? -ne 0 && echo REFUSES-UNDECLARED-PROFILE"
---

**A column an instrument displays but does not check is read as a checked column.** The footer
summarises as though it covered everything on screen, and a reader scanning for red finds none.

## The measured instance

`SV-CORPUS-GRAD.3.19` needed a 17-case repro matrix for a SystemVerilog fix, and sensibly also ran
it under `--profile verilog_2005` to see what that dialect did. The committed artifact
(`after_v2005.txt`) shows nine spellings IEEE 1364-2005 has **no production for** — `use .W(8)`,
`use adder .W(8)`, `use #(.WIDTH(32))` and friends — each with `verdict ACCEPT`, each with
`want ACCEPT`, under a footer reading:

```text
cases differing from the post-fix expectation: 0
```

Nine over-acceptances, filed green. The cause was two lines:

```python
# (file stem, LRM citation, expected verdict AFTER the fix under sv_2017)
...
if args.profile == "sv_2017" and verdict != expected:
```

plus a comment stating the problem outright — *"`want` is the post-fix expectation under sv_2017
only; other profiles print it for reference without judging."* **The author knew, wrote it down, and
the artifact still shipped as a pass.** Awareness in a comment does not make a report honest; the
report is what the next reader believes.

The defect was ultimately caught by a human who thought the v2005 column looked odd — exactly the
labour the instrument exists to remove.

## The repair, in three parts

1. **Expectation is per `(case, axis)`, not per instrument.** The `CASES` table carries one expected
   verdict per profile group, because the LRM answer genuinely differs per dialect.
2. **Every displayed axis is judged.** If it is on screen, it is in the footer's count.
3. **An axis with no declared expectation is a REFUSAL, not an unjudged run.**
   `--profile pcre2` exits nonzero with `REFUSE: no per-case expectation declared for profile`.
   ⛔ This is the load-bearing part: without it, the next person to add a dialect silently
   re-creates the original bug by running the tool before adding its expectations.

## The trick that turned the claim into evidence

A `before` measurement seemed lost — the grammar had been edited and the parser regenerated. It was
not: the **release binary had not finished relinking**, so the pre-fix parser was still on disk.
Running the *fixed* instrument against the *unchanged* binary produced `9 cases differing` from the
same executable that had reported `0`. ⭐ **When you fix an instrument, re-measure the old state with
the new instrument** — that delta is the reproducer, and it is far stronger than re-reading the old
artifact and asserting what it should have said.

## The general rule

Whenever an instrument grows a second axis — a profile, a dialect, a target, a platform, a seed —
ask *"is the new axis judged, or merely displayed?"* If the expectation table has one column and the
output has two, the instrument is now capable of certifying a defect. Related:
[[a-negative-control-can-disable-the-assertion-it-is-testing]] (the same failure one level in: the
assertion exists but the item was silently moved out of the checked population) and
[[a-rising-pass-rate-is-not-evidence-of-correctness]].
