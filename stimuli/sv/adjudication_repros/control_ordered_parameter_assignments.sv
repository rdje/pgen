// SV-CORPUS-GRAD.20 — the legal spelling of invalid_empty_ordered_parameter_assignment.sv:
// every ordered element is an expression (1364-2005 A.4.1.1 / 1800-2017 A.4.1.1).
module p #(parameter A = 1, B = 2) ();
endmodule
module top;
  p #(2, 4) u();
endmodule
