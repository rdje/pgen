---
id: a-single-character-terminal-that-may-touch-its-neighbour-must-not-carry-a-word-boundary
title: A single-character terminal that may touch its neighbour must not carry a word boundary — a closed symbol sub-language gets its own boundary-free terminals, and its rule names must not differ only by case
answers:
  - "UDP state table with adjacent symbols like (01) or ?? or 0x is rejected — why, and how was it fixed"
  - "when is trivia /x\\b/ the wrong terminal — symbols that may be followed by another symbol character"
  - "why do the SystemVerilog grammar's case-differing keywords carry hash suffixes (kw_s_a0f1490a vs kw_S_02aa629c)"
  - "my grammar lints green but the generated parser fails to compile with E0592 duplicate definitions RULE_"
  - "how do I prove a grammar terminal change is shape-neutral (adjacent vs spaced twin ASTs with spans stripped)"
tags: [grammar-authoring, terminals, systemverilog, udp, codegen, lint, sv-corpus]
date: 2026-09-03
status: current
evidence: "SV-CORPUS-GRAD.3.30 (PGEN-SV-CORPUS-GRAD-0349): level_symbol / output_symbol / edge_symbol referenced the general keyword terminals `trivia /0\\b/` … so `(01)`, `??`, `01`, `0x`, `(bx)` rejected on every profile while their spaced twins parsed — 16 sv_2017-lane and 6 verilog_2005-lane endtable rows, the largest family in the axis-2 bar; `--trace-rules level_symbol` at the `(01)` of Surelog/tests/Udp/dut.sv: `Terminal '?' failed at position 620 - found '0'`, `No match for regex pattern 'X\\b'` … Fourteen boundary-free `udp_sym_*` terminals fixed it with byte-identical ASTs for every spaced table; the first cut named `udp_sym_x` and `udp_sym_X` and the debug build failed with six E0592 (the generator uppercases rule names into RULE_ constants) although --lint-grammar had passed — routed ENGINE-UNIVERSAL-SERVICES.91."
reverify: "python3 stimuli/sv/run_adjudication_repros.py --verbose | grep -E 'udp_|ADJUDICATION-REPROS'   # the four .3.30 pins: two fixed, one control, one invalid, on all three profiles"
---

**Inside `table … endtable` the language is a closed sequence of single-character symbols, and IEEE
1800-2017 lets them touch; a `\b` on those terminals is a lexer assumption the grammar never made.**

## The rule

- A terminal for a symbol that may be immediately followed by another symbol character must not end in
  `\b`: `\b` needs a word/non-word transition and `0` followed by `1` has none. Give the closed
  sub-language its own boundary-free terminals (`udp_sym_0 := trivia /0/`, …) and reference them ONLY
  from the rules of that sub-language; leave the general keyword terminals — which serve `assert #0`,
  `$finish(0)`, `init_val`, `scalar_constant`, `z_or_x` — with their boundary, where a following word
  character must NOT be absorbed. Keep the `-> {kind: "…"}` shapes, so every input that parsed before
  yields a byte-identical typed AST.
- Prove shape-neutrality two ways: the spaced control's typed AST byte-identical across the fix on
  every profile, and an adjacent table's AST equal to its spaced twin's once source spans are stripped.
- Prove the widening is exactly the LRM's: an `invalid` pin with text no symbol production derives
  (`2 0 : 0 ;`, `0 a : 0 ;`) must keep rejecting.
- **Rule names must not differ only by case.** The generator uppercases a rule name into its `RULE_…`
  constant, so `udp_sym_x` and `udp_sym_X` emit one constant twice (`E0592`) — after `--lint-grammar`
  said green and the interpreter parsed every pin (it keys rules by name). Name the twins
  `udp_sym_<c>_upper`; the grammar's hash-suffixed keyword pairs are the same workaround. The lint arm
  that should refuse this is `ENGINE-UNIVERSAL-SERVICES.91`.

## Where it came from

The LRM's own tables are written with touching symbols (§29.3.4, `section-29-user-defined-primitives.md:204`
`(01) (10) 0 : 0 : 1 ;`), and §29.3.4 separates only the input fields *from the output field* — by a
colon. The corpus rows (Surelog `tests/Udp/dut.sv`, ispras 29.06 / 29.08 / 29.09, verilator
`t_udp_sequential*.v`, iverilog `udp_dff*.v`, `udp_bx.v`, the 1364-2005 `test_08_0[4-7]_00_1.v`) had sat in
the bar since the extractor first transcribed Annex A's symbols as keywords. Canonical homes:
`docs/tasks/SV-CORPUS-GRAD.md` `.3.30`; `grammars/systemverilog.ebnf` at `udp_sym_0`;
`docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` `.91`.
