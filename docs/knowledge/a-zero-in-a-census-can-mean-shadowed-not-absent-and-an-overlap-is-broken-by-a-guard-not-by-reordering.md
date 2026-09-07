---
id: a-zero-in-a-census-can-mean-shadowed-not-absent-and-an-overlap-is-broken-by-a-guard-not-by-reordering
title: A zero in a corpus census can mean SHADOWED rather than absent — when two alternatives both succeed on the same text the longest-match tie goes to declaration order, and the overlap is broken by a GUARD on the wider alternative, not by reordering: order is a generation contract too (the generator renders an unplanned node with the first of equally cheap alternatives, so reordering moved the certificate's own prelude onto a gated branch); verify the gate you cite exists on the rule you touch, and read every node in its parent's context before counting it
answers:
  - "why did the nettype alias form nettype wTsum nettypeid2 parse as a data-type declaration"
  - "why did the raw-envelope census read zero nettype alias rows when the corpus has alias declarations"
  - "how does PGEN break a longest-match tie between two alternatives that both succeed"
  - "when is it safe to reorder two alternatives without changing the accept set"
  - "why did reordering the nettype alias alternative alone misread nettype T wT as an alias — the ungated net_type_identifier"
  - "why did putting the gated nettype alias alternative first make checked_nettype_identifier and nettype_scope_prefix UNKNOWN in the union certificate"
  - "how does generate_or choose between alternatives it was not forced to — fewest references, then declaration order"
  - "what does RAW-ENVELOPE-CENSUS report and why does it need the parent's context"
  - "why can a typed clocking_event_sv_2017 expression node not be told from an event_control_sv_2017 node"
  - "what did release 1.0.219 change in the SystemVerilog typed AST — the partial-typing residue"
  - "which list_of_path_delay_expressions alternatives were removed and why"
  - "what is the class of an LRM production name extracted as a keyword terminal — tx_path_delay_expression, class_qualifier"
  - "how was the .3.45 prediction of moved files made two-sided before generation"
tags: [systemverilog, grammar, ast-shape, annotation-campaign, census, instrument, branch-policy, store-gate, sv-corpus-grad]
date: 2026-09-07
status: current
evidence: "SV-CORPUS-GRAD.3.45 (release 1.0.219, ledger SV-0092, schema 30 -> 31). (1) Protocol D on IEEE 1800-2017 §6.6.7's own example on the 1.0.218 probe: `PGEN_TRACE_VERBOSITY=debug parseability_probe --parse systemverilog ntd_lrm.sv --profile sv_2017 --trace-rules net_type_declaration_sv_2017` — `net_type_identifier_sv_2017` and `declared_nettype_identifier` both parse (the alias alternative SUCCEEDS) and then `🏁 Rule 'net_type_declaration_sv_2017' selected branch 1/2 consuming 28 chars (… branch_policy=longest_match)`; the same under sv_2023 for the fully typed twin `nettype_declaration_sv_2023`. (2) The corpus census on every lane row (RAW-ENVELOPE-CENSUS, 1.0.218 probe): sv_2017 rows=5356 typed=2201 RAW=2319 passthrough=836 unclassified=0 alias_shadowed=2 in 1 088 + 2 files; sv_2023 RAW=1779 alias_shadowed=2; verilog_2005 RAW=217; zero raw nettype rows on either SV lane. (3) The census's first context test missed a clocking event inside an optional group's list under the `clocking` key (`$rose(a, @(posedge clk))`): the pin pre-screen read PRESCREEN-PINS: jobs=781 verdict_moved=0 ast_moved=38 while the census predicted 37; with the context key carried through lists, 38 predicted = 38 observed, 361 stayers, 0 mismatches. (4) The seeded corpus A/B (29 files × 3 profiles, 87 arms): every verdict and position identical, DIFF exactly on the arms carrying a raw or alias_shadowed row. (4b) THE FIRST CANDIDATE WAS REFUTED BY THE SWEEP: reordering the alias alternative without gating its base — `net_type_identifier_sv_2017 := declaration_identifier`, ungated, a fact I had asserted the opposite of — read `AST-IDENTITY-SWEEP: profile=sv_2017 … ast_moved=1095 verdict_moved=0` against 1 094 predicted movers; the one moved-but-not-predicted file, ispras 06.07.02_01.sv, had `nettype T wT;` (T a struct typedef) re-read as `{kind: nettype, base: T}`; the trace `net_type_identifier_sv_2017 successfully parsed ' T'` with no predicate rejection confirmed the missing gate; the fix routes the base to `checked_nettype_identifier | wildcard_escape_nettype_identifier` (SV-0037's own precedent) and the file reads SAME. (4c) THE SECOND CANDIDATE — gated alias alternative still first — passed pins, lanes, the two-sided sweep and the final census and was refuted by the union certificate: `union UNKNOWN=2` residual `[checked_nettype_identifier, nettype_scope_prefix]`; Protocol A (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1 PGEN_REACH_PATH_DUMP=1`, canonical config, seed 0, new pair vs the 1.0.218 snapshot pair): identical reach path to the target, but the PRELUDE that declares the nettype changed from `nettype bit \foo ;` to `nettype \foo \foo ;` — `generate_or` renders an unplanned node with the first of the alternatives tied on reference count — and the gate refuses it (`parsed=false`); the third spelling keeps the shipped order and breaks the tie by a negative lookahead on the data-type alternative. (5) The corpus AST identity sweep on the final bytes, the 1.0.218 probe vs 1.0.219: AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9756 identical=8662 ast_moved=1094 verdict_moved=0 both_nonaccepting=6580; AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9780 identical=8959 ast_moved=821 verdict_moved=0 both_nonaccepting=6556; AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2190 ast_moved=120 verdict_moved=0 both_nonaccepting=296 — two-sided against the census's movers: `sv_2017`: predicted 1094, moved 1094, predicted-but-identical 0, moved-but-not-predicted 0; `sv_2023`: predicted 821, moved 821, predicted-but-identical 0, moved-but-not-predicted 0; `verilog_2005`: predicted 120, moved 120, predicted-but-identical 0, moved-but-not-predicted 0. Record: docs/tasks/SV-CORPUS-GRAD.md .3.45; docs/tasks/artifacts/sv_corpus_grad/partial_typing_residue/census.md."
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --emit-raw-ast-json rust/target/km_raw.json > /dev/null 2>&1 && python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/partial_typing_census.py rust/target/km_raw.json | grep -o 'RAW_ENVELOPE=[0-9]*'   # RAW_ENVELOPE=0; python3 docs/tasks/artifacts/sv_corpus_grad/partial_typing_residue/raw_envelope_census.py --self-test | head -c 34   # RAW-ENVELOPE-CENSUS SELF-TEST: OK; grep -n -A3 '^net_type_declaration_sv_2017 :=' grammars/systemverilog.ebnf | grep -c 'kind: \"nettype\"'   # 1 — the alias alternative is typed; grep -c '!( nettype_scope_prefix? checked_nettype_identifier declaration_identifier semi )' grammars/systemverilog.ebnf   # 2 — the alias-head lookahead on both twins' data-type alternative"
---

**The fact.** `net_type_declaration_sv_2017` had two alternatives in the order IEEE 1800-2017 A.2.1.3 prints them —
`nettype data_type name [with f];` then `nettype [scope] net_type_identifier name;` — and on the LRM's own §6.6.7 example
`nettype wTsum nettypeid2;` BOTH succeed: `wTsum` was declared by an earlier `nettype T wTsum with Tsum;`, which plants a
`type_name` fact, so the data-type alternative reads it as a `known_unscoped_data_type` while the alias alternative's
`net_type_identifier` (gated on `declaration_family: nettype`) reads it as a nettype. PGEN's `|` is a longest-match
tournament and the two consume the same 28 characters, so the tie went to declaration order and the alias alternative
never won — on either SV profile, the fully typed `sv_2023` twin included. The typed AST called the base nettype a data
type for the whole life of the parser, and `.3.44`'s partial-typing census listed the alternative as a raw envelope to
type, which would have typed nothing anyone could see.

**The rule — and the claim that had to be measured.** When two alternatives can both succeed on one text, the one that
should win the overlap must be declared first AND must carry a gate that is actually narrower: then it wins exactly the
inputs it alone describes and loses every other input to the wider sibling. I wrote that the alias gate was a
nettype-family `type_name` fact — a strict subset of the data-type gate — reordered, built, and the two-sided corpus sweep
read 1 095 moved files against 1 094 predicted. The odd file was ispras `06.07.02_01.sv`: `nettype T wT;`, T a struct
typedef, now an alias, because `net_type_identifier_sv_2017 := declaration_identifier` had NO gate at all (the gated rule,
`checked_nettype_identifier`, sat twenty lines below, used by `net_declaration` and, since SV-0037, by `net_port_type`).
The fix routes the alias base to `checked_nettype_identifier | wildcard_escape_nettype_identifier` — the exact
`net_declaration` precedent — after which the typedef file reads SAME, the two alias files per lane move, and one
LRM-illegal form (`nettype undeclared_t x;`, no wildcard import) stops being accepted. ⛔ Verify the gate exists on the
rule you touch; a gate cited from a neighbouring rule's comment is a gate on the neighbour.

**Order is a generation contract.** The gated alternative, still first, passed every corpus instrument and then broke
the certificate: `generate_or` renders an unplanned node with the first of the alternatives that tie on reference
count, so the prelude the planner writes to declare a nettype for `checked_nettype_identifier`'s witness turned from
`nettype bit \foo;` into `nettype \foo \foo;` — an alias with an undeclared base — and the gate refused the certificate's
own sample. The overlap is therefore broken by a GUARD on the wider alternative (`!( nettype_scope_prefix?
checked_nettype_identifier declaration_identifier semi )` before `data_type`), with the shipped order kept: the parser
reads the same language, the generator renders the same preludes, and only the winner on a real alias moves.

**A zero is a claim.** The raw-envelope census read ZERO raw nettype rows on a corpus that contains alias declarations.
The right reaction was not "the alternative is unused" but Protocol D: parse the LRM's own example and read the `🏁
selected branch` line. The census now reports the shadowed rows explicitly (`alias_shadowed`) and counts them among the
movers, so the two-sided sweep check sees them.

**Context before counting.** Once typed, a `clocking_event_sv_2017` `@( … )` is `{kind: "expression", body}` under a
`{kind: "sv_2017"}` wrapper — the SAME shape `event_control_sv_2017` emits under the same wrapper, and five other
`sv_2017`-wrapped rules emit a `{kind: "expression", body}`. A context-free classifier would have counted every `@(posedge
clk)` in every `always` block. The census reads the wrapper only under an `event` / `clocking` key, under a `{kind:
"clocking", body}` wrapper, or inside the two raw envelopes that hold one — and the pin pre-screen still caught a slot
it missed (the optional group's LIST under the `clocking` key of a system call), which is what a two-sided prediction is
for: 37 predicted against 38 observed is a defect in the instrument, found before a byte was generated.

**The extraction class.** `list_of_path_delay_expressions` carried, since the grammar's first promotion, a twelve-value
alternative whose middle four positions were the regex terminal `/tx_path_delay_expression\b/` — the LRM production NAME
extracted as literal source text — beside a duplicate six-value spelling that lost every tie to the LRM's. `.13c.2m` owns
the same class through `class_qualifier`; this slice deleted the member the census surfaced, with the three editions'
five-form production as proof.
