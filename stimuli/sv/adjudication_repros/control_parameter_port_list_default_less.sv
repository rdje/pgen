// SV-CORPUS-GRAD.3.60 CONTROL - a DEFAULT-LESS parameter after the comma stays a param_assignment (A.2.4 footnote 18):
// `T` alone (a typedef name used as a parameter name, followed by `)`) and `Q = 6` beside `Q`-less forms.
// Byte-identical before and after; the arm pins the default-less kind.
typedef logic [7:0] T;
module m1 #(parameter int P = 5, T) ();
endmodule
module m2 #(parameter int P = 5, Q = 6) ();
endmodule
module m3 #(parameter int P = 5, Q) ();
endmodule
