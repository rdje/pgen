---
id: a-long-tail-is-triaged-by-production-not-by-signature
title: A long tail is triaged by production, not by signature — when no stuck signature carries a slice, read every row against Annex A and the bar splits into text no edition derives (an answer-key defect, one production per ruling) and what owes a grammar leaf
answers:
  - "how was the SystemVerilog residual bar reduced from 79 to 53 rows without a grammar change"
  - "what did SV-CORPUS-GRAD.3.51 rule must_reject and on which Annex A productions"
  - "is a parameter value assignment without parentheses legal SystemVerilog — m1 #PAR m1();"
  - "are trailing commas in a module port list or parameter port list legal SystemVerilog"
  - "is for (x += 1; ...) legal — can a for_initialization be an operator assignment"
  - "is foreach (a[i]) ; with a null statement body legal SystemVerilog — statement vs statement_or_null in A.6.8"
  - "is foreach (y) without loop-variable brackets legal"
  - "is default :/ weight inside a dist legal in IEEE 1800-2017 or only in 1800-2023"
  - "does default disable iff take parentheses"
  - "can a delay value be a hierarchical name — assign #sub.delay"
  - "can a type be referenced through an interface instance — localparam type t = if0.rq_t"
  - "can an event trigger name a package-scoped event — -> pkg::e"
  - "can an enum base type be package-scoped or a bare dimension — enum pkg::t { } / enum [0:0] { }"
  - "is assert #0_0 the same as assert #0"
  - "can a method call appear inside an assignment target — a.get().x = 4"
  - "is vectored allowed on a variable declaration — logic vectored [15:0] a"
  - "why was the sv-tests 6.9.2--vector_vectored_inv.sv pin re-ruled"
  - "is 15'1 a legal literal"
  - "can an enum name index be negative — enum {e[-1]}"
tags: [systemverilog, answer-key, annex-a, edition-contrast, triage, sv-corpus-grad, pins]
date: 2026-09-08
status: current
evidence: "SV-CORPUS-GRAD.3.51 (PGEN-SV-CORPUS-GRAD-0381). cluster_rejects_valid.py on the 1.0.223 probe: 77 rows / 68 signatures, the largest with three rows. Twenty reproducer / control pairs under rust/target/s351/rep/ each REJECT at the production's own token and ACCEPT on the control on both SV profiles (m1 #PAR u1(); @28 vs m1 #(PAR) u1(); PASS; foreach (y[i]) ; @42 vs foreach (y[i]) y[i] = 0; PASS; default disable iff 1 dist { …, default :/ 4 }; @62 on sv_2017 and PASS on sv_2023; logic vectored [15:0] a = 0; @15 vs wire vectored [15:0] a; PASS …). Adjudicator: sv_2017 match 5990 → 6016, unexplained 79 → 53; sv_2023 match 5982 → 6007, unexplained 79 → 54 (edition_contrasts 20 → 21); verilog_2005 unchanged; observed verdicts moved on 0 rows. ADJUDICATION-REPROS: checked=973 armed=219 listed=497 multi_profile_rows=356 failures=0."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_foreach_null_statement.sv --profile sv_2017 2>&1 | grep -q 'furthest_position=' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_foreach_loop_variables.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_dist_default_under_2017.sv --profile sv_2017 2>&1 | grep -q 'furthest_position=' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_dist_default_sv2023.sv --profile sv_2023 2>&1 | grep -q 'parse_full passed' && grep -q 'pinned .3.51' stimuli/sv/adjudicate_external_corpus.py && echo OK"
---

**The fact.** Once every construct family with a grammar defect behind it has been taken, a corpus's residual
bar stops clustering: on the `1.0.223` probe the SystemVerilog bar was 77 rows over 68 stuck signatures, the
largest with three rows. Its rows do not share constructs; they share a provenance — each is one tool's
tolerance recorded as a key (verilator "expects success", sv2v "valid by suite contract", an ivtest golden with
"only post-parse errors", a kythe fixture). Read against Annex A, twenty-six of them stop at a token the cited
production does not derive, and the parser is right on every one:

