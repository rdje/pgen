---
id: an-operator-in-a-flat-operand-chain-is-a-chain-element-never-a-wrapper-around-its-left-operand
title: In a grammar that flattens binary expressions into ONE operand chain and leaves precedence to the consumer, a new operator is a CHAIN ELEMENT — spelling it as a wrapper around its left operand both rejects it as a left operand and mis-groups it on accepted text
answers:
  - "a inside {..} & ~b REJECTS while (a inside {..}) & ~b and b & a inside {..} ACCEPT — why, and what fixed it"
  - "IEEE 1800-2017 Table 11-2 puts inside on the relational row — how does PGEN's flat operand_chain encode that precedence"
  - "why did the SV inside_expression rule go away in release 1.0.213 and what does operand_chain.rest carry for inside now"
  - "how do I read an operand_chain rest entry that is an object rather than a [operator, attributes, operand] triple"
  - "can dist appear as a chain element like inside — where does IEEE 1800-2017 say dist cannot appear in other expressions"
  - "an inline ( a | b )* or ( a | b )? group inside an annotated multi-alternative rule typed its entries with the enclosing rule's kind templates — what is the workaround"
  - "why does a grammar spelling change that keeps every verdict still need a schema bump (wrong tree on accepted text)"
tags: [systemverilog, grammar, precedence, ast-shape, schema-bump, interpreter, release-ceremony, engine-defect]
date: 2026-09-04
status: current
evidence: "SV-CORPUS-GRAD.3.41 (PGEN-SV-CORPUS-GRAD-0368, release 1.0.213, ledger SV-0086, schema 26 -> 27). Until 1.0.212 `inside_expression := expression_base inside { open_range_list }` was a sibling of `expression_base` at the top of `expression`, so (i) `a inside {..} & ~b` REJECTED (furthest_position=630, the `&` after the `}`: the chain cannot continue past a wrapper) and (ii) `b & a inside {..}` derived as inside(expr: b & a) — `(b & a) inside {..}` on accepted text, measured with the interpreter's --interpret-parse-ast-json, against Table 11-2 (inside above &). One spelling fixes both: `expression_chain_step := binary_operator attribute_instance* expression_operand | inside_operator`, the set as a chain element `{kind: sv_2017|sv_2023, body: {kind: inside, ranges}}`. dist stays where A.2.10 puts it: IEEE 1800-2017 18.5 'dist expressions cannot appear in other expressions' (section-18-constrained-random-value-generation.md:444)."
reverify: "python3 stimuli/sv/run_adjudication_repros.py | tail -1   # failures=0; rejects_valid_inside_before_binary_operator.sv and fixed_inside_chain_two_sets.sv ACCEPT on both SV profiles with arm operand_chain>inside, control_inside_after_binary_operator.sv ACCEPT with the same arm, invalid_inside_without_set.sv REJECT, control_v2005_inside_identifier.sv ACCEPT under verilog_2005 with arm !inside; ./rust/target/debug/ast_pipeline docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/inline_alt.ebnf --interpret-parse docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/in.txt --interpret-parse-ast-json /tmp/x.json && grep -c invalid_sequence_access /tmp/x.json   # 1 until ENGINE-UNIVERSAL-SERVICES.96 lands"
---

**A flat operand chain is a promise: the grammar records the operators in source order and the
consumer applies Table 11-2.** `expression_base := expression_operand ( binary_operator … expression_operand )*`
keeps that promise for every binary operator — `b & a == c` is `[b, &, a, ==, c]` and the consumer
groups `a == c` first. Until release 1.0.212 the set-membership operator was the one exception:
`inside_expression := expression_base inside { open_range_list }` wrapped its LEFT operand, and
`expression := expression_base | inside_expression | conditional_expression` reached it only at the
top. Two defects, one cause:

