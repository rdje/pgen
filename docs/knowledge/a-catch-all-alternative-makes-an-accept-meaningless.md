---
id: a-catch-all-alternative-makes-an-accept-meaningless
title: In a rule whose last alternative is a catch-all expression, an ACCEPT is not evidence the intended alternative fired — force the alternative with a token the catch-all cannot swallow
answers:
  - "this construct parses, so the grammar supports it — is that a safe conclusion"
  - "how do I test that a specific alternative of a choice rule actually fires"
  - "why does intersect { 5, 6 } parse but intersect { 5, [1:3] } reject"
  - "how do I build a control that rules out an accidental parse route"
  - "my LR-eliminated rule seems to work — how do I check its non-seed continuations"
  - "why did a corpus pass rate never surface this missing alternative"
  - "how do I probe a grammar rule that ends in a general expression fallback"
tags: [grammar-authoring, instrument-soundness, adjudication, lrm-fidelity, systemverilog, toolbox, over-acceptance]
date: 2026-08-11
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaves .13c.2a / .13c.2a.1 / .13c.2a.2 / .13c.2a.3; grammars/systemverilog.ebnf `select_expression` (8 alternatives, last is cross_set_expression -> covergroup_expression -> expression) and `select_condition`; stimuli/sv/adjudication_repros/{defect_select_expression_with,control_select_expression_and,defect_select_expression_paren,defect_intersect_range_list,control_intersect_value_list}.sv; docs/systemverilog/2017/md/section-19-functional-coverage.md §19.6.2 + A.2.11
reverify: "mkdir -p tmp/r && for b in 'binsof(ca) \\&\\& binsof(cb)' 'binsof(ca) intersect { 1 } \\&\\& binsof(cb) intersect { 2 }' 'x with (a == 1)'; do printf 'module m; bit [2:0] a,b; covergroup cg; ca: coverpoint a; cb: coverpoint b; x: cross ca, cb { ignore_bins ib = %s; } endgroup endmodule\\n' \"$b\" > tmp/r/kc.sv; ./rust/target/release/parseability_probe --parse systemverilog tmp/r/kc.sv --profile sv_2017 >/dev/null 2>&1 && echo \"ACCEPT  $b\" || echo \"REJECT  $b\"; done"
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
| `binsof(ca) intersect {1} && binsof(cb) intersect {2}` | ACCEPT | the `&&` continuation genuinely fires |
| `( binsof(ca) )` | ACCEPT | **nothing** — a parenthesized expression |
| `( binsof(ca) intersect {1} ) && binsof(cb)` | REJECT | the `( select_expression )` arm is **inert** |
| `x with (a == 1)` | REJECT | the `with` continuation is **inert** |

The same trap one rule down: `select_condition ::= binsof ( … ) [ intersect { covergroup_range_list } ]`
has **literal** braces in the standard, and the grammar transcribed them as EBNF repetition. Yet
`intersect { 5, 6 }` parses — because `{5, 6}` is read as a *concatenation expression*. The braces
being gone only surfaces on `intersect { 5, [1:3] }` (the standard's own §19.6.2 example), because a
range is not an expression. ⇒ the rule was simultaneously **under**-accepting the legal form and
**over**-accepting through a route nobody designed.

## The technique: force the alternative with a keyword

Pick a token the catch-all cannot swallow — a **keyword** belonging to the alternative under test —
and put it where the fallback would otherwise absorb the input. `intersect` is what turned *"`&&`
seems to work"* into evidence, and what exposed the two arms that never fire.

Concretely, for any alternative `A` of a rule with a catch-all:

1. write the smallest input exercising `A`;
2. ask whether that input is *also* a plain expression — if it is, the probe is worthless;
3. add a keyword from `A` (or from a nested rule) until it is not;
4. keep the keyword-free version too, as a documented **control that proves only the boundary**, and
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

Related: [[a-rising-pass-rate-is-not-evidence-of-correctness]] (the same blindness at corpus scale),
[[annex-a-footnotes-license-derivations-the-productions-cannot-derive]] (when the standard's
examples outrank its productions), [[an-unchecked-search-step-can-move-away-from-the-answer]] (the
sibling discipline for automated diagnosis).
