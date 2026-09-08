---
id: a-routed-grammar-candidate-is-a-hypothesis-reproduce-it-in-isolation-first
title: A routed grammar candidate is a hypothesis — reproduce it in isolation before opening a grammar leaf; most of a long tail's candidates are tool spellings the standard never wrote or contextual stops that owe a bisection
answers:
  - "is foreach (A[i][j]) legal SystemVerilog or must it be foreach (A[i, j]) — A.6.8 loop_variables and 12.7.3"
  - "is -~c legal — can a unary operator apply to a unary expression in IEEE 1800 A.8.3"
  - "is @x[y] a legal event control — event_control and clocking_event in 1800-2017 and 1800-2023"
  - "can an event trigger target be a method call — -> b.get_event();"
  - "can int or integer carry a packed dimension — int [1:0] x"
  - "is output wire .p(expr) legal in an ANSI port list — ansi_port_declaration's port-expression alternative"
  - "can a hierarchical path contain a range select — intf[2:0].modport"
  - "can a $setuphold limit be a min:typ:max triple — timing_check_limit ::= expression"
  - "is (type) name; a legal declaration — Surelog's parenthesized type variable declaration"
  - "does footnote 18 cover class-body type parameters without a default — parameter type T; in a class"
  - "why does PGEN reject j = 1 + (i = i - 1) but accept j = (i = i - 1) — the parenthesised operator assignment as a binary operand"
  - "how were the .3.51 grammar candidates sorted — isolation reproducers on the release probe"
tags: [systemverilog, answer-key, annex-a, triage, foreach, unary, event-control, sv-corpus-grad, pins]
date: 2026-09-08
status: current
evidence: "SV-CORPUS-GRAD.3.52 (PGEN-SV-CORPUS-GRAD-0382). Thirty isolation reproducers under rust/target/s352/rep/ on the 1.0.223 probe: the class / scope group, 65'd3 ** a, the modport expression, inout wire reg, a port default with a call and a bare sequence instance PASS alone; ten reproducer / control pairs REJECT at the production's token and ACCEPT on the control (foreach (q[i][j]) @54 vs foreach (q[i, j]) PASS; -~c vs -(~c) PASS; @x[y]; @42 vs @(x[y]); PASS; int [1:0] t1 [3:0] @25 vs logic [1:0] t1 [3:0] PASS …); j = 1 + (i = i - 1); @38 and j = (i = i - 1) + 1; @43 while j = (i = i - 1); PASS. Twelve rows keyed must_reject; ADJUDICATION-REPROS: checked=1017 armed=219 listed=517 multi_profile_rows=376 failures=0 (its first run read failures=2: two verilog_2005 controls written with logic)."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_foreach_bracket_per_variable.sv --profile sv_2017 2>&1 | grep -q 'furthest_position=' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_foreach_comma_list.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_unary_on_parenthesised.sv --profile verilog_2005 2>&1 | grep -q 'parse_full passed' && grep -q 'pinned .3.52' stimuli/sv/adjudicate_external_corpus.py && echo OK"
---

**The fact.** A row routed as a "grammar candidate" from its stuck line is a hypothesis about the grammar, and the
cheapest test of it is a two-line reproducer on the release probe. `SV-CORPUS-GRAD.3.52` ran thirty such
reproducers over the groups `.3.51` had routed. The whole class / scope group parsed in isolation (a class-scoped
parameter in a comparison, a static member initialised with `new()`, `std::randomize(v) with { … }`, an `extern`
method in a class and in an interface, `endclass : Label`), as did a 65-bit sized literal under `**`, a modport
expression `(output .p(req[i]))`, `inout wire reg p`, a port default with a constant-function call, and a bare
sequence instance whether declared before or after its use — those rows stop elsewhere in their files and owe a
bisection each. What rejected in isolation was read against its production, and twelve of thirteen were
spellings the standard never wrote:

- A.6.8 `foreach ( array [ loop_variables ] )` with `loop_variables ::= [ index_variable_identifier ] { , [ index_variable_identifier ] }` — ONE bracket pair, a comma list, in the production and in §12.7.3's words and examples (`foreach( A [ i, j, k ] )`); `A[i][j]` is a tool spelling.
- A.8.3 `expression ::= primary | unary_operator { attribute_instance } primary | …` — a unary operator takes a primary; `-~c` needs `-(~c)`.
- A.6.5 `event_control ::= @ hierarchical_event_identifier | @ ( event_expression ) | …` (2023: `clocking_event ::= @ ps_identifier | @ hierarchical_identifier | @ ( event_expression )`) and A.9.3 `hierarchical_identifier` — a select only on a non-final element; `@x[y]` needs `@(x[y])`.
- A.6.5 `event_trigger ::= -> hierarchical_event_identifier ;` — no call: `->b.get_event();` is not derivable.
- A.2.2.1 `data_type ::= integer_vector_type [ signing ] { packed_dimension } | integer_atom_type [ signing ] | …` — `int`, `integer`, `byte`, … take no packed dimension.
- A.1.3 `ansi_port_declaration ::= … | [ port_direction ] . port_identifier ( [ expression ] )` — a direction only; `output wire .p(e)` is not derivable.
- A.9.3 `hierarchical_identifier ::= { identifier constant_bit_select . } identifier`, A.8.4 `constant_bit_select ::= { [ constant_expression ] }` — a bit-select, never a range: `intf[2:0].modport` is not derivable.
- A.7.5.1 / A.7.5.3 `timing_check_limit ::= expression` — a `min:typ:max` triple is not an expression.
- A.2.1.3 `data_declaration` — no parenthesised type: `(states_t) state1;` is Surelog's own extension by its comment.
- Annex A footnote 18 — a `type_assignment` may omit its data_type only within a `parameter_port_list`: `parameter type AT;` in a class body is not derivable.

The thirteenth was real: A.8.3 makes `( operator_assignment )` an `expression`, hence an operand of a binary
operator, and the grammar admits it only as an alternative of `expression_base` — `j = (i = i - 1);` parses,
`j = 1 + (i = i - 1);` does not. Routed as `SV-CORPUS-GRAD.3.53` with its AST-neutral fix shape drafted.

**The control rule.** A control fails for the construct's reason on every profile it is pinned on, or it pins
nothing. Two `verilog_2005` controls written with `logic` rejected at the keyword on the ratchet's first run;
respelled `reg` / `wire`, with their invalid twins, so every arm stops at the construct.
