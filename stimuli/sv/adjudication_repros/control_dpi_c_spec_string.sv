// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// import "DPI-C" task dpix_twice;
// IEEE 1800-2017 A.2.6 section-41-data-read-api.md:1114. isolates the defect to the string literal.
module t;
  import "DPI-C" task dpix_twice;
endmodule
