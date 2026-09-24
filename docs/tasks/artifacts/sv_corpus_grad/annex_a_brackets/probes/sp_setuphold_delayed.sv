module m(input clk, input [3:0] d, output [3:0] q); reg n; specify $setuphold(posedge clk, d, 1, 2, n, , , dclk, dd[1]); endspecify endmodule
