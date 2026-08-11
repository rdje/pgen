---
id: a-catch-all-alternative-makes-an-accept-meaningless
title: In a rule whose last alternative is a catch-all expression, an ACCEPT is not evidence the intended alternative fired — force the alternative with a token the catch-all cannot swallow
answers:
  - "this construct parses, so the grammar supports it — is that a safe conclusion"
  - "how do I test that a specific alternative of a choice rule actually fires"
  - "why does intersect { 5, 6 } parse but intersect { 5, [1:3] } reject"
  - "how do I build a control that rules out an accidental parse route"
  - "my left-recursive rule seems to work — how do I check its non-seed continuations"
  - "my control rules out one accidental parse route — is that enough"
  - "how do I assert WHICH alternative of a choice produced this AST"
  - "why did a corpus pass rate never surface this missing alternative"
  - "how do I probe a grammar rule that ends in a general expression fallback"
tags: [grammar-authoring, instrument-soundness, adjudication, lrm-fidelity, systemverilog, toolbox, over-acceptance]
date: 2026-08-11
status: current (row 2 of the probe table CORRECTED 2026-08-11 by SV-CORPUS-GRAD.13c.2a.2 — it claimed the `&&` continuation fired; the AST shows zero `and` nodes)
evidence: docs/tasks/SV-CORPUS-GRAD.md leaves .13c.2a / .13c.2a.1 / .13c.2a.2 / .13c.2a.3; grammars/systemverilog.ebnf `select_expression` (8 alternatives, last is cross_set_expression -> covergroup_expression -> expression) and `select_condition`; stimuli/sv/adjudication_repros/{fixed_select_expression_with,fixed_select_expression_paren,fixed_select_expression_or,control_select_expression_and,defect_intersect_range_list,control_intersect_value_list}.sv + MANIFEST.tsv `arm` column; docs/systemverilog/2017/md/section-19-functional-coverage.md §19.6.2 + A.2.11
reverify: "python3 stimuli/sv/run_adjudication_repros.py --verbose | grep -E 'select_expression|intersect'   # every armed row must print ok; the arm column is what makes the ACCEPTs mean something. To see the check bite, flip control_select_expression_and.sv's arm from !select_chain>and to select_chain>and in stimuli/sv/adjudication_repros/MANIFEST.tsv and re-run: the verdict stays ACCEPT and the run FAILS."
---

Many real grammar rules end in a **catch-all**: a final alternative that reaches the language's
general expression hierarchy. SystemVerilog's `select_expression` is one —

```ebnf
select_expression := select_condition | ! select_condition
                   | select_expression && select_expression
                   | select_expression || select_expression
                   | ( select_expression )
                   | select_expression with ( with_covergroup_expression ) [ matches … ]
                   | cross_identifier
                   | cross_set_expression        # -> covergroup_expression -> expression
```

The last arm will match anything that is a legal expression. That makes the rule **look** complete
and makes its gaps invisible to every accept-based measurement — a probe, a regression suite, a
corpus pass rate.

## The failure mode, measured

Three alternatives of that rule were inert, and each looked supported:

| probe | verdict | what it actually proves |
|---|---|---|
| `binsof(ca) && binsof(cb)` | ACCEPT | **nothing** — the whole thing is a legal expression |
| `binsof(ca) intersect {1} && binsof(cb) intersect {2}` | ACCEPT | ⛔ **nothing either** — see *the correction* below |
| `( binsof(ca) )` | ACCEPT | **nothing** — a parenthesized expression |
| `( binsof(ca) intersect {1} ) && binsof(cb)` | REJECT | the `&&` continuation is **inert** |
| `x with (a == 1)` | REJECT | the `with` continuation is **inert** |

