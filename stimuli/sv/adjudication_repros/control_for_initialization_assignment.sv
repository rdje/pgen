// SV-CORPUS-GRAD.3.51 - the twin of invalid_for_initialization_operator_assignment.sv with the plain assignment A.6.8 derives: ACCEPT on the SV profiles (the `int` declaration and `x++` step are SystemVerilog).
module m;
  int x, y;
  initial for (x = 1; x < y; x++) ;
endmodule
