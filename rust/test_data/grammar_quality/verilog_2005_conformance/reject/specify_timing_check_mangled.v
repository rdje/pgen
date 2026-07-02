module m (d, clk);
  input d;
  input clk;
  specify
    sv_dollar_setup(d, posedge clk, 1);
  endspecify
endmodule
