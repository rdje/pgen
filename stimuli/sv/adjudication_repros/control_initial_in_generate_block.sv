module m;
  generate
    genvar i;
    for (i = 0; i < 2; i = i + 1) begin : g
      initial $display("test");
    end
  endgenerate
endmodule
