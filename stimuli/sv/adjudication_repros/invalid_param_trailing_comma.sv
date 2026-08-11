// SV-CORPUS-GRAD.13c.2 — CORRECT REJECTION. A trailing comma in a named parameter value
// assignment list. IEEE 1800-2017 A.4.1.1:
//   list_of_parameter_assignments ::= … | named_parameter_assignment { , named_parameter_assignment }
// which admits no empty final element. ⛔ Accepting this would be an OVER-ACCEPTANCE defect.
// Expected FOREVER: REJECT.
module sub #(int A = 1, int B = 2) (); endmodule
module m;
  sub #(
    .A(1),
    .B(2),
  ) u_sub ();
endmodule
