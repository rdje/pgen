---
id: gate-the-arm-not-the-rule-a-profile-blind-alternative-is-an-over-acceptance-on-the-narrower-profile
title: Gate the arm, not the rule — an alternative that only one edition derives is an over-acceptance on every profile whose standard lacks it, and the `_sv_only` helper with a pass-through dispatcher keeps the wider profiles' AST byte-identical
answers:
  - "why did PGEN accept reg x [2]; under the verilog_2005 profile before release 1.0.222"
  - "is the unpacked size form x [N] legal in IEEE 1364-2005 — what does A.2.5 dimension say"
  - "where does IEEE 1800-2017 admit an unpacked dimension written as a size [N] — A.2.5 unpacked_dimension and 7.4.2"
  - "how is one alternative of a shared rule gated to the SystemVerilog profiles without changing the SV AST — the _sv_only helper idiom"
  - "what is unpacked_dimension_size_sv_only in the SystemVerilog grammar"
  - "how does a -> $1 pass-through template keep a dispatcher from adding a wrapper node"
  - "how did SV-CORPUS-GRAD.3.48's two-sided control find the verilog_2005 over-acceptance that .3.50 fixed"
  - "what does an accepts_invalid pin row become when its fix lands — the MANIFEST.tsv round trip"
  - "how was the verilog_2005 lane censused for the unpacked size form before the grammar changed"
tags: [systemverilog, verilog-2005, grammar, profiles, over-acceptance, answer-key, pins, sv-corpus-grad, census]
date: 2026-09-08
status: current
evidence: "SV-CORPUS-GRAD.3.50 (PGEN-SV-CORPUS-GRAD-0379, release 1.0.222, ledger SV-0095). Release probe 9932110eb1e90029… (1.0.221): stimuli/sv/adjudication_repros/accepts_invalid_v2005_unpacked_dimension_size.sv (`reg x [2];`) parse_full passed under --profile verilog_2005; interpreter A/B HEAD vs candidate on 11 files × 3 profiles: exactly the two verilog_2005 size-form arms accepted=true → false (812 → 800, 32 → 20, the `[`'s byte), every other arm ast=SAME. The token census over the 2 606 verilog_2005 lane rows read 0 declaration rows (17 hits, all `$time` arguments or escaped identifiers); the corpus AST identity sweep on the final bytes read ast_moved=0 on every lane and verdict_moved=0 on every lane."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/accepts_invalid_v2005_unpacked_dimension_size.sv --profile verilog_2005 2>&1 | grep -q 'furthest_position=800' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_unpacked_dimension_size.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_packed_dimension_range.sv --profile verilog_2005 2>&1 | grep -q 'parse_full passed' && grep -q '^accepts_invalid_v2005_unpacked_dimension_size.sv\tREJECT\tinvalid' stimuli/sv/adjudication_repros/MANIFEST.tsv && echo OK"
---

**The fact.** IEEE 1800-2017 A.2.5 `unpacked_dimension ::= [ constant_range ] | [ constant_expression ]`
(`docs/systemverilog/2017/md/section-41-data-read-api.md:1046-1050`; 1800-2023
`docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1079-1083`; §7.4.2) admits an unpacked dimension
written as a SIZE — `logic x [2];` is `logic x [0:1];`. IEEE 1364-2005 A.2.5 has no such form: `dimension ::=
[ dimension_constant_expression : dimension_constant_expression ]`
(`docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:309`) and `range ::= [ msb_constant_expression :
lsb_constant_expression ]` (`:310`) are ranges alone, so `reg x [2];` is not Verilog-2005. Through release `1.0.221` the
SystemVerilog grammar's `unpacked_dimension := lbrack constant_range rbrack | lbrack constant_expression rbrack` carried no
`@profiles` on its second alternative, and every carrier of an unpacked dimension — a net declaration, a variable
declaration, a port, an instance array — accepted the size form under `verilog_2005`. Nothing in the corpus wrote it
(the 2 606 lane rows carry 0 such declarations), which is why the over-acceptance was invisible to the lanes: it was
found by the *control* of another slice. `SV-CORPUS-GRAD.3.48` re-keyed the `[N]` PACKED dimension `must_reject` on
every profile and wrote the two-sided controls — the range form (accepts everywhere) and the UNPACKED size form (accepts
under IEEE 1800, must reject under IEEE 1364-2005). The 1364-2005 twin came back `expected REJECT, got ACCEPT`.

**The fix shape.** Gate the ARM, not the rule. The size arm is lifted into a named helper,
`unpacked_dimension_size_sv_only := lbrack constant_expression rbrack -> {kind: "expression", body: $2}` under
`@profiles: ["sv_2017", "sv_2023"]` (the `_sv_only` idiom the grammar already uses for `interconnect_net_declaration_sv_only`,
`string_type_sv_only`, `const_qualifier_sv_only`, …; the `-0289` ruling that a profile difference is a NAMED rule so the
profile lints, both certificate baselines and `rule_costs.tsv` see it), and the dispatcher's second alternative is
`unpacked_dimension_size_sv_only -> $1` — a pass-through template, so the node a consumer reads on the SV profiles is
the helper's own `{kind: "expression", body}` with no wrapper around it. That is what makes the change AST-neutral by
construction on the profiles that keep the form, and it is measured, not argued: the interpreter A/B read `ast=SAME` on
every SV arm and the corpus AST identity sweep on the final bytes read `ast_moved=0` on every lane. Under
`verilog_2005` the helper is pruned by the profile, the alternative cannot match, and `reg x [2];` stops at the `[`.

**The pin rule.** A known over-acceptance is filed in `stimuli/sv/adjudication_repros/MANIFEST.tsv` as
`class=accepts_invalid` / `expect=ACCEPT` — the fifth class, the one that goes RED when the defect is FIXED — and the
fixing slice flips the row to `REJECT` / `invalid` citing its work unit, so the same row guards the fix forever. The
round trip here: filed by `.3.48` (`PGEN-SV-CORPUS-GRAD-0378`), flipped by `.3.50` (`PGEN-SV-CORPUS-GRAD-0379`).

**The census rule.** Before the grammar moved, the `verilog_2005` lane was censused by token for a declaration-shaped
`name [expr-without-colon]` and for an instance array `type name [expr] (`: 17 hits in 13 files, every one a `$time`
argument or an escaped identifier (`reg \array[0] ;`) once the line was read — 0 rows. A regex census is only a seed
(`.3.38`: the candidate list that does not contain the known positive has not been tested), so the pre-screen carried the
pin and the reproducer by name, and the exhaustive sweep on the final bytes is the measurement the number is quoted from.
