// SV-CORPUS-GRAD.3.48 - a CONTROL: the nearest LEGAL twin (a range where the pin has a size) - must keep ACCEPTING on every profile.
// reg [1:0] b; reg [1:0][2:0] x; - A.2.5 packed_dimension ::= [ constant_range ] (section-41-data-read-api.md:1058-1062); IEEE 1364-2005 A.2.5 range (:310).
module m;
  reg [1:0] b;
  reg [1:0][2:0] x;
  wire [4:0] w [0:3];
endmodule
