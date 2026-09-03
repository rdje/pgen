// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// m u();
// IEEE 1800-2017 A.4.1.1 section-41-data-read-api.md:1809-1810. isolates the defect to the missing name.
module m; endmodule
module t;
  m u();
endmodule
