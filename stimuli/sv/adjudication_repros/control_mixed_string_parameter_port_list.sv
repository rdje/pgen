// SV-CORPUS-GRAD.3.61 - the mixed type/string parameter port list: the type tail and the declaration tail are clean
// arrays since 1.0.230 (the guard + comma envelope is gone); the arm pins the second type assignment's default.
module m #(type T = int, U = real, string S = "a", int P = 1) ();
endmodule
