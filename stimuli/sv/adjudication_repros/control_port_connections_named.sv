// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// m1 m0 (.a(j), .b(j)); m1 m2 (.*, .*); - all named, including the golden's duplicate .* (23.3.2 prose)
// IEEE 1800-2017 A.4.1.1 section-41-data-read-api.md:1811-1820. isolates the defect to the mixing.
module m1(input a, input b); endmodule
module t; logic j, a, b;
  m1 m0 (.a(j), .b(j));
  m1 m2 (.*, .*);
endmodule
