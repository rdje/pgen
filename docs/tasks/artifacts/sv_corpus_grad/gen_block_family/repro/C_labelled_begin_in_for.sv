module m;
  genvar i;
  generate
    for (i = 0; i < 2; i = i + 1) begin : A
      wire x;
    end
  endgenerate
endmodule