1. **A wrapper cannot be a left operand.** After `a inside {2'b10, 2'b11}` the chain has ended, so
   `& ~b` has nothing to attach to: `furthest_position=630`, the `&` after the `}`. Rejected on both
   SV profiles; `(a inside {..}) & ~b` and `b & a inside {..}` accepted — the classic asymmetry that
   names the operand side as the defect (Protocol B step 4: pair the reproducer with its accepting
   twin).
2. **A wrapper groups.** `b & a inside {..}` derived as `inside(expr: operand_chain(b & a), ranges)`,
   i.e. `(b & a) inside {..}`, while Table 11-2 (IEEE 1800-2017 11.3.2, row 7 `< <= > >= inside dist`)
   puts `inside` ABOVE `&`. A consumer evaluating that tree evaluates the wrong expression, silently.
   Measured with the interpreter's `--interpret-parse-ast-json` on a five-line control — the
   instrument this repository uses to prove shape-neutrality before generating is also the one that
   finds a shape that was never neutral.

**The precedence-neutral spelling is a chain element.** `expression_chain_step := binary_operator
attribute_instance* expression_operand | inside_operator`, with `inside_operator` = `inside { … }` as an
object `{kind: "sv_2017"|"sv_2023", body: {kind: "inside", ranges}}`. `a inside {..} & ~b` is
`[a, inside{..}, &, ~b]`, `b & a inside {..}` is `[b, &, a, inside{..}]`, `a + b inside {..}` is
`[a, +, b, inside{..}]` — and in every case the consumer's precedence pass, which already exists,
groups `inside` as a postfix operator on the relational row. The grouped `inside_expression`, its
`{kind: "inside"}` wrapper at `expression` and the sibling branch at `expression_or_cond_pattern` are
retired: two encodings of one construct is a consumer trap, and the second encoding was wrong.

**`dist` is NOT the same case, and the leaf that opened this one said it was.** Table 11-2 lists `dist`
on the same row for precedence, but IEEE 1800-2017 18.5 rules *"dist expressions cannot appear in other
expressions"* (`section-18-constrained-random-value-generation.md:444`) and A.2.10 reaches it only
through `expression_or_dist ::= expression [ dist { dist_list } ]` — the top of a constraint
expression. `expression_or_dist := expression ( dist { dist_list } )?` already spells exactly that.
Read the clause prose before promoting a table row to a grammar change.

**A verdict-neutral spelling change can still be a schema bump.** Every `x inside {..}` the parser
already accepted now yields a different tree (the chain element instead of the wrapper), so this is a
REPLACEMENT in the `SV-0053` sense and takes schema `26 → 27`, with the migration in the contract:
a consumer that read `expression.kind == "inside"` reads `operand_chain.rest` entries that are objects.
The `.3.35` rule "compare a legal control's kind multiset HEAD vs new" is satisfied on every
`inside`-FREE control (byte-identical), and deliberately violated on every `inside` control — which is
the point of measuring it: the diff is the change, listed by file, not discovered by a consumer.

**And the first spelling of the fix found an engine defect.** The inline group
`( binary_operator attribute_instance* expression_operand | inside_operator )*` typed the first binary
step of an `inside`-free control as `{kind: "tagged_union", body: {kind: "bitwise_and"}}` — the
enclosing rule's alternative-0 template applied to the inner alternation's branch 0. A five-rule
synthetic reproduces it (`docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/`);
the lift into a named rule is exact and is what shipped. Routed as `ENGINE-UNIVERSAL-SERVICES.96`.
Byte-identity on an inside-free control is what caught it, one interpreter run before any byte was
generated: the control that "cannot move" is the one to run first. ⛔ The OPTIONAL form `( a b | c )? d`
reproduces too (`scope: {kind: "tagged", …}` on one branch, `<invalid_sequence_access>` on the other),
and a textual census finds that shape in three shipped SV rules (`hierarchical_btf_identifier`,
`method_call_receiver_sv_2017/_sv_2023`) — their lift is `SV-CORPUS-GRAD.3.42`, sequenced before the
SV release; the engine fix (a frontend refusal guard first, then passthrough typing read through the
differential gate) is `.96`, post-release.
