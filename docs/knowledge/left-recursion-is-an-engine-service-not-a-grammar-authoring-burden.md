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
  - "why does the lint say left_recursion_unhandled and what does it cost me"
  - "does PGEN eliminate INDIRECT left recursion"
  - "how do I see ALL of a grammar's lint findings instead of the first ten"
  - "why is 8'(1) rejected in a parameter but accepted in a statement"
  - "how much does a surviving left-recursive cycle actually cost me"
tags: [grammar-authoring, left-recursion, codegen, linter, instrument-soundness, lrm-fidelity, systemverilog, engine-boundary]
date: 2026-08-11
status: current (REWRITTEN by GRAMMAR-WELLFORMED.A2.5, which fixed the defect this card used to describe; EXTENDED by A2.6, which made the linter's verdict derived and found the claim was false for 30 of SV's 30 surviving cycles; the pre-fix behaviour is kept below because shipped parsers built before it still show it)
evidence: rust/src/ast_pipeline/grammar_wellformedness.rs `classify_left_recursion` + `LeftRecursionUnhandled`; rust/src/ast_pipeline/mod.rs `LeftRecursionEliminationOutcome` + `normalize_direct_left_recursive_alternatives` + `retract_consumed_normalization_rules`; rust/src/parse_harness_combinator_suite.rs cases `direct_left_recursion`, `direct_left_recursion_multi_alt`, `direct_left_recursion_folded_ast`; docs/tasks/GRAMMAR-WELLFORMED.md leaf A2.5; docs/tasks/SV-CORPUS-GRAD.md leaves .13c.2a.2/.3/.4; grammars/systemverilog.ebnf `select_expression` + `block_event_expression`; stimuli/sv/adjudication_repros/fixed_select_expression_{with,paren,or}.sv + fixed_block_event_or.sv
reverify: "grep -q 'fn normalize_direct_left_recursive_alternatives' rust/src/ast_pipeline/mod.rs && grep -q 'direct_left_recursion_folded_ast' rust/src/parse_harness_combinator_suite.rs && grep -q 'pub fn classify_left_recursion' rust/src/ast_pipeline/grammar_wellformedness.rs && ! ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar 2>/dev/null | grep -q \"handled by PGEN's LR elimination\" && python3 stimuli/sv/run_adjudication_repros.py >/dev/null && echo DIRECT-LR-IS-AN-ENGINE-SERVICE"
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
parser generated before A2.5, this is still your behaviour; regenerate. ⛔ **And the message is gone
entirely since `A2.6`**: it was false for every surviving cycle, not only the direct ones (30 of 30
on SV), so the linter now derives the verdict from what the pass actually did — see the closing
section.

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

✅ **Both instrument defects this card used to warn about are FIXED (`GRAMMAR-WELLFORMED.A2.6`,
2026-08-12).** `--lint-grammar` no longer prints only the first 10 findings with no way to see the
rest — every class shares one cap of 40 plus a `PGEN_LINT_DUMP_ALL=1` escape that the truncation line
names. And the verdict is now DERIVED from the elimination pass's own outcome rather than asserted:
`left_recursion_eliminated=N` (the rules the pass rewrote, by name, split as `D direct + I indirect` — it counted only the DIRECT pass until `ENGINE-UNIVERSAL-SERVICES.27`, 2026-08-16, and so under-reported every purely-indirect grammar) versus
`left_recursion_unhandled=M` (a warning — the pass ran, these survived it). ⛔ The count that
uncapping revealed is the reason this matters: SV's linter was telling **30 of 30** surviving cycles
they were handled while the pass had rewritten **2**, and one of those 30 costs real text — the
LRM-legal cast chain `int'(2)'(3)` is rejected via `casting_type -> constant_primary ->
constant_cast -> casting_type` (`💥 Infinite recursion detected in rule 'casting_type'`). Eliminating
**indirect** left recursion is an engine capability PGEN does not have yet:
`ENGINE-UNIVERSAL-SERVICES.13`.

## ⭐ A surviving cycle costs NOTHING until it is the only road

The most useful thing to know about an `unhandled` cycle is when it will actually hurt you, and it is
narrower than "this rule is left-recursive". Measured on that same SV cycle
(`SV-CORPUS-GRAD.13c.2b`), in a *constant* expression:

| input | what `casting_type` can match | verdict |
|---|---|---|
| `parameter int K = int'(1);` | `simple_type` — branch 1/5 | ACCEPT |
| `parameter logic [7:0] K = W'(1);` | `simple_type → ps_type_identifier` — branch 1/5 | ACCEPT |
| `parameter logic [7:0] K = 8'(1);` | only `constant_primary` — branch 2/5 | **REJECT** |

⛔ The guard fires in **all three** — the `W'(1)` trace prints `💥 Infinite recursion detected in rule
'constant_primary'` and then `🏁 Rule 'casting_type' selected branch 1/5` and passes. ⇒ the cost is
not *"the cycle exists"*; it is *"no OTHER alternative of the re-entered rule can match this text"*.
So a count of surviving cycles is not a count of defects, and sizing one means finding the inputs
where every sibling alternative is dead — not reading the cycle list.

⛔ **And `left_recursion_unhandled=N` counts RULE ROWS, not cycles.** The detector starts a DFS from
every rule, so one 12-rule cycle is reported 12 times. Canonicalising by rotation turns
SystemVerilog's **30 rows into 7 distinct cycles** and `ebnf`'s **5 into 3**
(`docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/canonicalize_lint_cycles.py`).
`.13`'s adjudication then priced all 10: **8 reject text the standard licenses**, 2 are
*dead-but-covered* (the alternative is unreachable, a sibling rule derives the same text — proven per
row by the trace/AST, never by the probe merely passing). ⇒ when you size this class in your own
grammar, canonicalise first, then hunt the input where every sibling is dead.

⭐ **And do not reach for the grammar tier when you find one.** The same leaf proved
`constant_primary_sv_2017` (15 alternatives), `constant_primary_sv_2023` (16) and `casting_type` (5)
are order-identical to the Annex A extraction, so hand-splitting the cycle would trade a byte-for-byte
standard transcription for a workaround — reason 1 above, with a measurement attached. That cycle
alone blocks 2 vendored OpenTitan corpus files, whose only unparseable construct is a numeric size
cast in a package parameter.

Related: [[a-catch-all-alternative-makes-an-accept-meaningless]] (how the four dead arms looked
supported for so long), [[annex-a-footnotes-license-derivations-the-productions-cannot-derive]] (the
standard's text is not always the grammar you want),
[[a-rising-pass-rate-is-not-evidence-of-correctness]].
