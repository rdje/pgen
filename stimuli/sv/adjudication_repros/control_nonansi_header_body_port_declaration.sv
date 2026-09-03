// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// module t (clk); input clk; endmodule - the same body declaration after a NON-ANSI header
// IEEE 1800-2017 A.1.2 section-41-data-read-api.md:108-110; A.1.4 module_item ::= port_declaration ; section-41-data-read-api.md:327. isolates the defect to the header style.
module t (clk);
  input clk;
endmodule
