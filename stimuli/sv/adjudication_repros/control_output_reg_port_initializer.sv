// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// output reg [2:1] z = 2'b10; - the variable form the LRM gives an initializer
// IEEE 1800-2017 A.2.1.2 section-41-data-read-api.md:762-765, section-41-data-read-api.md:994. isolates the defect to the port kind.
module t (z);
  output reg [2:1] z = 2'b10;
endmodule
