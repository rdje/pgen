// SV-CORPUS-GRAD.3.51 - the twin of invalid_foreach_without_loop_variables.sv and invalid_foreach_null_statement.sv: the brackets and a statement body: ACCEPT.
module m;
  int y[3];
  initial foreach (y[i]) y[i] = 0;
endmodule
