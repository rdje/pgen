---
id: a-store-gate-that-decides-no-parse-is-a-resolution-check-annex-a-never-asked-for
title: A store gate that decides no parse is a resolution check Annex A never asked for — the checker-name fact gate rejected legal forward instantiations of a checker while the tie order already sent every plain instantiation to module_instantiation
answers:
  - "why did PGEN reject a checker instantiation whose checker is declared after the module — Surelog CheckerInst/dut.sv, ispras 14.14_05.sv"
  - "is it legal in IEEE 1800-2017 to instantiate a checker before its declaration — 3.2 design elements, 23.3.2 previously declared or one declared later"
  - "why did test dut(foo); inside a program block reject through PGEN 1.0.222 — iverilog program5b.v"
  - "what did the has_fact checker_name post gate on known_unscoped_checker_identifier decide — nothing, a declared checker with plain actuals is a module_instantiation"
  - "how does PGEN tell a checker instantiation from a module instantiation — the tournament tie goes to module_instantiation by order, sequence or property actuals go to checker_instantiation"
  - "why was phase: final not used for the checker-name forward reference in SV-CORPUS-GRAD.3.49"
  - "how to bisect a contextual rejection to its one variable — move the declaration, not the syntax"
  - "what was the 2026-03-21 semantic-fact pilot 3c63cf9b and why was its gate removed"
  - "does any other store gate in the SystemVerilog grammar reject a legal forward reference — the FORWARD-REFERENCE-CENSUS over the residual rows"
  - "why does the SystemVerilog grammar have checker_instantiation_beside_module with a !interface_instantiation lookahead — the parse-cost refusal of the ungated checker alternative"
tags: [systemverilog, grammar, semantic-store, store-gate, checker, forward-reference, bisection, parse-cost, sv-corpus-grad, census]
date: 2026-09-08
status: current
evidence: "SV-CORPUS-GRAD.3.49 (PGEN-SV-CORPUS-GRAD-0380, release 1.0.223, ledger SV-0096). On the 1.0.221 / 1.0.222 probes Surelog tests/CheckerInst/dut.sv rejects at furthest_position=303 (the second actual's `@`); sixteen bisection variants in rust/target/s349/ all reject with the stop sliding, and the three that move the checker declaration flip the verdict with its position (13_checker_before_module.sv passes, 15_checker_inside_module_last.sv rejects); 20_declared_plain.sv dumps {\"kind\":\"module_instantiation\"} for a declared checker with plain actuals. Interpreter A/B HEAD vs candidate, 29 files × 3 profiles: 34 verdict arms false → true on the SV profiles, 0 AST moves on any unchanged-verdict arm. CHECKER-CENSUS: lane_files=16336 with_endchecker=37 forward_instantiated=5; INSTANTIATION-HEAD-CENSUS: sv_2017 residual_rows=79 with_instantiation_head=8; FORWARD-REFERENCE-CENSUS: residual_rows=79 forward_declared_after_call_use=0."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_checker_forward_instantiation.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_checker_instantiation_in_program.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_checker_instantiation_v2005.sv --profile verilog_2005 2>&1 | grep -q 'furthest_position=' && ! grep -q 'has_fact, args: \\[checker_name, \\$body\\], phase: post }' <(grep -A1 -B1 '^known_unscoped_checker_identifier' grammars/systemverilog.ebnf) && echo OK"
---

**The fact.** IEEE 1800-2017 §3.2 lists the checker among the design elements of a compilation unit; §23.3.2 says an
instantiated name *"can be a module previously declared or one declared later"*; A.4.1.4 derives `checker_instantiation
::= ps_checker_identifier name_of_instance ( [ list_of_checker_port_connections ] ) ;` and A.9.3 `checker_identifier ::=
identifier`. Annex A resolves nothing about the name — whether it denotes a checker declared later, or in another file,
is elaboration semantics. Through release `1.0.222` the SystemVerilog grammar's unscoped checker name carried
`@predicate: { name: has_fact, args: [checker_name, $body], phase: post }`, landed on 2026-03-21 by the commit *"Pilot SV
checker semantic facts"* (`3c63cf9b`) to prove the fact model worked on a second seam. A `post` predicate fires at the
reference's own position, before a later declaration has emitted its fact, so `my_checker check(p2(a, b, c),
@($global_clock) a[*1:$] ##1 b);` rejected at the second actual whenever `checker my_checker … endchecker` followed the
module — which is how both corpus rows are written — and whenever the checker lived in another file.

**What the gate decided: nothing.** The obvious reading — the fact tells a checker instantiation from a module
instantiation — is false, and the AST says so: a DECLARED checker instantiated with plain actuals dumps
`{"kind":"module_instantiation"}`, because both alternatives match the same length and the tournament tie goes to
`module_instantiation` by order whether or not the fact exists. The only parses the checker alternative can win are
the ones whose actuals are sequence or property syntax, which no module port admits and which need no fact to be
recognised. So the gate could only ever REJECT: legal forward instantiations, legal cross-file instantiations, and
`test dut(foo);` inside a `program` (iverilog `program5b.v`, whose key had already read it as BNF-parseable
checker syntax). It was deleted; the fact is still emitted at the declaration, the scoped form is untouched.

**Why not `phase: final`.** The semantic store's book prescribes `final` for a legal forward reference, and it would
have carried the two bar rows. It would also have kept rejecting a checker declared in another compilation unit and
the program-block row — text the answer key calls BNF-parseable — and it would have given the checker name a check
`module_instantiation` never had. Refuted before the fix landed.

**The cost rule.** A store gate is also a cost device. With the gate simply gone the ceremony's first pass measured, on
the pinned sample, `committed +33.56 %`: at every plain-actual `ident inst(args);` the tournament now parsed the actuals a
second time as property actuals before losing the tie to `module_instantiation`. Costs are rejected, not traded, so the
contexts where an instantiation-shaped alternative competes reach a beside-module twin,
`checker_instantiation_beside_module := !interface_instantiation ps_checker_identifier name_of_instance ( … ) ;` —
`interface_instantiation` derives exactly the language `module_instantiation` derives (A.4.1.1–A.4.1.3) and is itself
tried at every such position, so the lookahead is a memo lookup that commits nothing; a checker body, a program and a
procedural statement keep the unguarded rule, because there nothing competes and the guard would re-reject
`test dut(foo);`.

**The bisection rule.** The rows had been filed as a CONTEXTUAL rejection because a hand-written reproducer parsed —
it had put the declaration inside the module before the instantiation. Sixteen variants of the corpus file that each
deleted one construct inside the module all still rejected with the stop sliding a few bytes; the three that moved the
declaration flipped the verdict with its position. Reproduce the corpus file's SHAPE (where things are), not just its
constructs; a "contextual" rejection is a declaration-ORDER rejection until the bisection says otherwise.

**The census rule.** Before the sweep, three censuses bounded the change: 37 lane files declare a checker and five
instantiate one first (two are the bar rows, one passes with plain actuals, two stop inside `ifdef`s); eight residual
rows head an instantiation with a demanded name and three are this defect; zero residual rows call a property,
sequence, let, covergroup or checker before its declaration — so no sibling `post` gate is owed a slice by the bar
today, and the claim that this was the only forward-reference over-rejection carries its enumerating command
(`rust/target/s349/forward_reference_census.py`).
