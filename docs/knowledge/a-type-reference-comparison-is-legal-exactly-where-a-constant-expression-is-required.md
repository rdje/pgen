---
id: a-type-reference-comparison-is-legal-exactly-where-a-constant-expression-is-required
title: A SystemVerilog type-reference comparison `type(a) == type(b)` is legal exactly where a constant_expression is required — Annex A of both editions puts type_reference in constant_primary and in no primary — and a leaf opened on "not a corpus row" owes a grep on the construct's own token before it is worked
answers:
  - "why does PGEN reject if (type(a) == type(b)) inside an initial block while generate if (type(a) == type(b)) parses"
  - "is a procedural case (type(x)) with type(...) items legal SystemVerilog"
  - "where may the SystemVerilog type operator appear in an expression — constant_primary, casting_type, data_type"
  - "what does IEEE 1800-2017 footnote 40 / 1800-2023 footnote 45 restrict about type_reference"
  - "why is sv-tests chapter-6/6.23--type_op_compare.sv keyed must_reject"
  - "why are verilator t_type_compare.v and t_type_expression_compare.v keyed must_reject"
  - "how was the SV-CORPUS-GRAD.3.32 partition question ruled and on what evidence"
  - "what does the parser stop on when it meets type(a) == type(b) in procedural code — the static cast's apostrophe"
  - "why did the .3.32 leaf say not a corpus row when three bar rows carried the construct"
  - "how does production_parity.py treat a wrongly cited Annex A line range — it compares the enclosing production faithfully"
tags: [systemverilog, lrm, annex-a, answer-key, adjudication, constant-expression, type-operator, sv-corpus-grad, census]
date: 2026-09-07
status: current
evidence: "SV-CORPUS-GRAD.3.32 (PGEN-SV-CORPUS-GRAD-0375, answer-key + pin tier, parser 1.0.219 unchanged). Release probe 011dbe9f8f1a2dd5… on both SV profiles: procedural `if (type(a) == type(b))` furthest_position=58, `initial case (type(a)) type(int): …` 42, `initial assert (type(a) == type(b))` 52 — each the byte after the first `type(…)`; `generate if` / `generate case` / `localparam bit EQ = (type(a) == type(b));` / `x = type(a)'(y);` / `if (type(a)'(y) == 3)` all PASS. Corpus grep on the construct's own token: 5 files, 3 of them bar rows (verilator t_type_compare.v:32 at 830, t_type_expression_compare.v:14 at 338, sv-tests 6.23--type_op_compare.sv:18 at 412), re-keyed must_reject; bar 95 -> 92, zero verdicts moved. IEEE 1800-2017 docs/systemverilog/2017/md/section-41-data-read-api.md:3109 (constant_primary), :3122-3140 (primary, no arm), :3560-3562 (fn 40), :841 (casting_type), :3186 (cast); 1800-2023 docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3066 / :3079-3094 / :3525-3527; 6.23 docs/systemverilog/2017/md/section-65536-216-bits.md:1610-1631; 11.2.1 docs/systemverilog/2017/md/section-11-operators-and-expressions.md:87-90."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_procedural_type_reference_comparison.sv --profile sv_2017 >/dev/null 2>&1; test $? -ne 0 && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_constant_context_type_reference_comparison.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && grep -c 'type_reference' docs/systemverilog/2017/md/section-41-data-read-api.md | grep -qx 4"
---

**The fact.** IEEE 1800-2017 §6.23 says a type reference *"can also be used in equality/inequality and case
equality/inequality comparisons with other type references, and such comparisons are considered to be constant
expressions (see 11.2.1)"*, and its only example is a `generate case (type(bus_t))`. Annex A agrees with the
sentence's frame, not with the intuition it invites: `type_reference` is an alternative of `constant_primary`
(`:3109`, footnote 40: legal as a `casting_type` and with the four equality operators only), of `data_type`
and of `casting_type` — and of no `primary` (`:3122-3140`). A procedural `if` takes
`cond_predicate → expression_or_cond_pattern → expression`, a procedural `case` takes `case_expression ::=
expression` with `case_item_expression ::= expression`, and `expression` reaches only `primary`. So the
comparison is derivable exactly where a `constant_expression` is required — generate `if` / `case`, parameter
and localparam initializers — and nowhere a plain `expression` is; the one way a type reference enters an
ordinary expression is a static cast, which is why the parser stops on the byte AFTER `type(a)`: the cast
derivation has consumed the type reference and wants its `'`. §11.2.1 names the frame: *"Some statement
constructs require an expression to be a constant expression."* The 2023 edition revised §6.23 (adding
`type(this)`) and kept every placement and the example — the `.3.43` tell that a restriction kept beside an
unchanged clause is the standard's intent.

**Prior art, read and outranked.** verilator's positive tests assert the procedural form (and its `_bad` test
parses `case (type(real))`, erroring on the ITEMS); slang's regression file and sv-tests' chapter-6.23 test write
it. No LRM text or example does. Under the north star (over-acceptance is a defect; the EBNF is the truth; spec
outranks tool) the ruling keeps the letter: three positive-test rows re-keyed `must_reject` on A.6.6 / A.6.7 /
A.8.4, the constant-context twins pinned as controls, no grammar byte moved.

**The process lesson.** The leaf was opened with *"Not a corpus row; found as a CONTROL while adjudicating
`t_type_compare_bad.v`"*. A grep on the construct's own token — `type\s*\([^)]*\)\s*(==|!=|===|!==)` and
`case\s*\(\s*type\s*\(` over the vendored suites — found five files, three of them `must_accept` bar rows on
both SV lanes. A "not a corpus row" is a CLAIM about a selector nobody ran (TOOLBOX §1.5c: select on the
construct's own token, never on a column or a memory of the census); it costs ten seconds to make it a
measurement, and the ruling's whole population turned out to be exactly those rows.

**The instrument lesson.** The `.13x.b` basis cited `primary` at `section-41-data-read-api.md:641-656` — the
`constraint_primary` / `constraint_expression` block. `production_parity.py` reads a cited line range as its
ENCLOSING production and compared that one between editions (`constraint_primary:DIFFERS`), which is faithful and
wrong; the clause-citation census judges clause NUMBERS against lines and never sees a production name. Corrected
to `:3122-3140`; a name-vs-line consistency arm for parity is routed to `SV-CORPUS-GRAD.14b.5`.