The same trap one rule down: `select_condition ::= binsof ( … ) [ intersect { covergroup_range_list } ]`
has **literal** braces in the standard, and the grammar transcribed them as EBNF repetition. Yet
`intersect { 5, 6 }` parses — because `{5, 6}` is read as a *concatenation expression*. The braces
being gone only surfaces on `intersect { 5, [1:3] }` (the standard's own §19.6.2 example), because a
range is not an expression. ⇒ the rule was simultaneously **under**-accepting the legal form and
**over**-accepting through a route nobody designed.

## ⛔ The correction this card had to make about itself

Row 2 above shipped in this card claiming the opposite, and the reasoning behind it sounded
airtight: both operands carry the keyword `intersect`, so the input **cannot** be passing as one
ordinary expression. That premise was true. The conclusion was false. It was not passing as an
expression — it was the **seed** consuming the whole remainder through its own
`covergroup_range_list*`, the repetition that should have been literal braces. Dumping the AST
settles in one line what the exit code cannot: one `condition` node and **zero** `and` nodes, with
`binsof(cb)` parsed as a plain subroutine call and `intersect` as a hierarchical identifier.

⭐⭐ **Ruling out ONE accidental route is not ruling out THE accidental route**, and no amount of
further argument closes the gap — there is always another route you did not think of. The keyword
technique below is a good way to *build* a probe. It is not a way to *verify* one.

## The technique: force the alternative with a keyword, then READ THE ARM

Pick a token the catch-all cannot swallow — a **keyword** belonging to the alternative under test —
and put it where the fallback would otherwise absorb the input. Then confirm, from the parser's own
output, which alternative fired.

Concretely, for any alternative `A` of a rule with a catch-all:

1. write the smallest input exercising `A`;
2. ask whether that input is *also* a plain expression — if it is, the probe is worthless;
3. add a keyword from `A` (or from a nested rule) until it is not;
4. ⭐ **verify the arm, do not argue it** — dump the AST (`--parse-dump-ast`) and check for a `kind`
   that only `A` can emit, or read the `🏁 … selected branch N/M` trace line. In PGEN's SV
   reproducer manifest this is the `arm` column: a `>`-separated chain of `kind` values, optionally
   negated with `!`, checked by `stimuli/sv/run_adjudication_repros.py`, so an accept down any other
   route **fails**. The falsified control above now carries `condition,!and` — its refutation
   written down as a check;
5. that check needs a `kind` **unique to the alternative** — which is a grammar-authoring
   constraint, not just a testing one: an alternative whose emitted `kind` is shared with five other
   rules cannot be pinned at all, so uniqueness of `kind` is a testability property;
6. keep the keyword-free version too, as a documented **control that proves only the boundary**, and
   say so in its comment. A control that quietly implies more than it shows is worse than none.

## Why this outranks "it parses"

`furthest_position` cannot help here: the parse *succeeds*. A trace can — the winning branch is
printed —

```
🏁 Rule 'select_expression' selected branch 7/8 consuming 2 chars (branch_policy=longest_match)
✅ Rule 'select_expression' successfully parsed from 127 to 129 (consumed 2 bytes: ' x')
```

— which is how the missing `with` continuation was caught: the seed won and nothing extended it.
So when a rule has a catch-all, **read the selected branch, not the verdict**
(TOOLBOX Protocol D).

For grammar authoring the corollary is stronger than a debugging tip: a catch-all arm means the
alternatives above it get no coverage from incidental traffic, so they need deliberate,
keyword-forced samples — or they will be missing for years and every corpus number will say the
rule is fine.

Related: [[a-directly-left-recursive-alternative-inside-a-choice-is-dead-code]] (what all three inert
arms above turned out to be), [[a-rising-pass-rate-is-not-evidence-of-correctness]] (the same
blindness at corpus scale), [[annex-a-footnotes-license-derivations-the-productions-cannot-derive]]
(when the standard's examples outrank its productions),
[[an-unchecked-search-step-can-move-away-from-the-answer]] (the sibling discipline for automated
diagnosis), [[a-check-whose-inputs-all-pass-has-not-been-tested]] (why the `arm` column had to be
proven able to fail before it was trusted).
