// SV-CORPUS-GRAD.3.52 - the twin of invalid_foreach_bracket_per_variable.sv in the standard spelling (A.6.8 loop_variables, one bracket pair, a comma list): ACCEPT.
module m;
  int q[2][2];
  int n;
  initial foreach (q[i, j]) n++;
endmodule
