// SV-CORPUS-GRAD.3.48 - a CONTROL: the nearest LEGAL twin (a range where the pin has a size) - must keep ACCEPTING on the IEEE 1800 profiles.
// NARROWED by SV-CORPUS-GRAD.50 (release 1.0.253): `reg [1:0][2:0] x;` is a SECOND packed range, which IEEE 1364-2005 A.2.1.3
// reg_declaration ::= reg [ signed ] [ range ] (docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:172)
// does not derive; the control had been listed on verilog_2005 through that over-acceptance (the .50 pins pre-screen read it
// accept -> reject there). The 1364-2005 one-range control is control_v2005_one_packed_dim_sites.sv.
// reg [1:0] b; reg [1:0][2:0] x; - A.2.5 packed_dimension ::= [ constant_range ] (section-41-data-read-api.md:1058-1062); IEEE 1364-2005 A.2.5 range (:310).
module m;
  reg [1:0] b;
  reg [1:0][2:0] x;
  wire [4:0] w [0:3];
endmodule
