// SV-CORPUS-GRAD.13c.2 — CORRECT REJECTION. The `.name` implicit-connection shorthand is a
// PORT form only. IEEE 1800-2017 A.4.1.1:
//   named_parameter_assignment ::= . parameter_identifier ( [ param_expression ] )
// — the parentheses are mandatory for a parameter, unlike named_port_connection.
// ⛔ Accepting this would be an OVER-ACCEPTANCE defect. Expected FOREVER: REJECT.
module sub #(int A = 1) (); endmodule
module m;
  parameter int A = 1;
  sub #(.A) u_sub ();
endmodule
