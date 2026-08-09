module m;
  parameter P = 1;
  generate
    if (P == 1) begin : A
      wire x;
    end
  endgenerate
endmodule
