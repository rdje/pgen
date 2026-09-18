// SV-CORPUS-GRAD.27 - the CONTROL for invalid_if_two_statements_then_else.sv - must keep ACCEPTING on every profile.
// The same two statements as a seq_block, the `else` paired (IEEE 1800-2017 A.6.6 / A.6.3).
module m;
  reg c, a, b;
  always @(posedge c) begin
    if (c) begin
      a <= 0;
      b <= 1;
    end else
      a <= 1;
  end
endmodule
