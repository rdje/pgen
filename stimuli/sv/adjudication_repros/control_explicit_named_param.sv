// Control for `invalid_implicit_named_param.sv`: the LRM-shaped `.A(A)` parses.
module sub #(int A = 1) (); endmodule
module m;
  parameter int A = 1;
  sub #(.A(A)) u_sub ();
endmodule
