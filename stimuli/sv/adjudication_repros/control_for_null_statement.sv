// SV-CORPUS-GRAD.3.51 - the twin of invalid_foreach_null_statement.sv on a loop A.6.8 gives statement_or_null: `for (...) ;` ACCEPTS.
module m;
  int y[3];
  initial for (int i = 0; i < 3; i++) ;
endmodule
