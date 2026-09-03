// SV-CORPUS-GRAD.13x.c - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// enum { A[2], B[1'sd1] } x; - a plain and a sized signed decimal, both integral_numbers
// IEEE 1800-2017 A.2.2.1 section-41-data-read-api.md:875-876; A.8.7 decimal_number section-41-data-read-api.md:3244-3250. isolates the defect to the literal kind; B[1'sd1] is sv2v enum_range_neg.sv, correctly keyed must_accept.
module top;
  enum { A[2], B[1'sd1] } x;
endmodule
