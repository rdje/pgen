---
id: a-transparent-rule-inherits-the-greed-of-the-rule-it-forwards-to
title: A rule whose alternative is a BARE reference inherits that rule's greed — so a starvation check on direct holders alone misses the holder one hop out
answers:
  - "left-recursion elimination made an unrelated construct stop parsing — why"
  - "how do I tell whether a rule is safe to rewrite into base ( suffix )*"
  - "my starvation analysis says the base rule is safe and a neighbouring rule regressed"
  - "why does 8'(1) parse in a constant expression but not in a statement after LR elimination"
  - "what does a bare-reference alternative propagate besides the AST"
  - "is checking direct holders of the base rule enough before eliminating left recursion"
tags: [engine, left-recursion, grammar-transform, peg, greedy-quantifier, starvation, ast-pipeline]
date: 2026-08-13
status: current
evidence: rust/src/ast_pipeline/indirect_lr_plan.rs (`rules_transparent_to` feeding `collect_starvation_sites`); grammars/systemverilog.ebnf:1029 (`cast := casting_type tick lparen expression rparen`) and :1033 (`casting_type := … | constant_primary`); the measured triple on the shipped grammar — `initial k = 8;` ACCEPT, `parameter logic [7:0] K = 8'(1);` ACCEPT, `initial k = 8'(1);` REJECT furthest_position=42; `stimuli/sv/run_adjudication_repros.py` reporting `FAIL control_size_cast_in_statement.sv control expect=ACCEPT got=REJECT`
reverify: "python3 stimuli/sv/run_adjudication_repros.py --verbose   # a CONTROL that stops parsing is the signature of this class"
---

**A bare-reference alternative (`A := … | B`, nothing after `B`) is transparent in two ways, and
only one of them is well known.** It is *AST-transparent* — it adds no wrapper node, which is why
annotations hoist through it. It is also **consumption-transparent**: `A` consumes exactly what `B`
consumes. So the moment `B` is rewritten into `B_base ( B_suffix )*`, with PGEN's greedy
non-backtracking `*`, **`A` becomes just as greedy as `B`** — and any rule holding `A` at a left
corner *with a residual* can be starved, even though it never mentions `B`.

Measured on SystemVerilog, two lines apart in the grammar:

```text
casting_type := … | constant_primary                          ← bare reference: TRANSPARENT
cast         := casting_type tick lparen expression rparen    ← holds it WITH a residual
```

Absorbing the indirect cycle at `constant_primary` made `constant_primary` greedy; `casting_type`
inherited it; `constant_primary` then swallowed `8'(1)` whole — a legal `constant_cast` in its own
right — and `cast` could never match its trailing `tick lparen expression rparen`. Three inputs one
difference apart pin it: `initial k = 8;` ACCEPT · `parameter logic [7:0] K = 8'(1);` ACCEPT ·
`initial k = 8'(1);` **REJECT**.

**So the criterion is a least fixed point, not a scan.** Transparent(X) = {X} ∪ {R : R has an
alternative that is a bare reference to something in Transparent}. A starvation site is any
alternative whose left corner is in Transparent(X) **and** whose residual is non-empty.

**Two things this class teaches beyond the fix:**

- **An isolating synthetic can be faithful to the DEFECT and blind to the FIX.** The synthetic that
  reproduced this cycle deleted its intermediates because nothing else reached them; the real
  grammar reaches them from elsewhere and keeps them. The synthetic's own header said so in prose —
  nobody had turned the sentence into a criterion. When a synthetic drops a rule "because nothing
  else uses it here", that deletion is a claim about the real grammar, and it needs its own check.
- **Only a two-sided ratchet with an accepting CONTROL can see it.** Every defect-side signal was
  green: the lint counter improved, the unit tests passed, ten generated parsers stayed
  byte-identical, the doctrine enforcer passed 18/18, and both target inputs flipped REJECT →
  ACCEPT. The single failing signal was a control row asserting that a *neighbouring* construct
  still parses.

Related: [[a-grammar-rewrite-must-verify-its-own-postcondition-because-a-static-criterion-only-answers-what-you-encoded]],
[[a-bare-rule-reference-alternative-is-ast-transparent]],
[[a-check-whose-inputs-all-pass-has-not-been-tested]].
