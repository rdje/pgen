// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// localparam type ok = int;
// IEEE 1800-2017 A.2.4 section-41-data-read-api.md:1013. isolates the defect to the right-hand side.
module t;
  localparam type ok = int;
endmodule
