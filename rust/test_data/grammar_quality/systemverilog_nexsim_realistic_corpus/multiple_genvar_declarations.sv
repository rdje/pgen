module multiple_genvar_declarations;
  genvar i, j;
  generate
    for (i = 0; i < 2; i = i + 1) begin : gi
      logic x;
    end
    for (j = 0; j < 2; j = j + 1) begin : gj
      logic y;
    end
  endgenerate
endmodule
