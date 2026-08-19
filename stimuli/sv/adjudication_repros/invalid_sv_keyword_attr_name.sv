// SV-CORPUS-GRAD.13c.2k — `attr_name ::= identifier` (1800-2023 A.9.1 :2092) and §5.6.2 forbids a
// reserved word as an identifier. `type` IS reserved in IEEE 1800 and is NOT in IEEE 1364-2005
// Annex B — so this text is ILLEGAL SystemVerilog and LEGAL Verilog-2005.
// PAIRED with control_v2005_keyword_attr_name.sv, which is the same text expected to ACCEPT under
// `verilog_2005`. Together they pin that PGEN distinguishes the dialects here; before .13c.2k both
// profiles wrongly accepted. Shape from iverilog/ivtest/ivltests/br930.v.
module m1((* type=1, name="a" *) input wire a);
endmodule
