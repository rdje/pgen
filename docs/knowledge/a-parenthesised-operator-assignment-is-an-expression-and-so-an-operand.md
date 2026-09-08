---
id: a-parenthesised-operator-assignment-is-an-expression-and-so-an-operand
title: A parenthesised operator assignment is an expression and so an operand — a grammar that flattens the binary chain into operand-step must admit every A.8.3 expression form as an operand, and the form is IEEE 1800's, not 1364's
answers:
  - "why did PGEN reject j = i++ + (i = i - 1) — IEEE 1800-2017 11.4.2's own example — before release 1.0.224"
  - "is a parenthesised assignment legal as an operand of a binary operator in SystemVerilog — A.8.3 ( operator_assignment )"
  - "is j = (i = i - 1) legal Verilog-2005 — IEEE 1364-2005 A.8.3 expression has no assignment inside an expression"
  - "what is paren_operator_assignment_operand / expression_chain_operand in the SystemVerilog grammar"
  - "why is expression_chain_operand an ordered choice and why does that change no verdict"
  - "how did the top-level parenthesised assignment keep a byte-identical AST when the operand form was added — expression_chain_step+ as the chain head"
  - "does a constant expression admit ( P = 3 ) — no, A.8.3 constant_expression has no assignment"
tags: [systemverilog, grammar, expressions, annex-a, profiles, verilog-2005, over-acceptance, sv-corpus-grad]
date: 2026-09-08
status: current
evidence: "SV-CORPUS-GRAD.3.53 (PGEN-SV-CORPUS-GRAD-0383, release 1.0.224, ledger SV-0097). On the 1.0.223 probe: j = (i = i - 1); parse_full passed, j = 1 + (i = i - 1); furthest_position=38, (i = i - 1) + 1 @43; ispras 11.04.02_01.sv @230, sv-tests 11.3.6--two_assign_in_expr.sv @476 and -sim @530; integer i, j; initial j = (i = i - 1); parse_full passed under verilog_2005. Interpreter A/B HEAD vs candidate, 17 files × 3 profiles: the three corpus rows and the operand control false → true on the SV profiles, the v2005 form true → false under verilog_2005 alone, every other arm ast=SAME. Census by token: 5 operand-form files and 13 top-level-form files on the SV lanes, 0 and 0 on the verilog_2005 lane."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_paren_operator_assignment_operand.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_paren_operator_assignment_top_level.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_paren_operator_assignment_v2005.sv --profile verilog_2005 2>&1 | grep -q 'furthest_position=' && grep -q '^paren_operator_assignment_operand :=' grammars/systemverilog.ebnf && echo OK"
---

**The fact.** IEEE 1800-2017 A.8.3 `expression ::= primary | unary_operator { attribute_instance } primary |
inc_or_dec_expression | ( operator_assignment ) | expression binary_operator { attribute_instance } expression | …`
makes a parenthesised operator assignment an `expression`, and `expression binary_operator expression` takes an
expression on either side — §11.4.2's own example is `j = i++ + (i = i - 1);`. The SystemVerilog grammar flattens the
binary hierarchy into one operand chain (`expression_operand expression_chain_step*`, the `.3.41` shape), and through
release `1.0.223` the parenthesised assignment sat beside that chain as an alternative of `expression_base` — legal
at the top of an expression, rejected as a chain operand: `j = (i = i - 1);` parsed, `j = 1 + (i = i - 1);` did
not. The same omission had already been repaired for hierarchical indices by `.3.31`. And the alternative was
unprofiled, so `verilog_2005` accepted `j = (i = i - 1);` although IEEE 1364-2005 A.8.3 `expression ::= primary |
unary_operator { attribute_instance } primary | expression binary_operator { attribute_instance } expression |
conditional_expression` has no assignment inside an expression.

**The fix shape.** `paren_operator_assignment_operand := lparen operator_assignment rparen -> {kind:
"paren_op_assign", body: $2}` under `@profiles: ["sv_2017", "sv_2023"]`, reached (i) from every chain step through
`expression_chain_operand := expression_operand -> $1 | paren_operator_assignment_operand -> $1` under
`@branch_policy: ordered` — the two alternatives are disjoint (an operator assignment carries an assignment operator
after its lvalue, which no `primary` derivation admits), so ordered choice changes no verdict and stops the parser
trying the assignment form at every parenthesised operand the primary takes; (ii) as a chain head with
`expression_chain_step+` (one or more steps), so the existing top-level alternative — now `paren_operator_assignment_operand
-> $1`, the same node — is never tied and every accepted tree stays byte-identical; (iii) from the index rule. A
constant expression is untouched: A.8.3 `constant_expression` has no assignment, and `localparam Q = (P = 3);` still
rejects on every profile.

**The reading rule.** Where a form sits in a grammar is a claim about the language. A production admitted at one
level and not another says "legal only there", and the standard's productions do not say that. When a grammar
flattens a recursive production into a chain, every alternative of the recursive production is an operand.
