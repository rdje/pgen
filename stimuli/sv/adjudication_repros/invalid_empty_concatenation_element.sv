// SV-CORPUS-GRAD.52 — ILLEGAL on every profile: an EMPTY element inside a concatenation (`,,`).
// IEEE 1364-2005 A.8.1 concatenation ::= { expression { , expression } }
// (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:841); IEEE 1800-2017 A.8.1 the same
// (docs/systemverilog/2017/md/section-41-data-read-api.md:2880-2883). iverilog 13.0 refuses it under -g2005 and
// -g2012 (`syntax error`, `Malformed statement`). From ivtest/ivltests/pr904.v:26. Control: control_concatenation.sv.
module t;
  reg [8:0] kuku;
  initial kuku = {3'd5,3'd5,,3'd5};
endmodule
