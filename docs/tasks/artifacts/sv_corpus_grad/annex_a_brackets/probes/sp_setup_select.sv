module m(input clk, input [3:0] d, output [3:0] q); specify $setup(d[1], posedge clk, 1); endspecify endmodule
