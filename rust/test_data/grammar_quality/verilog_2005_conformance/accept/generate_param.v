module gen_demo #(parameter N = 4) (input [N-1:0] a, output [N-1:0] y);
  genvar g;
  generate
    for (g = 0; g < N; g = g + 1) begin : bit_loop
      assign y[g] = ~a[g];
    end
  endgenerate
  localparam DOUBLE = N * 2;
endmodule
