module m(input clk, input [3:0] d, output [3:0] q); specify (d[1] => q[1]) = 1; endspecify endmodule
