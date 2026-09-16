module m; genvar i; generate for (i = 0; i < 2; i = i + 1) begin : g parameter A = i; end endgenerate endmodule
