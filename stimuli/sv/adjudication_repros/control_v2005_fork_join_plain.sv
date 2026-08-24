// SV-CORPUS-GRAD.13e.7 — an ACCEPTING CONTROL: legal IEEE 1364-2005 that must keep parsing.
// 1364-2005 A.6.3 par_block (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:529) ::= fork [ : block_identifier { block_item_declaration } ] { statement } join
// WHY: pins the join_any/join_none gates to exactly one difference: the TERMINATOR, not the fork statement
module test;
  initial fork join
endmodule
