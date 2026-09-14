// SV-CORPUS-GRAD.3.60 CONTROL - the type-list continuations that stay in the list (a default-less `U`, `U = real`) and
// the .3.35 continuation that leaves it (`T x = 0`, a typed parameter). Byte-identical before and after.
module m #(parameter type T = int, U) ();
endmodule
module m2 #(parameter type T = int, U = real) ();
endmodule
module m3 #(parameter type T = int, T x = 0) ();
endmodule
module m4;
  localparam type L = int, M = real;
endmodule
