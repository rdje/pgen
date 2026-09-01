// SV-CORPUS-GRAD.13o — an ACCEPTING CONTROL: legal IEEE 1364-2005 that must keep parsing.
// 1364-2005 A.6.5 event_expression (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:713-717) ::= … | posedge expression | negedge expression | …
// WHY: pins the gate to the `edge` BRANCH alone — if edge_identifier itself were gated out of verilog_2005, this control goes RED.
module test;
  reg c;
  always @(posedge c) ;
endmodule
