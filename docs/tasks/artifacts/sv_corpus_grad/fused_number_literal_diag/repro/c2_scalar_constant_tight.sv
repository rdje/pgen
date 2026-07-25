// CONTROL (ACCEPT at baseline, must HOLD): tight `1'b1` scalar_constant in a
// timing-check condition — IEEE 1800-2017 A.7.5.3 / A.7.5.1
module m (input clk, input d, input cond);
  specify
    $setup(posedge clk &&& cond == 1'b1, d, 10);
  endspecify
endmodule
