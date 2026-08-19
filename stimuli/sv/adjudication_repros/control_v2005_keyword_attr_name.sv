// SV-CORPUS-GRAD.13c.2k — the verilog_2005 half of the pair. Byte-identical construct to
// invalid_sv_keyword_attr_name.sv; `type` is not in IEEE 1364-2005 Annex B, so this is a legal
// Verilog-2005 attribute name and must ACCEPT.
module m1((* type=1, name="a" *) input wire a);
endmodule
