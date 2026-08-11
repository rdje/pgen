---
id: left-recursion-is-an-engine-service-not-a-grammar-authoring-burden
title: Left recursion is an ENGINE service, not a grammar-authoring burden — PGEN eliminates both shapes, so a standard's Annex A production is transcribed verbatim
answers:
  - "I transcribed a left-recursive production from the standard and the operator never parses"
  - "does PGEN handle left recursion"
  - "why does the linter say left-recursive is handled by PGEN when the alternative never fires"
  - "why is my binary operator continuation inert while the rule still parses its operands"
  - "when does PGEN's LR elimination actually fire"
  - "how do I write a left-recursive Annex A production in a PGEN grammar"
  - "what does 'Infinite recursion detected in rule X' at the seed position mean"
  - "how do I find every dead left-recursive alternative in a grammar"
  - "should I hand-flatten a left-recursive rule into seed plus continuation"
  - "who owns a parsing concern that every EBNF shares — the engine or the grammar"
tags: [grammar-authoring, left-recursion, codegen, linter, instrument-soundness, lrm-fidelity, systemverilog, engine-boundary]
date: 2026-08-11
status: current (REWRITTEN by GRAMMAR-WELLFORMED.A2.5, which fixed the defect this card used to describe; the pre-fix behaviour is kept below because shipped parsers built before it still show it)
evidence: rust/src/ast_pipeline/mod.rs `normalize_direct_left_recursive_alternatives` + `retract_consumed_normalization_rules`; rust/src/parse_harness_combinator_suite.rs cases `direct_left_recursion`, `direct_left_recursion_multi_alt`, `direct_left_recursion_folded_ast`; docs/tasks/GRAMMAR-WELLFORMED.md leaf A2.5; docs/tasks/SV-CORPUS-GRAD.md leaves .13c.2a.2/.3/.4; grammars/systemverilog.ebnf `select_expression` + `block_event_expression`; stimuli/sv/adjudication_repros/fixed_select_expression_{with,paren,or}.sv + fixed_block_event_or.sv
reverify: "grep -q 'fn normalize_direct_left_recursive_alternatives' rust/src/ast_pipeline/mod.rs && grep -q 'direct_left_recursion_folded_ast' rust/src/parse_harness_combinator_suite.rs && python3 stimuli/sv/run_adjudication_repros.py >/dev/null && echo DIRECT-LR-IS-AN-ENGINE-SERVICE"
---

**Write the production the way the standard writes it.** PGEN eliminates left recursion in *both*
shapes it can appear in, so an EBNF author never has to know which one they wrote — and never has to
hand-compile a grammar to get an operator to parse.

```ebnf
expr := expr "+" term      # direct — the self-reference inline in the choice (Annex A's spelling)
      | term

expr := expr_add | term    # indirect — the self-reference hoisted into a wrapper rule
expr_add := expr "+" term
```

Both become `expr := expr_lr_base ( expr_lr_suffix )*` internally, and both rebuild the standard's
**left-nested** `lhs`/`rhs` tree from the author's own `-> {…}` annotations. The direct form is
normalized into the indirect form first (`normalize_direct_left_recursive_alternatives`), because a
direct alternative is mechanically the wrapper shape with the wrapper inlined — so one tested planner
handles both, rather than two paths that can disagree.

## ⛔ Why this card exists: it was not always true, and the failure was silent

Until `GRAMMAR-WELLFORMED.A2.5` (2026-08-11) the planner matched **only** the wrapper shape. A direct
alternative matched nothing it looked for, reached codegen intact, and met the runtime cycle guard at
the seed position:

```
🚪 Entering branch 3/8 for rule 'select_expression' at position 127
💥 Infinite recursion detected in rule 'select_expression' at position 127
…
🏁 Rule 'select_expression' selected branch 7/8 consuming 2 chars (branch_policy=longest_match)
```

⭐ **The guard did not *handle* the alternative; it *rejected* it.** The seed won, nothing extended it,
and the rule still parsed its operands — so the construct looked supported right up until something
had to follow the first operand. Three alternatives of IEEE 1800-2017's `select_expression` (`&&`,
`||`, `with ( … )`) and one of `block_event_expression` (`or`) shipped dead this way, and the linter
called them clean:

```
[info] grammar info: rule 'select_expression' is left-recursive (cycle: select_expression -> select_expression)
       — handled by PGEN's LR elimination + runtime cycle-breaking (informational, not an error)
```

That message was true for the wrapper shape and false for the direct one — the well-formedness
contract's own item 3 (*no dead branches*) failing in the **passing** direction. ⚠️ If you maintain a
parser generated before A2.5, this is still your behaviour; regenerate.

## ⭐ The rule the fix is an instance of

> The ENGINE handles what is objectively common to ALL EBNFs; a grammar carries ONLY what is specific
> to the language it describes. *(director ruling, 2026-08-11)*

A grammar-tier repair was implemented, verified, and then **reverted unshipped** under that ruling,
and the three reasons are worth keeping because they generalize to any "just work around it in the
grammar" proposal:

1. It breaks **EBNF-as-sole-source-of-truth** — the grammar stops transcribing Annex A and starts
   transcribing a hand-compilation of it.
2. It produces a **less faithful AST** — a flat chain, where the eliminator's fold rebuilds the
   standard's left-nested binary production from the author's own annotations.
3. It costs a **schema break the real fix would have to break again** — the one thing you do not do
   twice to a downstream consumer.

⛔ So do **not** hand-flatten a left-recursive rule into `seed ( continuation )*` to make it parse.
That was this card's previous advice and it is now the wrong answer. Flatten only when the *language*
wants iteration, never to work around the engine.

## What the engine will still refuse

- **A rule whose alternatives are ALL left-recursive** derives nothing. It is deliberately not
  normalized — hoisting would only hide the non-termination behind a helper rule, so it stays visible
  to the linter's `non_terminating` error.
- **An annotation the fold cannot replay** on a left-recursive alternative — `$text`, `$0`, a
  quantified extraction, a nested chain, or a positional beyond that alternative's own body length.
  `lr_chain_fold::validate_chain_templates` hard-fails **generation**, not parse time, so you learn at
  build time (`ENGINE-UNIVERSAL-SERVICES.8`).

## Sizing it in your own grammar

The sweep below reads the **post-elimination** gen-AST, so anything it prints is a genuine dead
alternative rather than one the planner already rewrote. On PGEN's SystemVerilog grammar it printed
`2 rules / 4 alternatives` before A2.5 and prints **nothing** after.

```bash
# NOTE the feature set: SystemVerilog's grammar needs the generated annotation backend, so a
# debug ast_pipeline built without `generated_parsers` REFUSES it rather than mis-parsing it.
cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline
./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
  --generate-stimuli --count 1 --seed 0 --dump-gen-ast tmp/ga.json
# then: for each rule, report any alternative whose first element references the enclosing rule
```

⚠️ Do not read a small count as reassurance. `--lint-grammar` prints only the first **10** of a
grammar's left-recursive findings with no way to show the rest, so 22 of SystemVerilog's were
invisible from the CLI; the sweep had to go around the instrument to see them. Deriving the linter's
verdict from what the eliminator actually accepts is still owed (`GRAMMAR-WELLFORMED.A2.5`, linter
half).

Related: [[a-catch-all-alternative-makes-an-accept-meaningless]] (how the four dead arms looked
supported for so long), [[annex-a-footnotes-license-derivations-the-productions-cannot-derive]] (the
standard's text is not always the grammar you want),
[[a-rising-pass-rate-is-not-evidence-of-correctness]].
