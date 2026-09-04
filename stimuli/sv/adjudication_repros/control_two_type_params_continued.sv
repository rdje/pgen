// SV-CORPUS-GRAD.3.35 - a CONTROL - must keep ACCEPTING.
// #(type T = int, T2 = logic) - a second TYPE assignment continuing the type list (no second identifier)
// IEEE 1800-2017 A.2.4 list_of_type_assignments docs/systemverilog/2017/md/section-41-data-read-api.md:1010-1016
module top #(type T = int, T2 = logic) ();
  T a; T2 b;
endmodule
