// SV-CORPUS-GRAD.13o — an ACCEPTING CONTROL on the SystemVerilog profiles: `edge` in an event expression is IEEE 1800.
// 1800-2017 A.7.4 edge_identifier ::= posedge | negedge | edge; A.6.5 event_expression ::= [ edge_identifier ] expression [ iff expression ] | …
// WHY: the fix is a verilog_2005-only NARROWING; sv_2017 / sv_2023 must be unmoved, and this control goes RED if the gate is placed one level too high.
module test;
  reg c;
  always @(edge c) ;
endmodule
