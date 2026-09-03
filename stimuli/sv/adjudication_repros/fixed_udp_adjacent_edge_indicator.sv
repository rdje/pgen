// SV-CORPUS-GRAD.3.30 — FIXED in PGEN-SV-CORPUS-GRAD-0349 (release 1.0.207, ledger SV-0080).
// A UDP state table written with ADJACENT symbols, as IEEE 1800-2017 §29.3.4's own tables are
// (docs/systemverilog/2017/md/section-29-user-defined-primitives.md:204 "(01) (10) 0 : 0 : 1 ;";
// 1364-2005 §8.1.4 docs/verilog/2005/md/section-8-user-defined-primitives-udps.md:213). Annex A gives
// level_symbol ::= 0 | 1 | x | X | ? | b | B and edge_indicator ::= ( level_symbol level_symbol ) | edge_symbol
// as single tokens (1800-2017 A.5.3 section-41-data-read-api.md:1983-1988).
// WHY it rejected: the table's symbol rules reused the grammar's general keyword terminals `trivia /0\b/`
// etc., and `\b` cannot match a symbol followed by another symbol character. The spaced twin
// (control_udp_spaced_symbols.sv) always parsed. Corpus: Surelog tests/Udp/dut.sv, ispras 29/29.09_01.sv,
// iverilog ivltests/udp_dff.v, udp_bx.v, ispras 1364-2005 test_08_04/06/07_00_1.v.
primitive dff(q, clk, d);
  output q; reg q; input clk, d;
  table
    (01) 0 : ? : 0 ;
    (01) 1 : ? : 1 ;
    (1?) ? : ? : - ;
    (0x) ? : ? : - ;
  endtable
endprimitive
