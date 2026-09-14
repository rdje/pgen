// SVPP-EXPANSION.8 - the control that proves the REGION does the work. The same declaration with no
// `begin_keywords region: under IEEE 1800-2017 `logic` is a reserved keyword (Annex B Table B.1), so
// it is not a legal net name and the text is illegal SystemVerilog. It must stay rejected forever.
module top;
  reg [63:0] logic;
endmodule
