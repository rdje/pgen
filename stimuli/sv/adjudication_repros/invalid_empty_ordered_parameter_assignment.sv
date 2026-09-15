// SV-CORPUS-GRAD.20 — an EMPTY element in an ordered parameter value assignment.
// IEEE 1364-2005 A.4.1.1: `ordered_parameter_assignment ::= expression`; IEEE 1800-2017 A.4.1.1:
// `ordered_parameter_assignment ::= param_expression` — neither edition derives an empty element.
// Shape from iverilog/ivtest/ivltests/pr1716276.v (`param_test #( , 4) submod(...)`), which Icarus
// accepts as an extension; the legal spelling names the value or the parameter.
module p #(parameter A = 1, B = 2) ();
endmodule
module top;
  p #( , 4) u();
endmodule
