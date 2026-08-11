// Control for `invalid_param_trailing_comma.sv`: the same instantiation without the trailing
// comma parses, so the rejection is the comma and nothing else.
module sub #(int A = 1, int B = 2) (); endmodule
module m;
  sub #(
    .A(1),
    .B(2)
  ) u_sub ();
endmodule