- A.4.1.1 `parameter_value_assignment ::= # ( [ list_of_parameter_assignments ] )` — `m1 #PAR m1();`, `mreal #1.2 mr ();`
- A.1.3 `parameter_port_list` / `list_of_ports ::= ( port { , port } )` — `module ModuleA #(P=1,) (inp,);`
- A.6.8 `for_initialization ::= list_of_variable_assignments | …` with A.6.2 `variable_assignment ::= variable_lvalue = expression` — `for (x += 1; …)`
- A.2.2.1 `enum_name_declaration ::= enum_identifier [ [ integral_number [ : integral_number ] ] ] …` — `enum {ndef[-1]}`
- A.8.7 `unbased_unsized_literal ::= '0 | '1 | 'z_or_x`, `decimal_number ::= … [ size ] decimal_base unsigned_number` — `15'1`
- Annex B Table B.1 — `module medium();` (a charge strength, reserved in 1364-2005 too)
- A.2.10 `property_port_item` with `sequence_formal_type ::= data_type_or_implicit | sequence | untyped` — `property p(var int c)`
- A.8.5 `variable_lvalue` — `baz.get_bar().f.x = 4;`
- A.6.1 `delay_value ::= … | ps_identifier | …`, A.9.3 `ps_identifier ::= [ package_scope ] identifier` — `assign #sub.delay …`
- A.2.2.1 `data_type ::= … | [ class_scope | package_scope ] type_identifier { packed_dimension }` — `localparam type t = if0.rq_t;` (four rows)
- A.6.5 `event_trigger ::= -> hierarchical_event_identifier ;`, A.9.3 `hierarchical_identifier` — `#1 ->my_package::e;` (two rows)
- A.2.2.1 `enum_base_type ::= … | type_identifier [ packed_dimension ]` — `enum pkg1::uint8_t { … }`, `enum [0:0] { … }`
- A.6.10 `deferred_immediate_assert_statement ::= assert #0 ( expression ) action_block | …` — `assert #0_0 (1);`
- A.2.5 `packed_dimension ::= [ constant_range ] | unsized_dimension` — `logic [4] a, b;`, `S [WIDTH] s` (the `.3.48` ruling)
- A.1.2 `description ::= …` — iverilog's `$attribute(…)` at the top level
- A.6.8 `foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement` — `foreach (y) begin`, and `foreach (a[i]) ;`
- A.2.1.3 `net_declaration ::= net_type [ … ] [ vectored | scalared ] data_type_or_implicit …` — `logic vectored [15:0] a = 0;`

and one EDITION CONTRAST: `default disable iff 1 dist { …, default :/ 4 };` — the item takes no parentheses in
either edition, and `default` as a dist item is 1800-2023's (`dist_item ::= value_range [ dist_weight ] | default :/
expression`, §18.5.3), absent from 1800-2017's `dist_item ::= value_range [ dist_weight ]`: `must_reject` on the
2017 lane, `must_accept` on the 2023 lane, where the probe parses past it.

**The reading rule.** Read the production's right-hand side, not the family's. `foreach` is the one loop whose
body is `statement`; `forever`, `repeat`, `while`, `for` and `do` take `statement_or_null`, and the bare `;` is
`statement_or_null`'s own alternative — so `foreach (a[i]) ;` is not derivable although `for (…) ;` is. The
control first written for that row (`foreach (y[i]) ;`) rejected too; a control that rejects is a finding.

**The re-reading rule.** An earlier ruling of ours is re-read like any suite's. The `vectored` pin said *"the
syntax itself parses"* because the sv-tests header names a semantic failure (`:should_fail_because: bit selects
are not permitted on vectored vector nets`); the declaration line is `logic vectored …`, and `vectored` follows a
`net_type` in A.2.1.3 — the line was never derivable, which is where the parser stops.

**The pin rule.** One `invalid_*.sv` / `control_*.sv` pair per production, the invalid pinned on every profile
whose standard lacks the construct (a 1364-2005 profile rejects a trailing comma or an unparenthesised
parameter value for the same reason), the contrast as a pair split by profile (`invalid_dist_default_under_2017.sv`
on `sv_2017` / `verilog_2005`, `control_dist_default_sv2023.sv` on `sv_2023`).
