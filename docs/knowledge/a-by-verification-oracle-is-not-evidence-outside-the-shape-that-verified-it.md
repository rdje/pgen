---
id: a-by-verification-oracle-is-not-evidence-outside-the-shape-that-verified-it
title: A "byte-identical" oracle is a claim with a DOMAIN — on an un-eliminated left-recursive cycle the interpreter and the generated parser return OPPOSITE verdicts, in both directions
answers:
  - "can I trust ast_pipeline --interpret-parse / the interpreter for this grammar"
  - "when must I cross-check an interpreter verdict against the generated parser"
  - "why did my synthetic grammar accept in one harness and reject in the other"
  - "what does authoritative BY VERIFICATION actually cover"
  - "is the parse-harness equivalence gate's byte-identity claim true for every grammar"
  - "how do I probe a grammar whose --lint-grammar reports left_recursion_unhandled > 0"
tags: [parse-harness, interpreter, oracles, left-recursion, toolbox, measurement-discipline]
date: 2026-08-12
status: current
evidence: docs/tasks/artifacts/engine_universal_services/indirect_lr/README.md (the 3-probe x 5-input x 2-oracle matrix; P1 `t'(n)` gen=accept/interp=reject, P1 `n'(n)` gen=reject/interp=accept, and every P2/P3 row in agreement); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaves .13 slice 3 and .14; rust/src/parse_harness_interpreter.rs:746-749 (the generated parsers block on `check_cycle_id`, the interpreter has no cycle guard at all — only a whole-stack depth ceiling)
reverify: "docs/tasks/artifacts/engine_universal_services/indirect_lr/probe.sh   # the AGREE column must show DIVERGE on exactly the two P1 rows until ENGINE-UNIVERSAL-SERVICES.14 closes"
---

PGEN has two ways to parse an arbitrary grammar, and they are **not** equally authoritative:

- the **compile-and-run harness** and the **scratch slot** run the shipped codegen and the shipped
  runtime — authoritative **by construction**;
- the **grammar-AST interpreter** (`interpret_parse`, `ast_pipeline --interpret-parse`) is a second
  implementation — authoritative **by verification**, because `PARSE-HARNESS.5` and `.6.1` pin it
  byte-identical to the generated parser.

That second sentence is what makes the interpreter quotable, and it is a claim with a **domain**:
whatever the two gates' corpora actually exercised. The domain is not written on the tool, and the
tool does not warn you when you leave it.

## The measured hole

On a six-rule indirect-left-recursive synthetic (SystemVerilog knot A, minimised), the two
implementations return **opposite verdicts on two of five inputs — in both directions**:

| input | generated parser | interpreter |
|---|---|---|
| `t'(n)` | accept | **reject** |
| `n'(n)` | **reject** | accept |

Both *eliminated* variants of the same cycle agree on every row, so the divergence is specific to a
cycle that **survived** LR elimination — exactly the grammars where `--lint-grammar` reports
`left_recursion_unhandled > 0`. The mechanism was documented all along and its *consequence* was
not: a generated parser breaks a cycle with `check_cycle_id`, whose verdict names one blocking
frame, while the interpreter has no cycle guard at all and reaches the same situations through a
whole-stack depth ceiling.

The suites are green because their one indirect-LR case has an escape hop one rule in, so no probe
input ever needs the recursion to be *entered twice*.

## The rule

Before quoting an oracle that is authoritative *by verification*, ask **what shape verified it** —
and if the grammar's own lint reports a defect in that exact shape, the answer is "nothing did".
Concretely, for `--interpret-parse`: run `--lint-grammar` first; a non-zero
`left_recursion_unhandled` means cross-check every verdict against the scratch slot or the
compile-and-run harness before it backs a claim.

⛔ **This is not a hypothetical cost.** The first draft of the measurement that found the hole was
interpreter-only and had both divergent rows **inverted** — not under-reported, *misreported*. It
read as "even one cast level fails", which points a fix at a mechanism that does not exist. It was
caught only because the scratch slot happened to be run for an unrelated trace.

The generalisation of the sibling lesson
[[a-catch-all-alternative-makes-an-accept-meaningless]]: an ACCEPT is not evidence until you name the
winning branch, a REJECT is not evidence until you name the rejecting mechanism, and a verdict from
a by-verification oracle is not evidence *at all* outside the shape that verified it.
