// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// assign y = !(|v);
// IEEE 1800-2017 A.8.3 section-41-data-read-api.md:272-280 (a parenthesised expression is a primary). isolates the defect to the missing parentheses.
module t; wire [3:0] v; wire y;
  assign y = !(|v);
endmodule
